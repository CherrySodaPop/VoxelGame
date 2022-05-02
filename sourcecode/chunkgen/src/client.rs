use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
};

use gdnative::prelude::*;

use crate::{chunk::ChunkData, chunk_mesh::ChunkMeshData, macros::*, positions::*, ChunkNode};

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

fn create_view<'a>(chunks: &'a HashMap<ChunkPos, Chunk>) -> HashMap<ChunkPos, &'a ChunkData> {
    chunks
        .iter()
        .map(|(pos, chunk)| (*pos, &chunk.data))
        .collect()
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
