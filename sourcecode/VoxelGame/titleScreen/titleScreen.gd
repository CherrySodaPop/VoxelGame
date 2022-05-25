extends Control

const WORLDS_PATH: String = "user://worlds/"
onready var name_hint = $CenterContainer/VBoxContainer/NameHint
onready var line_edit = $CenterContainer/VBoxContainer/HBoxContainer/LineEdit

func _ready():
	# Hacky :)
	set_pause_mode(PAUSE_MODE_PROCESS)
	get_tree().paused = true

func ensureWorldExists(world_name: String):
	var dir = Directory.new()
	var make_code = dir.make_dir(WORLDS_PATH + world_name)
	if not (make_code == OK or make_code == ERR_ALREADY_EXISTS):
		printerr("Couldn't make directory for world %s, error code %d" % [world_name, make_code])

func _on_Button_pressed():
	var world_name = line_edit.text
	ensureWorldExists(world_name)
	Persistent.get_node("controllerNetwork").worldToLoad = world_name
	Persistent.get_node("controllerNetwork").StartWorld()
	get_tree().change_scene("res://world/world.tscn")

func checkWorldExists(world_name: String) -> bool:
	var dir = Directory.new()
	return dir.dir_exists(WORLDS_PATH + world_name)

func _on_LineEdit_text_changed(new_text):
	if new_text == "":
		name_hint.text = "Invalid world name."
		return
	if checkWorldExists(new_text):
		name_hint.text = "The world \"%s\" will be loaded." % new_text
	else:
		name_hint.text = "A new world \"%s\" will be created." % new_text
