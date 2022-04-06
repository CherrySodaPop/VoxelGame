extends Label

var username_variable = "USER" if OS.get_name() == "X11" else "USERNAME"

func _ready():
	text = "be sure to take short breaks " + OS.get_environment(username_variable).to_lower() + " :)"
