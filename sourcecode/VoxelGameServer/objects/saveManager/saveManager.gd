extends Node

var CLIENT_WORLDS_PATH: String = OS.get_data_dir() + "/VoxelGame/worlds/"
var currentWorld = null

func ensureWorldsDirectory(worldsPath: String):
	var dir = Directory.new()
	var make_code = dir.make_dir(worldsPath)
	if not (make_code == OK or make_code == ERR_ALREADY_EXISTS):
		printerr("Couldn't make worlds directory, error code %d" % make_code)

# Currently unused (this might be more useful for the client, especially the title screen):
# func getWorldList(worldsPath: String) -> Array:
# 	var worlds = []
# 	var dir = Directory.new()
# 	if dir.open(worldsPath) == OK:
# 		dir.list_dir_begin()
# 		while true:
# 			var path_name = dir.get_next()
# 			if path_name == "":
# 				break
# 			if path_name == "." or path_name == "..":
# 				continue
# 			if dir.current_is_dir():
# 				worlds.push_back(path_name)
# 	else:
# 		printerr("An error occurred when trying to access %s." % worldsPath)
# 	return worlds

func loadWorld(worldName: String):
	currentWorld = CLIENT_WORLDS_PATH + worldName
	var chunkCreator = preload("res://objects/chunkGenerator/chunkCreator.tscn").instance();
	get_tree().get_root().get_node("world").add_child(chunkCreator)


func _ready():
	ensureWorldsDirectory(CLIENT_WORLDS_PATH)
