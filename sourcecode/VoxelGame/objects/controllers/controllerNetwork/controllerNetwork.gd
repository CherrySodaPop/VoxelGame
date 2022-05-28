extends Node

signal connected
signal player_info_updated(client_id, position, camera_rot)
signal chunk_data_received(position, data)
signal player_appearance_received(client_id, skinBase64)

# secure info
var username = "Cherry";
var password = "my_password"
# networking
var peer = NetworkedMultiplayerENet.new();
var serverAddress = "localhost";
var serverPort = 25565;

func ConnectedToServer():
	print("Connected to server!")
	emit_signal("connected")

func FailedToConnect():
	print_debug("DEBUG: Failed to connect to %s" % (serverAddress + ":"+ str(serverPort)));

func ServerClosed():
	printerr("Server closed.")
	OS.alert("The server has been closed.", "Whoops.")

func _ready():
	# connect to server
	peer.create_client(serverAddress, serverPort)
	get_tree().network_peer = peer;
	# warning-ignore:RETURN_VALUE_DISCARDED
	get_tree().connect("connected_to_server", self, "ConnectedToServer");
	# warning-ignore:RETURN_VALUE_DISCARDED
	get_tree().connect("connection_failed", self, "FailedToConnect");
	# warning-ignore:RETURN_VALUE_DISCARDED
	get_tree().connect("server_disconnected", self, "ServerClosed");

func LoadSkin() -> String:
	var skinPath = "user://skin.png";
	var skinImage = File.new();
	if (!skinImage.file_exists(skinPath)):
		var defaultSkin:StreamTexture = load("res://assets/models/pm/skin.png");
		defaultSkin.get_data().save_png("user://skin.png");
	skinImage.open(skinPath, File.READ);
	return Marshalls.raw_to_base64(skinImage.get_buffer(skinImage.get_len()));

func GetClientInfo() -> Array:
	var passwordHashed = password.sha256_text();
	return [username, passwordHashed, LoadSkin()];

func SendClientInfo():
	var client_info = GetClientInfo()
	rpc_id(1, "HandleClientInfo", client_info[0], client_info[1], client_info[2])

remote func ClientDisconnected(client_id:int, reason:int):
	# check if we're disconnecting ourselves, if so, die!
	if client_id == get_tree().get_network_unique_id():
		get_tree().network_peer = null;
		print("DEBUG: Disconnected by server: %s" % reason);
		return;
	# no? we're disconnecting another player then, tell the butcher!
	emit_signal("player_disconnected", client_id)

######################################################################
# GAMEPLAY RELATED NETWORKING AFTER THIS POINT!
######################################################################

func sender_id() -> int:
	return get_tree().get_rpc_sender_id()

remote func PlayerAppearance(client_id: int, skinBase64: String):
	var skinImage = Image.new();
	# This data was sent from the server, we don't need to worry
	# if it's valid or not as the server has already checked it.
	skinImage.load_png_from_buffer(Marshalls.base64_to_raw(skinBase64));
	var skinTexture = ImageTexture.new()
	skinTexture.create_from_image(skinImage, 0);
	emit_signal("player_appearance_received", client_id, skinTexture)

remote func PlayerInfo(client_id: int, pos:Vector3, camRotation:Vector2):
	# todo: if their outside the viewdistance dont bother with their info
	if client_id == get_tree().get_network_unique_id():
		# This is our own info.
		return;
	emit_signal("player_info_updated", client_id, pos, camRotation);

remote func ChunkData(chunkData: PoolByteArray, chunkPos: Vector2):
	emit_signal("chunk_data_received", chunkPos, chunkData);

func RequestChunkData(chunkPos: Vector2):
	rpc_id(1, "RequestChunkData", chunkPos)

func RequestChunkDataAround(chunkPos: Vector2):
	var positions = PoolVector2Array();
	for x in range(-1, 2):
		for y in range(-1, 2):
			positions.push_back(chunkPos + Vector2(x, y));
	for position in positions:
		RequestChunkData(position);

func SendSetBlock(blockPos: Vector3, blockType: int):
	rpc_id(1, "SetBlock", blockPos, blockType)
