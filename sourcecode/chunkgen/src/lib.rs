#![allow(dead_code)]
// ^ Prevents "unused function" warnings, those functions will
// likely either be used in the future or by GDScript.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use gdnative::{
    api::{CollisionShape, MeshInstance, StaticBody},
    prelude::*,
};

mod block;
mod chunk;
mod chunk_mesh;
mod constants;
mod features;
mod generate;
mod macros;
mod mesh;
mod performance;
mod positions;

use crate::{
    block::BlockID, chunk::ChunkData, chunk_mesh::ChunkMeshData, generate::ChunkGenerator,
    macros::*, positions::*,
};

#[derive(Debug, Clone)]
pub struct NotLoadedError;

impl std::fmt::Display for NotLoadedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The chunk being modified has not been loaded")
    }
}
impl std::error::Error for NotLoadedError {}

pub struct Chunk {
    pub data: Arc<RwLock<ChunkData>>,
    node: Arc<Mutex<Instance<ChunkNode, Shared>>>,
}

#[derive(NativeClass)]
#[export]
#[inherit(StaticBody)]
#[user_data(gdnative::export::user_data::MutexData<ChunkNode>)]
struct ChunkNode {
    owner: Ref<StaticBody, Shared>,
    collision: Ref<CollisionShape, Shared>,
    mesh: Ref<MeshInstance, Shared>,
}

#[methods]
impl ChunkNode {
    fn new(owner: &StaticBody) -> Self {
        let collision = CollisionShape::new();
        let mesh = MeshInstance::new();
        let (collision, mesh) = (collision.into_shared(), mesh.into_shared());
        owner.add_child(collision, true);
        owner.add_child(mesh, true);
        ChunkNode {
            owner: unsafe { owner.assume_shared() },
            collision,
            mesh,
        }
    }

    fn update_mesh_data(&mut self, mesh_data: ChunkMeshData) {
        let collision = unsafe { self.collision.assume_safe() };
        let mesh = unsafe { self.mesh.assume_safe() };
        let build_collision_shape = mesh_data.build_collision_shape();
        unsafe {
            // This MUST be call_deferred, setting the shape when using the
            // Bullet physics engine is NOT thread-safe!
            collision.call_deferred(
                "set_shape",
                &[build_collision_shape.into_shared().to_variant()],
            );
        };
        mesh.set_mesh(mesh_data.build_mesh());
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
            .map(|chunk| chunk.data.read().unwrap().get(position.into()))
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
            chunk.data.write().unwrap().set(local_position, to);
        };
        let chunk = self
            .chunks
            .get(&local_position.chunk)
            .ok_or(NotLoadedError)?;
        self.update_mesh(chunk);
        for chunk_position in local_position.borders() {
            if let Some(chunk) = self.chunks.get(&chunk_position) {
                self.update_mesh(chunk);
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

    /// Returns a "view" into `World.chunks`, mapping `ChunkPos`s to `&ChunkData`s.
    fn chunk_data_view(&self) -> HashMap<ChunkPos, Arc<RwLock<ChunkData>>> {
        self.chunks
            .iter()
            .map(|(cp, c)| (*cp, c.data.clone()))
            .collect()
    }

    /// Updates the mesh for a specific `Chunk`.
    fn update_mesh(&self, chunk: &Chunk) {
        println!(
            "Updating mesh data for {:?}",
            chunk.data.read().unwrap().position
        );
        let view = self.chunk_data_view();
        let chunk_node = chunk.node.clone();
        let chunk_data = chunk.data.clone();
        // TODO: Somehow use a single thread that gets sent chunks to build meshes for,
        //       rather than creating a new one for each individually.
        std::thread::spawn(move || {
            let mesh_data = ChunkMeshData::new_from_chunk_data(chunk_data, view);
            let chunk_node = chunk_node.lock().unwrap();
            unsafe { chunk_node.assume_safe() }
                .map_mut(|cn: &mut ChunkNode, _base| {
                    cn.update_mesh_data(mesh_data);
                })
                .unwrap();
        });
    }

    /// Updates the meshes of chunks adjacent to `position`.
    fn update_nearby(&self, position: ChunkPos) {
        // TODO: This is mildly broken, may be related to threading?
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
    fn load_chunk(&mut self, position: ChunkPos) -> Chunk {
        let chunk_data = if false {
            todo!("Implement loading chunks from disk");
        } else {
            // The chunk is new.
            self.chunk_generator.generate_chunk(position)
        };
        self.chunk_generator
            .apply_waitlist(&mut self.chunk_data_view());
        let chunk_node = ChunkNode::new_instance();
        // Ugly godot-rust stuff, moves the chunk node into place.
        let chunk_node = chunk_node.into_shared();
        Chunk {
            data: Arc::new(RwLock::new(chunk_data)),
            node: Arc::new(Mutex::new(chunk_node)),
        }
    }

    /// Takes ownership of `chunk` and adds it to the `World.chunks` HashMap.
    ///
    /// This allows other chunks to see it when making face calculations,
    /// and for functions such as `World.set_block` to be able to modify it.
    fn add_chunk(&mut self, chunk: Chunk) {
        let chunk_position = chunk.data.read().unwrap().position;
        self.chunks.insert(chunk_position, chunk);
    }

    #[export]
    fn load_chunk_gd(&mut self, owner: &Node, chunk_position: Vector2) {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        if self.chunks.contains_key(&chunk_position) {
            // The chunk is already loaded.
            return;
        }
        let chunk = self.load_chunk(chunk_position);
        self.update_mesh(&chunk);
        unsafe {
            // let chunk = &chunk.read().unwrap();
            chunk
                .node
                .lock()
                .unwrap()
                .assume_safe()
                .map_mut(|_cn: &mut ChunkNode, base| {
                    let chunk_origin = chunk.data.read().unwrap().position.origin();
                    base.translate(vec3!(chunk_origin.x, chunk_origin.y, chunk_origin.z));
                })
                .unwrap();
            owner.call_deferred("add_child", &[chunk.node.lock().unwrap().to_variant()]);
        }
        self.add_chunk(chunk);
        // Update the meshes of nearby chunks to remove now-hidden faces
        self.update_nearby(chunk_position);
    }

    #[export]
    fn load_around_chunk_gd(&mut self, owner: &Node, chunk_position: Vector2) {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        let mut around = Vec::new();
        for x in -2..=2 {
            for z in -2..=2 {
                around.push(ChunkPos::new(chunk_position.x + x, chunk_position.z + z));
            }
        }
        for chunk_pos in around {
            self.load_chunk_gd(owner, vec2!(chunk_pos.x, chunk_pos.z));
        }
    }

    #[export]
    fn _ready(&mut self, owner: &Node) {
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
                self.load_chunk_gd(owner, vec2!(x, z));
            }
        }
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
