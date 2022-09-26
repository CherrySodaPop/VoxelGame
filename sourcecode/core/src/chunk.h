#ifndef CHUNK_H
#define CHUNK_H

#include <godot_cpp/core/binder_common.hpp>
#include <godot_cpp/classes/node3d.hpp>
#include <godot_cpp/classes/mesh_instance3d.hpp>
#include <godot_cpp/classes/static_body3d.hpp>
#include <godot_cpp/variant/vector3.hpp>
#include "shared.h"

using namespace godot;

namespace voxelgame {

class Chunk : public Node3D {
    GDCLASS(Chunk, Node3D);

protected:
    static void _bind_methods();

private:
    StaticBody3D *collision;
    MeshInstance3D *mesh;
    int blocks[CHUNK_WIDTH_LENGTH][CHUNK_WIDTH_LENGTH][CHUNK_HEIGHT];

public:
    void set_block(int x, int y, int z, int block_id);
    void gd_set_block(const Vector3 vector, int block_id);
};

};

#endif // CHUNK_H