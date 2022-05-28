extends Node

func _ready():
	print("Game executable: ", OS.get_executable_path())
	var args = OS.get_cmdline_args()
	if len(args) <= 2:
		# We are running as the client.
		get_tree().change_scene("res://titleScreen/titleScreen.tscn")
	else:
		# We are running as the server.
		var world_name = args[2]
		CurrentWorld.world_path = OS.get_user_data_dir() + "/worlds/" + world_name
		get_tree().change_scene("res://server/objects/server/server.tscn")
