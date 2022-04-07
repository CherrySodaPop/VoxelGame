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

const CHUNK_SIZE_X: usize = 32;
const CHUNK_SIZE_Y: usize = 256;
const CHUNK_SIZE_Z: usize = 32;

/// Represents a block's position in *world* space.
struct BlockPosition {
    // TODO: Use isizes once chunks can generate in the negative axes.
    x: usize,
    y: usize,
    z: usize,
    chunk: [usize; 2],
}

impl BlockPosition {
    fn new(x: usize, y: usize, z: usize) -> Self {
        BlockPosition {
            x,
            y,
            z,
            chunk: [x / CHUNK_SIZE_X, z / CHUNK_SIZE_Z],
        }
    }

    fn local_position(&self) -> [usize; 3] {
        [
            self.x - (self.chunk[0] * CHUNK_SIZE_X),
            self.y,
            self.z - (self.chunk[1] * CHUNK_SIZE_Z),
        ]
    }

    fn from_local_position(chunk: [usize; 2], local_position: [usize; 3]) -> Self {
        Self::new(
            local_position[0] + (chunk[0] * CHUNK_SIZE_X),
            local_position[1],
            local_position[2] + (chunk[1] * CHUNK_SIZE_Z),
        )
    }

    fn as_vector3(&self) -> Vector3 {
        Vector3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    // These dumb methods can be replaced once The Great isize Switch occurs
    fn offset_add(&self, x: usize, y: usize, z: usize) -> BlockPosition {
        BlockPosition::new(self.x + x, self.y + y, self.z + z)
    }

    fn offset_sub(&self, x: usize, y: usize, z: usize) -> BlockPosition {
        let new_x = self.x.checked_sub(x).unwrap_or(0);
        let new_y = self.y.checked_sub(y).unwrap_or(0);
        let new_z = self.z.checked_sub(z).unwrap_or(0);
        BlockPosition::new(new_x, new_y, new_z)
    }
}

struct Chunk {
    terrain: [[[u16; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X],
    origin: [usize; 2],
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
    chunks: Vec<Vec<Option<Chunk>>>,
}

#[methods]
impl ChunkGenerator {
    fn new(_owner: &Node) -> Self {
        ChunkGenerator {
            chunks: vec![vec![]],
        }
    }

    fn world_block(&self, block_position: BlockPosition) -> u16 {
        let chunk_origin = block_position.chunk;
        let chunk = self
            .chunks
            .get(chunk_origin[0])
            .and_then(|rock| rock.get(chunk_origin[1]))
            .unwrap_or(&None);
        if let Some(chunk) = chunk {
            let chunk_coords = block_position.local_position();
            chunk.terrain[chunk_coords[0]][chunk_coords[1]][chunk_coords[2]]
        } else {
            0
        }
    }

    fn all_chunks(&self) -> std::iter::Flatten<std::slice::Iter<'_, Vec<Option<Chunk>>>> {
        self.chunks.iter().flatten()
    }

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        let simplex_noise = OpenSimplexNoise::new();
        for z in 0..12usize {
            for x in 0..12usize {
                let mut new_chunk = Chunk::new([z, x]);
                godot_print!("Generating new chunk {:?}", new_chunk);
                new_chunk.generate(&*simplex_noise);
                self.chunks[z].push(Some(new_chunk));
            }
            self.chunks.push(Vec::new());
        }

        for chunk in self.all_chunks() {
            if let Some(chunk) = chunk {
                godot_print!("Constructing mesh for {:?}", chunk);
                chunk.construct_mesh(self);
                unsafe {
                    _owner.add_child(chunk.spatial.assume_shared(), true);
                }
            }
        }
    }
}

// Chunk implementation
impl Chunk {
    fn new(origin: [usize; 2]) -> Self {
        let spatial = Spatial::new();
        spatial.set_transform(spatial.transform().translated(Vector3::new(
            (origin[0] * CHUNK_SIZE_X) as f32,
            0.0,
            (origin[1] * CHUNK_SIZE_Z) as f32,
        )));
        Chunk {
            terrain: [[[0; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X],
            origin,
            spatial,
        }
    }

    fn generate(&mut self, simplex_noise: &OpenSimplexNoise) {
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                let world_block_x = x + (self.origin[0] * CHUNK_SIZE_X);
                let world_block_z = z + (self.origin[1] * CHUNK_SIZE_Z);
                let noise_height: f64 = simplex_noise
                    .get_noise_2dv(Vector2::new(world_block_x as f32, world_block_z as f32));
                let terrain_peak =
                    ((CHUNK_SIZE_Y as f64) * ((noise_height / 2.0) + 0.5) * 0.1) as usize;
                for y in 0..CHUNK_SIZE_Y {
                    if y > terrain_peak {
                        continue;
                    }
                    self.terrain[x][y][z] = 3;
                }
            }
        }
    }

    fn construct_face(
        &self,
        face_type: usize,
        surface_tool: &Ref<SurfaceTool, Unique>,
        local_position: [usize; 3],
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
        let mesh = MeshInstance::new();
        let surface_tool = SurfaceTool::new();
        surface_tool.begin(Mesh::PRIMITIVE_TRIANGLES);
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let block_id = self.terrain[x][y][z];
                    if block_id == 0 {
                        continue;
                    }
                    let local_position = [x, y, z];
                    let block_position =
                        BlockPosition::from_local_position(self.origin, local_position);

                    // Top, bottom, left, right, front, back
                    // TODO: Maybe switch left and right enum values to make this
                    //       section easily replacable with a for-loop

                    if self.check_nearby(block_position.offset_add(0, 1, 0), generator) == 0 {
                        self.construct_face(0, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset_sub(0, 1, 0), generator) == 0 {
                        self.construct_face(1, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset_sub(1, 0, 0), generator) == 0 {
                        self.construct_face(2, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset_add(1, 0, 0), generator) == 0 {
                        self.construct_face(3, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset_add(0, 0, 1), generator) == 0 {
                        self.construct_face(4, &surface_tool, local_position);
                    }
                    if self.check_nearby(block_position.offset_sub(0, 0, 1), generator) == 0 {
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
