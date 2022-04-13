use gdnative::api::{ArrayMesh, ConcavePolygonShape};
use gdnative::core_types::{VariantArray, Vector3, Vector3Array};
use gdnative::prelude::Unique;

use crate::chunk::Chunk;
use crate::constants::*;
use crate::positions::*;
use crate::world::World;

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

pub fn build_mesh_data(chunk: &Chunk, world: &World) -> MeshData {
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
                        let global_on_face_position =
                            local_position.offset_global(face.normal.into());
                        if let Some(block_id) = world.get_block(global_on_face_position) {
                            block_id == 0
                        } else {
                            false
                        }
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

pub struct GDMeshData {
    vertices: Vector3Array,
    normals: Vector3Array,
    uvs: Vector3Array,
}

impl GDMeshData {
    fn convert_vec(vec: &Vec<[isize; 3]>) -> Vector3Array {
        let mut gdarray = Vector3Array::new();
        // Hopefully this doesn't affect performance too much.
        vec.iter()
            .map(|val| Vector3::new(val[0] as f32, val[1] as f32, val[2] as f32))
            .collect()
    }
}

impl From<MeshData> for GDMeshData {
    fn from(mesh_data: MeshData) -> Self {
        GDMeshData {
            vertices: Self::convert_vec(&mesh_data.vertices),
            normals: Self::convert_vec(&mesh_data.normals),
            uvs: Self::convert_vec(&mesh_data.uvs),
        }
    }
}

// This function could accept Into<GDMeshData> to allow passing in MeshData structs
pub fn create_mesh(gd_mesh_data: &GDMeshData) -> gdnative::object::Ref<ArrayMesh, Unique> {
    println!("Creating mesh...");
    let mesh = ArrayMesh::new();
    let mut gdarray = VariantArray::new();
    gdarray.resize(ArrayMesh::ARRAY_MAX as i32);
    gdarray.set(ArrayMesh::ARRAY_VERTEX as i32, &gd_mesh_data.vertices);
    gdarray.set(ArrayMesh::ARRAY_NORMAL as i32, &gd_mesh_data.normals);
    // gdarray.set(ArrayMesh::ARRAY_TEX_UV as i32, uvs_vec);
    mesh.add_surface_from_arrays(
        gdnative::api::Mesh::PRIMITIVE_TRIANGLES,
        gdarray.into_shared(),
        VariantArray::new().into_shared(),
        2194432,
    );
    mesh
}

// This function taking ownership of the GDMeshData is stupid.
// Unfortunately, for reasons beyond me, ConcavePolygonShape.set_faces
// does not accept references to Vector3Arrays, it takes ownership of them.
// For optimization reasons, this function also just takes ownership of the
// GDMeshData to avoid having to clone.
pub fn create_collision_shape(
    gd_mesh_data: GDMeshData,
) -> gdnative::object::Ref<ConcavePolygonShape, Unique> {
    let collision_shape = ConcavePolygonShape::new();
    collision_shape.set_faces(gd_mesh_data.vertices);
    collision_shape
}

// TODO: tests
