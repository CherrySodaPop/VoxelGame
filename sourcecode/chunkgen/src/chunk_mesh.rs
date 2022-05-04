use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use gdnative::{
    api::{ArrayMesh, ConcavePolygonShape, Material, ResourceLoader, SpatialMaterial, Texture},
    object::Ref,
    prelude::{Shared, Unique},
};

use crate::{
    block::{BlockID, BLOCK_MANAGER},
    chunk::ChunkData,
    constants::*,
    mesh::{Face, GDMeshData, MeshData, FACES},
    positions::{ChunkPos, LocalBlockPos},
};

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
    fn add_to_mesh(&self, mesh: &mut Ref<ArrayMesh, Unique>, surface_no: usize) {
        let gd_mesh_data: GDMeshData = (&self.mesh_data).into();
        gd_mesh_data.add_to(mesh);
        mesh.surface_set_material(surface_no as i64, self.create_material());
    }
}

/// Enum representing `ChunkData` that is either in the
/// "current" chunk, or outside of it.
///
/// Provides `.get` to handle getting blocks from either
/// type without having to deal with locks explicitly.
enum CheckingData<'a> {
    SameChunk(&'a RwLockReadGuard<'a, ChunkData>),
    DifferentChunk(Arc<RwLock<ChunkData>>),
}

impl<'a> CheckingData<'a> {
    /// Gets the block at `position`, handling locking if `self` is `DifferentChunk`.
    fn get(&self, position: LocalBlockPos) -> BlockID {
        match self {
            CheckingData::SameChunk(lock_guard) => lock_guard.get(position),
            CheckingData::DifferentChunk(arc) => arc.read().unwrap().get(position),
        }
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
    pub fn new_from_chunk_data(
        chunk_data: Arc<RwLock<ChunkData>>,
        loaded_chunks: HashMap<ChunkPos, Arc<RwLock<ChunkData>>>,
    ) -> Self {
        let mut chunk_mesh = Self::new();
        let chunk_data_arc = chunk_data;
        let chunk_data = chunk_data_arc.read().unwrap();
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let position = LocalBlockPos::new(x, y, z, chunk_data.position);
                    let block_id = chunk_data.get(position);
                    if block_id == 0 {
                        // This is an air block, it has no faces.
                        continue;
                    };
                    // Set which of the six block faces should be rendered depending on
                    // whether the block they're adjacent to is transparent or solid.
                    for face in &FACES {
                        let offset = face.normal.into();
                        let checking_position = match position.offset(offset) {
                            Ok(pos) => Ok(pos),
                            // The block position to check is outside of the current chunk.
                            Err(_) => position.offset_global(offset).map(LocalBlockPos::from),
                        };
                        let should_draw = match checking_position {
                            Ok(checking_position) => {
                                let checking_data =
                                    if checking_position.chunk == chunk_data.position {
                                        // The block is inside the chunk we're building the mesh for,
                                        // just wrap up the current chunk_data.
                                        Some(CheckingData::SameChunk(&chunk_data))
                                    } else {
                                        // The block is outside the chunk we're building a mesh for.
                                        loaded_chunks
                                            .get(&checking_position.chunk)
                                            .map(|arc| CheckingData::DifferentChunk(arc.clone()))
                                    };
                                if let Some(checking_data) = checking_data {
                                    BLOCK_MANAGER
                                        .transparent_blocks
                                        .contains(&checking_data.get(checking_position))
                                } else {
                                    // Draw faces that are adjacent to unloaded chunks.
                                    true
                                }
                            }
                            // Draw faces at the bottom (y=0) and top (y=516) of the world.
                            Err(_) => true,
                        };
                        if should_draw {
                            chunk_mesh.add_face(block_id, face, position);
                        };
                    }
                }
            }
        }
        chunk_mesh
    }
}
