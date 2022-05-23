use std::collections::HashMap;

use crate::generate::ChunkGenerator;
use chunkcommon::{
    chunk::ChunkData,
    chunkmesh::{
        nodes::{ChunkCollisionShape, ChunkNode},
        ChunkMeshData,
    },
    errors::NotLoadedError,
    network::encode_and_compress,
    prelude::*,
    vec2,
};
use gdnative::prelude::*;

mod features;
mod generate;

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

struct ServerChunk {
    data: ChunkData,
    node: ChunkNode,
}

impl ServerChunk {
    fn new(data: ChunkData) -> Self {
        Self {
            data,
            node: ChunkNode::new(None),
        }
    }
}

#[derive(NativeClass)]
#[export]
#[inherit(Spatial)]
pub struct ServerChunkCreator {
    base: Ref<Spatial, Shared>,
    chunks: HashMap<ChunkPos, ServerChunk>,
    chunk_generator: ChunkGenerator,
    // #[property]
    // initial_generation_area: Option<Rect2>,
}

#[methods]
impl ServerChunkCreator {
    fn new(base: &Spatial) -> Self {
        Self {
            base: unsafe { base.assume_shared() },
            chunks: HashMap::new(),
            chunk_generator: ChunkGenerator::new(),
        }
    }

    /// Gets a block in _global_ space.
    ///
    /// Returns `None` if the block isn't loaded.
    fn get_block(&self, position: GlobalBlockPos) -> Option<BlockID> {
        let chunk_position = position.chunk();
        let chunk = self.chunks.get(&chunk_position)?;
        Some(chunk.data.get(position.into()))
    }

    /// Sets a block in _global_ space.
    ///
    /// Returns `NotLoadedError` if the block isn't loaded.
    fn set_block(&mut self, position: GlobalBlockPos, to: BlockID) -> Result<(), NotLoadedError> {
        // TODO: The collision shape needs to be updated here as well.
        let local_position: LocalBlockPos = position.into();
        let chunk = self
            .chunks
            .get_mut(&local_position.chunk)
            .ok_or(NotLoadedError)?;
        // HARDCODED
        // Don't allow for breaking silicate in any way.
        if chunk.data.get(local_position) != 25 {
            chunk.data.set(local_position, to);
        }
        Ok(())
    }

    #[export]
    fn set_block_gd(&mut self, _base: &Spatial, position: Vector3, to: BlockID) {
        let position = GlobalBlockPos::new(
            position.x as isize,
            position.y as isize,
            position.z as isize,
        );
        self.set_block(position, to);
    }

    #[export]
    fn get_block_gd(&mut self, _base: &Spatial, position: Vector3) -> Option<BlockID> {
        let position = GlobalBlockPos::new(
            position.x as isize,
            position.y as isize,
            position.z as isize,
        );
        self.get_block(position)
    }

    /// Returns a "view" into `ServerChunkCreator.chunks`, mapping `ChunkPos`s to `ChunkData`s.
    fn data_view(&mut self) -> HashMap<ChunkPos, &ChunkData> {
        self.chunks
            .iter()
            .map(|(pos, chunk)| (*pos, &chunk.data))
            .collect()
    }

    /// Loads a chunk from disk, or generates a new one.
    fn load_chunk(&mut self, position: ChunkPos) -> ChunkData {
        let data = if false {
            todo!("Implement loading chunks from disk");
        } else {
            // The chunk is new.
            self.chunk_generator.generate_chunk(position)
        };
        for chunk in self.chunks.values_mut() {
            self.chunk_generator.apply_waitlist_to(&mut chunk.data);
        }
        data
    }

    /// Takes ownership of `chunk` and adds it to the `ServerChunkCreator.chunks` HashMap.
    ///
    /// This allows other chunks to see it when making face calculations,
    /// and for functions such as `ServerChunkCreator.set_block` to be able to modify it.
    fn add_chunk(&mut self, data: ChunkData) {
        let position = data.position;
        let mut chunk = ServerChunk::new(data);
        chunk
            .node
            .spawn(&*unsafe { self.base.assume_safe() }, position);
        chunk.node.update(&ChunkMeshData::new_from_chunk_data(
            &chunk.data,
            self.data_view(),
        ));
        self.chunks.insert(position, chunk);
    }

    #[export]
    fn chunk_data_encoded(&self, _base: &Spatial, chunk_position: Vector2) -> Option<ByteArray> {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        println!("Encoding chunk data for {:?}", chunk_position);
        self.chunks
            .get(&chunk_position)
            .map(|chunk| ByteArray::from_vec(encode_and_compress(&chunk.data)))
    }

    #[export]
    /// Loads the chunk at `chunk_position`.
    ///
    /// Returns `true` if that chunk is new, otherwise `false`.
    /// Note that "new" here refers to whether or not the server has seen it before
    /// in *this session*, not whether it was loaded from the disk or not.
    fn load_chunk_gd(&mut self, _base: &Spatial, chunk_position: Vector2) -> bool {
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
    fn load_around_chunk_gd(&mut self, base: &Spatial, chunk_position: Vector2) -> Vec<Vector2> {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        let mut around = Vec::new();
        for x in -2..=2 {
            for z in -2..=2 {
                let chunk_pos = vec2!(chunk_position.x + x, chunk_position.z + z);
                self.load_chunk_gd(base, chunk_pos);
                around.push(chunk_pos);
            }
        }
        around
    }

    // TODO: a "update_lightlevel" function, goes through the entire
    // terrain info, lowering the light level depending on it's distance from a light source or sky

    #[export]
    fn _ready(&mut self, _base: &Spatial) {
        godot_print!("ServerChunkCreator ready!");
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ServerChunkCreator>();
    handle.add_class::<ChunkCollisionShape>();
}

godot_init!(init);
