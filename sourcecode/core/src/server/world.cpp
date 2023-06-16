#include "world.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

#include "shared.hpp"

using namespace godot;
using namespace voxelgame;

World::World() {

}

World::~World() {

}

void World::_ready() {
    GAME_LOGIC_CHECK
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
    this->add_child(_chunk);
}

Chunk *World::generate_chunk(int x, int y) {
    // base terrain
    Chunk *_chunk = new Chunk;
    for (int xx = 0; xx < CHUNK_WIDTH_LENGTH; xx++) {
        for (int yy = 0; yy < CHUNK_WIDTH_LENGTH; yy++) {
            int xpos = xx + (x * CHUNK_WIDTH_LENGTH);
            int ypos = yy + (y * CHUNK_WIDTH_LENGTH);
            for (int zz = 0; zz < CHUNK_HEIGHT; zz++) {
                // send it to hell
                _chunk->blocks[xx][yy][zz] = generator_block_type(xpos, ypos, zz);
            }
        }
    }
    return _chunk;
}

block World::generator_block_type(int x, int y, int z) const {
    block _block;
    _block.datapack = String("voxelgame");
    _block.id = String("air");
    // todo: figures out the block to be placed here depending on biome, height, etc.
    int terrain_height = int(noiseTerrain->get_noise_2d(x, y));
    if ((z > terrain_height) == false) {
        _block.id = String("dirt"); // todo: replace me :)
    }
    return _block;
}