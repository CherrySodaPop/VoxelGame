use crate::block::BlockID;

pub const CHUNK_SIZE_X: usize = 32;
pub const CHUNK_SIZE_Y: usize = 512;
pub const CHUNK_SIZE_Z: usize = 32;

pub type TerrainData = [[[BlockID; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X];
// TODO: Use a u8
pub type LightLevelData = [[[u16; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X];
