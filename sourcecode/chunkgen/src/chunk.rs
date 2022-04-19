use crate::block::{BlockID, BLOCK_MANAGER};
use crate::{constants::*, mesh::Face, positions::*};

pub struct ChunkData {
    pub position: ChunkPos,
    pub terrain: [[[BlockID; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z],
}

impl ChunkData {
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            terrain: [[[0; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z],
        }
    }
    pub fn get(&self, position: LocalBlockPos) -> BlockID {
        self.terrain[position.x][position.y][position.z]
    }
    pub fn set(&mut self, position: LocalBlockPos, to: BlockID) {
        self.terrain[position.x][position.y][position.z] = to;
    }

    /// Returns `Ok(true)` if `face` is visible (e.g. is not blocked by a
    /// solid block) at `position`.
    ///
    /// This checks in **local space**, and will return `TooLargeError` if
    /// the block to check for transparency is outside the range of this chunk.
    pub fn is_face_visible(
        &self,
        position: LocalBlockPos,
        face: &Face,
    ) -> Result<bool, TooLargeError> {
        position.offset(face.normal.into()).map(|check_position| {
            BLOCK_MANAGER
                .transparent_blocks
                .contains(&self.get(check_position))
        })
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
