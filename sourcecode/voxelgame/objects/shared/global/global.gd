extends Node

var gamePath = OS.get_executable_path().get_base_dir() + "/"

var initialBootFinished:bool = false
var isDedicatedServer:bool = false

# the self contained server instance
var serverTree:SceneTree = null

# client
var scene_controllerUI = preload("res://objects/client/controllers/controllerUI/controllerUI.tscn")
var scene_controllerClient = preload("res://objects/client/controllers/controllerClient/controllerClient.tscn")
# server
var scene_controllerServer = preload("res://objects/server/controllers/controllerServer/controllerServer.tscn")

func _ready():
	InitialBoot()

# this will handle how the game should boot, are we a dedicated server?
# then create only the server node, and vice versa,
func InitialBoot():
	# TODO: make this accept terminal arguments, for now its just a bool we manually set
	# CheckDedicatedServer();
	if (isDedicatedServer):
		CreateServer()
	else:
		ClientStart()
	initialBootFinished = true

func CreateServer():
	pGlobal.serverTree = SceneTree.new()
	pGlobal.serverTree.root.add_child(controllerServer.instantiate())

func ClientStart():
	add_child(scene_controllerUI.instantiate())
	add_child(scene_controllerClient.instantiate())
