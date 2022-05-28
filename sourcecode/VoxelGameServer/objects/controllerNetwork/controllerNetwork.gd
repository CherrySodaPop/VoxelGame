extends Node

signal block_set(client_id, position, block_id)
signal chunk_data_requested(client_id, position)

# networking
var peer = NetworkedMultiplayerENet.new();
enum disconnectTypes {
	INVALID_INFO,
	INVALID_CREDS,
	SERVER_CLOSED,
	LEFT,
	ADMIN_KICK,
	CHEATING,
};
# macros!
const playerCredsPath = "res://gameinfo/player_creds.json";
# secure info (defined in _ready)
var playerCreds:Dictionary;

func DisconnectClient(id:int, reason:int):
	rpc("ClientDisconnected", id, reason);

func ClientConnected(client_id:int):
	print_debug("DEBUG: Client %s connected." % client_id);

func ClientDisconnected(client_id:int):
	DisconnectClient(client_id, disconnectTypes.LEFT);
	print_debug("DEBUG: Client %s disconnected." % client_id);

func _ready():
	playerCreds = LoadPlayerCredentials();
	# create server
	peer.create_server(25565, 32);
	get_tree().network_peer = peer;
	# connect a few base functions
	get_tree().connect("network_peer_connected", self, "ClientConnected");
	get_tree().connect("network_peer_disconnected", self, "ClientDisconnected");

func sender_id() -> int:
	return get_tree().get_rpc_sender_id()

func LoadPlayerCredentials() -> Dictionary:
	var file = File.new();
	file.open(playerCredsPath, File.READ);
	playerCreds = parse_json(file.get_as_text());
	return playerCreds

func SavePlayerCredentials():
	var file = File.new();
	file.open(playerCredsPath, File.WRITE);
	file.store_string(JSON.print(playerCreds, "\t"));
	file.close();

remote func HandleClientInfo(username, passwordHashed, skinBase64):
	var client_id = sender_id()

	# make sure the sent info is proper
	if (typeof(username) != TYPE_STRING || typeof(passwordHashed) != TYPE_STRING || typeof(skinBase64) != TYPE_STRING):
		DisconnectClient(client_id, disconnectTypes.INVALID_INFO);
		return;
	# check skin
	var skinImage = Image.new();
	var loadOutput = skinImage.load_png_from_buffer(Marshalls.base64_to_raw(skinBase64));
	if (loadOutput != OK || len(skinImage.data["data"]) != 16384):
		DisconnectClient(client_id, disconnectTypes.INVALID_INFO);
		return;

	# check credentials (pass and username)
	# - first, do they exist already? - then lets check their info if it's valid
	if (playerCreds.has(username)):
		if (playerCreds[username]["passhash"] != passwordHashed):
			DisconnectClient(client_id, disconnectTypes.INVALID_CREDS);
			return;
		playerCreds[username]["skin"] = skinBase64;
	# - info doesnt exist yet, lets make a new "account"
	else:
		playerCreds[username] = {"passhash" : passwordHashed, "skin" : skinBase64};

	# TODO: REMOVE! DONT WRITE ALL THE TIME
	SavePlayerCredentials();
	rpc("PlayerAppearance", client_id, skinBase64);

######################################################################
# GAMEPLAY RELATED NETWORKING AFTER THIS POINT!
######################################################################

remote func HandlePlayerInfo(pos:Vector3, camRotation:Vector2):
	var client_id = sender_id();
	rpc("PlayerInfo", client_id, pos, camRotation);

remote func RequestChunkData(chunkPos: Vector2):
	emit_signal("chunk_data_requested", sender_id(), chunkPos)

func SendChunkData(toClientID: int, chunkPos: Vector2, data: PoolByteArray):
	rpc_id(toClientID, "ChunkData", data, chunkPos);

func SendChunkDataAll(chunkPos: Vector2, data: PoolByteArray):
	rpc("ChunkData", data, chunkPos);

remote func SetBlock(blockPos:Vector3, blockID:int):
	# TODO: Re-implement the server-side distance checking
	emit_signal("block_set", sender_id(), blockPos, blockID)
