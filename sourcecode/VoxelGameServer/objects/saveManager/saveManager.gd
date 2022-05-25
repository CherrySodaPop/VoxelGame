extends Node

var CLIENT_WORLDS_PATH: String = OS.get_data_dir() + "/VoxelGame/worlds/"
var currentWorld = null

func loadWorld(worldName: String):
	currentWorld = CLIENT_WORLDS_PATH + worldName
	var chunkCreator = preload("res://objects/chunkGenerator/chunkCreator.tscn").instance();
	get_tree().get_root().get_node("world").add_child(chunkCreator)
