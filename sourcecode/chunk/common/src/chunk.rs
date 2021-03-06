use crate::{block::BlockID, constants::*, positions::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub position: ChunkPos,
    // These fields are Box-ed to prevent the stack from overflowing.
    // We're storing a whole lot of data!
    pub terrain: Box<TerrainData>,
    pub skylightlevel: Box<LightLevelData>,
}

impl ChunkData {
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            terrain: Box::new(TerrainData::from_elem(
                [CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z],
                0,
            )),
            skylightlevel: Box::new(LightLevelData::from_elem(
                [CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z],
                0,
            )),
        }
    }
    pub fn get(&self, position: LocalBlockPos) -> BlockID {
        self.terrain[[position.x, position.y, position.z]]
    }
    pub fn set(&mut self, position: LocalBlockPos, to: BlockID) {
        self.terrain[[position.x, position.y, position.z]] = to;
    }

    /// Gets the y-level of the first air block at `x` and `z` (local-space).
    ///
    /// Returns `None` if there's no air blocks at any y-level.
    pub fn get_air_start(&self, x: usize, z: usize) -> Option<usize> {
        for y in 0..CHUNK_SIZE_Y {
            let position = LocalBlockPos::new(x, y, z, self.position);
            if self.get(position) == 0 {
                return Some(y);
            }
        }
        None
    }
}

impl std::fmt::Debug for ChunkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("position", &self.position)
            .finish()
    }
}
