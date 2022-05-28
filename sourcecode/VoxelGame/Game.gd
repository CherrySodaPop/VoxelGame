extends Node

const TICK_RATE: float = 1.0 / 30.0

onready var connectionTimeout = $ConnectionTimeout
onready var network = $network
onready var chunkLoader = $world/chunkLoader
onready var networkedPlayers = $world/networkedPlayers
onready var player = $world/player

var tick: float = 0.0

func _ready():
	var server_ip = get_tree().get_root().get_meta("server_ip") # Set by the title screen
	network.connect_to(server_ip)

func _on_network_connected():
	connectionTimeout.stop()
	connectionTimeout.queue_free()
	connectionTimeout = null
	network.RequestChunkDataAround(Vector2(0, 0))

func _on_controllerNetwork_player_info_updated(client_id: int, position: Vector3, camera_rot: Vector2):
	if not networkedPlayers.has(client_id):
		networkedPlayers.add_player(client_id)
	networkedPlayers.update_player(client_id, position, camera_rot)

func _on_network_player_disconnected(client_id):
	networkedPlayers.remove_player(client_id)

func _on_controllerNetwork_chunk_data_received(position: Vector2, data: PoolByteArray):
	chunkLoader.update_chunk(position, data)

func _physics_process(delta):
	tick += delta
	if tick >= TICK_RATE:
		tick = 0
	else:
		return
	if not network.connected:
		return
	var player_info = player.TickInfo()
	network.SendPlayerInfo(player_info[0], player_info[1])

func _on_player_entered_chunk(chunk_position: Vector2):
	network.RequestChunkDataAround(chunk_position)

func _on_player_block_broken(position):
	network.SendSetBlock(position, 0)

func _on_player_block_placed(position, block_id):
	network.SendSetBlock(position, block_id)

func _on_ConnectionTimeout_timeout():
	if network.connected:
		return
	OS.alert("Couldn't establish a connection to the server.", "Error")
	get_tree().quit(1)
