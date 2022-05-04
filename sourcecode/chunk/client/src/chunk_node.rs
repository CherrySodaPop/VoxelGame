use gdnative::{
    api::{CollisionShape, MeshInstance, StaticBody},
    prelude::*,
};

use crate::chunk_mesh::ChunkMeshData;

#[derive(NativeClass)]
#[export]
#[inherit(StaticBody)]
#[user_data(gdnative::export::user_data::MutexData<ChunkNode>)]
pub struct ChunkNode {
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

    pub fn update_mesh_data(&mut self, mesh_data: ChunkMeshData) {
        let collision = unsafe { self.collision.assume_safe() };
        let mesh = unsafe { self.mesh.assume_safe() };
        let collision_shape = mesh_data.build_collision_shape();
        unsafe {
            // This MUST be call_deferred, setting the shape when using the
            // Bullet physics engine is NOT thread-safe!
            collision.call_deferred("set_shape", &[collision_shape.into_shared().to_variant()]);
        };
        mesh.set_mesh(mesh_data.build_mesh());
    }
}
