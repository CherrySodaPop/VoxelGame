extends Node

signal block_set(client_id, position, block_id)
signal chunk_data_requested(client_id, position)

const PLAYER_CREDS_PATH := Constants.SERVER_PATH + "gameinfo/creds.json"

enum DisconnectType {
	INVALID_INFO,
	INVALID_CREDS,
	SERVER_CLOSED,
	LEFT,
	ADMIN_KICK,
	CHEATING,
}

var port: int = 25565
var network := NetworkedMultiplayerENet.new()

var player_creds: Dictionary

func broadcast_client_disconnect(client_id: int, reason: int):
	rpc("ClientDisconnected", client_id, reason)

func on_client_connected(client_id: int):
	print_debug("Client %d connected." % client_id)

func on_client_disconnected(client_id: int):
	broadcast_client_disconnect(client_id, DisconnectType.LEFT)
	print_debug("Client %d disconnected." % client_id)

func load_player_credentials():
	var file := File.new()
	if file.open(PLAYER_CREDS_PATH, File.READ) != OK:
		printerr("Could not open player credentials file.")
	return parse_json(file.get_as_text())

func save_player_credentials():
	var file := File.new();
	if file.open(PLAYER_CREDS_PATH, File.WRITE) != OK:
		printerr("Could not open player credentials file (for saving).")
	file.store_string(JSON.print(player_creds, "\t"))
	file.close()

func sender_id() -> int:
	return multiplayer.get_rpc_sender_id()

func _ready():
	# warning-ignore:RETURN_VALUE_DISCARDED
	multiplayer.connect("network_peer_connected", self, "on_client_connected");
	# warning-ignore:RETURN_VALUE_DISCARDED
	multiplayer.connect("network_peer_disconnected", self, "on_client_disconnected");

func start():
	player_creds = load_player_credentials()
	if network.create_server(port, 32) != OK:
		printerr("Could not create server.")
		return
	multiplayer.set_network_peer(network)

remote func HandleClientInfo(username, password_hashed, skin_base64):
	var client_id = sender_id()

	if (typeof(username) & typeof(password_hashed) & typeof(skin_base64)) != TYPE_STRING:
		broadcast_client_disconnect(client_id, DisconnectType.INVALID_INFO)
		return
	# check skin
	var skin_img = Image.new();
	var load_code = skin_img.load_png_from_buffer(Marshalls.base64_to_raw(skin_base64));
	if not (load_code == OK and len(skin_img.data["data"]) == 16384):
		broadcast_client_disconnect(client_id, DisconnectType.INVALID_INFO)
		return
	if not (username in player_creds):
		# info doesnt exist yet, lets make a new "account"
		player_creds[username] = {
			"passhash": password_hashed,
			"skin": skin_base64,
		}
	else:
		if (player_creds[username]["passhash"] != password_hashed):
			broadcast_client_disconnect(client_id, DisconnectType.INVALID_CREDS);
			return;
		player_creds[username]["skin"] = skin_base64;

	save_player_credentials()
	rpc("PlayerAppearance", client_id, skin_base64);

######################################################################
# GAMEPLAY RELATED NETWORKING AFTER THIS POINT!
######################################################################

remote func HandlePlayerInfo(pos:Vector3, camRotation:Vector2):
	var client_id = sender_id();
	rpc("PlayerInfo", client_id, pos, camRotation);

remote func RequestChunkData(chunkPos: Vector2):
	emit_signal("chunk_data_requested", sender_id(), chunkPos)

# Sends chunk data to a specific client.
func send_chunk_data(toClientID: int, chunkPos: Vector2, data: PoolByteArray):
	rpc_id(toClientID, "ChunkData", data, chunkPos);

# Sends chunk data to all connected clients.
func send_chunk_data_all(chunkPos: Vector2, data: PoolByteArray):
	rpc("ChunkData", data, chunkPos);

remote func SetBlock(blockPos:Vector3, blockID:int):
	# TODO: Re-implement the server-side distance checking
	emit_signal("block_set", sender_id(), blockPos, blockID)
