extends Control

onready var label = $RichTextLabel
onready var player = Persistant.get_node("clientPlayer")

var username_variable = "USER" if OS.get_name() == "X11" else "USERNAME"
var username = OS.get_environment(username_variable)
var flavor_text = "%s, you're awesome!" % username

# TODO: An actual Debug Menu node!
func _process(delta):
	var playerLooking = player.lookingAtBlock;
	var playerLookingID = Persistant.get_node("chunkGeneration").get_block_gd(playerLooking);
	var playerLookingBlock = Persistant.get_node("blockManager").get_block_by_id(playerLookingID).name
	var playerPosition = player.global_transform.origin
	var debugText = (
		"VoxelGame Indev - "
		+ flavor_text
		+ "\nPosition:"
		+ "\n  X: " + str(playerPosition.x)
		+ "\n  Y: " + str(playerPosition.y)
		+ "\n  Z: " + str(playerPosition.z)
		+ "\nLooking at block:"
		+ "\n  Position: " + str(playerLooking)
		+ "\n  Type    : %s (ID %d)" % [playerLookingBlock, playerLookingID]
	)
	label.text = debugText
