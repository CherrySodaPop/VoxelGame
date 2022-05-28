extends Node

# This entire node's purpose is to provide easy access
# to the current world name from the Rust side of things.

var WORLDS_PATH: String = OS.get_user_data_dir() + "/worlds/"
var currentWorld: String

func _ready():
	var args = OS.get_cmdline_args()
	var worldName = "default"
	if len(args) > 1:
		worldName = args[1]
	currentWorld = WORLDS_PATH + worldName
