use std::collections::HashMap;

use gdnative::api::OpenSimplexNoise;

use crate::{
    chunk::Chunk,
    constants::BlockID,
    positions::{ChunkPos, GlobalBlockPos},
};

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
    pub fn get_block(&self, position: GlobalBlockPos) -> Option<BlockID> {
        if let Some(chunk) = self.chunks.get(&position.chunk()) {
            Some(chunk.get(position.into()))
        } else {
            None
        }
    }
    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position, chunk);
    }
    pub fn generate_rect(&mut self, start: [isize; 2], end: [isize; 2], noise: &OpenSimplexNoise) {
        use crate::chunk::TerrainGenerator;
        for x in start[0]..end[0] {
            for z in start[1]..end[1] {
                let mut chunk = Chunk::new(ChunkPos::new(x, z));
                chunk.generate(noise);
                self.add_chunk(chunk);
            }
        }
    }
    pub fn chunks(&self) -> std::collections::hash_map::Values<'_, ChunkPos, Chunk> {
        self.chunks.values()
    }
    pub fn chunks_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, ChunkPos, Chunk> {
        self.chunks.values_mut()
    }
}

// Necessary for ChunkGenerator deriving Default in lib.rs
impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
