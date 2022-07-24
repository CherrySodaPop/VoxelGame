extends Node

# working path

# secure info
var playerData:Dictionary = {}; # password hash, world location, items, etc.
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
var networkTickTimer:float = 0.0;
var networkTick:float = 1.0/30.0;
# entites
var playerEntites:Dictionary = {};

func _ready():
	# loading phase
	PrepareGameInfo();
	# create server
	peer.create_server(25565, 32);
	MultiplayerAPI.peer_connected.connect(ClientConnected);

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
# first connection type network functions
######################################################################
func ClientConnected(id:int):
	rpc_id(id, "InitialHandshake", id);

@rpc(any_peer)
func HandlePlayerInfo(username, passwordHashed):
	# newly connecting player?
	if (!playerData.has(username)):
		playerData[username]["password"];
