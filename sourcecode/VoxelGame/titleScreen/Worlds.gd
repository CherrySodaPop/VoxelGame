extends VBoxContainer

signal world_chosen(world_name)

const WORLDS_PATH := "user://worlds/"

onready var worlds_list: ItemList = $WorldsList
onready var create_button: Button = $HBoxContainer2/CreateWorldButton
onready var world_name_le: LineEdit = $HBoxContainer2/WorldNameLE

func get_worlds() -> Array:
	var dir := Directory.new()
	if dir.open(WORLDS_PATH) != OK:
		printerr("Could not open worlds directory")
		return []
	# warning-ignore:RETURN_VALUE_DISCARDED
	var worlds := []
	dir.list_dir_begin()
	while true:
		var path = dir.get_next()
		if path == "":
			break
		if dir.current_is_dir() and not path.begins_with("."):
			worlds.push_back(path)
	dir.list_dir_end()
	return worlds

func ensure_worlds_path():
	var dir := Directory.new()
	var make_code = dir.make_dir(WORLDS_PATH)
	if not (make_code == OK or make_code == ERR_ALREADY_EXISTS):
		printerr("Could not create worlds directory")

func world_exists(world_name: String) -> bool:
	var dir := Directory.new()
	return dir.open(WORLDS_PATH + world_name) == OK

func update_create_button():
	create_button.disabled = world_exists(world_name_le.text)

func new_world_requested():
	var new_world_name = world_name_le.text
	world_name_le.text = ""
	update_create_button()
	var world_path: String = WORLDS_PATH + new_world_name + "/"
	var dir := Directory.new()
	var make_code = dir.make_dir(world_path)
	if make_code != OK:
		var error_text = "Could not create world (error code %d)" % make_code
		if make_code == ERR_ALREADY_EXISTS:
			error_text = "That world already exists!"
		OS.alert(error_text, "Cannot create")
		return
	var info_file = File.new()
	randomize()
	var world_info = {
		"seed": randi()
	}
	info_file.open(world_path + "info.json", File.WRITE)
	info_file.store_string(JSON.print(world_info, "\t"))
	info_file.close()
	worlds_list.update_list(get_worlds())

func _ready():
	ensure_worlds_path()
	worlds_list.update_list(get_worlds())
	update_create_button()

func _on_WorldNameLE_text_changed(_new_text):
	update_create_button()

func _on_WorldNameLE_text_entered(_new_text):
	new_world_requested()

func _on_CreateWorldButton_pressed():
	new_world_requested()

func _on_StartButton_pressed():
	var selected_world = worlds_list.get_selected_items()
	if len(selected_world) == 0:
		OS.alert("No world selected", "Cannot start")
		return
	var world_name = worlds_list.get_item_text(selected_world[0])
	emit_signal("world_chosen", world_name)
