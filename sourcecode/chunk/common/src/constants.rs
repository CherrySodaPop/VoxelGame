use crate::block::BlockID;

pub const CHUNK_SIZE_X: usize = 32;
pub const CHUNK_SIZE_Y: usize = 512;
pub const CHUNK_SIZE_Z: usize = 32;

pub type TerrainData = ndarray::Array3<BlockID>;
// TODO: Use a u8
pub type LightLevelData = ndarray::Array3<u16>;
