use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use chunkcommon::{
    chunk::ChunkData,
    chunkmesh::{
        nodes::{ChunkCollisionShape, ChunkMeshInstance},
        ChunkMeshData,
    },
    network::decode_compressed,
    prelude::*,
    vec3,
};
use gdnative::{api::StaticBody, prelude::*};

/// This type is **not** thread-safe and needs to be wrapped in a Mutex to be shared
/// between threads.
struct ClientChunkNode {
    body: Ref<StaticBody, Shared>,
    collision: Instance<ChunkCollisionShape, Shared>,
    mesh: Instance<ChunkMeshInstance, Shared>,
}

impl ClientChunkNode {
    fn new() -> Self {
        let body = StaticBody::new();
        let collision = ChunkCollisionShape::new_instance().into_shared();
        let mesh = ChunkMeshInstance::new_instance().into_shared();
        body.add_child(&collision, true);
        body.add_child(&mesh, true);
        Self {
            body: body.into_shared(),
            collision,
            mesh,
        }
    }

    fn spawn(&mut self, parent: &Node, position: ChunkPos) {
        let origin = position.origin();
        unsafe { self.body.assume_safe() }.set_translation(vec3!(origin.x, origin.y, origin.z));
        parent.add_child(self.body, true);
    }

    /// Update the mesh and collision nodes with `mesh_data`.
    ///
    /// This can only be called from a thread with **exclusive access** to this `ClientChunkNode`.
    fn update(&mut self, mesh_data: &ChunkMeshData) {
        unsafe { self.collision.assume_safe() }
            .map_mut(|collision, _base| collision.update_shape(mesh_data))
            .unwrap();
        unsafe { self.mesh.assume_safe() }
            .map_mut(|mesh, _base| mesh.update_mesh(mesh_data))
            .unwrap();
    }
}

pub struct Chunk {
    /// The chunk's position. This is determined by `Chunk.data.position` and
    /// is copied here for convenience, to prevent requiring locking to access
    /// something so simple.
    pub position: ChunkPos,
    /// The chunk's terrain data.
    pub data: Arc<RwLock<ChunkData>>,
    /// The chunk node, its manifestation in the Godot world.
    node: Arc<Mutex<ClientChunkNode>>,
}

impl Chunk {
    fn new(data: ChunkData, node: ClientChunkNode) -> Self {
        Self {
            position: data.position,
            data: Arc::new(RwLock::new(data)),
            node: Arc::new(Mutex::new(node)),
        }
    }
}

#[derive(NativeClass)]
#[export]
#[inherit(Node)]
pub struct ClientChunkLoader {
    base: Ref<Node, Shared>,
    chunks: HashMap<ChunkPos, Chunk>,
}

#[methods]
impl ClientChunkLoader {
    fn new(owner: &Node) -> Self {
        Self {
            base: unsafe { owner.assume_shared() },
            chunks: HashMap::new(),
        }
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
            let mesh_data = ChunkMeshData::new_from_chunk_data_threaded(chunk_data, view);
            let mut chunk_node = chunk_node.lock().unwrap();
            chunk_node.update(&mesh_data);
        });
    }

    fn update_nearby_meshes(&self, position: ChunkPos) {
        let nearby = position.adjacent();
        for nearby_position in nearby {
            if let Some(chunk) = self.chunks.get(&nearby_position) {
                self.update_mesh(chunk);
            }
        }
    }

    fn spawn_chunk(&mut self, data: ChunkData) {
        println!("Spawning chunk {:?}", data.position);
        let mut node = ClientChunkNode::new();
        node.spawn(&*unsafe { self.base.assume_safe() }, data.position);
        let chunk = Chunk::new(data, node);
        self.update_mesh(&chunk);
        self.chunks.insert(chunk.position, chunk);
    }

    #[export]
    fn receive_chunk(&mut self, _base: &Node, data: ByteArray, position: Vector2) {
        let position = ChunkPos::new(position.x as isize, position.y as isize);
        let data = data.read();
        let received_chunk_data: ChunkData = decode_compressed(&*data);
        if let Some(loaded_chunk) = self.chunks.get(&position) {
            let mut chunk_data_write = loaded_chunk.data.write().unwrap();
            // This should just replace `loaded_chunk.data`, but it can't
            // without violating borrowing rules at the moment.
            chunk_data_write.terrain = received_chunk_data.terrain;
            chunk_data_write.skylightlevel = received_chunk_data.skylightlevel;
            self.update_mesh(loaded_chunk);
        } else {
            self.spawn_chunk(received_chunk_data);
        }
    }

    fn _ready(&self, _owner: &Node) {
        godot_print!("ClientChunkLoader ready!");
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ChunkCollisionShape>();
    handle.add_class::<ChunkMeshInstance>();
    handle.add_class::<ClientChunkLoader>();
}

godot_init!(init);
