#![allow(dead_code)]
// ^ Prevents "unused function" warnings, those functions will
// likely either be used in the future or by GDScript.

use std::{collections::HashMap, isize};

use gdnative::{
    api::{CollisionShape, MeshInstance, StaticBody},
    prelude::*,
};

mod block;
mod chunk;
mod chunk_mesh;
mod constants;
mod generate;
mod macros;
mod mesh;
mod performance;
mod positions;

use crate::{
    block::BlockID,
    chunk::ChunkData,
    chunk_mesh::{ChunkMeshData, ChunkMeshDataGenerator},
    generate::ChunkGenerator,
    macros::*,
    mesh::*,
    performance::Timings,
    positions::*,
};

#[derive(Debug, Clone)]
pub struct NotLoadedError;

impl std::fmt::Display for NotLoadedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The chunk being modified has not been loaded")
    }
}
impl std::error::Error for NotLoadedError {}

struct Chunk {
    data: ChunkData,
    node: Instance<ChunkNode, Shared>,
}

#[derive(NativeClass)]
#[export]
#[inherit(StaticBody)]
#[user_data(gdnative::export::user_data::MutexData<ChunkNode>)]
struct ChunkNode {
    collision: Ref<CollisionShape, Shared>,
    mesh: Ref<MeshInstance, Shared>,
}

#[methods]
impl ChunkNode {
    fn new(owner: &StaticBody) -> Self {
        let collision = CollisionShape::new().into_shared();
        let mesh = MeshInstance::new().into_shared();
        owner.add_child(collision, true);
        owner.add_child(mesh, true);
        ChunkNode { collision, mesh }
    }

    fn update_mesh_data(&mut self, mesh_data: ChunkMeshData) {
        unsafe {
            self.collision
                .assume_safe()
                .set_shape(mesh_data.build_collision_shape());
            self.mesh.assume_safe().set_mesh(mesh_data.build_mesh());
        }
    }
}

/// Helper struct for `Rect2` -> `(x1, y1, x2, y2)` conversion
struct PositionRange {
    x1: isize,
    y1: isize,
    x2: isize,
    y2: isize,
}

impl From<Rect2> for PositionRange {
    fn from(rect2: Rect2) -> Self {
        println!("{:?}", rect2);
        let x = rect2.position.x as isize;
        let y = rect2.position.y as isize;
        let w = rect2.size.x as isize;
        let h = rect2.size.y as isize;
        Self {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

#[derive(NativeClass)]
#[export]
#[inherit(Node)]
#[user_data(gdnative::export::user_data::MutexData<World>)]
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
    chunk_generator: ChunkGenerator,
    #[property]
    initial_generation_area: Option<Rect2>,
}

#[methods]
impl World {
    // constructor
    fn new(_owner: &Node) -> Self {
        World {
            ..Default::default()
        }
    }

    /// Gets a block in _global_ space.
    ///
    /// Returns `None` if the block isn't loaded.
    fn get_block(&self, position: GlobalBlockPos) -> Option<BlockID> {
        let chunk_position = position.chunk();
        self.chunks
            .get(&chunk_position)
            .map(|chunk| chunk.data.get(position.into()))
    }

    /// Sets a block in _global_ space. This will update the
    /// chunk the block is in, as well as its neighbors if necessary.
    ///
    /// Returns `NotLoadedError` if the block isn't loaded.
    fn set_block(&mut self, position: GlobalBlockPos, to: BlockID) -> Result<(), NotLoadedError> {
        let local_position: LocalBlockPos = position.into();
        // Rust foolishness to prevent mutability mishaps
        {
            let chunk = self
                .chunks
                .get_mut(&local_position.chunk)
                .ok_or(NotLoadedError)?;
            chunk.data.set(local_position, to);
        };
        let chunk = self
            .chunks
            .get(&local_position.chunk)
            .ok_or(NotLoadedError)?;
        self.update_mesh(chunk);
        for chunk_position in local_position.borders() {
            if let Some(chunk) = self.chunks.get(&chunk_position) {
                self.update_mesh(chunk)
            }
        }
        Ok(())
    }

    #[export]
    fn set_block_gd(&mut self, _owner: &Node, position: Vector3, to: BlockID) {
        let position = GlobalBlockPos::new(
            position.x as isize,
            position.y as isize,
            position.z as isize,
        );
        self.set_block(position, to);
    }

    #[export]
    fn get_block_gd(&mut self, _owner: &Node, position: Vector3) -> Option<BlockID> {
        let position = GlobalBlockPos::new(
            position.x as isize,
            position.y as isize,
            position.z as isize,
        );
        self.get_block(position)
    }

    /// Returns `true` if `face` is visible (e.g. is not blocked by a
    /// solid block) at `position`.
    ///
    /// This checks in world space, meaning block faces checked on chunk borders
    /// will be accurate.
    fn is_face_visible(&self, position: GlobalBlockPos, face: &Face) -> bool {
        let check = position.offset(face.normal.into());
        match self.get_block(check) {
            Some(block_id) => block_id == 0,
            None => true,
        }
    }

    /// Updates the mesh for a specific `Chunk`.
    fn update_mesh(&self, chunk: &Chunk) {
        let mesh_generator = ChunkMeshDataGenerator::new(self, &chunk.data);
        let mesh_data = mesh_generator.generate();
        // TODO: This function seems to cause issues when using multithreading.
        unsafe { chunk.node.assume_safe() }
            .map_mut(|cn: &mut ChunkNode, _base| {
                cn.update_mesh_data(mesh_data);
            })
            .unwrap();
    }

    /// Updates the meshes of chunks adjacent to `position`.
    fn update_nearby(&self, position: ChunkPos) {
        let adjacent = position.adjacent();
        for adj_position in adjacent {
            if let Some(chunk) = self.chunks.get(&adj_position) {
                self.update_mesh(chunk);
            }
        }
    }

    /// Loads a chunk from disk, or generates a new one.
    ///
    /// This does not create the mesh, see `World.update_mesh`.
    fn load_chunk(&self, position: ChunkPos) -> Chunk {
        let chunk_data = if false {
            todo!("Implement loading chunks from disk");
        } else {
            // The chunk is new.
            self.chunk_generator.generate_chunk(position)
        };
        let chunk_node = ChunkNode::new_instance();
        // Ugly godot-rust stuff, moves the chunk node into place.
        chunk_node
            .map_mut(|_cn: &mut ChunkNode, base| {
                let chunk_origin = position.origin();
                base.translate(vec3!(chunk_origin.x, chunk_origin.y, chunk_origin.z));
            })
            .unwrap();
        let chunk_node = chunk_node.into_shared();
        Chunk {
            data: chunk_data,
            node: chunk_node,
        }
    }

    /// Takes ownership of `chunk` and adds it to the `World.chunks` HashMap.
    ///
    /// This allows other chunks to see it when making face calculations,
    /// and for functions such as `World.set_block` to be able to modify it.
    fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.data.position, chunk);
    }

    #[export]
    fn load_chunk_gd(&mut self, _owner: &Node, chunk_position: Vector2) {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        if self.chunks.contains_key(&chunk_position) {
            // The chunk is already loaded.
            return;
        }
        let chunk = self.load_chunk(chunk_position);
        self.update_mesh(&chunk);
        _owner.add_child(&chunk.node, true);
        self.add_chunk(chunk);
        // Update the meshes of nearby chunks to remove now-hidden faces
        self.update_nearby(chunk_position);
    }

    #[export]
    fn load_around_chunk_gd(&mut self, _owner: &Node, chunk_position: Vector2) {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        let mut around = Vec::new();
        for x in -2..=2 {
            for z in -2..=2 {
                around.push(ChunkPos::new(chunk_position.x + x, chunk_position.z + z));
            }
        }
        for chunk_pos in around {
            self.load_chunk_gd(_owner, vec2!(chunk_pos.x, chunk_pos.z));
        }
    }

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        let mut timings = Timings::new();
        let generate_range = if let Some(initial_generation_area) = self.initial_generation_area {
            initial_generation_area.into()
        } else {
            godot_warn!("Missing default chunk generation rect, using (-2, -2) w=4 h=4");
            PositionRange {
                x1: -2,
                y1: -2,
                x2: 2,
                y2: 2,
            }
        };
        // Generate some initial "spawn area" chunks.
        for x in generate_range.x1..=generate_range.x2 {
            for z in generate_range.y1..=generate_range.y2 {
                let start_time = std::time::Instant::now();
                let loaded_chunk = self.load_chunk(ChunkPos::new(x, z));
                timings.generate_chunk.push(start_time.elapsed());
                self.add_chunk(loaded_chunk);
            }
        }
        for chunk in self.chunks.values() {
            let start_time = std::time::Instant::now();
            self.update_mesh(chunk);
            timings.build_mesh.push(start_time.elapsed());
            _owner.add_child(&chunk.node, true);
        }
        println!("{}", timings);
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            chunks: Default::default(),
            chunk_generator: ChunkGenerator::new(),
            initial_generation_area: Default::default(),
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<World>();
    handle.add_class::<ChunkNode>();
}

godot_init!(init);
