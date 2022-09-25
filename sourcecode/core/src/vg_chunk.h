#ifndef VG_CHUNK_H
#define VG_CHUNK_H

#include <godot_cpp/core/binder_common.hpp>
#include <godot_cpp/classes/node3d.hpp>
#include <godot_cpp/variant/vector3.hpp>
#include "vg_shared.h"

using namespace godot;

class VGChunk : public Node3D {
    GDCLASS(VGChunk, Node3D);

protected:
    static void _bind_methods();

private:
    int blocks[CHUNK_WIDTH_LENGTH][CHUNK_WIDTH_LENGTH][CHUNK_HEIGHT];

public:
    void set_block(int x, int y, int z, int block_id);
    void gd_set_block(const Vector3 vector, int block_id);
};

#endif VG_CHUNK_H