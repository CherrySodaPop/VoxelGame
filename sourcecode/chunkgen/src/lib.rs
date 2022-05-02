#![allow(dead_code)]
// ^ Prevents "unused function" warnings, those functions will
// likely either be used in the future or by GDScript.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use constants::{CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};
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
    /// is copied here for convenience, to prevent requiring locking to access
    /// something so simple.
    pub position: ChunkPos,
    /// The chunk's terrain data.
    pub data: Arc<RwLock<ChunkData>>,
    /// The chunk node, its manifestation in the Godot world.
    node: Arc<Mutex<Instance<ChunkNode, Shared>>>,
}

impl Chunk {
    fn new(data: ChunkData, node: Instance<ChunkNode, Shared>) -> Self {
        Self {
            position: data.position,
            data: Arc::new(RwLock::new(data)),
            node: Arc::new(Mutex::new(node)),
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

#[derive(NativeClass)]
#[export]
#[inherit(Node)]
pub struct ClientChunkLoader {
    owner: Ref<Node, Shared>,
    chunks: HashMap<ChunkPos, Chunk>,
}

#[methods]
impl ClientChunkLoader {
    fn new(owner: &Node) -> Self {
        Self {
            owner: unsafe { owner.assume_shared() },
            chunks: HashMap::new(),
        }
    }

    fn get(&self, chunk_position: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&chunk_position)
    }

    /// Returns a "view" into `ClientChunkLoader.chunks`,
    /// mapping `ChunkPos`s to `RwLock`ed `ChunkData`s.
    fn data_view(&self) -> HashMap<ChunkPos, Arc<RwLock<ChunkData>>> {
        self.chunks
            .iter()
            .map(|(pos, chunk)| (*pos, chunk.data.clone()))
            .collect()
    }

    fn update_mesh(&self, chunk: &Chunk) {
        println!("Updating mesh data for {:?}", chunk.position);
        let view = self.data_view();
        let chunk_node = chunk.node.clone();
        let chunk_data = chunk.data.clone();
        // TODO: Somehow use a single thread that gets sent chunks to build meshes for,
        //       rather than creating a new one for each individually.
        std::thread::spawn(move || {
            let mesh_data = ChunkMeshData::new_from_chunk_data(chunk_data, view);
            let chunk_node = chunk_node.lock().unwrap();
            unsafe { chunk_node.assume_safe() }
                .map_mut(|chunk_node, _owner| {
                    chunk_node.update_mesh_data(mesh_data);
                })
                .expect("updating ChunkNode mesh failed");
        });
    }

    fn update_nearby_meshes(&self, position: ChunkPos) {
        let nearby = position.adjacent();
        for nearby_position in nearby {
            if let Some(chunk) = self.get(nearby_position) {
                self.update_mesh(chunk);
            }
        }
    }

    fn spawn_chunk(&mut self, data: ChunkData) {
        println!("Spawning chunk {:?}", data.position);
        let node = ChunkNode::new_instance();
        let origin = data.position.origin();
        node.map_mut(|_chunk_node, owner| {
            owner.set_translation(vec3!(origin.x, origin.y, origin.z));
        })
        .expect("setting ChunkNode translation failed");
        let node = node.into_shared();
        unsafe { self.owner.assume_safe() }.add_child(&node, true);
        let chunk = Chunk::new(data, node);
        self.update_mesh(&chunk);
        self.chunks.insert(chunk.position, chunk);
    }

    fn decode_u8_chunk_data(
        &self,
        compressed: &[u8],
    ) -> Box<[[[BlockID; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]> {
        let mut decompressed = Vec::new();
        lzzzz::lz4f::decompress_to_vec(compressed, &mut decompressed).unwrap();

        // TODO: This got merged into a single function from the original
        //       terrain and light-level specific ones, however it'll actually
        //       have to be split up again or use some generics/closure magic
        //       once light level data gets stored as a u8 instead of a u16.
        let flat: Box<Vec<u16>> = Box::new(
            decompressed
                .chunks_exact(2)
                // TODO: We could maybe use u16::from_le_bytes here
                .map(|bytes| ((bytes[0] as u16) << 8) + (bytes[1] as u16))
                .collect(),
        );
        let mut packed = Box::new([[[0; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]);
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let idx = z + CHUNK_SIZE_X * (y + CHUNK_SIZE_Y * x);
                    packed[x][y][z] = flat[idx];
                }
            }
        }
        packed
    }

    #[export]
    fn receive_chunk(
        &mut self,
        _owner: &Node,
        terrain_data: ByteArray,
        skylightlevel_data: ByteArray,
        position: Vector2,
    ) {
        let position = ChunkPos::new(position.x as isize, position.y as isize);
        let terrain_data_read = terrain_data.read();
        let skylightlevel_data_read = skylightlevel_data.read();
        let terrain = self.decode_u8_chunk_data(&*terrain_data_read);
        let skylightlevel = self.decode_u8_chunk_data(&*skylightlevel_data_read);
        // TODO: That was a lot of repetition, which could be fixed in many interesting ways...
        //       Generic functions, closures, or perhaps a unified BlockInfo type...
        if let Some(loaded_chunk) = self.chunks.get(&position) {
            let mut chunk_data_write = loaded_chunk.data.write().unwrap();
            chunk_data_write.terrain = terrain;
            chunk_data_write.skylightlevel = skylightlevel;
            self.update_mesh(loaded_chunk);
        } else {
            let data = ChunkData {
                position,
                terrain,
                skylightlevel,
            };
            self.spawn_chunk(data);
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

    fn encode_terrain_data(&self, data: &ChunkData) -> Vec<u8> {
        data.terrain
            .iter()
            .flatten()
            .flatten()
            .flat_map(|block_id| [(block_id >> 8) as u8, (block_id & 0xff) as u8])
            .collect()
    }

    fn encode_skylightlevel_data(&self, data: &ChunkData) -> Vec<u8> {
        data.skylightlevel
            .iter()
            .flatten()
            .flatten()
            .flat_map(|level| [(level >> 8) as u8, (level & 0xff) as u8])
            .collect()
    }

    fn compress_to_bytearray(source: &[u8]) -> ByteArray {
        let compression_prefs = lzzzz::lz4f::Preferences::default();
        let mut compressed_buffer =
            vec![0; lzzzz::lz4f::max_compressed_size(source.len(), &compression_prefs)];

        let compressed_size =
            lzzzz::lz4f::compress(source, &mut compressed_buffer, &compression_prefs).unwrap();
        let compressed: Vec<u8> = compressed_buffer[..compressed_size].into();

        ByteArray::from_vec(compressed)
    }

    #[export]
    fn terrain_encoded(&self, _owner: &Node, chunk_position: Vector2) -> Option<ByteArray> {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        println!("Encoding terrain data for {:?}", chunk_position);
        let flat_terrain = self.encode_terrain_data(self.chunks.get(&chunk_position)?);
        Some(Self::compress_to_bytearray(&flat_terrain))
    }

    #[export]
    fn skylightlevel_encoded(&self, _owner: &Node, chunk_position: Vector2) -> Option<ByteArray> {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        println!("Encoding skylightlevel data for {:?}", chunk_position);
        let flat_skylightlevel = self.encode_skylightlevel_data(self.chunks.get(&chunk_position)?);
        Some(Self::compress_to_bytearray(&flat_skylightlevel))
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
