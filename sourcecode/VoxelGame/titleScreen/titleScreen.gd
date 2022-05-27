extends Control

const WORLDS_PATH: String = "user://worlds/"
onready var line_edit = $CenterContainer/VBoxContainer/NewWorldVBC/HBoxContainer/LineEdit
onready var name_hint = $CenterContainer/VBoxContainer/NewWorldVBC/NameHint
onready var create_button = $CenterContainer/VBoxContainer/NewWorldVBC/HBoxContainer/CreateButton

func _ready():
	# Hacky :)
	set_pause_mode(PAUSE_MODE_PROCESS)
	get_tree().paused = true

	ensureWorldsDirectory(WORLDS_PATH)
	line_edit.grab_focus()

func ensureWorldsDirectory(worlds_path: String):
	var dir = Directory.new()
	var make_code = dir.make_dir(worlds_path)
	if not (make_code == OK or make_code == ERR_ALREADY_EXISTS):
		printerr("Couldn't make worlds directory, error code %d" % make_code)

func createWorldInfo() -> Dictionary:
	randomize()
	return {
		"seed": randi()
	}

func writeWorldInfo(world_path: String, world_info: Dictionary):
	var file = File.new()
	file.open(world_path + "info.json", File.WRITE)
	file.store_string(JSON.print(world_info, "\t"));
	file.close()

func ensureWorldExists(world_name: String):
	var dir = Directory.new()
	var world_path = WORLDS_PATH + world_name + "/"
	var make_code = dir.make_dir(world_path)
	if make_code == OK:
		writeWorldInfo(world_path, createWorldInfo())
	elif make_code == ERR_ALREADY_EXISTS:
		# That's fine.
		return
	else:
		printerr("Couldn't make directory for world %s, error code %d" % [world_name, make_code])

func worldSelected(world_name: String):
	ensureWorldExists(world_name)
	Persistent.controllerNetwork.worldToLoad = world_name
	Persistent.controllerNetwork.StartWorld()
	get_tree().change_scene("res://world/world.tscn")

func checkWorldExists(world_name: String) -> bool:
	var dir = Directory.new()
	return dir.dir_exists(WORLDS_PATH + world_name)

func _on_LineEdit_text_changed(new_text):
	if new_text == "":
		name_hint.text = "Invalid world name."
		create_button.disabled = true
		return
	if checkWorldExists(new_text):
		name_hint.text = "The world \"%s\" already exists." % new_text
		create_button.disabled = true
	else:
		name_hint.text = "A new world \"%s\" will be created." % new_text
		create_button.disabled = false

func newWorld():
	worldSelected(line_edit.text)

func _on_CreateButton_pressed():
	newWorld()

func _on_LineEdit_text_entered(_new_text):
	if create_button.disabled:
		return
	newWorld()
