use std::{collections::BTreeMap, isize};

use chunk::Chunk;
use gdnative::{
    api::{
        CollisionShape, ConcavePolygonShape, Material, Mesh, MeshInstance, OpenSimplexNoise,
        StaticBody, SurfaceTool,
    },
    prelude::*,
};
use positions::ChunkPos;
use world::World;

mod chunk;
mod constants;
mod mesh;
mod positions;
mod world;

use crate::constants::*;
use crate::mesh::*;

// For UV calculations, hence f32.
const UV_TEXTURE_WIDTH: f32 = 256.0;
const TEXTURE_WIDTH: f32 = 16.0;

// TODO: Make it an actual node
struct ChunkNode<'a> {
    chunk: &'a Chunk,
    spatial: Ref<StaticBody, Unique>,
    collision: Ref<CollisionShape, Unique>,
    mesh: Ref<MeshInstance, Unique>,
}

impl<'a> std::fmt::Debug for ChunkNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkNode")
            .field("position", &self.chunk.position)
            .finish()
    }
}

// Chunk generator implementation
#[derive(NativeClass, Default)]
#[export]
#[inherit(Node)]
pub struct ChunkGenerator {
    world: World,
    #[property]
    material: Option<Ref<Material, Shared>>,
    #[property]
    initial_generation_area: Option<Rect2>,
}

#[methods]
impl ChunkGenerator {
    // constructor
    fn new(_owner: &Node) -> Self {
        ChunkGenerator {
            world: World::new(),
            ..Default::default()
        }
    }

    /*
    // get the block id
    fn world_block(&self, block_position: BlockPosition) -> u16 {
        let chunk_origin = block_position.chunk;
        let chunk = self.chunks.get(&chunk_origin);
        if let Some(chunk) = chunk {
            let chunk_coords = block_position.local_position();
            chunk.terrain[chunk_coords[0]][chunk_coords[1]][chunk_coords[2]]
        } else {
            0
        }
    }

    #[export]
    // constructs the specified chunk mesh - godot specific
    fn generate_chunk_mesh(&mut self, _owner: &Node, _origin: Vector2) {
        let origin: [isize; 2] = [_origin.x as isize, _origin.y as isize];
        //let origin = _origin.as_slice();
        let _chunk = self.chunks.get(&origin);
        if let Some(_chunk) = _chunk {
            _chunk.construct_mesh(self);
        } else {
            godot_print!("chunkgeneration: attempted to generate unloaded chunk mesh!");
        }
    }

    #[export]
    // check if the chunk is loaded or not - godot specific
    fn chunk_loaded_godot(&self, _owner: &Node, _origin: Vector2) -> bool {
        let origin: [isize; 2] = [_origin.x as isize, _origin.y as isize];
        self.chunk_loaded(origin)
    }

    // check if the chunk is loaded or not - internal
    fn chunk_loaded(&self, _origin: [isize; 2]) -> bool {
        self.chunks.contains_key(&_origin)
    }

    // set block - godot
    #[export]
    fn set_block_godot(&mut self, _owner: &Node, _origin: Vector3, block_id: u16) {
        let origin: [isize; 3] = [_origin.x as isize, _origin.y as isize, _origin.z as isize];
        self.set_block(origin, block_id);
    }

    // set block - internal
    fn set_block(&mut self, _origin: [isize; 3], block_id: u16) {
        let block_data = BlockPosition::new(_origin[0], _origin[1], _origin[2]);
        let block_chunk_pos = block_data.chunk;

        if self.chunk_loaded(block_chunk_pos) {
            let chunk = self.chunks.get_mut(&block_chunk_pos).unwrap();
            let block_local_pos = block_data.local_position();
            chunk.set_block(block_local_pos, block_id);
            return;
        }
        godot_print!("chunkgeneration: attempting to set block on unloaded chunk")
    }

    #[export]
    // returns chunk node - godot specific
    fn chunk_node_id_gd(&self, _owner: &Node, _origin: Vector2) -> i64 {
        let origin: [isize; 2] = [_origin.x as isize, _origin.y as isize];
        let _chunk = self.chunks.get(&origin);
        let _chunk = _chunk.unwrap();
        _chunk.spatial.get_instance_id()
    }

    */

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        // generate chunks
        let simplex_noise = OpenSimplexNoise::new();
        let [start, end] = if let Some(initial_generation_area) = self.initial_generation_area {
            let x = initial_generation_area.position.x as isize;
            let y = initial_generation_area.position.y as isize;
            let w = initial_generation_area.size.x as isize;
            let h = initial_generation_area.size.y as isize;
            [[x, y], [x + w, y + h]]
        } else {
            godot_warn!("Missing default chunk generation rect, using (-2, -2) w=4 h=4");
            [[-2, -2], [2, 2]]
        };
        self.world.generate_rect(start, end, &*simplex_noise);

        // generate mesh (to be removed! - cherry)
        for chunk in self.world.chunks() {
            let mut chunk_node = ChunkNode::new(chunk);
            chunk_node.construct_mesh(&self.world);
            chunk_node.apply_mesh();
            // TODO: Move this to the ChunkNode.apply_mesh method
            unsafe {
                _owner.add_child(chunk_node.spatial.assume_shared(), true);
            }
        }
    }
}

// Chunk implementation
impl<'a> ChunkNode<'a> {
    fn new(chunk: &'a Chunk) -> Self {
        let spatial = StaticBody::new();
        let collision = CollisionShape::new();
        let mesh = MeshInstance::new();
        let spatial_transform = Self::spatial_transform(chunk.position.x, chunk.position.z);
        spatial.set_transform(spatial.transform().translated(Vector3::new(
            spatial_transform[0] as f32,
            spatial_transform[1] as f32,
            spatial_transform[2] as f32,
        )));
        ChunkNode {
            chunk,
            spatial,
            collision,
            mesh,
        }
    }

    fn spatial_transform(x: isize, z: isize) -> [isize; 3] {
        [x * CHUNK_SIZE_X as isize, 0, z * CHUNK_SIZE_Z as isize]
    }

    // fn construct_mesh(&mut self, world: &World) {
    fn construct_mesh(&mut self, world: &World) {
        let gd_mesh_data: GDMeshData = build_mesh_data(&self.chunk, world).into();
        let mesh = create_mesh(&gd_mesh_data);
        let collision_shape = create_collision_shape(gd_mesh_data);
        self.mesh.set_mesh(mesh);
        self.collision.set_shape(collision_shape);
    }

    fn apply_mesh(&mut self) {
        unsafe {
            self.spatial.add_child(self.collision.assume_shared(), true);
            self.spatial.add_child(self.mesh.assume_shared(), true);
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ChunkGenerator>();
}

godot_init!(init);
