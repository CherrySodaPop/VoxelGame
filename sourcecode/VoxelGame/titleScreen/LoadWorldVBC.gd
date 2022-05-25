extends VBoxContainer

signal world_selected(world_name)

onready var world_buttons = $WorldButtons

func getWorldList() -> Array:
	var worlds = []
	var dir = Directory.new()
	if dir.open(owner.WORLDS_PATH) == OK:
		dir.list_dir_begin()
		while true:
			var path_name = dir.get_next()
			if path_name == "":
				break
			if path_name == "." or path_name == "..":
				continue
			if dir.current_is_dir():
				worlds.push_back(path_name)
	return worlds

func _ready():
	var worlds = getWorldList()
	if not worlds:
		var empty_label = Label.new()
		empty_label.text = "\tThere are no saved worlds."
		empty_label.add_color_override("font_color", Color(0.8, 0.8, 0.8, 1.0))
		world_buttons.add_child(empty_label)
		return
	for world in worlds:
		var world_button = Button.new()
		world_button.text = world
		world_button.connect("pressed", owner, "worldSelected", [world])
		world_button.size_flags_horizontal = 0
		world_buttons.add_child(world_button)
