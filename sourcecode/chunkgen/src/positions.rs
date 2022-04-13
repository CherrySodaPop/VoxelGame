use std::fmt::Debug;

use crate::constants::*;

#[derive(Debug, Clone)]
pub struct TooLargeError;

impl std::fmt::Display for TooLargeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LocalBlockPosition was offset beyond its boundaries")
    }
}
// impl std::error::Error for TooLargeError {}

/// A chunk in the world.
///
/// Keep in mind that the x and z values here do not represent *block* positions.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct ChunkPos {
    pub x: isize,
    pub z: isize,
}

impl ChunkPos {
    // pub const SIZE_X: isize = 32;
    // pub const SIZE_Y: isize = 256;
    // pub const SIZE_Z: isize = 32;

    pub fn new(x: isize, z: isize) -> Self {
        ChunkPos { x, z }
    }
    /// Returns the origin (in global coordinates) of this chunk.
    pub fn origin(&self) -> GlobalBlockPos {
        let x = self.x * CHUNK_SIZE_X as isize;
        let z = self.z * CHUNK_SIZE_Z as isize;
        GlobalBlockPos::new(x, 0, z)
    }
}

/// An offset from some block position. Does not represent anything
/// specifically in local/global space.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct BlockOffset {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl BlockOffset {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }
}

impl From<[isize; 3]> for BlockOffset {
    fn from(offset: [isize; 3]) -> Self {
        Self::new(offset[0], offset[1], offset[2])
    }
}

/// A local block position, i.e. one that is tied to a chunk.
///
/// (all xyz values are within the range 0-31)
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct LocalBlockPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub chunk: ChunkPos,
}

impl LocalBlockPos {
    pub fn new(x: usize, y: usize, z: usize, chunk: ChunkPos) -> Self {
        Self { x, y, z, chunk }
    }
    pub fn offset(&self, offset: BlockOffset) -> Result<Self, TooLargeError> {
        let x = self.x as isize + offset.x;
        let y = self.y as isize + offset.y;
        let z = self.z as isize + offset.z;
        if !(0 < x && x < 32 && 0 < y && y < 256 && 0 < z && z < 32) {
            Err(TooLargeError)
        } else {
            Ok(Self::new(x as usize, y as usize, z as usize, self.chunk))
        }
    }
    pub fn offset_global(&self, offset: BlockOffset) -> GlobalBlockPos {
        let origin = self.chunk.origin();
        let x = origin.x + self.x as isize + offset.x;
        let y = self.y as isize + offset.y;
        let z = origin.z + self.z as isize + offset.z;
        GlobalBlockPos::new(x, y, z)
    }
}

impl From<GlobalBlockPos> for LocalBlockPos {
    fn from(global_position: GlobalBlockPos) -> Self {
        let chunk = global_position.chunk();
        let chunk_origin = chunk.origin();
        let x = (global_position.x - chunk_origin.x).abs() as usize;
        let y = global_position.y.abs() as usize;
        let z = (global_position.z - chunk_origin.z).abs() as usize;
        Self::new(x, y, z, chunk)
    }
}

/// A global block position, i.e. one that is anywhere in the world.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct GlobalBlockPos {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl GlobalBlockPos {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }
    pub fn chunk(&self) -> ChunkPos {
        let xn = if self.x < 0 { 1 } else { 0 };
        let zn = if self.z < 0 { 1 } else { 0 };
        //                                             -------------- Round *downwards* in the negatives, instead of
        //                                             |              towards zero.
        //                                             |
        //                        ----------------------------------- Avoid "flicking" to the next chunk at -32, -64, etc.,
        //                        |                    |              do it at -33, -65, and so on instead.
        //                        |                    |
        //                        |                    |
        let chunk_x: isize = ((xn + self.x) / CHUNK_SIZE_X as isize) - xn;
        let chunk_z: isize = ((zn + self.z) / CHUNK_SIZE_Z as isize) - zn;
        ChunkPos::new(chunk_x, chunk_z)
    }
    pub fn offset(&self, offset: BlockOffset) -> Self {
        Self {
            x: self.x + offset.x,
            y: self.y + offset.y,
            z: self.z + offset.z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! gbp_test {
        (
            $base_global_position:expr, $chunk_position:expr, $local_position:expr
        ) => {
            let gbp = GlobalBlockPos::new(
                $base_global_position[0],
                $base_global_position[1],
                $base_global_position[2],
            );
            let expected_chunk_pos = ChunkPos::new($chunk_position[0], $chunk_position[1]);
            assert_eq!(gbp.chunk(), expected_chunk_pos);
            let expected_local_pos = LocalBlockPos::new(
                $local_position[0],
                $local_position[1],
                $local_position[2],
                expected_chunk_pos,
            );
            assert_eq!(LocalBlockPos::from(gbp), expected_local_pos);
        };
    }

    macro_rules! origin_test {
        ($base_chunk_position:expr, $global_position:expr) => {
            let chunk = ChunkPos::new($base_chunk_position[0], $base_chunk_position[1]);
            assert_eq!(
                chunk.origin(),
                GlobalBlockPos::new(
                    $global_position[0],
                    $global_position[1],
                    $global_position[2]
                )
            );
        };
    }

    #[test]
    fn test_global_block_pos() {
        // Global position (base), chunk position (expected), local position (expected)
        gbp_test!([0, 0, 0], [0, 0], [0, 0, 0]);
        gbp_test!([0, 0, 0], [0, 0], [0, 0, 0]);
        gbp_test!([31, 0, 31], [0, 0], [31, 0, 31]);
        gbp_test!([32, 0, 32], [1, 1], [0, 0, 0]);
        gbp_test!([63, 0, 63], [1, 1], [31, 0, 31]);
        gbp_test!([64, 0, 64], [2, 2], [0, 0, 0]);
        gbp_test!([-1, 0, -1], [-1, -1], [31, 0, 31]);
        gbp_test!([-32, 0, -32], [-1, -1], [0, 0, 0]);
        gbp_test!([-33, 0, -33], [-2, -2], [31, 0, 31]);
        gbp_test!([-64, 0, -100], [-2, -4], [0, 0, 28]);
        gbp_test!([-64, 0, -97], [-2, -4], [0, 0, 31]);
        gbp_test!([-64, 0, -96], [-2, -3], [0, 0, 0]);
        gbp_test!([32, 0, -64], [1, -2], [0, 0, 0]);
        gbp_test!([36, 0, -60], [1, -2], [4, 0, 4]);
        gbp_test!([36, 0, -68], [1, -3], [4, 0, 28]);
    }

    #[test]
    fn test_chunk_origin() {
        // Chunk position (base), origin block position (expected)
        origin_test!([-1, -1], [-32, 0, -32]);
        origin_test!([0, 0], [0, 0, 0]);
        origin_test!([1, 1], [32, 0, 32]);
        origin_test!([2, 2], [64, 0, 64]);
        origin_test!([-2, -2], [-64, 0, -64]);
        origin_test!([-2, -1], [-64, 0, -32]);
        origin_test!([-4, 3], [-128, 0, 96]);
    }

    // TODO: More tests
}
