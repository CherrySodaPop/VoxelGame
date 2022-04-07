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

struct Chunk {
    terrain: [[[u16; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X],
    origin: [usize; 2],
    spatial: Ref<Spatial, Unique>
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

    fn world_to_chunk_coords(chunk_origin: [usize; 2], world_coords: Vector3) -> Vector3 {
        Vector3::new(
            world_coords.x - (chunk_origin[0] * CHUNK_SIZE_X) as f32,
            world_coords.y,
            world_coords.z - (chunk_origin[1] * CHUNK_SIZE_Z) as f32
        )
    }

    fn chunk_of_world_position(world_coords: Vector3) -> [usize; 2] {
        [
            world_coords.x as usize / CHUNK_SIZE_X,
            world_coords.z as usize / CHUNK_SIZE_Z,
        ]
    }

    fn world_block(&self, block_position: Vector3) -> u16 {
        let chunk_origin = Self::chunk_of_world_position(block_position);
        let chunk = self
            .chunks
            .get(chunk_origin[0])
            .and_then(|rock| rock.get(chunk_origin[1]))
            .unwrap_or(&None);
        if let Some(chunk) = chunk {
            let chunk_coords = Self::world_to_chunk_coords(chunk.origin, block_position);
            chunk.terrain[chunk_coords.x as usize][chunk_coords.y as usize][chunk_coords.z as usize]
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
                // let chunk_node = Spatial::new();
                // new_chunk.construct_mesh(&chunk_node, self);
                // unsafe {
                //     _owner.add_child(chunk_node.assume_shared(), true);
                // }
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
        //godot_print!("{:#?}", self.chunks);
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ChunkGenerator>();
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
            spatial
        }
    }

    fn generate(&mut self, simplex_noise: &OpenSimplexNoise) {
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                let world_block_x = x + (self.origin[0] * CHUNK_SIZE_X);
                let world_block_z = z + (self.origin[1] * CHUNK_SIZE_Z);
                let noise_height: f64 = simplex_noise.get_noise_2dv(Vector2::new(world_block_x as f32, world_block_z as f32));
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
        // //godot_print!("{:?}", self.terrain);
    }

    fn construct_face(
        &self,
        face_type: usize,
        surface_tool: &Ref<SurfaceTool, Unique>,
        x: usize,
        y: usize,
        z: usize,
    ) {
        let position = Vector3::new(x as f32, y as f32, z as f32);
        surface_tool.add_uv(Vector2::new(0.0, 0.0));
        surface_tool.add_normal(MESH_FACE_NORMALS[face_type]);
        for vertex in MESH_FACE_POSITIONS[face_type] {
            surface_tool.add_vertex(vertex + position);
        }
    }

    fn check_nearby(
        &self,
        world_position: Vector3,
        offset: Vector3,
        generator: &ChunkGenerator,
    ) -> u16 {
        let checking_world_position = world_position + offset;
        if ChunkGenerator::chunk_of_world_position(checking_world_position) == self.origin {
            let local_position =
                ChunkGenerator::world_to_chunk_coords(self.origin, checking_world_position);
            self.terrain[local_position.x as usize][local_position.y as usize]
                [local_position.z as usize]
        } else {
            generator.world_block(checking_world_position)
        }
    }

    fn construct_mesh(&self, generator: &ChunkGenerator) {
        let mesh = MeshInstance::new();
        let surface_tool = SurfaceTool::new();
        surface_tool.begin(Mesh::PRIMITIVE_TRIANGLES);
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let world_block_x = x + (self.origin[0] * CHUNK_SIZE_X);
                    let world_block_z = z + (self.origin[1] * CHUNK_SIZE_Z);
                    let world_position =
                        Vector3::new(world_block_x as f32, y as f32, world_block_z as f32);
                    //godot_print!("Constructing block @ {:?}", world_position);
                    // println!("Building faces for {:?}", world_position);
                    let block_id = self.terrain[x][y][z];
                    if block_id == 0 {
                        continue;
                    }

                    // top bottom left right front back
                    // if (ShouldBuildFace(0, 1, 0))
                    // if (ShouldBuildFace(0, -1, 0))
                    // if (ShouldBuildFace(-1, 0, 0))
                    // if (ShouldBuildFace(1, 0, 0))
                    // if (ShouldBuildFace(0, 0, 1))
                    // if (ShouldBuildFace(0, 0, -1))

                    if self.check_nearby(world_position, Vector3::new(0.0, 1.0, 0.0), generator)
                        == 0
                    {
                        self.construct_face(0, &surface_tool, x, y, z);
                    }
                    if self.check_nearby(world_position, Vector3::new(0.0, -1.0, 0.0), generator)
                        == 0
                    {
                        self.construct_face(1, &surface_tool, x, y, z);
                    }
                    if self.check_nearby(world_position, Vector3::new(-1.0, 0.0, 0.0), generator)
                        == 0
                    {
                        self.construct_face(2, &surface_tool, x, y, z);
                    }
                    if self.check_nearby(world_position, Vector3::new(1.0, 0.0, 0.0), generator)
                        == 0
                    {
                        self.construct_face(3, &surface_tool, x, y, z);
                    }
                    if self.check_nearby(world_position, Vector3::new(0.0, 0.0, 1.0), generator)
                        == 0
                    {
                        self.construct_face(4, &surface_tool, x, y, z);
                    }
                    if self.check_nearby(world_position, Vector3::new(0.0, 0.0, -1.0), generator)
                        == 0
                    {
                        self.construct_face(5, &surface_tool, x, y, z);
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
        // owner.add_child(mesh, true);
    }
}

godot_init!(init);
