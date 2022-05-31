extends Node

func parse_args(args: Array) -> Array:
	var kwargs := {}
	var flags := []
	var idx: int = 0
	while true:
		if idx >= len(args):
			break
		var arg = args[idx]
		var arg_prefix = arg.substr(0, 2)
		if arg_prefix == "--":
			var key = arg.trim_prefix("--")
			var value = args[idx+1]
			kwargs[key] = value
			idx += 2
		elif arg_prefix[0] == "-":
			var key = arg.trim_prefix("-")
			flags.append(key)
			idx += 1
		else:
			break
	return [kwargs, flags]


func _ready():
	print("Game executable: ", OS.get_executable_path())
	var args = OS.get_cmdline_args()
	var parsed_args = parse_args(args)
	var kwargs: Dictionary = parsed_args[0]
	var flags: Array = parsed_args[1]
	print("Parsed CLI flags: ", flags)
	print("Parsed CLI kwargs: ", kwargs)

	if not ("world" in kwargs):
		# We are running as the client.
		get_tree().change_scene("res://titleScreen/titleScreen.tscn")
	else:
		# We are running as the server.
		var world_name = kwargs["world"]
		CurrentWorld.world_path = ProjectSettings.globalize_path("user://worlds/" + world_name)
		get_tree().get_root().set_meta("local_server", false)
		if "local_server" in flags:
			# This is a local server.
			print("Running as local server.")
			get_tree().get_root().set_meta("local_server", true)
			get_tree().change_scene("res://server/objects/server/server.tscn")
