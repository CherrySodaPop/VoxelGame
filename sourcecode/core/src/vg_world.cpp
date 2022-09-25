#include "vg_world.h"

#include <godot_cpp/core/class_db.hpp>

using namespace godot;

VGWorld::VGWorld() {
    noiseTerrain = new FastNoiseLite;
}

void VGWorld::_bind_methods() {
    
}

void VGWorld::_process(double delta) {

}