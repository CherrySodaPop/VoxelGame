#ifndef VG_WORLD_H
#define VG_WORLD_H

#include <map>

#include <godot_cpp/core/binder_common.hpp>
#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/classes/fast_noise_lite.hpp>
#include <godot_cpp/variant/vector2.hpp>
#include <godot_cpp/variant/dictionary.hpp>

#include "chunk.hpp"

using namespace std;
using namespace godot;

namespace voxelgame {

class Chunk;

class World : public Node {
    GDCLASS(World, Node);

protected:
    static void _bind_methods() {}

private:
    FastNoiseLite *noiseTerrain;
    FastNoiseLite *noiseBiome;

    map<Vector2i, Chunk> chunks;

public:
    World();
    ~World();

    void _ready() override;
    void _process(double delta) override;

    void load_chunk(int x, int y);
    //Chunk *file_load_chunk(int x, int y);
    Chunk *generate_chunk(int x, int y);
    String generate_block_type(int x, int y, int z) const;
};

}

#endif // VG_WORLD_H