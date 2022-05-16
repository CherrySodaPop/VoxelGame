//! Chunk nodes, for use by the server and client.

use gdnative::{
    api::{CollisionShape, MeshInstance},
    prelude::*,
};

use crate::chunkmesh::ChunkMeshData;

#[derive(NativeClass)]
#[export]
#[inherit(CollisionShape)]
#[user_data(gdnative::export::user_data::MutexData<ChunkCollisionShape>)]
pub struct ChunkCollisionShape {
    base: Ref<CollisionShape, Shared>,
}

#[methods]
impl ChunkCollisionShape {
    fn new(base: &CollisionShape) -> Self {
        Self {
            base: unsafe { base.assume_shared() },
        }
    }

    pub fn update_shape(&mut self, mesh_data: &ChunkMeshData) {
        let new_collision_shape = mesh_data.build_collision_shape();
        unsafe {
            // This MUST be call_deferred, setting the shape when using the
            // Bullet physics engine is NOT thread-safe!
            self.base.assume_safe().call_deferred(
                "set_shape",
                &[new_collision_shape.into_shared().to_variant()],
            );
        };
    }
}

#[derive(NativeClass)]
#[export]
#[inherit(MeshInstance)]
#[user_data(gdnative::export::user_data::MutexData<ChunkMeshInstance>)]
pub struct ChunkMeshInstance {
    base: Ref<MeshInstance, Shared>,
}

#[methods]
impl ChunkMeshInstance {
    fn new(base: &MeshInstance) -> Self {
        Self {
            base: unsafe { base.assume_shared() },
        }
    }

    pub fn update_mesh(&mut self, mesh_data: &ChunkMeshData) {
        let new_mesh = mesh_data.build_mesh();
        unsafe { self.base.assume_safe() }.set_mesh(new_mesh);
    }
}
