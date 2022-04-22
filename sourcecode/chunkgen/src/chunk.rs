use crate::block::BlockID;
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
        position
            .offset(face.normal.into())
            .map(|check_position| self.get(check_position) == 0)
    }
}

impl std::fmt::Debug for ChunkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("position", &self.position)
            .finish()
    }
}
