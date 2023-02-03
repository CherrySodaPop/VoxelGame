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

void Database::load_datapacks(const PackedStringArray &_datapack_paths) {
    String _datapacks_path = get_game_path() + "data/";
    int _datapack_count = _datapack_paths.size();

    // load the json to the dictionary
    for (int i = 0; i < _datapack_count; i++) {
        String _datapack_check_path = _datapacks_path + _datapack_paths[i] + "/";
        load_datapack(_datapack_check_path);
    }
    // perfect, now lets parse it all into engine resources (materials, models, sounds)
    Array _datapack_titles = get_game_data().keys();
    int _datapacks_count = get_game_data().size();
    for (int i = 0; i < _datapacks_count; i++) {
        parse_datapack(_datapack_titles[i]);
    }
}

void Database::parse_datapack(const String &_datapack_title) {
    String _pack_path = get_game_path() + "data/" + _datapack_title + "/";
    Dictionary _pack_data = get_game_data()[_datapack_title];
    Dictionary _block_data = _pack_data["blockData"];
    Dictionary _item_data = _pack_data["itemData"];
    // blocks
    Array _block_keys = _block_data.keys();
    int _block_count = _block_keys.size();
    for (int i = 0; i < _block_count; i++) {
        Dictionary _block = _block_data[_block_keys[i]];
        if (_block["material"] != "") {
            String _block_mat_path = _pack_path + String(_block["material"]);
            Ref<FileAccess> _block_mat_file = FileAccess::open(_block_mat_path, FileAccess::READ);
            String _block_mat_text = _block_mat_file->get_as_text();

            Dictionary _block_mat_info = JSON::parse_string(_block_mat_text); // read info
            StandardMaterial3D *_mat = memnew(StandardMaterial3D); // material info

            apply_material_info_dictionary(_mat, _block_mat_info, _pack_path);
            _block["material"] = _mat;
        }
    }
    // items, but first autocreate the block items
    for (int i = 0; i < _block_count; i++) {

    }
    // the only item data loaded is the texture/model
}

void Database::load_datapack(const String &_datapack_path) {
    // check if datapack exists
    String _gameinfo_path = _datapack_path + String("gameinfo.json");
    if (!FileAccess::file_exists(_gameinfo_path)) {
        UtilityFunctions::push_error(String("Missing gameinfo.json! (%s)") % _datapack_path);
        return;
    }
    // parse JSON
    Ref<FileAccess> _gameinfo_file = FileAccess::open(_gameinfo_path, FileAccess::READ);
    String _gameinfo_text = _gameinfo_file->get_as_text();
    Dictionary _gameinfo = JSON::parse_string(_gameinfo_text);
    if (_gameinfo == nullptr) {
        UtilityFunctions::push_error(String("Can't parse gameinfo.json! (%s)") % _datapack_path);
        return;
    }
    // add the data!
    game_data.merge(_gameinfo);
    UtilityFunctions::print("Finished loading datapacks");
}

Ref<ImageTexture> Database::load_texture(const String &_texture_path) {
    Ref<Image> _image = Image::load_from_file(_texture_path);
    Ref<ImageTexture> _image_texture = ImageTexture::create_from_image(_image);
    return _image_texture;
}

void Database::apply_material_info_dictionary(StandardMaterial3D *_material, Dictionary _dictionary, String _pack_path) {
    // TODO: more values!
    _material->set_texture(StandardMaterial3D::TEXTURE_ALBEDO, load_texture(_pack_path + String(_dictionary["albedo"])));
    _material->set_texture_filter((StandardMaterial3D::TextureFilter)int(_dictionary["texture_filter"]));
}