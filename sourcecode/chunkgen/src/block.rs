use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;

const BLOCKS_JSON: &str = include_str!("../../VoxelGame/assets/blocks.json"); // HARDCODED
pub type BlockID = u16;

#[derive(Debug, Deserialize)]
pub struct Block {
    pub id: BlockID,
    pub name: String,
    #[serde(default)]
    pub transparent: bool,
    pub durability: f64,
    // FUTURE: This will house fields such as toughness,
    //         tool preference, etc.
}

/// Loads and provides access to block data from the `blocks.json` file.
pub struct BlockManager {
    blocks: HashMap<String, Block>,
    // TODO: `transparent_blocks` shouldn't be necessary. `ChunkData.terrain` should
    //       be updated to store a type like `Block`, however as it stands that's
    //       going to waste a lot of memory. We need to find a way to make something
    //       like consts/enum variants for each block type.
    pub transparent_blocks: Vec<BlockID>,
}

impl BlockManager {
    fn new() -> Self {
        // TODO: Learn serde.
        let mut game_dict: HashMap<String, serde_json::Value> =
            serde_json::from_str(BLOCKS_JSON).unwrap();
        let blocks: HashMap<String, Block> =
            serde_json::from_value(game_dict.remove("blocks").unwrap()).unwrap();
        println!("{:?}", blocks);
        let transparent_blocks = blocks
            .iter()
            // TODO: Replace with `.then_some(block.id)` if `bool_to_option` gets stabilized
            //       (https://github.com/rust-lang/rust/issues/80967)
            .filter_map(|(_, block)| block.transparent.then(|| block.id))
            .collect();
        Self {
            blocks,
            transparent_blocks,
        }
    }

    pub fn block(&self, block_name: &str) -> Option<&Block> {
        self.blocks.get(block_name)
    }
}

lazy_static! {
    pub static ref BLOCK_MANAGER: BlockManager = BlockManager::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_manager() {
        let block = BLOCK_MANAGER.block("air").unwrap();
        assert_eq!(block.id, 0);
        assert_eq!(block.transparent, true);
        let block = BLOCK_MANAGER.block("dirt").unwrap();
        assert_eq!(block.id, 21);
        assert_eq!(block.transparent, false);

        assert!(BLOCK_MANAGER.transparent_blocks.contains(&24));
        assert!(BLOCK_MANAGER.transparent_blocks.contains(&0));
        assert!(!BLOCK_MANAGER.transparent_blocks.contains(&21));
    }
}
