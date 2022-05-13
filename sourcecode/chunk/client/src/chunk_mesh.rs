//! Chunk mesh creation facilities.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use gdnative::{
    api::{ArrayMesh, ConcavePolygonShape, Material, ResourceLoader, SpatialMaterial, Texture},
    object::Ref,
    prelude::{Shared, Unique},
};

use chunkcommon::{
    block::BLOCK_MANAGER,
    chunk::ChunkData,
    errors::{NotLoadedError, OffsetError},
    prelude::*,
};

use crate::mesh::{Face, GDMeshData, MeshData, FACES};

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
                material.set_flag(SpatialMaterial::FLAG_ALBEDO_FROM_VERTEX_COLOR, true);
                material.set_flag(SpatialMaterial::FLAG_DISABLE_AMBIENT_LIGHT, true);
                material.upcast::<Material>().into_shared()
            }
        }
    }
    /// Adds this `BlockSurface` to `mesh` as a surface,
    /// setting the surface's material depending on `self.block_id`.
    fn add_to_mesh(&self, mesh: &mut Ref<ArrayMesh, Unique>) {
        let gd_mesh_data: GDMeshData = (&self.mesh_data).into();
        let surf_idx = gd_mesh_data.add_to(mesh);
        mesh.surface_set_material(surf_idx, self.create_material());
    }
}

/// Gets the block at `position` from `position.chunk` if it is present in `loaded_chunks`.
fn get_global(
    loaded_chunks: &HashMap<ChunkPos, Arc<RwLock<ChunkData>>>,
    position: LocalBlockPos,
) -> Result<BlockID, NotLoadedError> {
    loaded_chunks
        .get(&position.chunk)
        .map(|data| data.read().unwrap().get(position))
        .ok_or(NotLoadedError)
}

/// Returns `true` if:
/// * The face is adjacent to a block that is transparent
/// * The face is adjacent to a block that is in an unloaded chunk
/// * The face is adjacent to a block that is outside of y=`0..512`
///
/// Returns `false` if:
/// * The face is adjacent to a non-transparent block
fn should_draw_face(
    face: &Face,
    chunk_data: &ChunkData,
    loaded_chunks: &HashMap<ChunkPos, Arc<RwLock<ChunkData>>>,
    position: LocalBlockPos,
) -> bool {
    let offset = face.normal.into();
    let block_id = match position.offset(offset) {
        Ok(position) => chunk_data.get(position),
        // Draw faces at the bottom (y=0) and top (y=512) of the world.
        Err(OffsetError::OutOfBounds) => return true,
        Err(OffsetError::DifferentChunk) => {
            let position = position
                .offset_global(offset)
                .map(LocalBlockPos::from)
                // This is safe to unwrap because if it was an out-of-bounds position
                // we would have already caught that above.
                .unwrap();
            match get_global(loaded_chunks, position) {
                Ok(block_id) => block_id,
                // Draw faces adjacent to unloaded chunks.
                Err(_) => return true,
            }
        }
    };
    BLOCK_MANAGER.transparent_blocks.contains(&block_id)
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
        block_surface.mesh_data.add_face(
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
        for block_surface in self.surfaces.values() {
            block_surface.add_to_mesh(&mut mesh);
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
    pub fn new_from_chunk_data(
        chunk_data: Arc<RwLock<ChunkData>>,
        loaded_chunks: HashMap<ChunkPos, Arc<RwLock<ChunkData>>>,
    ) -> Self {
        let mut chunk_mesh = Self::new();
        let chunk_data = chunk_data.read().unwrap();
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let position = LocalBlockPos::new(x, y, z, chunk_data.position);
                    let block_id = chunk_data.get(position);
                    if block_id == 0 {
                        // This is an air block, it has no faces.
                        continue;
                    };
                    for face in &FACES {
                        if should_draw_face(face, &*chunk_data, &loaded_chunks, position) {
                            chunk_mesh.add_face(block_id, face, position);
                        };
                    }
                }
            }
        }
        chunk_mesh
    }
}
