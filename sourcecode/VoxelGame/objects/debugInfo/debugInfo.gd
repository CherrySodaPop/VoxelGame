extends Control

onready var label = $RichTextLabel
onready var player = Persistant.get_node("player")

var username_variable = "USER" if OS.get_name() == "X11" else "USERNAME"
var username = OS.get_environment(username_variable)
var flavor_text = "I have my doubts about %s." % username
# TODO: A better text system than *raw indices*!!
var debugText = PoolStringArray([
	"VoxelGame Indev - " + flavor_text,
	"Position:",
	"",
	"",
	"",
	"(waiting for lock)",
	"Looking at block:",
	"",
	"",
	"ClientChunkLoader:",
	""
])

func _ready():
	VisualServer.set_debug_generate_wireframes(true)

func _process(_delta):
	var playerPosition = player.global_transform.origin
	debugText[2] = "  X: " + str(playerPosition.x)
	debugText[3] = "  Y: " + str(playerPosition.y)
	debugText[4] = "  Z: " + str(playerPosition.z)
	label.text = debugText.join("\n")

func _on_RefreshTimer_timeout():
	# ClientChunkLoader lock-dependent stuff.
	var playerLooking = player.lookingAtBlock;
	var debugInfo = Persistant.get_node("chunkLoader").debug_info(playerLooking);
	if debugInfo == null:
		debugText[5] = "(waiting for lock)"
		label.text = debugText.join("\n")
		return
	debugText[5] = ""
	var playerLookingID = debugInfo[0]
	var playerLookingBlock = (
		null if playerLookingID == null else
		Persistant.get_node("blockManager").get_block_by_id(playerLookingID).name
		)
	debugText[7] = "  Position: " + str(playerLooking)
	debugText[8] = "  Type    : %s (ID %s)" % [str(playerLookingBlock), str(playerLookingID)]
	var chunksLoaded = debugInfo[1]
	debugText[10] = "  Loaded: %d" % chunksLoaded
	label.text = debugText.join("\n")

func _input(event):
	if event.is_action_pressed("debugToggleWireframe"):
		var viewport = get_viewport()
		var wireframe = not (viewport.debug_draw == VisualServer.VIEWPORT_DEBUG_DRAW_WIREFRAME)
		viewport.debug_draw = (
			VisualServer.VIEWPORT_DEBUG_DRAW_WIREFRAME
			if wireframe else
			VisualServer.VIEWPORT_DEBUG_DRAW_DISABLED
		)
