use crate::{block::BlockID, constants::*, positions::*};

pub struct ChunkData {
    pub position: ChunkPos,
    // These fields are Box-ed to prevent the stack from overflowing.
    // We're storing a whole lot of data!
    pub terrain: Box<TerrainData>,
    pub skylightlevel: Box<LightLevelData>,
}

fn compress(source: &[u8]) -> Vec<u8> {
    let compression_prefs = lzzzz::lz4f::Preferences::default();
    let mut compressed_buffer =
        vec![0; lzzzz::lz4f::max_compressed_size(source.len(), &compression_prefs)];

    let compressed_size =
        lzzzz::lz4f::compress(source, &mut compressed_buffer, &compression_prefs).unwrap();
    let compressed: Vec<u8> = compressed_buffer[..compressed_size].into();

    compressed
}

impl ChunkData {
    pub fn new(position: ChunkPos) -> Self {
        Self {
            position,
            terrain: Box::new([[[0; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z]),
            skylightlevel: Box::new([[[0; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z]),
        }
    }
    pub fn get(&self, position: LocalBlockPos) -> BlockID {
        self.terrain[position.x][position.y][position.z]
    }
    pub fn set(&mut self, position: LocalBlockPos, to: BlockID) {
        self.terrain[position.x][position.y][position.z] = to;
    }

    /// Gets the y-level of the first air block at `x` and `z` (local-space).
    ///
    /// Returns `None` if there's no air blocks at any y-level.
    pub fn get_air_start(&self, x: usize, z: usize) -> Option<usize> {
        for y in 0..CHUNK_SIZE_Y {
            let position = LocalBlockPos::new(x, y, z, self.position);
            if self.get(position) == 0 {
                return Some(y);
            }
        }
        None
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut combined = Vec::new();
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let block = self.terrain[x][y][z];
                    let skylightlevel = self.skylightlevel[x][y][z];
                    combined.push(((block as u32) << 16) + (skylightlevel as u32));
                }
            }
        }
        let split: Vec<u8> = combined
            .into_iter()
            .flat_map(|num| num.to_le_bytes())
            .collect();
        compress(&split)
    }

    pub fn unpack(position: ChunkPos, packed: &[u8]) -> Self {
        let mut split = Vec::new();
        lzzzz::lz4f::decompress_to_vec(packed, &mut split).unwrap();

        let combined: Vec<u32> = split
            .chunks_exact(4)
            .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            .collect();

        let mut terrain = Box::new([[[0; CHUNK_SIZE_X]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z]);
        let mut skylightlevel = Box::new([[[0; CHUNK_SIZE_X]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z]);
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let idx = z + CHUNK_SIZE_X * (y + CHUNK_SIZE_Y * x);
                    skylightlevel[x][y][z] = (combined[idx] & 0xffffff) as u16;
                    terrain[x][y][z] = (combined[idx] >> 16) as u16;
                }
            }
        }
        let mut data = ChunkData::new(position);
        data.terrain = terrain;
        data.skylightlevel = skylightlevel;
        data
    }
}

impl std::fmt::Debug for ChunkData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("position", &self.position)
            .finish()
    }
}
