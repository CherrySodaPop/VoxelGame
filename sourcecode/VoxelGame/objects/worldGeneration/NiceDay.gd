extends Label

var username_variable = "USER" if OS.get_name() == "X11" else "USERNAME"

func _ready():
	text = "Hello " + OS.get_environment(username_variable) + " :)"
