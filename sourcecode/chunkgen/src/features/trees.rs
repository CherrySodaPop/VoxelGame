//! Trees!

use gdnative::{api::OpenSimplexNoise, prelude::Unique};

use crate::{
    chunk::ChunkData,
    constants::{CHUNK_SIZE_X, CHUNK_SIZE_Z},
    positions::{GlobalBlockPos, LocalBlockPos},
};

use super::{Feature, FeatureWaitlist};

pub struct Trees {
    noise: gdnative::object::Ref<OpenSimplexNoise, Unique>,
}

impl Trees {
    pub fn new() -> Self {
        let noise = OpenSimplexNoise::new();
        // TODO: Real world seeds. (This is only here to prevent the noise from using
        //       the same seed as the terrain generation, which would look weird)
        noise.set_seed(20);
        // TODO: Make all of these controllable by a single "rarity" parameter.
        noise.set_octaves(5);
        noise.set_period(2.0);
        noise.set_lacunarity(2.0);
        noise.set_persistence(1.0);
        Self { noise }
    }
}

impl Trees {
    // TODO: Store all of this somewhere else!
    const LEAVES: [[isize; 3]; 130] = [
        [-2, 3, -1],
        [-1, 3, 4],
        [1, 3, -4],
        [1, 3, 3],
        [-2, 4, -1],
        [-2, 4, 0],
        [-2, 4, 2],
        [-1, 4, -1],
        [-1, 4, 0],
        [-1, 4, 1],
        [-1, 4, 2],
        [-1, 4, 3],
        [-1, 4, 4],
        [0, 4, -3],
        [0, 4, -2],
        [0, 4, 2],
        [0, 4, 3],
        [0, 4, 4],
        [1, 4, -4],
        [1, 4, -3],
        [1, 4, -1],
        [1, 4, 3],
        [1, 4, 4],
        [2, 4, -1],
        [2, 4, 0],
        [2, 4, 1],
        [2, 4, 2],
        [4, 4, 0],
        [-3, 5, 1],
        [-2, 5, -2],
        [-2, 5, -1],
        [-2, 5, 0],
        [-2, 5, 1],
        [-2, 5, 2],
        [-2, 5, 3],
        [-1, 5, -3],
        [-1, 5, -2],
        [-1, 5, -1],
        [-1, 5, 0],
        [-1, 5, 1],
        [-1, 5, 2],
        [-1, 5, 3],
        [-1, 5, 4],
        [0, 5, -4],
        [0, 5, -3],
        [0, 5, -2],
        [0, 5, -1],
        [0, 5, 2],
        [0, 5, 3],
        [0, 5, 4],
        [1, 5, -3],
        [1, 5, -2],
        [1, 5, -1],
        [1, 5, 2],
        [1, 5, 3],
        [1, 5, 4],
        [2, 5, -3],
        [2, 5, -2],
        [2, 5, -1],
        [2, 5, 0],
        [2, 5, 1],
        [2, 5, 2],
        [2, 5, 3],
        [2, 5, 4],
        [3, 5, -2],
        [3, 5, -1],
        [3, 5, 0],
        [3, 5, 1],
        [3, 5, 2],
        [3, 5, 3],
        [4, 5, 0],
        [4, 5, 1],
        [-3, 6, -1],
        [-2, 6, -1],
        [-2, 6, 0],
        [-2, 6, 1],
        [-2, 6, 2],
        [-1, 6, -1],
        [-1, 6, 0],
        [-1, 6, 1],
        [-1, 6, 2],
        [-1, 6, 3],
        [0, 6, -2],
        [0, 6, -1],
        [0, 6, 0],
        [0, 6, 1],
        [0, 6, 2],
        [0, 6, 3],
        [1, 6, -2],
        [1, 6, -1],
        [1, 6, 0],
        [1, 6, 1],
        [1, 6, 2],
        [2, 6, -1],
        [2, 6, 0],
        [2, 6, 1],
        [2, 6, 2],
        [3, 6, -1],
        [3, 6, 0],
        [3, 6, 1],
        [4, 6, -1],
        [-2, 7, 0],
        [-2, 7, 1],
        [-1, 7, -1],
        [-1, 7, 0],
        [-1, 7, 1],
        [0, 7, -2],
        [0, 7, -1],
        [0, 7, 0],
        [0, 7, 1],
        [0, 7, 2],
        [1, 7, -2],
        [1, 7, -1],
        [1, 7, 0],
        [1, 7, 1],
        [1, 7, 2],
        [1, 7, 3],
        [2, 7, -1],
        [2, 7, 0],
        [2, 7, 1],
        [3, 7, 0],
        [3, 7, 1],
        [-1, 8, 0],
        [-1, 8, 1],
        [0, 8, 0],
        [0, 8, 1],
        [1, 8, 0],
        [1, 8, 1],
        [2, 8, 0],
        [2, 8, 1],
    ];
    const TRUNK: [[isize; 3]; 19] = [
        [0, 0, 0],
        [0, 0, 1],
        [1, 0, 0],
        [0, 1, 1],
        [1, 1, 0],
        [0, 2, 0],
        [0, 2, 1],
        [1, 2, 1],
        [0, 3, 0],
        [0, 3, 1],
        [1, 3, 0],
        [0, 4, 0],
        [0, 4, 1],
        [1, 4, 0],
        [1, 4, 1],
        [0, 5, 0],
        [0, 5, 1],
        [1, 5, 0],
        [1, 5, 1],
    ];
}

impl Feature for Trees {
    fn add_to_chunk(&self, chunk_data: &mut ChunkData) -> FeatureWaitlist {
        let mut waitlist = FeatureWaitlist::new();
        let mut tree_positions = Vec::new();
        // Pick some random positions within this chunk to be the origins
        // of trees.
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                // `global_pos` is only used to get a value from the noise map.
                let global_pos: GlobalBlockPos =
                    LocalBlockPos::new(x, 0, z, chunk_data.position).into();
                if self
                    .noise
                    .get_noise_2d(global_pos.x as f64, global_pos.z as f64)
                    > 0.46
                {
                    tree_positions.push((x, z));
                }
            }
        }

        for (x, z) in tree_positions {
            // Spawn trees where the ground turns to air.
            let air_start = chunk_data.get_air_start(x, z);
            match air_start {
                Some(air_start) => {
                    let origin = LocalBlockPos::new(x, air_start, z, chunk_data.position);
                    waitlist.merge(self.fill(chunk_data, origin, &Self::LEAVES, 24));
                    waitlist.merge(self.fill(chunk_data, origin, &Self::TRUNK, 23));
                }
                None => {}
            };
        }

        waitlist
    }
}
