//! Chunk generation, like features, biomes, etc.

use gdnative::{api::OpenSimplexNoise, core_types::Vector2, object::Ref, prelude::Unique};

use chunkcommon::{block::BLOCK_MANAGER, chunk::ChunkData, prelude::*, vec2};

use crate::features::{trees::Trees, Feature, FeatureWaitlist};

macro_rules! blockid {
    ($name:expr) => {
        BLOCK_MANAGER.block($name).unwrap().id
    };
}

struct GenerationConfig {
    top: BlockID,
    layers: Vec<(isize, BlockID)>,
    filler: BlockID,
    bottom: BlockID,
    features: Vec<Box<dyn Feature>>,
}

pub struct ChunkGenerator {
    noise: Ref<OpenSimplexNoise, Unique>,
    config: GenerationConfig,
    waitlist: FeatureWaitlist,
}

impl ChunkGenerator {
    pub fn new() -> Self {
        let layers = vec![(4, blockid!("dirt")), (8, blockid!("pebbled_dirt"))];
        Self {
            noise: OpenSimplexNoise::new(),
            config: GenerationConfig {
                top: blockid!("grass"),
                layers,
                filler: blockid!("stone"),
                bottom: blockid!("silicate"),
                features: vec![Box::new(Trees::new())],
            },
            waitlist: FeatureWaitlist::new(),
        }
    }
    pub fn generate_block(&self, y: isize, terrain_peak: isize) -> BlockID {
        if y == terrain_peak {
            self.config.top
        } else if y == 0 {
            self.config.bottom
        } else if y > terrain_peak {
            0
        } else {
            let distance_from_peak = terrain_peak - y;
            for (gen_before, block_id) in &self.config.layers {
                if distance_from_peak <= *gen_before {
                    return *block_id;
                }
            }
            // We've run out of layers.
            self.config.filler
        }
    }
    fn get_terrain_peak(&self, x: isize, z: isize) -> isize {
        let noise_height: f64 = self.noise.get_noise_2dv(vec2!(x, z));
        let peak = CHUNK_SIZE_Y as f64 * ((noise_height / 14.0) + 0.1);
        peak as isize
    }
    pub fn add_features(&mut self, chunk_data: &mut ChunkData) {
        for feature in &self.config.features {
            self.waitlist.merge(feature.add_to_chunk(chunk_data));
        }
    }
    pub fn apply_waitlist_to(&mut self, data: &mut ChunkData) {
        if let Some(add_blocks) = self.waitlist.chunks.remove(&data.position) {
            for (pos, block_id) in add_blocks {
                data.set(pos, block_id);
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
