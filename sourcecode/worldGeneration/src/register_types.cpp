#include "register_types.h"

#include <godot/gdnative_interface.h>

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/core/defs.hpp>
#include <godot_cpp/godot.hpp>

#include "vg_world.h"
#include "vg_chunk.h"

using namespace godot;

void register_types() {
    ClassDB::register_class<VGWorld>();
    ClassDB::register_class<VGChunk>();
}