use std::{collections::BTreeMap, isize};

use gdnative::{
    api::{
        CollisionShape, ConcavePolygonShape, Material, Mesh, MeshInstance, OpenSimplexNoise,
        StaticBody, SurfaceTool,
    },
    prelude::*,
};
use positions::ChunkPos;

mod constants;
mod mesh;
mod positions;

use crate::constants::*;
use crate::mesh::*;

// For UV calculations, hence f32.
const UV_TEXTURE_WIDTH: f32 = 256.0;
const TEXTURE_WIDTH: f32 = 16.0;

// TODO: Make it an actual node
struct ChunkNode {
    chunk: Chunk,
    spatial: Ref<StaticBody, Unique>,
    collision: Ref<CollisionShape, Unique>,
    mesh: Ref<MeshInstance, Unique>,
}

impl std::fmt::Debug for ChunkNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("origin", &self.chunk)
            .finish()
    }
}

// Chunk generator implementation
#[derive(NativeClass, Default)]
#[export]
#[inherit(Node)]
pub struct ChunkGenerator {
    chunks: BTreeMap<[isize; 2], ChunkNode>,
    #[property]
    material: Option<Ref<Material, Shared>>,
}

#[methods]
impl ChunkGenerator {
    // constructor
    fn new(_owner: &Node) -> Self {
        ChunkGenerator {
            chunks: BTreeMap::new(),
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
        for x in -4..5isize {
            for z in -4..5isize {
                let origin = [x, z];
                let mut new_chunk = ChunkNode::new(x, z);
                godot_print!("Generating new chunk {:?}", new_chunk);
                new_chunk.generate(&*simplex_noise);
                self.chunks.insert(origin, new_chunk);
            }
        }
        // generate mesh (to be removed! - cherry)

        for chunk in self.chunks.values_mut() {
            godot_print!("Constructing mesh for {:?}", chunk);
            chunk.construct_mesh();
            unsafe {
                chunk
                    .spatial
                    .add_child(chunk.collision.assume_shared(), true);
                chunk.spatial.add_child(chunk.mesh.assume_shared(), true);
                _owner.add_child(chunk.spatial.assume_shared(), true);
            }
        }
    }
}

// Chunk implementation
impl ChunkNode {
    fn new(x: isize, z: isize) -> Self {
        let chunk = Chunk::new(ChunkPos::new(x, z));
        let spatial = StaticBody::new();
        let collision = CollisionShape::new();
        let mesh = MeshInstance::new();
        let spatial_transform = Self::spatial_transform(x, z);
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
        [(x * CHUNK_SIZE_X as isize), 0, (z * CHUNK_SIZE_Z as isize)]
    }

    fn set_block(&mut self, pos: [usize; 3], block_id: u16) {
        // self.terrain[pos[0]][pos[1]][pos[2]] = block_id;
    }

    /// Generates the chunk's terrain data, storing it in `Chunk.terrain`.
    fn generate(&mut self, simplex_noise: &OpenSimplexNoise) {
        let chunk_origin = self.chunk.position.origin();
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                let world_block_x = x as isize + chunk_origin.x;
                let world_block_z = z as isize + chunk_origin.z;
                let noise_height: f64 = simplex_noise
                    .get_noise_2dv(Vector2::new(world_block_x as f32, world_block_z as f32));
                let terrain_peak =
                    ((CHUNK_SIZE_Y as f64) * ((noise_height / 2.0) + 0.5) * 0.1) as isize;
                for y in 0..CHUNK_SIZE_Y {
                    let y = y as isize;
                    if y > terrain_peak {
                        break;
                    }
                    let block_id = if y > 6 { 1 } else { 2 }; // TODO: implement actual block ID system
                    self.chunk.terrain[x as usize][y as usize][z as usize] = block_id;
                }
            }
        }
    }

    fn construct_mesh(&mut self) {
        let mesh_data = build_mesh_data(&self.chunk);
        let mesh = create_mesh(mesh_data);
        self.mesh.set_mesh(mesh);
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<ChunkGenerator>();
}

godot_init!(init);

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_position() {
        let bp = BlockPosition::new(0, 0, 0);
        assert_eq!(bp.chunk, [0, 0]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(31, 0, 31);
        assert_eq!(bp.chunk, [0, 0]);
        assert_eq!(bp.local_position(), [31, 0, 31]);
        let bp = BlockPosition::new(32, 0, 32);
        assert_eq!(bp.chunk, [1, 1]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(63, 0, 63);
        assert_eq!(bp.chunk, [1, 1]);
        assert_eq!(bp.local_position(), [31, 0, 31]);
        let bp = BlockPosition::new(64, 0, 64);
        assert_eq!(bp.chunk, [2, 2]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(-1, 0, -1);
        assert_eq!(bp.chunk, [-1, -1]);
        let bp = BlockPosition::new(-32, 0, -32);
        assert_eq!(bp.chunk, [-1, -1]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
        let bp = BlockPosition::new(-33, 0, -33);
        assert_eq!(bp.chunk, [-2, -2]);
        assert_eq!(bp.local_position(), [31, 0, 31]);
        let bp = BlockPosition::new(-64, 0, -100);
        assert_eq!(bp.chunk, [-2, -4]);
        assert_eq!(bp.local_position(), [0, 0, 28]);
        let bp = BlockPosition::new(-64, 0, -97);
        assert_eq!(bp.chunk, [-2, -4]);
        assert_eq!(bp.local_position(), [0, 0, 31]);
        let bp = BlockPosition::new(-64, 0, -96);
        assert_eq!(bp.chunk, [-2, -3]);
        assert_eq!(bp.local_position(), [0, 0, 0]);
    }

    #[test]
    fn test_spatial_transform() {
        let origin = [0, 0];
        assert_eq!(ChunkNode::spatial_transform(origin), [0, 0, 0]);
        let origin = [-1, -1];
        assert_eq!(ChunkNode::spatial_transform(origin), [-32, 0, -32]);
        let origin = [-2, -2];
        assert_eq!(ChunkNode::spatial_transform(origin), [-64, 0, -64]);
        let origin = [3, 3];
        assert_eq!(ChunkNode::spatial_transform(origin), [96, 0, 96]);
    }

    #[test]
    fn test_from_local_position() {
        let bp = BlockPosition::from_local_position([0, 0], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [0, 0, 0]);
        let bp = BlockPosition::from_local_position([1, 1], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [32, 0, 32]);
        let bp = BlockPosition::from_local_position([1, 1], [31, 0, 31]);
        assert_eq!([bp.x, bp.y, bp.z], [63, 0, 63]);
        let bp = BlockPosition::from_local_position([2, 2], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [64, 0, 64]);
        let bp = BlockPosition::from_local_position([-1, -2], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [-32, 0, -64]);
        let bp = BlockPosition::from_local_position([-2, -3], [1, 0, 1]);
        assert_eq!([bp.x, bp.y, bp.z], [-63, 0, -95]);
        let bp = BlockPosition::from_local_position([-2, -2], [31, 0, 31]);
        assert_eq!([bp.x, bp.y, bp.z], [-33, 0, -33]);
        let bp = BlockPosition::from_local_position([-1, -1], [0, 0, 0]);
        assert_eq!([bp.x, bp.y, bp.z], [-32, 0, -32]);
        let bp = BlockPosition::from_local_position([-1, -1], [31, 0, 31]);
        assert_eq!([bp.x, bp.y, bp.z], [-1, 0, -1]);
    }

    #[test]
    #[should_panic]
    fn test_invalid_from_local_position() {
        BlockPosition::from_local_position([-1, -1], [32, 0, 32]);
    }
}
*/
