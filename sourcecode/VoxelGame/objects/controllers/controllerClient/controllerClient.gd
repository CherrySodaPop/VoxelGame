extends Node

# secure info
var username:String = "Cherry";
var password:String = "my_password";
# networking
var peer = ENetMultiplayerPeer.new();
var serverAddress = "localhost";
var serverPort = 25565;
var networkTickTimer:float = 0.0;
var networkTick:float = 1.0/30.0;
var networkID:int = -1; # the unique network id given by the server for the client

func _ready():
	peer.create_client(serverAddress, serverPort);
	MultiplayerAPI.connected_to_server.connect(ConnectedToServer);
	MultiplayerAPI.connection_failed.connect(FailedToConnect);

######################################################################
# first connection type network functions
######################################################################
func ConnectedToServer():
	pass

func FailedToConnect():
	pass

@rpc
func InitialHandshake(id:int):
	networkID = id;
	SendPlayerInfo();

func SendPlayerInfo():
	var passwordHashed = password.sha256_text();
	rpc_id(1, "HandlePlayerInfo", username, passwordHashed)
