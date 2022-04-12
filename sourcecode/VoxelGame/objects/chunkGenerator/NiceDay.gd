extends Label

var username_variable = "USER" if OS.get_name() == "X11" else "USERNAME"

func _process(delta):
	text = "VoxelGame Indev - " + OS.get_environment(username_variable) + " stinks!\n" + str(Persistant.get_node("clientPlayer").global_transform.origin);
