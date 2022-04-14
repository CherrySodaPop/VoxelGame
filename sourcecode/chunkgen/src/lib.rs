use std::{
    collections::{BTreeMap, HashMap},
    isize,
    sync::{Mutex, Arc},
};

use chunk::ChunkData;
use gdnative::{
    api::{
        CollisionShape, ConcavePolygonShape, Material, Mesh, MeshInstance, OpenSimplexNoise,
        StaticBody, SurfaceTool,
    },
    prelude::*,
};
use positions::{ChunkPos, GlobalBlockPos};
// use world::World;

mod chunk;
mod constants;
mod macros;
mod mesh;
mod positions;
mod world;

use crate::macros::*;
use crate::mesh::*;
use crate::{constants::*, positions::LocalBlockPos};

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

    fn update_mesh_data(&mut self, mesh_data: &GDMeshData) {
        unsafe {
            self.collision
                .assume_safe()
                .set_shape(mesh_data.create_collision_shape());
            self.mesh.assume_safe().set_mesh(mesh_data.create_mesh());
        }
    }
}

struct Chunk {
    data: ChunkData,
    node: Instance<ChunkNode, Shared>,
}

struct ChunkGenerator {
    noise: Ref<OpenSimplexNoise, Unique>,
}

impl ChunkGenerator {
    fn new() -> Self {
        Self {
            noise: OpenSimplexNoise::new(),
        }
    }
    fn generate_block(&self, y: isize, terrain_peak: isize) -> BlockID {
        if y > terrain_peak {
            return 0;
        }
        let block_id = if y > 6 { 1 } else { 2 }; // TODO: implement actual block ID system
        block_id
    }
    fn get_terrain_peak(&self, x: isize, z: isize) -> isize {
        let noise_height: f64 = self.noise.get_noise_2dv(vec2!(x, z));
        let terrain_peak = ((CHUNK_SIZE_Y as f64) * ((noise_height / 2.0) + 0.5) * 0.1) as isize;
        terrain_peak
    }
    pub fn generate_chunk(&self, position: ChunkPos) -> ChunkData {
        println!("Generating terrain data for chunk {:?}", position);
        let mut terrain = [[[0u16; CHUNK_SIZE_Z]; CHUNK_SIZE_Y]; CHUNK_SIZE_X];
        let chunk_origin = position.origin();
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                let global_x = x as isize + chunk_origin.x;
                let global_z = z as isize + chunk_origin.z;
                let terrain_peak = self.get_terrain_peak(global_x, global_z);
                for y in 0..CHUNK_SIZE_Y {
                    terrain[x][y][z] = self.generate_block(y as isize, terrain_peak);
                }
            }
        }
        ChunkData { position, terrain }
    }
}

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
    generator: ChunkGenerator,
    #[property]
    material: Option<Ref<Material, Shared>>,
    #[property]
    initial_generation_area: Option<Rect2>,
}

#[derive(Debug, Clone)]
pub struct NotLoadedError;

impl std::fmt::Display for NotLoadedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The chunk attempting to be modified has not been loaded")
    }
}
impl std::error::Error for NotLoadedError {}

#[methods]
impl World {
    // constructor
    fn new(_owner: &Node) -> Self {
        World {
            // chunks: HashMap::new(),
            // generator: ChunkGenerator::new(),
            ..Default::default()
            // material: None,
            // initial_generation_area: None,
        }
    }

    /// Gets a block in _global_ space.
    ///
    /// Returns `None` if the block isn't loaded.
    fn get_block(&self, position: GlobalBlockPos) -> Option<BlockID> {
        let chunk_position = position.chunk();
        self.chunks
            .get(&chunk_position)
            .and_then(|chunk| Some(chunk.data.get(position.into())))
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
        self.update_mesh(&chunk);
        for chunk_position in local_position.borders() {
            match self.chunks.get(&chunk_position) {
                Some(chunk) => {
                    self.update_mesh(&chunk);
                }
                None => {}
            };
        }
        Ok(())
    }

    #[export]
    fn set_block_gd(&mut self, _owner: &Node, x: isize, y: isize, z: isize, to: BlockID) {
        self.set_block(GlobalBlockPos::new(x, y, z), to);
    }

    /// Returns `true` if `face` is visible (e.g. is not blocked by a
    /// solid block) at `position`.
    ///
    /// This checks in world space, meaning block faces on chunk borders
    /// won't always be marked as visible.
    fn is_face_visible(&self, position: GlobalBlockPos, face: &Face) -> bool {
        let check = position.offset(face.normal.into());
        match self.get_block(check) {
            Some(block_id) => block_id == 0,
            None => true,
        }
    }

    /// Creates `MeshData` for a given `ChunkData`.
    ///
    /// Handles things like chunk border face checking.
    fn create_mesh_data(&self, chunk_data: &ChunkData) -> MeshData {
        println!("Building mesh data for {:?}", chunk_data);
        let mut mesh_data = MeshData::new();
        for x in 0..CHUNK_SIZE_X {
            for y in 0..CHUNK_SIZE_Y {
                for z in 0..CHUNK_SIZE_Z {
                    let block_id = chunk_data.terrain[x as usize][y as usize][z as usize];
                    if block_id == 0 {
                        // This is an air block, it has no faces.
                        continue;
                    }
                    let local_position = LocalBlockPos::new(x, y, z, chunk_data.position);
                    for face in &FACES {
                        let face_visible = match chunk_data.is_face_visible(local_position, face) {
                            Ok(face_visible) => face_visible,
                            Err(_) => self.is_face_visible(local_position.into(), face),
                        };
                        if !face_visible {
                            continue;
                        }
                        mesh_data.add_face(
                            face,
                            [
                                local_position.x as isize,
                                local_position.y as isize,
                                local_position.z as isize,
                            ],
                        );
                    }
                }
            }
        }
        mesh_data
    }

    fn update_mesh(&self, chunk: &Chunk) {
        let mesh_data = self.create_mesh_data(&chunk.data);
        // HERE!
        unsafe { chunk.node.assume_safe() }
            .map_mut(|cn: &mut ChunkNode, _base| {
                cn.update_mesh_data(&mesh_data.into());
            })
            .unwrap();
    }

    fn update_nearby(&self, position: ChunkPos) {
        let adjacent = position.adjacent();
        for adj_position in adjacent {
            match self.chunks.get(&adj_position) {
                Some(chunk) => {
                    self.update_mesh(chunk);
                }
                None => {}
            }
        }
    }

    fn load_chunk(&self, position: ChunkPos) -> Chunk {
        let chunk_data = if false {
            todo!("Implement loading chunks from disk");
        } else {
            // The chunk is new.
            self.generator.generate_chunk(position)
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
        let chunk = Chunk {
            data: chunk_data,
            node: chunk_node,
        };
        chunk
    }

    fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.data.position, chunk);
    }

    #[export]
    fn load_chunk_gd(&mut self, _owner: &Node, chunk_position: Vector2) {
        let chunk_position = ChunkPos::new(chunk_position.x as isize, chunk_position.y as isize);
        if self.chunks.contains_key(&chunk_position) {
            return;
        }
        let chunk = self.load_chunk(chunk_position);
        self.update_mesh(&chunk);
        // _owner.add_child(&chunk.node, true);
        unsafe {
            _owner.call_deferred("add_child", &[chunk.node.to_variant()]);
        }

        self.add_chunk(chunk);
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
        for x in generate_range.x1..=generate_range.x2 {
            for z in generate_range.y1..=generate_range.y2 {
                self.add_chunk(self.load_chunk(ChunkPos::new(x, z)));
            }
        }
        for chunk in self.chunks.values() {
            self.update_mesh(chunk);
            _owner.add_child(&chunk.node, true);
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            chunks: Default::default(),
            generator: ChunkGenerator::new(),
            material: Default::default(),
            initial_generation_area: Default::default(),
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<World>();
    handle.add_class::<ChunkNode>();
}

godot_init!(init);
