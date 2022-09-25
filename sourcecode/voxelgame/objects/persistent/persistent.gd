extends Node

var initialBootFinished:bool = false;
var isDedicatedServer:bool = false;

# the self contained server instance
var serverTree:SceneTree = null;

# client
var player = preload("res://objects/client/entities/player/player.tscn");
var controllerUI = preload("res://objects/client/controllers/controllerUI/controllerUI.tscn");
var controllerClient = preload("res://objects/client/controllers/controllerClient/controllerClient.tscn");
# server
var controllerServer = preload("res://objects/server/controllers/controllerServer/controllerServer.tscn");

func _ready():
	InitialBoot();

# this will handle how the game should boot, are we a dedicated server?
# then create only the server node, and vice versa,
func InitialBoot():
	# TODO: make this accept terminal arguments, for now its just a bool we manually set
	# CheckDedicatedServer();
	if (isDedicatedServer):
		CreateServer();
	else:
		ClientStart();
	initialBootFinished = true;

func CreateServer():
	Persistent.serverTree = SceneTree.new();
	Persistent.serverTree.root.add_child(controllerServer.instantiate());

func ClientStart():
	add_child(controllerUI.instantiate());
	add_child(controllerClient.instantiate());
	add_child(player.instantiate());
