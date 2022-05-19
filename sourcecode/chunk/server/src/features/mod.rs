//! Provides the `Feature` trait and related structs.
//!
//! Also holds `Feature`s themselves, like `Trees`.

use std::collections::{hash_map::Entry, HashMap};

use chunkcommon::{chunk::ChunkData, errors::OffsetError, prelude::*};

pub mod trees;

/// Struct containing information about blocks `Feature`s
/// *wanted* to generate, but couldn't, because the block
/// positions to be set were in a different chunk.
pub struct FeatureWaitlist {
    pub chunks: HashMap<ChunkPos, Vec<(LocalBlockPos, BlockID)>>,
}

impl FeatureWaitlist {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    /// Merges another `FeatureWaitlist` into this one.
    ///
    /// **NOTE**: There is *no* precedence given to certain block types,
    /// or certain features. This means if a block gets set to air by one
    /// feature, but leaves by another, it's entirely up to fate which gets
    /// generated in the end.
    pub fn merge(&mut self, other: FeatureWaitlist) {
        for (chunk_pos, mut add_blocks) in other.chunks.into_iter() {
            match self.chunks.entry(chunk_pos) {
                Entry::Occupied(mut entry) => {
                    // Append all the new blocks from `other` to this chunk's existing vector.
                    let current_blocks = entry.get_mut();
                    current_blocks.append(&mut add_blocks);
                }
                Entry::Vacant(entry) => {
                    // This `ChunkPos` wasn't in the current feature list, add it
                    // and the blocks to be generated.
                    entry.insert(add_blocks);
                }
            }
        }
    }
}

pub trait Feature {
    /// Fills an array of offsets (relative to `origin`) with `block_id`,
    /// while populating a `FeatureWaitlist` if any of the blocks to set
    /// are outside of `chunk_data.position`.
    fn fill(
        &self,
        chunk_data: &mut ChunkData,
        origin: LocalBlockPos,
        // TODO: Accept Into<BlockOffset> instead of [isize; 3]
        offsets: &[[isize; 3]],
        block_id: BlockID,
    ) -> FeatureWaitlist {
        let mut waitlist = FeatureWaitlist::new();
        for offset in offsets {
            let offset = (*offset).into();
            match origin.offset(offset) {
                Ok(position) => chunk_data.set(position, block_id),
                Err(_) => {
                    // That position is outside of this chunk, add it to the waitlist.
                    // TODO: This should probably be a method of FeatureWaitlist itself.
                    let outside_position: Result<LocalBlockPos, OffsetError> =
                        origin.offset_global(offset).map(|global| global.into());
                    if let Ok(outside_position) = outside_position {
                        let outside_blocks = waitlist
                            .chunks
                            .entry(outside_position.chunk)
                            .or_insert_with(Vec::new);
                        outside_blocks.push((outside_position, block_id));
                    }
                }
            }
        }
        waitlist
    }

    /// Adds this `Feature` to a chunk.
    ///
    /// Returns a `FeatureWaitlist`.
    fn add_to_chunk(&self, chunk_data: &mut ChunkData) -> FeatureWaitlist;
}
