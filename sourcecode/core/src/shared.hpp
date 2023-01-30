#ifndef VG_SHARED_H
#define VG_SHARED_H

#include <godot_cpp/variant/utility_functions.hpp>
#include <godot_cpp/classes/engine.hpp>

#define CHUNK_WIDTH_LENGTH 32
#define CHUNK_HEIGHT 256

#define GAME_LOGIC_CHECK if (Engine::get_singleton()->is_editor_hint()) { return; }

#endif // VG_SHARED_H