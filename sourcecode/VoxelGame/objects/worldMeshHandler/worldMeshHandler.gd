extends Node

func _ready():
	pass

func GenerateChunkMesh(pos:Vector2):
	if (Persistant.has_node("chunkGeneration")):
		print(pos);
		Persistant.get_node("chunkGeneration").load_around_chunk_gd(pos);