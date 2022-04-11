use std::collections::BTreeMap;

use gdnative::{
    api::{Material, Mesh, MeshInstance, OpenSimplexNoise, SpatialMaterial, SurfaceTool},
    prelude::*,
};

const MESH_FACE_POSITIONS: [[Vector3; 6]; 6] = [
    [
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
    ],
    [
        Vector3::new(0.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
    ],
    [
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(1.0, -1.0, 0.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(1.0, 0.0, 0.0),
    ],
    [
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, -1.0, 1.0),
        Vector3::new(0.0, -1.0, 0.0),
    ],
    [
        Vector3::new(0.0, -1.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(1.0, -1.0, 1.0),
        Vector3::new(0.0, -1.0, 1.0),
    ],
    [
        Vector3::new(1.0, -1.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(1.0, -1.0, 0.0),
    ],
];

const MESH_FACE_NORMALS: [Vector3; 6] = [
    Vector3::new(0.0, 1.0, 0.0),
    Vector3::new(0.0, -1.0, 0.0),
    Vector3::new(1.0, 0.0, 0.0),
    Vector3::new(-1.0, 0.0, 0.0),
    Vector3::new(0.0, 0.0, 1.0),
    Vector3::new(0.0, 0.0, -1.0),
];

// These don't need to be isizes, but it reduces the amount
// of "as isize" that would be present otherwise.
const CHUNK_SIZE_X: isize = 32;
const CHUNK_SIZE_Y: isize = 256;
const CHUNK_SIZE_Z: isize = 32;

// For UV calculations, hence f32.
const UV_TEXTURE_WIDTH: f32 = 256.0;
const TEXTURE_WIDTH: f32 = 16.0;

#[derive(PartialEq, Eq, Clone, Copy)]
enum BlockFace {
    Top,    // Y+
    Bottom, // Y-
    Right,  // X+
    Left,   // X-
    Front,  // Z+
    Back,   // Z-
}

impl BlockFace {
    /// Whether or not this block face is on the X or Z axes.
    fn is_side(&self) -> bool {
        // This enum stuff is making me *miss* C++, somehow.
        let i = *self as u8;
        i != 0 && i != 1
    }
    /// Whether or not this block face is on the X axis (+ or -).
    fn is_x_axis(&self) -> bool {
        let i = *self as u8;
        i == 2 || i == 3
    }
    /// Whether or not this block face is on the Z axis (+ or -).
    fn is_z_axis(&self) -> bool {
        let i = *self as u8;
        i == 4 || i == 5
    }
    /// Whether or not this block face is on the Y axis (+ or -).
    fn is_y_axis(&self) -> bool {
        let i = *self as u8;
        i == 0 || i == 1
    }
    /// Returns the offset of a block that would be resting on this face.
    ///
    /// e.g., the block resting on `BlockFace::Top` would be 1 above the
    /// block this face represents, so `[0, 1, 0]`.
    fn block_offset(&self) -> [isize; 3] {
        match self {
            BlockFace::Top => [0, 1, 0],
            BlockFace::Bottom => [0, -1, 0],
            BlockFace::Right => [1, 0, 0],
            BlockFace::Left => [-1, 0, 0],
            BlockFace::Front => [0, 0, 1],
            BlockFace::Back => [0, 0, -1],
        }
    }
    /// The texture atlas holds textures in the order top, sides, bottom.
    ///
    /// As such, this will return 0 for top faces, 1 for left, right, front,
    /// or back faces, and 2 for bottom faces.
    fn atlas_offset(&self) -> isize {
        if self.is_side() {
            1
        } else {
            match *self {
                Self::Top => 0,
                Self::Bottom => 2,
                _ => unreachable!(),
            }
        }
    }
}

const FACES: [BlockFace; 6] = [
    BlockFace::Top,
    BlockFace::Bottom,
    BlockFace::Right,
    BlockFace::Left,
    BlockFace::Front,
    BlockFace::Back,
];

#[derive(Debug)]
/// Represents a block's position in *world* space.
struct BlockPosition {
    x: isize,
    y: isize,
    z: isize,
    chunk: [isize; 2],
}

impl BlockPosition {
    fn new(x: isize, y: isize, z: isize) -> Self {
        let xn = if x < 0 { 1 } else { 0 };
        let zn = if z < 0 { 1 } else { 0 };
        //                                             -------------- Round *downwards* in the negatives, instead of
        //                                             |              towards zero.
        //                                             |
        //                        ----------------------------------- Avoid "flicking" to the next chunk at -32, -64, etc.,
        //                        |                    |              do it at -33, -65, and so on instead.
        //                        |                    |
        //                        |                    |
        let chunk_x: isize = ((xn + x) / CHUNK_SIZE_X) - xn;
        let chunk_z: isize = ((zn + z) / CHUNK_SIZE_Z) - zn;
        BlockPosition {
            x,
            y,
            z,
            chunk: [chunk_x, chunk_z],
        }
    }

    fn local_position(&self) -> [usize; 3] {
        [
            (self.x - (self.chunk[0] * CHUNK_SIZE_X)).abs() as usize,
            self.y.abs() as usize,
            (self.z - (self.chunk[1] * CHUNK_SIZE_Z)).abs() as usize,
        ]
    }

    fn from_local_position(chunk: [isize; 2], local_position: [isize; 3]) -> Self {
        if local_position[0] > CHUNK_SIZE_X - 1 || local_position[2] > CHUNK_SIZE_Z - 1 {
            panic!("invalid local_position: {:?}", local_position);
        }
        Self::new(
            local_position[0] as isize + (chunk[0] * CHUNK_SIZE_X),
            local_position[1] as isize,
            local_position[2] as isize + (chunk[1] * CHUNK_SIZE_Z),
        )
    }

    fn offset(&self, by: [isize; 3]) -> BlockPosition {
        BlockPosition::new(self.x + by[0], self.y + by[1], self.z + by[2])
    }
}

impl From<[isize; 3]> for BlockPosition {
    fn from(position: [isize; 3]) -> Self {
        Self::new(position[0], position[1], position[2])
    }
}

impl From<BlockPosition> for Vector3 {
    fn from(block_position: BlockPosition) -> Self {
        Self::new(
            block_position.x as f32,
            block_position.y as f32,
            block_position.z as f32,
        )
    }
}

// impl std::ops::Add for BlockPosition {
//     type Output = Self;

//     fn add(self, rhs: Self) -> Self::Output {
//         BlockPosition::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
//     }
// }

struct Chunk {
    terrain: [[[u16; CHUNK_SIZE_Z as usize]; CHUNK_SIZE_Y as usize]; CHUNK_SIZE_X as usize],
    origin: [isize; 2],
    spatial: Ref<Spatial, Unique>,
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("origin", &self.origin)
            .finish()
    }
}

// Chunk generator implementation
#[derive(NativeClass, Default)]
#[export]
#[inherit(Node)]
pub struct ChunkGenerator {
    chunks: BTreeMap<[isize; 2], Chunk>,
    #[property]
    material: Option<Ref<Material, Shared>>,
}

#[methods]
impl ChunkGenerator {
    fn new(_owner: &Node) -> Self {
        ChunkGenerator {
            chunks: BTreeMap::new(),
            ..Default::default()
        }
    }

    fn world_block(&self, block_position: BlockPosition) -> u16 {
        let chunk_origin = block_position.chunk;
        let chunk = self.chunks.get(&chunk_origin);
        if let Some(chunk) = chunk {
            let chunk_coords = block_position.local_position();
            chunk.terrain[chunk_coords[0]][chunk_coords[1]][chunk_coords[2]]
        } else {
            0
        }
    }

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        let simplex_noise = OpenSimplexNoise::new();
        for x in -4..5isize {
            for z in -4..5isize {
                let origin = [x, z];
                let mut new_chunk = Chunk::new(origin);
                godot_print!("Generating new chunk {:?}", new_chunk);
                new_chunk.generate(&*simplex_noise);
                self.chunks.insert(origin, new_chunk);
            }
        }

        for chunk in self.chunks.values() {
            godot_print!("Constructing mesh for {:?}", chunk);
            chunk.construct_mesh(self);
            unsafe {
                _owner.add_child(chunk.spatial.assume_shared(), true);
            }
        }
    }
}

/// Returns the UV coordinates for this vertex, accounting for what face it's on.
fn vertex_uv(vertex: Vector3, face: BlockFace) -> [f32; 2] {
    [
        if face.is_x_axis() { vertex.z } else { vertex.x },
        if face.is_side() {
            vertex.y.abs()
        } else {
            vertex.z
        },
    ]
}

// Chunk implementation
impl Chunk {
    fn new(origin: [isize; 2]) -> Self {
        let spatial = Spatial::new();
        let spatial_transform = Self::spatial_transform(origin);
        spatial.set_transform(spatial.transform().translated(Vector3::new(
            spatial_transform[0] as f32,
            spatial_transform[1] as f32,
            spatial_transform[2] as f32,
        )));
        Chunk {
            terrain: [[[0; CHUNK_SIZE_Z as usize]; CHUNK_SIZE_Y as usize]; CHUNK_SIZE_X as usize],
            origin,
            spatial,
        }
    }

    fn spatial_transform(origin: [isize; 2]) -> [isize; 3] {
        [(origin[0] * CHUNK_SIZE_X), 0, (origin[1] * CHUNK_SIZE_Z)]
    }

    /// Generates the chunk's terrain data, storing it in `Chunk.terrain`.
    fn generate(&mut self, simplex_noise: &OpenSimplexNoise) {
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                let world_block_x = x + (self.origin[0] * CHUNK_SIZE_X);
                let world_block_z = z + (self.origin[1] * CHUNK_SIZE_Z);
                let noise_height: f64 = simplex_noise
                    .get_noise_2dv(Vector2::new(world_block_x as f32, world_block_z as f32));
                let terrain_peak =
                    ((CHUNK_SIZE_Y as f64) * ((noise_height / 2.0) + 0.5) * 0.1) as isize;
                for y in 0..CHUNK_SIZE_Y {
                    if y > terrain_peak {
                        break;
                    }
                    let block_id = if y > 6 { 1 } else { 2 };
                    self.terrain[x as usize][y as usize][z as usize] = block_id;
                }
            }
        }
    }

    /// Builds a block face in the mesh, and handles its texture/UV mapping.
    fn construct_face(
        &self,
        face: BlockFace,
        surface_tool: &Ref<SurfaceTool, Unique>,
        local_position: [isize; 3],
        block_id: u16,
    ) {
        surface_tool.add_uv(Vector2::new(0.0, 0.0));
        let face_type_index = face as usize;
        surface_tool.add_normal(MESH_FACE_NORMALS[face_type_index]);
        for vertex in MESH_FACE_POSITIONS[face_type_index] {
            // "Normalized" UV (only 0 or 1)
            let mut uv = vertex_uv(vertex, face);
            // Align the UV to its position within the texture atlas
            let texture_x =
                ((3.0 * block_id as f32) + face.atlas_offset() as f32 + uv[0]) * TEXTURE_WIDTH;
            uv = [texture_x / UV_TEXTURE_WIDTH, uv[1]];
            surface_tool.add_uv(Vector2::new(uv[0], uv[1]));
            let position = Vector3::new(
                vertex.x + local_position[0] as f32,
                vertex.y + local_position[1] as f32,
                vertex.z + local_position[2] as f32,
            );
            surface_tool.add_vertex(position);
        }
    }

    /// Gets the block ID of a "nearby" block.
    ///
    /// This is more optimized than using just `ChunkGenerator.world_block`,
    /// as most "nearby" blocks are within the current chunk and thus do not
    /// require a dictionary lookup.
    fn check_nearby(&self, block_position: BlockPosition, generator: &ChunkGenerator) -> u16 {
        if block_position.chunk == self.origin {
            let local_position = block_position.local_position();
            self.terrain[local_position[0]][local_position[1]][local_position[2]]
        } else {
            generator.world_block(block_position)
        }
    }

    /// Builds the chunk mesh using the current `Chunk.terrain` data.
    fn construct_mesh(&self, generator: &ChunkGenerator) {
        let mesh = MeshInstance::new();
        let surface_tool = SurfaceTool::new();
        surface_tool.begin(Mesh::PRIMITIVE_TRIANGLES);
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let block_id = self.terrain[x as usize][y as usize][z as usize];
                    if block_id == 0 {
                        // This is an air block, it has no faces.
                        continue;
                    }
                    let local_position = [x, y, z];
                    let block_position =
                        BlockPosition::from_local_position(self.origin, local_position);

                    // Check each face to see if it's visible, and if so, add it to the mesh.
                    for face_type in FACES {
                        let block_offset = block_position.offset(face_type.block_offset());
                        if self.check_nearby(block_offset, generator) == 0 {
                            self.construct_face(face_type, &surface_tool, local_position, block_id);
                        }
                    }
                }
            }
        }
        mesh.set_mesh(
            surface_tool
                .commit(Null::null(), Mesh::ARRAY_COMPRESS_DEFAULT)
                .unwrap(),
        );
        if let Some(material) = &generator.material {
            mesh.set_surface_material(0, material);
        }
        self.spatial.add_child(mesh, true);
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ChunkGenerator>();
}

godot_init!(init);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_position() {
        let bp = BlockPosition::new(0, 0, 0);
        assert_eq!(bp.chunk, [0, 0]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(31, 0, 31);
        assert_eq!(bp.chunk, [0, 0]);
        assert_eq!(bp.local_position(), [31, 0, 31]);
        let bp = BlockPosition::new(32, 0, 32);
        assert_eq!(bp.chunk, [1, 1]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(63, 0, 63);
        assert_eq!(bp.chunk, [1, 1]);
        assert_eq!(bp.local_position(), [31, 0, 31]);
        let bp = BlockPosition::new(64, 0, 64);
        assert_eq!(bp.chunk, [2, 2]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(-1, 0, -1);
        assert_eq!(bp.chunk, [-1, -1]);
        let bp = BlockPosition::new(-32, 0, -32);
        assert_eq!(bp.chunk, [-1, -1]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(-33, 0, -33);
        assert_eq!(bp.chunk, [-2, -2]);
        assert_eq!(bp.local_position(), [31, 0, 31]);
        let bp = BlockPosition::new(-64, 0, -100);
        assert_eq!(bp.chunk, [-2, -4]);
        assert_eq!(bp.local_position(), [0, 0, 28]);
        let bp = BlockPosition::new(-64, 0, -97);
        assert_eq!(bp.chunk, [-2, -4]);
        assert_eq!(bp.local_position(), [0, 0, 31]);
        let bp = BlockPosition::new(-64, 0, -96);
        assert_eq!(bp.chunk, [-2, -3]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
    }

    #[test]
    fn test_spatial_transform() {
        let origin = [0, 0];
        assert_eq!(Chunk::spatial_transform(origin), [0, 0, 0]);
        let origin = [-1, -1];
        assert_eq!(Chunk::spatial_transform(origin), [-32, 0, -32]);
        let origin = [-2, -2];
        assert_eq!(Chunk::spatial_transform(origin), [-64, 0, -64]);
        let origin = [3, 3];
        assert_eq!(Chunk::spatial_transform(origin), [96, 0, 96]);
    }

    #[test]
    fn test_from_local_position() {
        let bp = BlockPosition::from_local_position([0, 0], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [0, 0, 0]);
        let bp = BlockPosition::from_local_position([1, 1], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [32, 0, 32]);
        let bp = BlockPosition::from_local_position([1, 1], [31, 0, 31]);
        assert_eq!([bp.x, bp.y, bp.z], [63, 0, 63]);
        let bp = BlockPosition::from_local_position([2, 2], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [64, 0, 64]);
        let bp = BlockPosition::from_local_position([-1, -2], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [-32, 0, -64]);
        let bp = BlockPosition::from_local_position([-2, -3], [1, 0, 1]);
        assert_eq!([bp.x, bp.y, bp.z], [-63, 0, -95]);
        let bp = BlockPosition::from_local_position([-2, -2], [31, 0, 31]);
        assert_eq!([bp.x, bp.y, bp.z], [-33, 0, -33]);
        let bp = BlockPosition::from_local_position([-1, -1], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [-32, 0, -32]);
        let bp = BlockPosition::from_local_position([-1, -1], [31, 0, 31]);
        assert_eq!([bp.x, bp.y, bp.z], [-1, 0, -1]);
    }

    #[test]
    #[should_panic]
    fn test_invalid_from_local_position() {
        BlockPosition::from_local_position([-1, -1], [32, 0, 32]);
    }
}
