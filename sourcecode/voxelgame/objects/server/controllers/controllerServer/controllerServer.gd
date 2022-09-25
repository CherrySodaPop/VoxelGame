extends Node

var gamePath = OS.get_executable_path().get_base_dir() + "/";

# secure info
var userData:Dictionary = {}; # username, pass
# networking
var peer = ENetMultiplayerPeer.new();
enum disconnectTypes {
	INVALID_INFO,
	INVAILD_CREDS,
	SERVER_CLOSED,
	LEFT,
	ADMIN_KICK,
	ADMIN_BAN,
	CHEATING, };
var maxClients:int = 32;
var networkTickTimer:float = 0.0;
var networkTick:float = 1.0/30.0;
# entites
var playerData:Dictionary = {}; # world location, items, etc. not to be accessed constantly, just for game saving
var playerEntites:Dictionary = {}; # easy access to all player entities

func _ready():
	# loading phase
	PrepareGameInfo();
	# create server
	peer.peer_connected.connect(ClientConnected);
	peer.create_server(25565, maxClients);
	multiplayer.set_multiplayer_peer(peer);

func PrepareGameInfo():
	# load block info, entity info, etc.
	LoadPlayerCredentials();

func LoadPlayerCredentials():
	#var file = File.new();
	pass

func PrepareDimensions():
	# TODO: some file that describes the dimensions to create, terrain style, etc.
	var dimensionViewportContainer = SubViewportContainer.new();
	var dimensionSubViewport = SubViewport.new();
	add_child(dimensionViewportContainer)
	dimensionViewportContainer.add_child(dimensionSubViewport);

######################################################################
# connection type network functions
######################################################################
func ClientConnected(id:int):
	rpc_id(id, "InitialHandshake", id);

@rpc(any_peer)
func HandleUserInfo(username, passwordHashed, skin):
	var id = peer.get_remote_sender_id();
	var doubleHashedPass = passwordHashed.sha256_text();
	
	# verify info
	var skinImage = Image.new();
	var loadOutput = skinImage.load_png_from_buffer(Marshalls.base64_to_raw(skin));
	if (typeof(username) != TYPE_STRING || typeof(passwordHashed) != TYPE_STRING || typeof(skin) != TYPE_STRING ||
		username.length() > 20 || passwordHashed != 64 || loadOutput != OK || len(skinImage.data["data"]) != 65536):
		DisconnectPlayer(id, disconnectTypes.INVALID_INFO);
		return;
	
	# newly connecting player?
	if (!userData.has(username)):
		userData[username] = {
			"password" : doubleHashedPass,
			"skin" : skin,
		};
	if (!playerData.has(username)):
		playerData[username] = {
			"dimension" : "overworld",
			"pos" : Vector3.ZERO,
			"stats" : {
				"health" : 20,
				},
			"items" : {
				"0" : {"count": 64, "metadata": {}},
				"32" : {"count": 64, "metadata": {}},
				},
		};
	
	# invalid creds
	if (userData[username]["password"] != doubleHashedPass):
		DisconnectPlayer(id, disconnectTypes.INVAILD_CREDS);
		return;
	# everything is correct, create player
	

func DisconnectPlayer(id, disconnectType):
	rpc("DisconnectPlayer", id, disconnectType);
	peer.get_peer(id).peer_disconnect_now();