extends Node

func _ready():
	pass

func GenerateChunkMesh(pos:Vector2):
	print("Entered ", pos);
	Persistant.get_node("controllerNetwork").rpc_id(1, "SendChunkData", pos);
