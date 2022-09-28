#include "chunk.h"

#include <godot_cpp/core/class_db.hpp>

using namespace godot;
using namespace voxelgame;

Chunk::Chunk() {

}

Chunk::~Chunk() {

}

void Chunk::_bind_methods() {

}

void Chunk::set_block(int x, int y, int z, int block_id) {
    if (x >= CHUNK_WIDTH_LENGTH || y >= CHUNK_WIDTH_LENGTH || z >= CHUNK_HEIGHT || 0 > x || 0 > y || 0 > z) {
        return;
    }
    // todo: update_chunk()
    blocks[x][y][z] = block_id;
}