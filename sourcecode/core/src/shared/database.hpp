#ifndef VG_GAMEDATA_H
#define VG_GAMEDATA_H

#include <godot_cpp/core/binder_common.hpp>
#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/classes/image_texture.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/classes/standard_material3d.hpp>

using namespace godot;

namespace voxelgame {

class Database : public Node {
    GDCLASS(Database, Node);

private:
    String game_path;
    Dictionary game_data;

protected:
    static void _bind_methods() {
        ClassDB::bind_method(D_METHOD("datapack_search"), &Database::datapack_search);
        ClassDB::bind_method(D_METHOD("set_game_data", "game_data"), &Database::set_game_data);
        ClassDB::bind_method(D_METHOD("get_game_data"), &Database::get_game_data);
        ADD_PROPERTY(PropertyInfo(Variant::DICTIONARY, "game_data"), "set_game_data", "get_game_data");
    }

public:
    Database();
    ~Database();

    void _ready() override;

    PackedStringArray datapack_search();
    void load_datapacks(const PackedStringArray &_pack_folders);
    bool can_parse_datapack(const String &_pack_path);
    void parse_datapack(const String &_pack_title);

    Ref<ImageTexture> load_texture(const String &_texture_path);
    void apply_material_info_dictionary(StandardMaterial3D *_material, Dictionary _dictionary, String _pack_path);
    Dictionary parse_json_filepath(const String &_path);

    String get_game_path() { return game_path; }
    Dictionary get_game_data () { return game_data; }
    void set_game_data(Dictionary _game_data) { game_data = _game_data; }
    
};

}

#endif // VG_GAMEDATA_H