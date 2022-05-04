use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crate::{chunk_mesh::ChunkMeshData, chunk_node::ChunkNode};
use chunkcommon::{chunk::ChunkData, prelude::*, vec3};
use gdnative::prelude::*;

mod chunk_mesh;
mod chunk_node;
mod mesh;

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
        data: &[u8],
    ) -> Box<[[[BlockID; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X]> {
        // TODO: This got merged into a single function from the original
        //       terrain and light-level specific ones, however it'll actually
        //       have to be split up again or use some generics/closure magic
        //       once light level data gets stored as a u8 instead of a u16.
        let flat: Box<Vec<u16>> = Box::new(
            data.chunks_exact(2)
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
        // NOTE: These data were decompressed via PoolByteArray.decompress() in the networkController.
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

fn init(handle: InitHandle) {
    handle.add_class::<ChunkNode>();
    handle.add_class::<ClientChunkLoader>();
}

godot_init!(init);
