extends Node

var gamePath = OS.get_executable_path().get_base_dir() + "/"

var initialBootFinished:bool = false
var isDedicatedServer:bool = false

# the self contained server instance
var serverTree:SceneTree = null

# client
var scene_uxHead = preload("res://objects/client/ux/uxHead/uxHead.tscn")
var scene_clientHead = preload("res://objects/client/clientHead/clientHead.tscn")
# server
var scene_serverHead = preload("res://objects/server/serverHead/serverHead.tscn")

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
	pGlobal.serverTree.root.add_child(scene_serverHead.instantiate())

func ClientStart():
	add_child(scene_uxHead.instantiate())
	add_child(scene_clientHead.instantiate())
