#![allow(dead_code)]
// ^ Prevents "unused function" warnings, those functions will
// likely either be used in the future or by GDScript.

use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

use gdnative::{
    api::{CollisionShape, MeshInstance, StaticBody},
    prelude::*,
};

mod block;
mod chunk;
mod chunk_mesh;
mod constants;
mod features;
mod generate;
mod macros;
mod mesh;
mod performance;
mod positions;

use crate::{
    block::BlockID, chunk::ChunkData, chunk_mesh::ChunkMeshData, generate::ChunkGenerator,
    macros::*, positions::*,
};

#[derive(Debug, Clone)]
pub struct NotLoadedError;

impl std::fmt::Display for NotLoadedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The chunk being modified has not been loaded")
    }
}
impl std::error::Error for NotLoadedError {}

pub struct Chunk {
    /// The chunk's position. This is determined by `Chunk.data.position` and
    /// is copied here for convenience.
    pub position: ChunkPos,
    /// The chunk's terrain data.
    pub data: ChunkData,
    /// The chunk node, its manifestation in the Godot world.
    node: Instance<ChunkNode, Shared>,
}

impl Chunk {
    fn new(data: ChunkData, node: Instance<ChunkNode, Shared>) -> Self {
        Self {
            position: data.position,
            data,
            node,
        }
    }
}

#[derive(NativeClass)]
#[export]
#[inherit(StaticBody)]
#[user_data(gdnative::export::user_data::MutexData<ChunkNode>)]
struct ChunkNode {
    owner: Ref<StaticBody, Shared>,
    collision: Ref<CollisionShape, Shared>,
    mesh: Ref<MeshInstance, Shared>,
}

#[methods]
impl ChunkNode {
    fn new(owner: &StaticBody) -> Self {
        let collision = CollisionShape::new();
        let mesh = MeshInstance::new();
        let (collision, mesh) = (collision.into_shared(), mesh.into_shared());
        owner.add_child(collision, true);
        owner.add_child(mesh, true);
        ChunkNode {
            owner: unsafe { owner.assume_shared() },
            collision,
            mesh,
        }
    }

    fn update_mesh_data(&mut self, mesh_data: ChunkMeshData) {
        let collision = unsafe { self.collision.assume_safe() };
        let mesh = unsafe { self.mesh.assume_safe() };
        let collision_shape = mesh_data.build_collision_shape();
        unsafe {
            // This MUST be call_deferred, setting the shape when using the
            // Bullet physics engine is NOT thread-safe!
            collision.call_deferred("set_shape", &[collision_shape.into_shared().to_variant()]);
        };
        mesh.set_mesh(mesh_data.build_mesh());
    }
}

/// Helper struct for `Rect2` -> `(x1, y1, x2, y2)` conversion
struct PositionRange {
    x1: isize,
    y1: isize,
    x2: isize,
    y2: isize,
}

impl From<Rect2> for PositionRange {
    fn from(rect2: Rect2) -> Self {
        let x = rect2.position.x as isize;
        let y = rect2.position.y as isize;
        let w = rect2.size.x as isize;
        let h = rect2.size.y as isize;
        Self {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

fn create_view<'a>(chunks: &'a HashMap<ChunkPos, Chunk>) -> HashMap<ChunkPos, &'a ChunkData> {
    chunks
        .iter()
        .map(|(pos, chunk)| (*pos, &chunk.data))
        .collect()
}

enum ChunkUpdate {
    Create(ChunkData),
    Update(ChunkData),
    Exists,
}

#[derive(NativeClass)]
#[export]
#[inherit(Node)]
pub struct ClientChunkLoader {
    owner: Ref<Node, Shared>,
    chunks: Arc<Mutex<HashMap<ChunkPos, Chunk>>>,
    send_channel: Sender<LoadThreadChannelContents>,
}

type LoadThreadChannelContents = (ChunkPos, Option<Vec<u8>>);

struct ChunkLoaderThread {
    node: Ref<Node, Shared>,
    chunks_map: Arc<Mutex<HashMap<ChunkPos, Chunk>>>,
    receiver: Receiver<LoadThreadChannelContents>,
}

impl ChunkLoaderThread {
    fn create_chunk(&self, position: ChunkPos, data: ChunkData) -> Chunk {
        let node = ChunkNode::new_instance();
        node.map_mut(|_, owner| {
            // Align the chunk node in the world and add it to the parent node.
            let origin = position.origin();
            owner.set_translation(vec3!(origin.x, origin.y, origin.z));
            unsafe {
                self.node
                    .assume_safe()
                    .call_deferred("add_child", &[owner.assume_shared().to_variant()]);
            }
        })
        .expect("couldn't initialize new ChunkNode");
        Chunk::new(data, node.into_shared())
    }
    fn handle_chunk(&self, position: ChunkPos, new_data: Option<ChunkData>) {
        let start_time = std::time::Instant::now();
        let mut chunks = self.chunks_map.lock().unwrap();
        let chunk = {
            if let Some(with_data) = new_data {
                // We have some data, either update an existing chunk
                // with it or use it to create a new one.
                match chunks.remove(&position) {
                    Some(mut chunk) => {
                        // The chunk exists, update its data.
                        chunk.data = with_data;
                        chunk
                    }
                    // The chunk doesn't exist, create it with the new data
                    // and add it to the parent node.
                    None => self.create_chunk(position, with_data),
                }
            } else if let Some(chunk) = chunks.remove(&position) {
                // No data was provided, however the chunk does exist,
                // so we'll update its mesh assuming something like the
                // chunks bordering it changed.
                chunk
            } else {
                // We've been told to update a chunk that doesn't exist
                // and have not been given the data required to make it exist.
                return;
            }
        };
        let mesh_data = {
            let view = create_view(&chunks);
            ChunkMeshData::new_from_chunk_data(&chunk.data, view)
        };
        unsafe { chunk.node.assume_safe() }
            .map_mut(|chunk_node, _owner| {
                chunk_node.update_mesh_data(mesh_data);
            })
            .expect("updating ChunkNode mesh failed");
        chunks.insert(position, chunk);
        let took = start_time.elapsed();
        println!(
            "[{}] Generated mesh in {:.2} ms",
            position,
            took.as_micros() as f64 / 1000.0
        );
    }
    fn recv(&self) {
        let (position, packed_data) = self.receiver.recv().unwrap();
        let new_data = packed_data.map(|packed_data| ChunkData::unpack(position, &packed_data));
        self.handle_chunk(position, new_data)
    }
}

#[methods]
impl ClientChunkLoader {
    fn new(owner: &Node) -> Self {
        let owner = unsafe { owner.assume_shared() };
        let chunks = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver): (
            Sender<LoadThreadChannelContents>,
            Receiver<LoadThreadChannelContents>,
        ) = mpsc::channel();
        let loader_thread = ChunkLoaderThread {
            node: owner.clone(),
            chunks_map: chunks.clone(),
            receiver,
        };
        std::thread::spawn(move || loop {
            loader_thread.recv()
        });
        Self {
            owner,
            chunks,
            send_channel: sender,
        }
    }

    /// Tells the chunk updater thread to update the chunk at `position`
    /// with `new_data_packed`. The chunk's mesh is always updated, and
    /// its terrain/light level data is updated if `new_data_packed` is `Some`.
    ///
    /// Does nothing if the chunk at `position` doesn't exist.
    fn queue_update(&self, position: ChunkPos, new_data_packed: Option<Vec<u8>>) {
        self.send_channel.send((position, new_data_packed)).unwrap();
    }

    /// Runs `queue_update` on `position`, as well as the chunks adjacent
    /// to it.
    fn queue_update_with_nearby(&self, position: ChunkPos, new_data: Option<Vec<u8>>) {
        self.queue_update(position, new_data);
        for nearby_position in position.adjacent() {
            self.queue_update(nearby_position, None);
        }
    }

    #[export]
    fn receive_chunk(
        &mut self,
        _owner: &Node,
        packed_data: ByteArray,
        position: Vector2,
        update_nearby: bool,
    ) {
        let position = ChunkPos::new(position.x as isize, position.y as isize);
        let new_data_packed = Some(packed_data.read().to_vec());
        if update_nearby {
            self.queue_update_with_nearby(position, new_data_packed);
        } else {
            self.queue_update(position, new_data_packed);
        }
    }

    fn _ready(&self, _owner: &Node) {
        godot_print!("ClientChunkLoader ready!");
    }
}

#[derive(NativeClass)]
#[export]
#[inherit(Node)]
pub struct ServerChunkCreator {
    chunks: HashMap<ChunkPos, ChunkData>,
    chunk_generator: ChunkGenerator,
    // #[property]
    // initial_generation_area: Option<Rect2>,
}

#[methods]
impl ServerChunkCreator {
    fn new(_owner: &Node) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_generator: ChunkGenerator::new(),
        }
    }

    /// Gets a block in _global_ space.
    ///
    /// Returns `None` if the block isn't loaded.
    fn get_block(&self, position: GlobalBlockPos) -> Option<BlockID> {
        let chunk_position = position.chunk();
        let chunk_data = self.chunks.get(&chunk_position)?;
        Some(chunk_data.get(position.into()))
    }

    /// Sets a block in _global_ space.
    ///
    /// Returns `NotLoadedError` if the block isn't loaded.
    fn set_block(&mut self, position: GlobalBlockPos, to: BlockID) -> Result<(), NotLoadedError> {
        let local_position: LocalBlockPos = position.into();
        let data = self
            .chunks
            .get_mut(&local_position.chunk)
            .ok_or(NotLoadedError)?;
        data.set(local_position, to);
        Ok(())
    }

    #[export]
    fn set_block_gd(&mut self, _owner: &Node, position: Vector3, to: BlockID) {
        let position = GlobalBlockPos::new(
            position.x as isize,
            position.y as isize,
            position.z as isize,
        );
        self.set_block(position, to);
    }

    #[export]
    fn get_block_gd(&mut self, _owner: &Node, position: Vector3) -> Option<BlockID> {
        let position = GlobalBlockPos::new(
            position.x as isize,
            position.y as isize,
            position.z as isize,
        );
        self.get_block(position)
    }

    /// Returns a "view" into `ServerChunkCreator.chunks`, mapping `ChunkPos`s to `ChunkData`s.
    fn data_view(&mut self) -> HashMap<ChunkPos, &ChunkData> {
        self.chunks.iter().map(|(pos, data)| (*pos, data)).collect()
    }

    /// Loads a chunk from disk, or generates a new one.
    fn load_chunk(&mut self, position: ChunkPos) -> ChunkData {
        let data = if false {
            todo!("Implement loading chunks from disk");
        } else {
            // The chunk is new.
            self.chunk_generator.generate_chunk(position)
        };
        self.chunk_generator.apply_waitlist(&mut self.chunks);
        data
    }

    /// Takes ownership of `chunk` and adds it to the `ServerChunkCreator.chunks` HashMap.
    ///
    /// This allows other chunks to see it when making face calculations,
    /// and for functions such as `ServerChunkCreator.set_block` to be able to modify it.
    fn add_chunk(&mut self, data: ChunkData) {
        self.chunks.insert(data.position, data);
    }

    #[export]
    fn chunk_data_packed(&self, _owner: &Node, position: Vector2) -> Option<Vec<u8>> {
        let position = ChunkPos::new(position.x as isize, position.y as isize);
        self.chunks.get(&position).map(|data| data.pack())
    }

    #[export]
    /// Loads the chunk at `chunk_position`.
    ///
    /// Returns `true` if that chunk is new, otherwise `false`.
    /// Note that "new" here refers to whether or not the server has seen it before
    /// in *this session*, not whether it was loaded from the disk or not.
    fn load_chunk_gd(&mut self, _owner: &Node, chunk_position: Vector2) -> bool {
        let position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        if self.chunks.contains_key(&position) {
            return false;
        }
        let chunk = self.load_chunk(position);
        self.add_chunk(chunk);
        true
    }

    #[export]
    /// Loads a 4x4 square of chunks around `chunk_position`.
    ///
    /// Returns the positions of those chunks.
    fn load_around_chunk_gd(&mut self, owner: &Node, chunk_position: Vector2) -> Vec<Vector2> {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        let mut around = Vec::new();
        for x in -2..=2 {
            for z in -2..=2 {
                let chunk_pos = vec2!(chunk_position.x + x, chunk_position.z + z);
                self.load_chunk_gd(owner, chunk_pos);
                around.push(chunk_pos);
            }
        }
        around
    }

    // TODO: a "update_lightlevel" function, goes through the entire
    // terrain info, lowering the light level depending on it's distance from a light source or sky

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        godot_print!("ServerChunkCreator ready!");
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ChunkNode>();
    handle.add_class::<ServerChunkCreator>();
    handle.add_class::<ClientChunkLoader>();
}

godot_init!(init);
