//! Chunk generation. Seperate from the `chunk` module as world generation will
//! likely be expanded a lot more (e.g. biomes).

use gdnative::{api::OpenSimplexNoise, core_types::Vector2, object::Ref, prelude::Unique};

use crate::block::BlockID;
use crate::block::BLOCK_MANAGER;
use crate::chunk::ChunkData;
use crate::constants::*;
use crate::macros::*;
use crate::positions::ChunkPos;

struct GenerationConfig {
    top_layer: BlockID,
    bottom_layer: BlockID,
}

pub struct ChunkGenerator {
    noise: Ref<OpenSimplexNoise, Unique>,
    config: GenerationConfig,
}

impl ChunkGenerator {
    pub fn new() -> Self {
        let top_layer = BLOCK_MANAGER.block("grass").unwrap().id;
        let bottom_layer = BLOCK_MANAGER.block("dirt").unwrap().id;
        Self {
            noise: OpenSimplexNoise::new(),
            config: GenerationConfig {
                top_layer,
                bottom_layer,
            },
        }
    }
    pub fn generate_block(&self, y: isize, terrain_peak: isize) -> BlockID {
        if y > terrain_peak {
            return 0;
        }
        let block_id = if y > 6 {
            self.config.top_layer
        } else {
            self.config.bottom_layer
        };
        block_id
    }
    fn get_terrain_peak(&self, x: isize, z: isize) -> isize {
        let noise_height: f64 = self.noise.get_noise_2dv(vec2!(x, z));
        ((CHUNK_SIZE_Y as f64) * ((noise_height / 2.0) + 0.5) * 0.1) as isize
    }
    pub fn generate_chunk(&self, position: ChunkPos) -> ChunkData {
        println!("Generating terrain data for chunk {:?}", position);
        let mut terrain = [[[0u16; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X];
        let chunk_origin = position.origin();
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                let global_x = x as isize + chunk_origin.x;
                let global_z = z as isize + chunk_origin.z;
                let terrain_peak = self.get_terrain_peak(global_x, global_z);
                for y in 0..CHUNK_SIZE_Y {
                    terrain[x][y][z] = self.generate_block(y as isize, terrain_peak);
                }
            }
        }
        ChunkData { position, terrain }
    }
}
