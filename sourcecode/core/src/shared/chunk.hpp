#ifndef VG_CHUNK_H
#define VG_CHUNK_H

#include <godot_cpp/core/binder_common.hpp>
#include <godot_cpp/classes/node3d.hpp>
#include <godot_cpp/classes/mesh_instance3d.hpp>
#include <godot_cpp/classes/static_body3d.hpp>
#include <godot_cpp/classes/collision_shape3d.hpp>
#include <godot_cpp/variant/vector3.hpp>

#include "shared.hpp"
#include "world.hpp"

using namespace godot;

namespace voxelgame {

// internal block info holder
struct block {
    String datapack;
    String id;
    Dictionary data;
};

// the physical chunk, mesh, and block data
class Chunk : public StaticBody3D {
    GDCLASS(Chunk, Node3D);

protected:
    static void _bind_methods() {}

private:
    CollisionShape3D *collision;
    MeshInstance3D *mesh;

public:
    block blocks[CHUNK_WIDTH_LENGTH][CHUNK_WIDTH_LENGTH][CHUNK_HEIGHT];

public:
    Chunk();
    ~Chunk();

    // generation
    void generate();

    // after generation
    void set_block(int x, int y, int z, block &_block);
};

};

#endif // VG_CHUNK_H