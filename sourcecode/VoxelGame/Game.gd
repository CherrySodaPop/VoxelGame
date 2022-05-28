extends Node

const TICK_RATE: float = 1.0 / 30.0

onready var network = $network
onready var chunkLoader = $world/chunkLoader
onready var networkedPlayers = $world/networkedPlayers
onready var player = $world/player

var tick: float = 0.0

func _ready():
	yield(network, "connected")
	# TODO: no
	network.RequestChunkDataAround(Vector2(0, 0))

func _on_controllerNetwork_player_info_updated(client_id: int, position: Vector3, camera_rot: Vector3):
	if not networkedPlayers.has(client_id):
		networkedPlayers.add_player(client_id)
	networkedPlayers.update_player(client_id, position, camera_rot)

func _on_controllerNetwork_chunk_data_received(position: Vector2, data: PoolByteArray):
	chunkLoader.update_chunk(position, data)

func _physics_process(delta):
	tick += delta
	if tick >= TICK_RATE:
		tick = 0
		# TODO: Tick stuff

func _on_player_entered_chunk(chunk_position: Vector2):
	network.RequestChunkDataAround(chunk_position)

func _on_player_block_broken(position):
	network.SendSetBlock(position, 0)

func _on_player_block_placed(position, block_id):
	network.SendSetBlock(position, block_id)
