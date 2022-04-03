extends Label

# LGPL 3.0 License (c) MAQUE 2022

# Written by MAQUE. :)
var username_variable = "USER" if OS.get_name() == "X11" else "USERNAME"

func _ready():
	text = "Hello " + OS.get_environment(username_variable) + " :)"
