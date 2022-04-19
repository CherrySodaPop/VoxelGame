use std::{
    borrow::BorrowMut,
    collections::{hash_map::Entry, HashMap},
};

use crate::{
    block::BlockID,
    chunk::ChunkData,
    positions::{BlockOffset, ChunkPos, LocalBlockPos},
};

pub mod trees;

pub struct FeatureWaitlist {
    pub chunks: HashMap<ChunkPos, Vec<(LocalBlockPos, BlockID)>>,
}

impl FeatureWaitlist {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn merge(&mut self, other: FeatureWaitlist) {
        for (chunk_pos, mut add_blocks) in other.chunks.into_iter() {
            match self.chunks.entry(chunk_pos) {
                Entry::Occupied(mut entry) => {
                    let current_blocks = entry.get_mut();
                    current_blocks.append(&mut add_blocks);
                }
                Entry::Vacant(entry) => {
                    entry.insert(add_blocks);
                }
            }
        }
    }
}

pub trait Feature {
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
                    let outside_position: LocalBlockPos = origin.offset_global(offset).into();
                    let outside_blocks = waitlist
                        .chunks
                        .entry(outside_position.chunk)
                        .or_insert_with(Vec::new);
                    outside_blocks.push((outside_position, block_id));
                }
            }
        }
        waitlist
    }
    fn add_to_chunk(&self, chunk_data: &mut ChunkData) -> FeatureWaitlist;
}
