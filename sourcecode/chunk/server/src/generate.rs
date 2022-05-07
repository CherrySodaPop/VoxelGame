//! Chunk generation. Seperate from the `chunk` module as world generation will
//! likely be expanded a lot more (e.g. biomes).

use std::collections::HashMap;

use gdnative::{api::OpenSimplexNoise, core_types::Vector2, object::Ref, prelude::Unique};

use chunkcommon::{block::BLOCK_MANAGER, chunk::ChunkData, prelude::*, vec2};

use crate::features::{trees::Trees, Feature, FeatureWaitlist};

struct GenerationConfig {
    top_layer: BlockID,
    bottom_layer: BlockID,
    features: Vec<Box<dyn Feature>>,
}

pub struct ChunkGenerator {
    noise: Ref<OpenSimplexNoise, Unique>,
    config: GenerationConfig,
    waitlist: FeatureWaitlist,
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
                features: vec![Box::new(Trees::new())],
            },
            waitlist: FeatureWaitlist::new(),
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
    pub fn add_features(&mut self, chunk_data: &mut ChunkData) {
        for feature in &self.config.features {
            self.waitlist.merge(feature.add_to_chunk(chunk_data));
        }
    }
    pub fn apply_waitlist(&mut self, chunks: &mut HashMap<ChunkPos, ChunkData>) {
        for (chunk_pos, chunk_data) in chunks.iter_mut() {
            match self.waitlist.chunks.remove(chunk_pos) {
                Some(add_blocks) => {
                    for (pos, block_id) in add_blocks {
                        chunk_data.set(pos, block_id);
                    }
                }
                None => continue,
            }
        }
    }
    pub fn generate_chunk(&mut self, position: ChunkPos) -> ChunkData {
        let mut data = ChunkData::new(position);

        println!("Generating terrain data for chunk {:?}", position);
        let chunk_origin = position.origin();
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                let global_x = x as isize + chunk_origin.x;
                let global_z = z as isize + chunk_origin.z;
                let terrain_peak = self.get_terrain_peak(global_x, global_z);
                for y in 0..CHUNK_SIZE_Y {
                    data.terrain[[x, y, z]] = self.generate_block(y as isize, terrain_peak);
                }
            }
        }

        println!("Generating light level data for chunk {:?}", position);
        // TODO: "update_lightlevel" in lib.rs
        // TODO: use an unsigned 8 bit int!
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                for y in 0..CHUNK_SIZE_Y {
                    data.skylightlevel[[x, y, z]] = 0;
                }
            }
        }

        self.add_features(&mut data);
        if let Some(add_blocks) = self.waitlist.chunks.remove(&position) {
            for (pos, block_id) in add_blocks {
                data.set(pos, block_id);
            }
        }
        data
    }
}
