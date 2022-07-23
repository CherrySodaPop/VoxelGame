extends Node

var initialBootFinished:bool = false;
var isDedicatedServer:bool = true;

# the self contained server instance
var serverTree:SceneTree = null;

var controllerClient = preload("res://objects/controllers/controllerClient/controllerClient.tscn");
var controllerServer = preload("res://objects/controllers/controllerServer/controllerServer.tscn");

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
	add_child(controllerClient.instantiate());
