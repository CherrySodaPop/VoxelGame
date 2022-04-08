use std::collections::BTreeMap;

use gdnative::{
    api::{Mesh, MeshInstance, OpenSimplexNoise, SurfaceTool},
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
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, -1.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, -1.0, 1.0),
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
    Vector3::new(-1.0, 0.0, 0.0),
    Vector3::new(1.0, 0.0, 0.0),
    Vector3::new(0.0, 0.0, 1.0),
    Vector3::new(0.0, 0.0, -1.0),
];

// These don't need to be isizes, but it reduces the amount
// of "as isize" that would be present otherwise.
const CHUNK_SIZE_X: isize = 32;
const CHUNK_SIZE_Y: isize = 256;
const CHUNK_SIZE_Z: isize = 32;

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

    fn as_vector3(&self) -> Vector3 {
        Vector3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    fn offset(&self, x: isize, y: isize, z: isize) -> BlockPosition {
        BlockPosition::new(self.x + x, self.y + y, self.z + z)
    }
}

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
#[derive(NativeClass)]
#[inherit(Node)]
pub struct ChunkGenerator {
    chunks: BTreeMap<[isize; 2], Chunk>,
}
 

#[methods]
impl ChunkGenerator {
    // constructor
    fn new(_owner: &Node) -> Self {
        ChunkGenerator {
            chunks: BTreeMap::new(),
        }
    }

    // get the block id
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
    fn generate_chunk_mesh(&mut self, _owner: &Node, _origin: Vector2) {
    //fn generate_chunk_mesh(&mut self, _owner: &Node, _origin: Vec<isize>) {
        let origin: [isize; 2] = [
            _origin.x as isize,
            _origin.y as isize,
        ];
        //let origin = _origin.as_slice();
        let _chunk = self.chunks.get(&origin);
        if let Some(_chunk) = _chunk {
            _chunk.construct_mesh(self);
        } else {
            godot_print!("chunkgeneration: attempted to generate unloaded chunk!");
        }
    }

    fn chunk_node(&mut self, _owner: &Node, _origin: [isize; 2]) -> Option<Ref<Spatial, Unique>> {
        let _chunk = self.chunks.get(&_origin);
        if let Some(_chunk) = _chunk {
            Some(_chunk.spatial)
        }
        else
        {
            None
        }
    }

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        // generate chunks
        let simplex_noise = OpenSimplexNoise::new();
        for x in -0..1isize {
            for z in -0..1isize {
                let origin = [x, z];
                let mut new_chunk = Chunk::new(origin);
                godot_print!("Generating new chunk {:?}", new_chunk);
                new_chunk.generate(&*simplex_noise);
                self.chunks.insert(origin, new_chunk);
            }
        }
        // generate mesh (to be removed! - cherry)
        
        for chunk in self.chunks.values() {
            godot_print!("Constructing mesh for {:?}", chunk);
            chunk.construct_mesh(self);
            unsafe {
                _owner.add_child(chunk.spatial.assume_shared(), true);
            }
        }
        
    }
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
                        continue;
                    }
                    self.terrain[x as usize][y as usize][z as usize] = 3;
                }
            }
        }
    }

    fn construct_face(
        &self,
        face_type: usize,
        surface_tool: &Ref<SurfaceTool, Unique>,
        local_position: [isize; 3],
    ) {
        surface_tool.add_uv(Vector2::new(0.0, 0.0));
        surface_tool.add_normal(MESH_FACE_NORMALS[face_type]);
        for vertex in MESH_FACE_POSITIONS[face_type] {
            let position = Vector3::new(
                vertex.x + local_position[0] as f32,
                vertex.y + local_position[1] as f32,
                vertex.z + local_position[2] as f32,
            );
            surface_tool.add_vertex(vertex + position);
        }
    }

    fn check_nearby(&self, block_position: BlockPosition, generator: &ChunkGenerator) -> u16 {
        if block_position.chunk == self.origin {
            let local_position = block_position.local_position();
            self.terrain[local_position[0]][local_position[1]][local_position[2]]
        } else {
            generator.world_block(block_position)
        }
    }

    fn construct_mesh(&self, generator: &ChunkGenerator) {
        let chunkNode: &Node = generator.chunk_node;
        if (ch) MeshInstance::new();
        let surface_tool = SurfaceTool::new();
        surface_tool.begin(Mesh::PRIMITIVE_TRIANGLES);
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let block_id = self.terrain[x as usize][y as usize][z as usize];
                    if block_id == 0 {
                        continue;
                    }
                    let local_position = [x, y, z];
                    let block_position =
                        BlockPosition::from_local_position(self.origin, local_position);

                    // Top, bottom, left, right, front, back
                    // TODO: Maybe switch left and right enum values to make this
                    //       section easily replacable with a for-loop

                    if self.check_nearby(block_position.offset(0, 1, 0), generator) == 0 {
                        self.construct_face(0, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset(0, -1, 0), generator) == 0 {
                        self.construct_face(1, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset(-1, 0, 0), generator) == 0 {
                        self.construct_face(2, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset(1, 0, 0), generator) == 0 {
                        self.construct_face(3, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset(0, 0, 1), generator) == 0 {
                        self.construct_face(4, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset(0, 0, -1), generator) == 0 {
                        self.construct_face(5, &surface_tool, local_position);
                    }
                }
            }
        }
        mesh.set_mesh(
            surface_tool
                .commit(Null::null(), Mesh::ARRAY_COMPRESS_DEFAULT)
                .unwrap(),
        );
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
