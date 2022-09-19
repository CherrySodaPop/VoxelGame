#ifndef VG_WORLD_H
#define VG_WORLD_H

#include <map>
#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/core/binder_common.hpp>
#include "vg_chunk.h"

using namespace std;
using namespace godot;

class VGWorld : public Node {
    GDCLASS(VGWorld, Node);

protected:
    static void _bind_methods();

private:
    map<int[], VGChunk> chunks;

public:
    void _process(double delta);
};

#endif // VG_WORLD_H