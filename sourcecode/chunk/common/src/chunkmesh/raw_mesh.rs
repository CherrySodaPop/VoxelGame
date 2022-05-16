//! Mesh data structs and Godot `ArrayMesh` generation.

use gdnative::{
    api::ArrayMesh,
    core_types::{Color, ColorArray, VariantArray, Vector2, Vector2Array, Vector3, Vector3Array},
    prelude::Unique,
};

use crate::{color, vec2, vec3};

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
    pub colors: Vec<[u8; 4]>,
    pub uvs: Vec<[f32; 2]>,
}

impl MeshData {
    pub fn new() -> Self {
        // This could be replaced with deriving Default
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            colors: Vec::new(),
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

            // TODO: should be based on the block this face is facing, not the current block position
            self.colors.push([255, 255, 255, 255]);
        }
    }
    /// Converts this `MeshData` into a `VariantArray` for use with
    /// Godot's `ArrayMesh` and its associated concepts.
    pub fn to_gd_array(&self) -> VariantArray<Unique> {
        let gdarray = VariantArray::new();
        gdarray.resize(ArrayMesh::ARRAY_MAX as i32);
        gdarray.set(
            ArrayMesh::ARRAY_VERTEX as i32,
            Vector3Array::from_iter(self.vertices.iter().map(|v| vec3!(v))),
        );
        gdarray.set(
            ArrayMesh::ARRAY_NORMAL as i32,
            Vector3Array::from_iter(self.normals.iter().map(|v| vec3!(v))),
        );
        gdarray.set(
            ArrayMesh::ARRAY_TEX_UV as i32,
            Vector2Array::from_iter(self.uvs.iter().map(|v| vec2!(v))),
        );
        gdarray.set(
            ArrayMesh::ARRAY_COLOR as i32,
            ColorArray::from_iter(self.colors.iter().map(|v| color!(v))),
        );
        gdarray
    }
}

/// Adds `array_data` to `mesh` as a new surface.
///
/// Returns the new surface's "surface index".
pub fn add_surface(array_data: VariantArray<gdnative::prelude::Shared>, mesh: &ArrayMesh) -> i64 {
    let surf_idx = mesh.get_surface_count();
    mesh.add_surface_from_arrays(
        gdnative::api::Mesh::PRIMITIVE_TRIANGLES,
        array_data,
        VariantArray::new().into_shared(),
        2194432,
    );
    surf_idx
}

// TODO: tests
