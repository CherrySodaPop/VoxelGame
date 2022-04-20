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

remote func HandlePlayerInfo(username, passwordHashed):
	var id = get_tree().get_rpc_sender_id();
	# make sure the sent info is proper
	if (typeof(username) != TYPE_STRING || typeof(passwordHashed) != TYPE_STRING): 
		DisconnectPlayer(id, disconnectTypes.INVALID_INFO);
		return;
	# check credentials (pass and username)
	# - first, do they exist already? - then lets check their info if it's valid
	if (playerCreds.has(username)):
		if (playerCreds[username]["passhash"] != passwordHashed):
			DisconnectPlayer(id, disconnectTypes.INVALID_CREDS);
			return;
	# - info doesnt exist yet, lets make a new "account"
	else:
		playerCreds[username] = {"passhash" : passwordHashed};
	
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
	var id = get_tree().get_rpc_sender_id();
	if (playerInstances.has(id) && is_instance_valid(playerInstances[id])):
		var obj:Spatial = playerInstances[id];
		obj.global_transform.origin = pos;
		obj.camRotation = camRotation;
