#include "database.hpp"

#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/classes/os.hpp>
#include <godot_cpp/classes/file_access.hpp>
#include <godot_cpp/classes/dir_access.hpp>
#include <godot_cpp/classes/json.hpp>
#include <godot_cpp/classes/standard_material3d.hpp>
#include "shared.hpp"

using namespace godot;
using namespace voxelgame;

Database::Database() {}

Database::~Database() {}

void Database::_ready() {
    GAME_LOGIC_CHECK
    game_path = OS::get_singleton()->get_executable_path().get_base_dir() + "/";
    load_datapacks(datapack_search());
}

PackedStringArray Database::datapack_search() {
    String _datapacks_path = get_game_path() + "data/";
    Ref<DirAccess> _data_access = DirAccess::open(_datapacks_path);

    PackedStringArray _pack_string_array = _data_access->get_directories();

    return _pack_string_array;
}

void Database::load_datapacks(const PackedStringArray &_pack_folders) {
    String _data_path = get_game_path() + "data/";
    int _datapack_count = _pack_folders.size();
    PackedStringArray _valid_pack_folders;

    // load the json to the dictionary
    for (int i = 0; i < _datapack_count; i++) {
        String _datapack_check_path = _data_path + _pack_folders[i] + "/";
        if (can_parse_datapack(_datapack_check_path)) {
            _valid_pack_folders.append(_pack_folders[i]);
        }
    }

    // perfect, now lets parse it all into engine resources (materials, models, sounds)
    for (int i = 0; i < _valid_pack_folders.size(); i++) {
        parse_datapack(_valid_pack_folders[i]);
    }
}

void Database::parse_datapack(const String &_pack_title) {
    // file path
    String _pack_path = get_game_path() + "data/" + _pack_title + "/";
    // pack dictionary
    Dictionary _data_to_parse = get_game_data()[_pack_title];
    // data dictionaries
    Dictionary _block_data = _data_to_parse["blockData"];
    Dictionary _item_data = _data_to_parse["itemData"];
    // blocks
    Array _block_keys = _block_data.keys();
    int _block_count = _block_keys.size();
    for (int i = 0; i < _block_count; i++) {
        Dictionary _block = _block_data[_block_keys[i]];
        // parse material to standard material 3d
        if (_block["material"] != "") {
            StandardMaterial3D *_mat = memnew(StandardMaterial3D);
            apply_material_info_dictionary(_mat, parse_json_filepath(_pack_path + String(_block["material"])), _pack_path);
            _block["material"] = _mat;
        }
    }
    // items
    
    // autocreate items, so block item version to actually place and use in crafting
    for (int i = 0; i < _block_count; i++) {

    }
}

bool Database::can_parse_datapack(const String &_pack_path) {
    // check if datapack exists
    String _gameinfo_path = _pack_path + String("gameinfo.json");
    if (!FileAccess::file_exists(_gameinfo_path)) {
        UtilityFunctions::push_error(String("Missing gameinfo.json! (%s)") % _pack_path);
        return false;
    }
    // parse JSON
    Ref<FileAccess> _gameinfo_file = FileAccess::open(_gameinfo_path, FileAccess::READ);
    String _gameinfo_text = _gameinfo_file->get_as_text();
    Dictionary _gameinfo = JSON::parse_string(_gameinfo_text);
    if (_gameinfo == nullptr) {
        UtilityFunctions::push_error(String("Can't parse gameinfo.json! (%s)") % _pack_path);
        return false;
    }
    // add the data!
    UtilityFunctions::print(String("Finished loading datapack! (%s)") % _pack_path);
    game_data.merge(_gameinfo);
    return true;
}

Ref<ImageTexture> Database::load_texture(const String &_texture_path) {
    Ref<Image> _image = Image::load_from_file(_texture_path);
    Ref<ImageTexture> _image_texture = ImageTexture::create_from_image(_image);
    return _image_texture;
}

void Database::apply_material_info_dictionary(StandardMaterial3D *_material, Dictionary _dictionary, String _pack_path) {
    // check the class definitions, yes you have to set each manually
    _material->set_texture(StandardMaterial3D::TEXTURE_ALBEDO, load_texture(_pack_path + String(_dictionary["albedo"])));
    _material->set_texture_filter((StandardMaterial3D::TextureFilter)int(_dictionary["texture_filter"]));
}

Dictionary Database::parse_json_filepath(const String &_path) {
    if (!FileAccess::file_exists(_path)) { UtilityFunctions::push_error(String("Failed to find [%s]") % _path); }
    Ref<FileAccess> _file = FileAccess::open(_path, FileAccess::READ);
    String _json_text = _file->get_as_text();
    Dictionary _json_parsed = JSON::parse_string(_json_text);
    if (_json_parsed == nullptr) { UtilityFunctions::push_error(String("Failed to parse [%s]") % _path); }
    return _json_parsed;
}
