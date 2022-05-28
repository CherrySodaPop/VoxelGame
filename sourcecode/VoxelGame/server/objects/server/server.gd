extends Node

onready var network = $network
var chunkCreator: Node

func _ready():
	# This is instantiated in _ready to allow for the "CurrentWorld" autoload
	# to, well, load.
	chunkCreator = load(
		Constants.SERVER_PATH + "objects/chunkCreator/chunkCreator.tscn"
	).instance()
	call_deferred("add_child", chunkCreator)
	network.start()

func chunkPosOf(blockPos: Vector3) -> Vector2:
	return Vector2(
		# HARDCODED
		floor(blockPos.x / 32),
		floor(blockPos.z / 32)
	)

func _on_network_block_set(client_id: int, position: Vector3, block_id: int):
	var chunkPos = chunkPosOf(position)
	chunkCreator.load_chunk_gd(chunkPos)
	chunkCreator.set_block_gd(position, block_id)
	var chunk_data = chunkCreator.chunk_data_encoded(chunkPos)
	network.send_chunk_data_all(chunkPos, chunk_data)


func _on_network_chunk_data_requested(client_id: int, position: Vector2):
	chunkCreator.load_chunk_gd(position)
	var chunk_data = chunkCreator.chunk_data_encoded(position)
	network.send_chunk_data(client_id, position, chunk_data)
