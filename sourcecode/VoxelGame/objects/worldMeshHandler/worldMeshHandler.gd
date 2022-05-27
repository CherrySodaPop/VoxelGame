extends Node

func _ready():
	pass

func GenerateChunkMesh(pos:Vector2):
	print("Entered ", pos);
	Persistent.controllerNetwork.rpc_id(1, "SendChunkDataAround", pos);
