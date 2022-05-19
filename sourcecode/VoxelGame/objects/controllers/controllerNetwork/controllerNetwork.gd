extends Node

# secure info
var username = "Cherry";
var password = "my_password"
# networking
var peer = NetworkedMultiplayerENet.new();
var serverAddress = "localhost";
var serverPort = 25565;
var networkTickTimer:float = 0.0;
var networkTick:float = 1/30;
# instances
var playerInstances:Dictionary = {};
var playerDisconnectedInstances:Dictionary = {};
var npcInstances:Dictionary = {};
var npcDestroyedInstances:Dictionary = {};
# preload info
var objClientPlayer = preload("res://objects/clientPlayer/clientPlayer.tscn");
var npcInfo = preload("res://objects/controllers/controllerNetwork/npcInfo.gd");

func _ready():
	# connect to server
	peer.create_client(serverAddress, serverPort)
	get_tree().network_peer = peer;
	# connect a few base functions
	get_tree().connect("connected_to_server", self, "ClientConnectedToServer");
	get_tree().connect("connection_failed", self, "ClientFailedToConnect");
	get_tree().connect("server_disconnected", self, "ClientServerClosed");

func _process(delta):
	if (HasTicked()): networkTickTimer = 0.0;
	networkTickTimer += delta;

func HasTicked() -> bool:
	return (networkTickTimer >= networkTick);

######################################################################
# first connection type network functions
######################################################################
func ClientConnectedToServer():
	# succesfuly connected
	pass

remote func ServerID(id:int):
	Persistant.get_node("player").networkID = id;
	SendPlayerInfo();

func ClientFailedToConnect():
	print_debug("DEBUG: Failed to connect to %s" % (serverAddress + ":"+ str(serverPort)));

func ClientServerClosed():
	pass

func SendPlayerInfo():
	# password!
	var passwordHashed = password.sha256_text();
	# skin!
	var skinPath = "user://skin.png";
	var skinImage = File.new();
	var skinBase64 = "";
	if (!skinImage.file_exists(skinPath)):
		var defaultSkin:StreamTexture = load("res://assets/models/pm/skin.png");
		defaultSkin.get_data().save_png("user://skin.png");
	skinImage.open(skinPath, File.READ);
	skinBase64 = Marshalls.raw_to_base64(skinImage.get_buffer(skinImage.get_len()));
	rpc_id(1, "HandlePlayerInfo", username, passwordHashed, skinBase64);

remote func PlayerAppearance(objID:int, skinBase64:String):
	var obj:Spatial = instance_from_id(objID);
	var skinImage = Image.new();
	var skinTexture = ImageTexture.new();
	skinImage.load_png_from_buffer(Marshalls.base64_to_raw(skinBase64));
	skinTexture.create_from_image(skinImage, 0);
	if (is_instance_valid(obj)):
		var mesh:MeshInstance = obj.get_node("model/PM/Skeleton/PMMeshObj")
		var under:SpatialMaterial = mesh.get("material/0");
		var top:SpatialMaterial = mesh.get("material/1");
		under.albedo_texture = skinTexture;
		top.albedo_texture = skinTexture;

remote func DisconnectClient(id:int, reason:int):
	# check if we're disconnecting ourselves, if so, die!
	if (Persistant.get_node("player").networkID == id):
		get_tree().network_peer = null;
		print("DEBUG: Disconnected by server: %s" % reason);
		return;
	# no? we're disconnecting another player then, kill em!
	if (playerInstances.has(id)):
		var playerNode:Spatial = playerInstances[id];
		if (is_instance_valid(playerNode)):
			playerNode.queue_free();
			playerInstances.erase(id);
			playerDisconnectedInstances[id] = true;
			return;
	push_warning("controllerNetwork: No client to disconnect!")

######################################################################
# GAMEPLAY RELATED NETWORKING AFTER THIS POINT!
######################################################################

remote func ChunkData(chunkData:PoolByteArray, chunkPos:Vector2):
	Persistant.chunkLoader.receive_chunk(chunkData, chunkPos);

remote func PlayerInfo(networkID:int, pos:Vector3, camRotation:Vector2):
	# not our own info
	if (Persistant.get_node("player").networkID == networkID): return;
	# not a disconnected player's info
	if (playerDisconnectedInstances.has(networkID)): return;
	# todo: if their outside the viewdistance dont bother with their info and remove them
	if (!playerInstances.has(networkID)):
		var tmpObj = objClientPlayer.instance();
		get_tree().current_scene.add_child(tmpObj);
		tmpObj.networkID = networkID;
		playerInstances[networkID] = tmpObj;
		rpc_id(1, "SendPlayerAppearance", tmpObj.get_instance_id(), networkID);
	var obj:Spatial = playerInstances[networkID];
	if (is_instance_valid(obj)):
		obj.global_transform.origin = pos;
		obj.camRotation = camRotation;

remote func NPCBaseInfo(id:int, pos:Vector3, camRotation:Vector2):
	# not an already destroyed npc
	if (npcDestroyedInstances.has(id)):
		return;
	if (!npcInstances.has(id)):
		var tmpObj = npcInfo.npcList[npcInfo.npcID.BASE].instance();
		get_tree().current_scene.add_child(tmpObj);
		tmpObj.networkID = id;
		npcInstances[id] = tmpObj;
	var obj:Spatial = npcInstances[id];
	if (is_instance_valid(obj)):
		obj.global_transform.origin = pos;
		obj.camRotation = camRotation;
