#include "world.h"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>
#include "shared.h"

using namespace godot;
using namespace voxelgame;

World::World() {

}

World::~World() {

}

void World::_bind_methods() {
    
}

void World::_init() {
    noiseTerrain = new FastNoiseLite;
    noiseBiome = new FastNoiseLite;
}

void World::_process(double delta) {

}

void World::load_chunk(int x, int y) {
    // todo: check if the chunk already exists in the filesystem and load that instead
    // if (chunk_file_exists) {}
    // else
    Chunk *_chunk = generate_chunk(x, y);
    if (_chunk != nullptr) {
        this->add_child(_chunk);
    }
}

Chunk *World::generate_chunk(int x, int y) {
    if (chunks.count(Vector2i(x,y)) == false) return nullptr;
    Chunk *_chunk = new Chunk;
    for (int xx = 0; xx < CHUNK_WIDTH_LENGTH; xx++) {
        for (int yy = 0; yy < CHUNK_WIDTH_LENGTH; yy++) {
            int xpos = xx + (x * CHUNK_WIDTH_LENGTH);
            int ypos = yy + (y * CHUNK_WIDTH_LENGTH);
            for (int zz = 0; zz < CHUNK_HEIGHT; zz++) {
                BLOCK_IDS _block = generate_block_type(xpos, ypos, zz);
                _chunk->blocks[xx][yy][zz] = _block;
            }
        }
    }
    return _chunk;
}

BLOCK_IDS World::generate_block_type(int x, int y, int z) {
    // todo: figures out the block to be placed here depending on biome, height, etc.
    BLOCK_IDS _block = BLOCK_IDS::AIR;

    int terrain_height = int(noiseTerrain->get_noise_2d(x, y));
    if ((z > terrain_height) == false) {
        _block = BLOCK_IDS::OAK_WOOD_LOG; // todo: replace me :)
    }

    return _block;
}