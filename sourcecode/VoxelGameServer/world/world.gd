extends Spatial

onready var network = $network
var chunkCreator:Node

func _ready():
	chunkCreator = preload("res://objects/chunkGenerator/chunkCreator.tscn").instance()
	add_child(chunkCreator)

func chunkPosOf(blockPos: Vector3) -> Vector2:
	return Vector2(
		# HARDCODED
		floor(blockPos.x / 32),
		floor(blockPos.z / 32)
	)

func _on_controllerServer_block_set(client_id: int, position: Vector3, block_id: int):
	chunkCreator.set_block_gd(position, block_id)
	_on_controllerServer_chunk_data_requested(client_id, chunkPosOf(position))


func _on_controllerServer_chunk_data_requested(client_id: int, position: Vector2):
	chunkCreator.load_chunk_gd(position)
	var chunk_data = chunkCreator.chunk_data_encoded(position)
	network.SendChunkData(client_id, position, chunk_data)
