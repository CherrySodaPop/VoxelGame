extends Node

# secure info
var playerCreds:Dictionary = {};
# networking
var peer = NetworkedMultiplayerENet.new();
enum disconnectTypes {
	INVALID_INFO,
	INVALID_CREDS,
	SERVER_CLOSED,
	LEFT,
	ADMIN_KICK,
	CHEATING, };
var networkTickTimer:float = 0.0;
var networkTick:float = 1/30;
# instances
var playerInstances:Dictionary = {};
var objPlayer = preload("res://objects/player/player.tscn");
onready var chunkLoader = get_node("chunkCreator")
# macros!
const gameinfoPlayerCredsPath = "res://gameinfo/player_creds.json";

func _ready():
	# load any info before starting (creds, world, etc)
	PrepareGameInfo();
	# create server
	peer.create_server(25565, 32);
	get_tree().network_peer = peer;
	# connect a few base functions
	get_tree().connect("network_peer_connected", self, "ClientConnected");
	get_tree().connect("network_peer_disconnected", self, "ClientDisconnected");

func PrepareGameInfo():
	LoadPlayerCredentials();

# player creds loader and saver
func LoadPlayerCredentials():
	var file = File.new();
	file.open(gameinfoPlayerCredsPath, File.READ);
	playerCreds = parse_json(file.get_as_text());

func SavePlayerCredentials():
	var file = File.new();
	file.open(gameinfoPlayerCredsPath, File.WRITE);
	file.store_string(JSON.print(playerCreds, "\t"));
	file.close();

func _process(delta):
	if (HasTicked()): networkTickTimer = 0.0;
	networkTickTimer += delta;

func HasTicked() -> bool:
	return (networkTickTimer >= networkTick);

######################################################################
# first connection type network functions
######################################################################
func ClientConnected(id:int):
	print_debug("DEBUG: Client %s connected." % id);
	rpc_id(id, "ServerID", id);

func ClientDisconnected(id:int):
	DisconnectPlayer(id, disconnectTypes.LEFT);
	print_debug("DEBUG: Client %s disconnected." % id);

remote func HandlePlayerInfo(username, passwordHashed, skinBase64):
	var id = get_tree().get_rpc_sender_id();

	# make sure the sent info is proper
	if (typeof(username) != TYPE_STRING || typeof(passwordHashed) != TYPE_STRING || typeof(skinBase64) != TYPE_STRING):
		DisconnectPlayer(id, disconnectTypes.INVALID_INFO);
		return;
	# check skin
	var skinImage = Image.new();
	var loadOutput = skinImage.load_png_from_buffer(Marshalls.base64_to_raw(skinBase64));
	print(len(skinImage.data["data"]));
	if (loadOutput != OK || len(skinImage.data["data"]) != 16384):
		DisconnectPlayer(id, disconnectTypes.INVALID_INFO);
		return;

	# check credentials (pass and username)
	# - first, do they exist already? - then lets check their info if it's valid
	if (playerCreds.has(username)):
		if (playerCreds[username]["passhash"] != passwordHashed):
			DisconnectPlayer(id, disconnectTypes.INVALID_CREDS);
			return;
		playerCreds[username]["skin"] = skinBase64;
	# - info doesnt exist yet, lets make a new "account"
	else:
		playerCreds[username] = {"passhash" : passwordHashed, "skin" : skinBase64};

	# all is good, create the player
	var tmpObj = objPlayer.instance();
	get_tree().current_scene.add_child(tmpObj);
	# set player info
	tmpObj.networkID = id;
	tmpObj.username = username;
	# now store it for ease of access
	playerInstances[id] = tmpObj;

	# TODO: REMOVE! DONT WRITE ALL THE TIME
	SavePlayerCredentials();

remote func SendPlayerAppearance(doNotTouchID:int, requestedID:int):
	var senderID = get_tree().get_rpc_sender_id();
	if (playerInstances.has(requestedID)):
		var objPlayer = playerInstances[requestedID];
		var skinBase64 = playerCreds[objPlayer.username]["skin"];
		rpc_id(senderID, "PlayerAppearance", doNotTouchID, skinBase64);
		return;
	print("Failed to send player appearance.")

func DisconnectPlayer(id:int, reason:int):
	if (playerInstances.has(id)):
		playerInstances[id].queue_free();
		print("destroyed!!!")
		playerInstances.erase(id);
	rpc("DisconnectClient", id, reason);

######################################################################
# GAMEPLAY RELATED NETWORKING AFTER THIS POINT!
######################################################################

remote func PlayerInfo(pos:Vector3, camRotation:Vector2):
	var senderID = get_tree().get_rpc_sender_id();
	if (playerInstances.has(senderID) && is_instance_valid(playerInstances[senderID])):
		var obj:Spatial = playerInstances[senderID];
		obj.global_transform.origin = pos;
		obj.camRotation = camRotation;

remote func SendChunkData(chunkPos: Vector2):
	var senderID = get_tree().get_rpc_sender_id();
	var positions = chunkLoader.load_around_chunk_gd(chunkPos);
	for chunkPos in positions:
		var chunkDataPacked:PoolByteArray = chunkLoader.chunk_data_packed(chunkPos);
		if chunkDataPacked != null:
			rpc_id(senderID, "ChunkData", chunkDataPacked, chunkPos, true);

remote func SetBlock(blockPos:Vector3, blockID:int):
	var senderID = get_tree().get_rpc_sender_id();
	if (playerInstances.has(senderID) && is_instance_valid(playerInstances[senderID])):
		var obj:Spatial = playerInstances[senderID];
		if (obj.global_transform.origin.distance_to(blockPos) <= 4.0):
			Persistant.get_node("controllerNetwork/chunkCreator").set_block_gd(blockPos, blockID);
			var chunkPos:Vector2 = Vector2(floor(blockPos.x / 32), floor(blockPos.z / 32));
			var chunkDataPacked:PoolByteArray = chunkLoader.chunk_data_packed(chunkPos);
			if (chunkDataPacked != null):
				# TODO: Make update_nearby `true` if this is a chunk boundary.
				rpc_unreliable("ChunkData", chunkDataPacked, chunkPos, false);
