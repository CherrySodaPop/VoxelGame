use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;

const BLOCKS_JSON: &str = include_str!("../../VoxelGame/assets/blocks.json"); // HARDCODED
pub type BlockID = u16;

#[derive(Debug, Deserialize)]
pub struct Block {
    pub id: BlockID,
    pub name: String,
    // FUTURE: This will house fields such as toughness,
    //         tool preference, etc.
}

/// Loads and provides access to block data from the `blocks.json` file.
pub struct BlockManager {
    blocks: HashMap<String, Block>,
}

impl BlockManager {
    fn new() -> Self {
        // TODO: Learn serde.
        let mut game_dict: HashMap<String, serde_json::Value> =
            serde_json::from_str(BLOCKS_JSON).unwrap();
        let blocks: HashMap<String, Block> =
            serde_json::from_value(game_dict.remove("blocks").unwrap()).unwrap();
        println!("{:?}", blocks);
        Self { blocks }
    }

    pub fn block(&self, block_name: &str) -> Option<&Block> {
        self.blocks.get(block_name)
    }
}

lazy_static! {
    pub static ref BLOCK_MANAGER: BlockManager = BlockManager::new();
}
