extends Node

# secure info
var username:String = "Cherry";
var password:String = "my_password";
# networking
var peer = ENetMultiplayerPeer.new();
var serverAddress = "localhost";
var serverPort = 25565;
var networkTickTimer
