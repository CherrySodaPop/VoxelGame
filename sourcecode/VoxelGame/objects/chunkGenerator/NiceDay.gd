extends Label

var username_variable = "USER" if OS.get_name() == "X11" else "USERNAME"

# TODO: An actual Debug Menu node!
func _process(delta):
	var playerLooking = Persistant.get_node("clientPlayer").lookingAtBlock;
	var playerLookingID = get_parent().get_block_gd(playerLooking);
	var playerLookingBlock = Persistant.get_node("blockManager").get_block_by_id(playerLookingID).name
	text = (
		"VoxelGame Indev - "
		+ OS.get_environment(username_variable)
		+ " stinks!"
		+ "\nPosition: " + str(Persistant.get_node("clientPlayer").global_transform.origin)
		+ "\nLooking at: " + str(playerLooking)
		+ "\nLooking at block type: " + playerLookingBlock
	)
