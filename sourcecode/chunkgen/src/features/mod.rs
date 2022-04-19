use std::collections::HashMap;

use crate::{
    block::BlockID,
    chunk::ChunkData,
    positions::{BlockOffset, ChunkPos, LocalBlockPos},
};

pub mod trees;

pub type FeatureWaitlist = HashMap<ChunkPos, Vec<(LocalBlockPos, BlockID)>>;

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
