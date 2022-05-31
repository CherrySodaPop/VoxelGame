extends Node

const TICK_RATE: float = 1.0 / 30.0

onready var connectionTimeout = $ConnectionTimeout
onready var network = $network
onready var chunkLoader = $world/chunkLoader
onready var networkedPlayers = $world/networkedPlayers
onready var player = $world/player

var tick: float = 0.0

func _ready():
	var root = get_tree().get_root()
	var world_name = root.get_meta("world_name")
	var server_ip = "localhost"
	if world_name != null:
		# A local world was selected on the title screen
		var args = ["--no-window", "--world", world_name, "-local_server"]
		var server_pid = OS.execute(OS.get_executable_path(), args, false)
		print("Server process started with PID %d." % server_pid)
	else:
		# Multiplayer was selected on the title screen
		server_ip = get_tree().get_root().get_meta("server_ip")
	network.connect_to(server_ip)

func _on_network_connected():
	connectionTimeout.stop()
	connectionTimeout.queue_free()
	connectionTimeout = null
	network.RequestChunkDataAround(Vector2(0, 0))

func _on_controllerNetwork_player_info_updated(client_id: int, position: Vector3, camera_rot: Vector2):
	networkedPlayers.ensure_has(client_id)
	networkedPlayers.update_player(client_id, position, camera_rot)

func _on_network_player_appearance_received(client_id: int, skin_texture: ImageTexture):
	networkedPlayers.ensure_has(client_id)
	networkedPlayers.update_player_appearance(client_id, skin_texture)

func _on_network_player_disconnected(client_id):
	networkedPlayers.remove_player(client_id)

func _on_controllerNetwork_chunk_data_received(position: Vector2, data: PoolByteArray):
	chunkLoader.update_chunk(position, data)

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
