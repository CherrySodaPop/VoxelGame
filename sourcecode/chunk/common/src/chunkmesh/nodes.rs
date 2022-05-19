//! Chunk nodes, for use by the server and client.

use gdnative::{
    api::{CollisionShape, MeshInstance, StaticBody},
    prelude::*,
};

use crate::{chunkmesh::ChunkMeshData, prelude::ChunkPos, vec3};

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
    fn update(&mut self, mesh_data: &ChunkMeshData) {
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
    fn update(&mut self, mesh_data: &ChunkMeshData) {
        let new_mesh = mesh_data.build_mesh();
        unsafe { self.base.assume_safe() }.set_mesh(new_mesh);
    }
}

pub struct ChunkNode {
    body: Ref<StaticBody, Shared>,
    mesh: Option<Instance<ChunkMeshInstance, Shared>>,
    collision: Instance<ChunkCollisionShape, Shared>,
}

impl ChunkNode {
    pub fn new(mesh: Option<Instance<ChunkMeshInstance, Unique>>) -> Self {
        let body = StaticBody::new();
        let collision = ChunkCollisionShape::new_instance().into_shared();
        body.add_child(&collision, true);
        let mesh = mesh.map(|m| m.into_shared());
        if let Some(ref mesh) = mesh {
            body.add_child(mesh, true);
        }
        Self {
            body: body.into_shared(),
            collision,
            mesh,
        }
    }

    pub fn new_with_mesh() -> Self {
        Self::new(Some(ChunkMeshInstance::new_instance()))
    }

    pub fn spawn(&mut self, parent: &Node, position: ChunkPos) {
        let origin = position.origin();
        unsafe { self.body.assume_safe() }.set_translation(vec3!(origin.x, origin.y, origin.z));
        parent.add_child(self.body, true);
    }

    pub fn update(&mut self, mesh_data: &ChunkMeshData) {
        unsafe { self.collision.assume_safe() }
            .map_mut(|collision, _base| collision.update(mesh_data))
            .unwrap();
        if let Some(ref mesh) = self.mesh {
            unsafe { mesh.assume_safe() }
                .map_mut(|mesh, _base| {
                    mesh.update(mesh_data);
                })
                .unwrap();
        }
    }
}
