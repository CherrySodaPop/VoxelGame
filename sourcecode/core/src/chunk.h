#ifndef VG_CHUNK_H
#define VG_CHUNK_H

#include <godot_cpp/core/binder_common.hpp>
#include <godot_cpp/classes/node3d.hpp>
#include <godot_cpp/classes/mesh_instance3d.hpp>
#include <godot_cpp/classes/static_body3d.hpp>
#include <godot_cpp/classes/collision_shape3d.hpp>
#include <godot_cpp/variant/vector3.hpp>

#include "shared.h"

using namespace godot;

namespace voxelgame {

class Chunk : public StaticBody3D {
    GDCLASS(Chunk, Node3D);

protected:
    static void _bind_methods();

private:
    CollisionShape3D *collision;
    MeshInstance3D *mesh;

public:
    int blocks[CHUNK_WIDTH_LENGTH][CHUNK_WIDTH_LENGTH][CHUNK_HEIGHT];

public:
    Chunk();
    ~Chunk();

    void set_block(int x, int y, int z, int block_id);
};

};

#endif // VG_CHUNK_H