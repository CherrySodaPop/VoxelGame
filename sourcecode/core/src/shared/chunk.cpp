#include "chunk.hpp"

#include <godot_cpp/core/class_db.hpp>

using namespace godot;
using namespace voxelgame;

Chunk::Chunk() {

}

Chunk::~Chunk() {

}

void Chunk::set_block(int x, int y, int z, block &_block) {
    if (x >= CHUNK_WIDTH_LENGTH || y >= CHUNK_WIDTH_LENGTH || z >= CHUNK_HEIGHT || 0 > x || 0 > y || 0 > z) {
        UtilityFunctions::push_error(String("Attempted to set block outside chunk size! (%s)") % Vector3i(x,y,z));
        return;
    }
    // todo: update_chunk()
    blocks[x][y][z] = _block;
}