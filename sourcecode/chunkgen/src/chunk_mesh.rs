use std::collections::HashMap;

use gdnative::api::{
    ArrayMesh, ConcavePolygonShape, Material, ResourceLoader, SpatialMaterial, Texture,
};
use gdnative::object::Ref;
use gdnative::prelude::{Shared, Unique};

use crate::block::BlockID;
use crate::chunk::ChunkData;
use crate::constants::*;
use crate::mesh::{Face, GDMeshData, MeshData, FACES};
use crate::positions::LocalBlockPos;

/// Block-type specific `MeshData`, to allow for different block types
/// to have their own specific materials.
///
/// Provides convenience functions like `BlockSurface.add_to_mesh` for
/// adding to an `ArrayMesh` as a new surface, with the material specific
/// to `self.block_id`.
struct BlockSurface {
    mesh_data: MeshData,
    block_id: BlockID,
}

impl BlockSurface {
    fn new(block_id: BlockID) -> Self {
        Self {
            mesh_data: MeshData::new(),
            block_id,
        }
    }
    /// Loads the texture for `self.block_id` from the game assets.
    fn get_albedo_texture(&self) -> Ref<Texture, Shared> {
        let tex_path = format!("res://assets/textures/blocks/{}.png", self.block_id);
        let resource_loader = ResourceLoader::godot_singleton();
        // TODO: Fail more gracefully if the texture isn't found
        let texture: Ref<Texture, Shared> = resource_loader
            .load(tex_path, "", false)
            .unwrap()
            .cast()
            .unwrap();
        unsafe { texture.assume_safe() }.set_flags(Texture::FLAGS_DEFAULT ^ Texture::FLAG_FILTER);
        texture
    }
    /// Creates the block-type-specific material for this surface.
    fn create_material(&self) -> Ref<Material, Shared> {
        // TODO: This means we're going to be making a new material per-chunk, per-block-type.
        //       That is, quite obviously, not good for performance or ergonomics.
        let resource_loader = ResourceLoader::godot_singleton();
        // Check if a custom material for this block type exists in `assets/materials`.
        let material_path = format!("res://assets/materials/{}.tres", self.block_id); // HARDCODED
        let material = if resource_loader.exists(&material_path, "") {
            // Prevent Godot error spam by checking for the material before attempting
            // to load it.
            resource_loader.load(material_path, "", false)
        } else {
            None
        };
        match material {
            Some(material) => material.cast().unwrap(),
            None => {
                // Make a new material containing the block's texture.
                let material = SpatialMaterial::new();
                material.set_texture(SpatialMaterial::TEXTURE_ALBEDO, self.get_albedo_texture());
                material.upcast::<Material>().into_shared()
            }
        }
    }
    /// Adds this `BlockSurface` to `mesh` as a surface,
    /// setting the surface's material depending on `self.block_id`.
    fn add_to_mesh(&self, mesh: &mut Ref<ArrayMesh, Unique>, surface_no: usize) {
        let gd_mesh_data: GDMeshData = (&self.mesh_data).into();
        gd_mesh_data.add_to(mesh);
        mesh.surface_set_material(surface_no as i64, self.create_material());
    }
}

/// Chunk mesh information, such as vertices.
///
/// Mesh information is stored per-block-type as `BlockSurface`s.
pub struct ChunkMeshData {
    surfaces: HashMap<BlockID, BlockSurface>,
}

impl ChunkMeshData {
    fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
        }
    }
    /// Adds a block face to this `ChunkMeshData`, putting it in the appropriate `BlockSurface`.
    fn add_face(&mut self, block_id: BlockID, face: &Face, position: LocalBlockPos) {
        let block_surface = self
            .surfaces
            .entry(block_id)
            .or_insert_with(|| BlockSurface::new(block_id));
        block_surface.mesh_data.add_face_with_uv(
            face,
            [
                position.x as isize,
                position.y as isize,
                position.z as isize,
            ],
        );
    }
    /// Constructs an `ArrayMesh` from this `ChunkMeshData`.
    pub fn build_mesh(&self) -> Ref<ArrayMesh, Unique> {
        let mut mesh = ArrayMesh::new();
        for (i, block_surface) in self.surfaces.values().enumerate() {
            block_surface.add_to_mesh(&mut mesh, i);
        }
        mesh
    }
    /// Constructs a `ConcavePolygonShape` from this `ChunkMeshData`.
    pub fn build_collision_shape(&self) -> Ref<ConcavePolygonShape, Unique> {
        // Pool every vertex from every block type in the mesh data together.
        let all_vertices: Vec<[isize; 3]> = self
            .surfaces
            .values()
            // TODO: No clone >:(
            .flat_map(|bs| bs.mesh_data.vertices.clone())
            .collect();
        let collision_shape = ConcavePolygonShape::new();
        collision_shape.set_faces(GDMeshData::convert_vec3(&all_vertices));
        collision_shape
    }
}

/// Struct for creating `ChunkMeshData`.
///
/// As the face-checking process can be quite involved, this is
/// a seperate type from `ChunkMeshData` itself.
//  ^ Could we fix this somehow (i.e. merge the two structs) using some wacky Rust stuff like traits?
pub struct ChunkMeshDataGenerator<'a> {
    chunk_data: &'a ChunkData,
    world: &'a crate::World,
}

impl<'a> ChunkMeshDataGenerator<'a> {
    pub fn new(world: &'a crate::World, chunk_data: &'a ChunkData) -> Self {
        Self { chunk_data, world }
    }
    /// Checks to see if `face` of the block at `local_position` should be visible.
    ///
    /// Handles checking for faces on chunk boundaries.
    fn face_visible(&self, local_position: LocalBlockPos, face: &Face) -> bool {
        match self.chunk_data.is_face_visible(local_position, face) {
            Ok(face_visible) => face_visible,
            Err(_) => self.world.is_face_visible(local_position.into(), face),
        }
    }
    /// Adds a block's visible faces to `chunk_mesh`.
    fn add_block(&self, chunk_mesh: &mut ChunkMeshData, position: LocalBlockPos) {
        let block_id = self.chunk_data.terrain[position.x][position.y][position.z];
        if block_id == 0 {
            // This is an air block, it has no faces.
            return;
        }
        for face in &FACES {
            let face_visible = self.face_visible(position, face);
            if face_visible {
                chunk_mesh.add_face(block_id, face, position);
            }
        }
    }
    /// Builds the final `ChunkMeshData`.
    pub fn generate(&self) -> ChunkMeshData {
        let mut chunk_mesh = ChunkMeshData::new();
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let position = LocalBlockPos::new(x, y, z, self.chunk_data.position);
                    self.add_block(&mut chunk_mesh, position);
                }
            }
        }
        chunk_mesh
    }
}
