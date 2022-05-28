extends Node
class_name Server

onready var network = $main/network

# Thank you to https://github.com/LudiDorici/gd-custom-multiplayer.
# region

func _init():
	custom_multiplayer = MultiplayerAPI.new()
	custom_multiplayer.set_root_node(self)

func _notification(what):
	if what == NOTIFICATION_ENTER_TREE:
		# warning-ignore:RETURN_VALUE_DISCARDED
		get_tree().connect("node_added", self, "_on_add_node")
		_customize_children()
	elif what == NOTIFICATION_EXIT_TREE:
		get_tree().disconnect("node_added", self, "_on_add_node")

func _process(_delta):
	if not custom_multiplayer.has_network_peer():
		return
	custom_multiplayer.poll()

func _on_add_node(node):
	var path = str(node.get_path())
	var mypath = str(get_path())
	if path.substr(0, mypath.length()) != mypath:
		return
	var rel = path.substr(mypath.length(), path.length())
	if rel.length() > 0 and not rel.begins_with("/"):
		return
	node.custom_multiplayer = custom_multiplayer

func _customize_children():
	var frontier = []
	for c in get_children():
		frontier.append(c)
	while not frontier.empty():
		var node = frontier.pop_front()
		frontier += node.get_children()
		node.custom_multiplayer = custom_multiplayer

# endregion

func start():
	network.start()
	print("Server started :)")
