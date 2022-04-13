use gdnative::api::OpenSimplexNoise;

use crate::{constants::*, positions::*};

pub trait TerrainGenerator {
    fn generate(&mut self, noise: &OpenSimplexNoise);
}

pub struct Chunk {
    pub position: ChunkPos,
    pub terrain: [[[BlockID; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z],
}

impl Chunk {
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
}

impl TerrainGenerator for Chunk {
    fn generate(&mut self, noise: &OpenSimplexNoise) {
        println!("Generating terrain data for chunk {:?}", self);
        let chunk_origin = self.position.origin();
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                // TODO: Use a trait method for per-block noise sampling?
                let world_block_x = x as isize + chunk_origin.x;
                let world_block_z = z as isize + chunk_origin.z;
                let noise_height: f64 = noise.get_noise_2dv(gdnative::core_types::Vector2::new(
                    world_block_x as f32,
                    world_block_z as f32,
                ));
                let terrain_peak =
                    ((CHUNK_SIZE_Y as f64) * ((noise_height / 2.0) + 0.5) * 0.1) as isize;
                for y in 0..CHUNK_SIZE_Y {
                    let y = y as isize;
                    if y > terrain_peak {
                        break;
                    }
                    let block_id = if y > 6 { 1 } else { 2 }; // TODO: implement actual block ID system
                    self.terrain[x as usize][y as usize][z as usize] = block_id;
                }
            }
        }
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("position", &self.position)
            .finish()
    }
}
