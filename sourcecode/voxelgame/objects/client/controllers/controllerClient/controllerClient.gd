extends Node

# secure info
var username:String = "Cherry"
var password:String = "my_password"
# networking
var peer:ENetMultiplayerPeer = null
var serverAddress = "localhost"
var serverPort = 25565
var networkTickTimer:float = 0.0
var networkTick:float = 1.0/30.0
var networkID:int = -1 # the unique network id given by the server for the client

func _ready():
	multiplayer.connected_to_server.connect(ConnectedToServer)
	multiplayer.connection_failed.connect(FailedToConnect)

func ConnectToServer():
	if (peer == null):
		peer = ENetMultiplayerPeer.new()
		peer.create_client(serverAddress, serverPort)
		multiplayer.set_multiplayer_peer(peer)

func FreePeer():
	peer.free()
	peer = null

######################################################################
# first connection type network functions
######################################################################
func ConnectedToServer():
	print("Connected to server")

func FailedToConnect():
	FreePeer()

@rpc
func InitialHandshake(id:int):
	networkID = id
	SendUserInfo()

func SendUserInfo():
	# pass
	var passwordHashed = password.sha256_text()
	# skin
	var skinPath:String = pPersistent.gamePath + "data/client/skin.png"
	var skinBase64 = ""
	# default skin fallback
	if not FileAccess.file_exists(skinPath):
		var defaultSkin:Image = load("res://assets/models/player/skin.png")
		defaultSkin.save_png(skinPath)
	var skinImage = FileAccess.open(skinPath, FileAccess.READ)
	skinBase64 = Marshalls.raw_to_base64(skinImage.get_buffer(skinImage.get_length()))
	skinImage.free()
	rpc_id(1, "HandleUserInfo", username, passwordHashed, skinBase64)
