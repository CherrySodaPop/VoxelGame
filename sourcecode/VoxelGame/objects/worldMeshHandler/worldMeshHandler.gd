extends Node

func _ready():
	pass

func GenerateChunkMesh(pos:Vector2):
	print("Entered ", pos);
	Persistent.get_node("controllerNetwork").rpc_id(1, "SendChunkDataAround", pos);
