use gdnative::api::ArrayMesh;
use gdnative::core_types::Variant;
use gdnative::core_types::VariantArray;
use gdnative::core_types::Vector3;
use gdnative::core_types::Vector3Array;
use gdnative::prelude::Unique;

use crate::constants::*;
use crate::positions::*;

struct Face {
    vertices: [[isize; 3]; 6],
    normal: [isize; 3],
}

const FACES: [Face; 6] = [
    // Top, Y+
    Face {
        vertices: [
            [0, 0, 0],
            [1, 0, 0],
            [0, 0, 1],
            [1, 0, 0],
            [1, 0, 1],
            [0, 0, 1],
        ],
        normal: [0, 1, 0],
    },
    // Bottom, Y-
    Face {
        vertices: [
            [0, -1, 1],
            [1, -1, 1],
            [0, -1, 0],
            [1, -1, 1],
            [1, -1, 0],
            [0, -1, 0],
        ],
        normal: [0, -1, 0],
    },
    // Right, X+
    Face {
        vertices: [
            [1, 0, 1],
            [1, 0, 0],
            [1, -1, 1],
            [1, -1, 0],
            [1, -1, 1],
            [1, 0, 0],
        ],
        normal: [1, 0, 0],
    },
    // Left, X-
    Face {
        vertices: [
            [0, 0, 0],
            [0, 0, 1],
            [0, -1, 0],
            [0, 0, 1],
            [0, -1, 1],
            [0, -1, 0],
        ],
        normal: [-1, 0, 0],
    },
    // Front, Z+
    Face {
        vertices: [
            [0, -1, 1],
            [0, 0, 1],
            [1, 0, 1],
            [1, 0, 1],
            [1, -1, 1],
            [0, -1, 1],
        ],
        normal: [0, 0, 1],
    },
    // Back, Z-
    Face {
        vertices: [
            [1, -1, 0],
            [1, 0, 0],
            [0, 0, 0],
            [0, 0, 0],
            [0, -1, 0],
            [1, -1, 0],
        ],
        normal: [0, 0, -1],
    },
];

/*
fn construct_face(
    &self,
    face: BlockFace,
    surface_tool: &Ref<SurfaceTool, Unique>,
    local_position: [isize; 3],
    block_id: u16,
    vertex_pool: &mut Vector3Array,
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
        vertex_pool.push(position);
    }
}
*/

pub struct MeshData {
    vertices: Vec<[isize; 3]>,
    normals: Vec<[isize; 3]>,
    uvs: Vec<[isize; 3]>,
}

impl MeshData {
    fn new() -> Self {
        // This could be replaced with deriving Default
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        }
    }
}

type BlockID = u16;
// type TerrainArray = [[[BlockID; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_Z];

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

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("position", &self.position)
            .finish()
    }
}

fn add_face(face: Face, position: LocalBlockPos, mesh_data: &mut MeshData) {
    for vertex in face.vertices {
        mesh_data.normals.push(face.normal); // hm?
        mesh_data.vertices.push([
            position.x as isize + vertex[0],
            position.y as isize + vertex[1],
            position.z as isize + vertex[2],
        ]);
    }
}

pub fn build_mesh_data(chunk: &Chunk) -> MeshData {
    println!("Building mesh data for {:?}", chunk);
    let mut mesh_data = MeshData::new();
    for x in 0..CHUNK_SIZE_X {
        for y in 0..CHUNK_SIZE_Y {
            for z in 0..CHUNK_SIZE_Z {
                let block_id = chunk.terrain[x as usize][y as usize][z as usize];
                if block_id == 0 {
                    // This is an air block, it has no faces.
                    continue;
                }
                let local_position = LocalBlockPos::new(x, y, z, chunk.position);
                for face in FACES {
                    let on_face_position = local_position.offset(face.normal.into());
                    let face_visible = if let Ok(on_face_position) = on_face_position {
                        chunk.get(on_face_position) == 0
                    } else {
                        false
                    };
                    if !face_visible {
                        continue;
                    }
                    add_face(face, local_position, &mut mesh_data);
                }
            }
        }
    }
    mesh_data
}

pub fn create_mesh(mesh_data: MeshData) -> gdnative::object::Ref<ArrayMesh, Unique> {
    println!("Creating mesh...");
    let mesh = ArrayMesh::new();
    let mut gdarray = VariantArray::new();
    gdarray.resize(ArrayMesh::ARRAY_MAX as i32);
    // How bad is all this garbage on performance?
    let vertices_vec: Vector3Array = mesh_data
        .vertices
        .into_iter()
        .map(|vert| Vector3::new(vert[0] as f32, vert[1] as f32, vert[2] as f32))
        .collect();
    let normals_vec: Vector3Array = mesh_data
        .normals
        .into_iter()
        .map(|normal| Vector3::new(normal[0] as f32, normal[1] as f32, normal[2] as f32))
        .collect();
    let uvs_vec: Vector3Array = mesh_data
        .uvs
        .into_iter()
        .map(|uv| Vector3::new(uv[0] as f32, uv[1] as f32, uv[2] as f32))
        .collect();
    gdarray.set(ArrayMesh::ARRAY_VERTEX as i32, vertices_vec);
    gdarray.set(ArrayMesh::ARRAY_NORMAL as i32, normals_vec);
    // gdarray.set(ArrayMesh::ARRAY_TEX_UV as i32, uvs_vec);
    println!("gdarray len: {}", gdarray.len());
    mesh.add_surface_from_arrays(
        gdnative::api::Mesh::PRIMITIVE_TRIANGLES,
        gdarray.into_shared(),
        VariantArray::new().into_shared(),
        2194432,
    );
    mesh
}

/*
let chunk_mesh = surface_tool
    .commit(Null::null(), Mesh::ARRAY_COMPRESS_DEFAULT)
    .unwrap();
self.mesh.set_mesh(chunk_mesh);
if let Some(material) = &generator.material {
    self.mesh.set_surface_material(0, material);
}
collision_shape.set_faces(vertex_pool);
self.collision.set_shape(collision_shape);
 */
