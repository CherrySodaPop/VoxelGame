#ifndef VG_GAMEDATA_H
#define VG_GAMEDATA_H

#include <godot_cpp/core/binder_common.hpp>
#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/variant/dictionary.hpp>

using namespace godot;

namespace voxelgame {

class Database : public Node {
    GDCLASS(Database, Node);

private:
    String game_path;

    Dictionary *block_info; // construct on runtime an internal list of the blocks info
    Dictionary *item_info; // construct on runtime an internal list of items, should also auto generate block item versions

protected:
    static void _bind_methods() {}

public:
    Database();
    ~Database();

    void _ready() override;

    void load_data(const String &_datapack_path);

};

}

#endif // VG_GAMEDATA_H