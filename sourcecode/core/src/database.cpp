#include "database.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/os.hpp>

using namespace godot;
using namespace voxelgame;

Database::Database() {
    game_path = OS::get_singleton()->get_executable_path().get_base_dir() + "/";
}

Database::~Database() {}

void Database::load_data(const String &_datapack_path) {

}