//! Mesh data structs and Godot `ArrayMesh` generation.

use std::borrow::Borrow;

use gdnative::{
    api::{ArrayMesh, ConcavePolygonShape},
    core_types::{VariantArray, Vector2, Vector2Array, Vector3, Vector3Array, Color, ColorArray},
    object::Ref,
    prelude::Unique,
};

use crate::macros::*;

#[derive(Clone, Copy, Debug)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn uv(axes: [Self; 2], vertex: [isize; 3]) -> [f32; 2] {
        let x_idx = axes[0] as usize;
        let z_idx = axes[1] as usize;
        [vertex[x_idx].abs() as f32, vertex[z_idx].abs() as f32]
    }
}

#[derive(Debug)]
pub struct Face {
    pub vertices: [[isize; 3]; 6],
    pub normal: [isize; 3],
    uv_use: [Axis; 2],
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
        uv_use: [Axis::X, Axis::Z],
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
        uv_use: [Axis::X, Axis::Z],
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
        uv_use: [Axis::Z, Axis::Y],
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
        uv_use: [Axis::Z, Axis::Y],
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
        uv_use: [Axis::X, Axis::Y],
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
        uv_use: [Axis::X, Axis::Y],
    },
];

/// Mesh data, like vertices, normals, and UVs.
pub struct MeshData {
    pub vertices: Vec<[isize; 3]>,
    pub normals: Vec<[isize; 3]>,
    pub color: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
}

impl MeshData {
    pub fn new() -> Self {
        // This could be replaced with deriving Default
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            color: Vec::new(),
        }
    }
    /// Adds a `Face` at `position`.
    pub fn add_face(&mut self, face: &Face, position: [isize; 3]) {
        for vertex in face.vertices {
            self.normals.push(face.normal);
            self.vertices.push([
                position[0] + vertex[0],
                position[1] + vertex[1],
                position[2] + vertex[2],
            ]);
            // self.uvs.push(Axis::uv(face.uv_use, vertex));
        }
    }
    /// Adds a `Face` at `position`, with UV coordinates.
    pub fn add_face_with_uv(&mut self, face: &Face, position: [isize; 3]) {
        self.add_face(face, position);
        for vertex in face.vertices {
            let uv = Axis::uv(face.uv_use, vertex);
            // TODO: This is tied to our specific texture system,
            //       making it more general may be a good idea.
            let face_offset = match face.normal[1] {
                1 => 0.0,
                0 => 1.0,
                -1 => 2.0,
                _ => unreachable!(),
            };
            let uv = [(face_offset + uv[0]) * 16.0, uv[1] * 16.0];
            let uv = [uv[0] / 48.0, uv[1] / 16.0];
            self.uvs.push(uv);
            self.color.push([0.2, 0.2, 0.2, 1.0]);
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
    color: ColorArray,
    uvs: Vector2Array,
}

impl GDMeshData {
    pub fn convert_vec3_color(vec: &Vec<[f32; 4]>) -> ColorArray {
        vec.iter()
            .map(|val| vecColor!(val[0], val[1], val[2], val[3]))
            .collect()
    }

    pub fn convert_vec3(vec: &Vec<[isize; 3]>) -> Vector3Array {
        // Hopefully this doesn't affect performance too much.
        vec.iter()
            .map(|val| vec3!(val[0], val[1], val[2]))
            .collect()
    }

    pub fn convert_vec2(vec: &Vec<[f32; 2]>) -> Vector2Array {
        vec.iter().map(|val| vec2!(val[0], val[1])).collect()
    }

    /// Creates an `ArrayMesh` from this `GDMeshData`.
    pub fn create_mesh(&self) -> Ref<ArrayMesh, Unique> {
        let mut mesh = ArrayMesh::new();
        self.add_to(&mut mesh);
        mesh
    }

    /// Adds this `GDMeshData` to `mesh` as a new surface.
    pub fn add_to(&self, mesh: &mut Ref<ArrayMesh, Unique>) {
        let gdarray = VariantArray::new();
        gdarray.resize(ArrayMesh::ARRAY_MAX as i32);
        gdarray.set(ArrayMesh::ARRAY_VERTEX as i32, self.vertices.clone());
        gdarray.set(ArrayMesh::ARRAY_NORMAL as i32, self.normals.clone());
        gdarray.set(ArrayMesh::ARRAY_COLOR as i32, self.color.clone());
        gdarray.set(ArrayMesh::ARRAY_TEX_UV as i32, self.uvs.clone());
        mesh.add_surface_from_arrays(
            gdnative::api::Mesh::PRIMITIVE_TRIANGLES,
            gdarray.into_shared(),
            VariantArray::new().into_shared(),
            2194432,
        );
    }

    /// Creates a `ConcavePolygonShape` from this `GDMeshData`.
    pub fn create_collision_shape(&self) -> Ref<ConcavePolygonShape, Unique> {
        let collision_shape = ConcavePolygonShape::new();
        collision_shape.set_faces(self.vertices.clone());
        collision_shape
    }
}

impl<T: Borrow<MeshData>> From<T> for GDMeshData {
    fn from(mesh_data_ref: T) -> Self {
        let mesh_data: &MeshData = mesh_data_ref.borrow();
        GDMeshData {
            vertices: Self::convert_vec3(&mesh_data.vertices),
            normals: Self::convert_vec3(&mesh_data.normals),
            uvs: Self::convert_vec2(&mesh_data.uvs),
            color: Self::convert_vec3_color(&mesh_data.color),
        }
    }
}

// TODO: tests
