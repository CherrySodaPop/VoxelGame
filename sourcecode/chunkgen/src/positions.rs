use std::fmt::{Debug, Display};

use crate::constants::*;

#[derive(Debug, Clone)]
pub struct OutOfBoundsError;

impl std::fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "<Local/Global>BlockPosition was offset beyond its boundaries"
        )
    }
}
impl std::error::Error for OutOfBoundsError {}

/// A chunk in the world.
///
/// Keep in mind that the x and z values here do not represent *block* positions.
/// Use `ChunkPos.origin` if you want the position of the chunk's origin block.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct ChunkPos {
    pub x: isize,
    pub z: isize,
}

impl ChunkPos {
    pub fn new(x: isize, z: isize) -> Self {
        ChunkPos { x, z }
    }
    /// Returns the origin (in global coordinates) of this chunk.
    pub fn origin(&self) -> GlobalBlockPos {
        let x = self.x * CHUNK_SIZE_X as isize;
        let z = self.z * CHUNK_SIZE_Z as isize;
        GlobalBlockPos::new(x, 0, z)
    }
    /// Returns `ChunkPos` that are adjacent to this one.
    ///
    /// This does not include diagonals.
    pub fn adjacent(&self) -> [ChunkPos; 4] {
        [
            ChunkPos::new(self.x + 1, self.z),
            ChunkPos::new(self.x - 1, self.z),
            ChunkPos::new(self.x, self.z + 1),
            ChunkPos::new(self.x, self.z - 1),
        ]
    }
}

impl Display for ChunkPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Chunk ({}, {})", self.x, self.z)
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

// TODO: Find a way to make this work.
// impl<T: Borrow<[isize; 3]>> From<T> for BlockOffset {
//     fn from(offset: T) -> Self {
//         let offset = offset.borrow();
//         Self::new(offset[0], offset[1], offset[2])
//     }
// }

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

/// Checks to see if `<param 1>` is within the range `(0..<param 2>)`.
macro_rules! in_urange {
    ($val:expr, $range_end_exclusive:expr) => {
        (0..$range_end_exclusive).contains($val)
    };
}

impl LocalBlockPos {
    pub fn new(x: usize, y: usize, z: usize, chunk: ChunkPos) -> Self {
        Self { x, y, z, chunk }
    }
    /// Offsets this block position by `offset`.
    ///
    /// Returns `TooLargeError` if the resulting block position would
    /// end up in another chunk. (See `LocalBlockPos.offset_global`)
    pub fn offset(&self, offset: BlockOffset) -> Result<Self, OutOfBoundsError> {
        let x = self.x as isize + offset.x;
        let y = self.y as isize + offset.y;
        let z = self.z as isize + offset.z;
        if in_urange!(&x, CHUNK_SIZE_X as isize)
            && in_urange!(&y, CHUNK_SIZE_Y as isize)
            && in_urange!(&z, CHUNK_SIZE_Z as isize)
        {
            Ok(Self::new(x as usize, y as usize, z as usize, self.chunk))
        } else {
            Err(OutOfBoundsError)
        }
    }
    /// Offsets this block position by `offset`, returning a `GlobalBlockPos`.
    ///
    /// To offset in local space, see `LocalBlockPos.offset`.
    pub fn offset_global(&self, offset: BlockOffset) -> Result<GlobalBlockPos, OutOfBoundsError> {
        let global_position: GlobalBlockPos = (*self).into();
        global_position.offset(offset)
    }
    /// Returns `ChunkPos` that this block position borders.
    ///
    /// For example: x=0 borders the chunk to the left (X-), z=31 borders
    /// the chunk below (Z+), etc.
    pub fn borders(&self) -> Vec<ChunkPos> {
        let mut borders = Vec::new();
        if self.x == 0 {
            borders.push(ChunkPos::new(self.chunk.x - 1, self.chunk.z));
        } else if self.x == CHUNK_SIZE_X - 1 {
            borders.push(ChunkPos::new(self.chunk.x + 1, self.chunk.z));
        }
        if self.z == 0 {
            borders.push(ChunkPos::new(self.chunk.x, self.chunk.z - 1));
        } else if self.z == CHUNK_SIZE_Z - 1 {
            borders.push(ChunkPos::new(self.chunk.x, self.chunk.z + 1));
        }
        borders
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
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct GlobalBlockPos {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl GlobalBlockPos {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }
    /// Returns the `ChunkPos` that this block resides in.
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
    /// Offsets this block position by `offset`.
    pub fn offset(&self, offset: BlockOffset) -> Result<Self, OutOfBoundsError> {
        let y = self.y + offset.y;
        if !(0..=255).contains(&y) {
            Err(OutOfBoundsError)
        } else {
            Ok(Self {
                x: self.x + offset.x,
                y,
                z: self.z + offset.z,
            })
        }
    }
}

impl From<LocalBlockPos> for GlobalBlockPos {
    fn from(local_position: LocalBlockPos) -> Self {
        let chunk_origin = local_position.chunk.origin();
        let x = chunk_origin.x + local_position.x as isize;
        let y = chunk_origin.y + local_position.y as isize;
        let z = chunk_origin.z + local_position.z as isize;
        Self::new(x, y, z)
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

    macro_rules! lbp_to_gbp_test {
        ($local_position_base:expr, $chunk_position_base:expr, $global_position:expr) => {
            let chunk_pos = ChunkPos::new($chunk_position_base[0], $chunk_position_base[1]);
            let lbp = LocalBlockPos::new(
                $local_position_base[0],
                $local_position_base[1],
                $local_position_base[2],
                chunk_pos,
            );
            let converted = GlobalBlockPos::from(lbp);
            assert_eq!(
                converted,
                GlobalBlockPos::new(
                    $global_position[0],
                    $global_position[1],
                    $global_position[2]
                )
            );
            assert_eq!(chunk_pos, converted.chunk());
        };
    }

    macro_rules! test_lbp_offset {
        ($local_position_base:expr, $chunk_position_base:expr, $offset_base:expr, $end_position:expr, $end_chunk_position:expr) => {
            let initial = LocalBlockPos::new(
                $local_position_base[0],
                $local_position_base[1],
                $local_position_base[2],
                ChunkPos::new($chunk_position_base[0], $chunk_position_base[1]),
            );
            let offset = BlockOffset::new($offset_base[0], $offset_base[1], $offset_base[2]);
            assert_eq!(
                initial.offset(offset).unwrap(),
                LocalBlockPos::new(
                    $end_position[0],
                    $end_position[1],
                    $end_position[2],
                    ChunkPos::new($end_chunk_position[0], $end_chunk_position[1])
                )
            );
        };
    }

    macro_rules! test_lbp_global_offset {
        ($local_position_base:expr, $chunk_position_base:expr, $offset_base:expr, $end_position:expr) => {
            let initial = LocalBlockPos::new(
                $local_position_base[0],
                $local_position_base[1],
                $local_position_base[2],
                ChunkPos::new($chunk_position_base[0], $chunk_position_base[1]),
            );
            let offset = BlockOffset::new($offset_base[0], $offset_base[1], $offset_base[2]);
            assert_eq!(
                initial.offset_global(offset).unwrap(),
                GlobalBlockPos::new($end_position[0], $end_position[1], $end_position[2],)
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

    #[test]
    fn test_conversions() {
        // Local block position (base), chunk position (base), global block position (expected)
        lbp_to_gbp_test!([0, 0, 0], [0, 0], [0, 0, 0]);
        lbp_to_gbp_test!([1, 0, 1], [0, 0], [1, 0, 1]);
        lbp_to_gbp_test!([31, 0, 31], [0, 0], [31, 0, 31]);
        lbp_to_gbp_test!([0, 0, 0], [1, 1], [32, 0, 32]);
        lbp_to_gbp_test!([0, 0, 0], [-1, -1], [-32, 0, -32]);
    }

    // TODO: More tests

    #[test]
    fn test_local_offest() {
        test_lbp_offset!([4, 0, 4], [0, 0], [0, 0, 0], [4, 0, 4], [0, 0]);
        test_lbp_offset!([20, 0, 20], [0, 0], [2, 0, 2], [22, 0, 22], [0, 0]);
    }

    #[test]
    fn test_global_offset() {
        test_lbp_global_offset!([10, 0, 10], [0, 0], [0, 0, 0], [10, 0, 10]);
        test_lbp_global_offset!([31, 0, 31], [0, 0], [1, 0, 1], [32, 0, 32]);
        test_lbp_global_offset!([31, 0, 31], [1, 1], [1, 0, 1], [64, 0, 64]);
        test_lbp_global_offset!([0, 0, 0], [-1, 0], [0, 0, 0], [-32, 0, 0]);
        test_lbp_global_offset!([0, 0, 0], [-1, 0], [4, 2, 0], [-28, 2, 0]);
        test_lbp_global_offset!([0, 10, 0], [-1, 0], [-4, 0, 0], [-36, 10, 0]);
    }

    #[should_panic]
    fn test_local_offset_panic() {
        test_lbp_offset!([31, 0, 31], [0, 0], [1, 0, 1], [0, 0, 0], [1, 1]);
    }

    #[should_panic]
    fn test_local_offset_panic_2() {
        test_lbp_global_offset!([0, 0, 0], [0, 0], [0, -1, 0], [0, 0, 0]);
    }
}
