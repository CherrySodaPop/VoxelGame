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
var objClientPlayer = preload("res://objects/clientPlayer/clientPlayer.tscn");

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
	var passwordHashed = password.sha256_text();
	rpc_id(1, "HandlePlayerInfo", username, passwordHashed);

remote func DisconnectClient(id:int, reason:int):
	# check if we're disconnecting ourselves, if so, die!
	if (Persistant.get_node("player").networkID == id):
		get_tree().network_peer = null;
		print("DEBUG: Disconnected by server.")
		return;
	# no? we're disconnecting another player then, kill em!
	if (playerInstances.has(id)):
		var playerNode:Spatial = playerInstances[id];
		if (is_instance_valid(playerNode)):
			playerNode.queue_free();
			playerInstances.erase(id);
			return;
	push_warning("controllerNetwork: No client to disconnect!")

######################################################################
# GAMEPLAY RELATED NETWORKING AFTER THIS POINT!
######################################################################

remote func PlayerInfo(networkID:int, pos:Vector3, camRotation:Vector2):
	if (Persistant.get_node("player").networkID == networkID): return;
	# todo: if their outside the viewdistance dont bother with their info and remove them
	if (!playerInstances.has(networkID)):
		var tmpObj = objClientPlayer.instance();
		get_tree().current_scene.add_child(tmpObj);
		playerInstances[networkID] = tmpObj;
	var obj:Spatial = playerInstances[networkID];
	obj.global_transform.origin = pos;
	
	var skeleton:Skeleton = obj.get_node("model").get_node("PM/Skeleton");
	print("gaming");
	var newTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
	newTransform = newTransform.rotated(Vector3.UP, camRotation.y + deg2rad(180));
	skeleton.set_bone_pose(skeleton.find_bone("core"), newTransform);
