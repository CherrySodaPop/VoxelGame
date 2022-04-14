/// Mesh data structs and Godot `ArrayMesh` generation.

use gdnative::api::{ArrayMesh, ConcavePolygonShape};
use gdnative::core_types::{VariantArray, Vector3, Vector3Array};
use gdnative::object::Ref;
use gdnative::prelude::Unique;

use crate::macros::vec3;

pub struct Face {
    pub vertices: [[isize; 3]; 6],
    pub normal: [isize; 3],
}

pub const FACES: [Face; 6] = [
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

/// Mesh data, like vertices, normals, and UVs.
pub struct MeshData {
    vertices: Vec<[isize; 3]>,
    normals: Vec<[isize; 3]>,
    uvs: Vec<[isize; 3]>,
}

impl MeshData {
    pub fn new() -> Self {
        // This could be replaced with deriving Default
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
        }
    }
    /// Adds a `Face` at `position`.
    pub fn add_face(&mut self, face: &Face, position: [isize; 3]) {
        // TODO: UV coordinates
        for vertex in face.vertices {
            self.normals.push(face.normal);
            self.vertices.push([
                position[0] + vertex[0],
                position[1] + vertex[1],
                position[2] + vertex[2],
            ]);
        }
    }
}

/// Like `MeshData`, but using Godot types.
///
/// This allows abstraction over mesh data, while still permitting
/// things like passing `self.vertices.clone()` to Godot.
///
/// Keep in mind that `Vector3Array`s are reference-counted, meaning
/// that `Vector3Array.clone()` does not actually clone the *data*.
/// See godot-rust's documentation on `PoolArray` for more info.
pub struct GDMeshData {
    vertices: Vector3Array,
    normals: Vector3Array,
    uvs: Vector3Array,
}

impl GDMeshData {
    fn convert_vec(vec: &Vec<[isize; 3]>) -> Vector3Array {
        // Hopefully this doesn't affect performance too much.
        vec.iter()
            .map(|val| vec3!(val[0], val[1], val[2]))
            .collect()
    }

    /// Creates an `ArrayMesh` from this `GDMeshData`.
    pub fn create_mesh(&self) -> Ref<ArrayMesh, Unique> {
        let mesh = ArrayMesh::new();
        let gdarray = VariantArray::new();
        gdarray.resize(ArrayMesh::ARRAY_MAX as i32);
        gdarray.set(ArrayMesh::ARRAY_VERTEX as i32, self.vertices.clone());
        gdarray.set(ArrayMesh::ARRAY_NORMAL as i32, self.normals.clone());
        // TODO: UV coordinates
        // gdarray.set(ArrayMesh::ARRAY_TEX_UV as i32, uvs_vec);
        mesh.add_surface_from_arrays(
            gdnative::api::Mesh::PRIMITIVE_TRIANGLES,
            gdarray.into_shared(),
            VariantArray::new().into_shared(),
            2194432,
        );
        mesh
    }

    /// Creates a `ConcavePolygonShape` from this `GDMeshData`.
    pub fn create_collision_shape(&self) -> Ref<ConcavePolygonShape, Unique> {
        let collision_shape = ConcavePolygonShape::new();
        collision_shape.set_faces(self.vertices.clone());
        collision_shape
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

// TODO: tests
