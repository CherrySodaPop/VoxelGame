extends Control

# all your UI elements in one place, how nice!

var overrideInput:bool = true; # disables any player interactions till disabled

func _ready():
	$mainmenu/multiplayer.pressed.connect(Enter_MultiplayerMenu);
	# multiplayer
	$multiplayermenu/join.pressed.connect(MultiplayerMenu_JoinServer);
	$multiplayermenu/cancel.pressed.connect(MultiplayerMenu_Cancel);
	# settings

# multiplayer
func Enter_MultiplayerMenu():
	$mainmenu.visible = false;
	$multiplayermenu.visible = true;

func MultiplayerMenu_JoinServer():
	if ($multiplayermenu/port.text.is_valid_int()):
		$multiplayermenu/joiningscreen.visible = true;
		var controller = Persistent.get_node("controllerClient");
		controller.serverAddress = $multiplayermenu/address.text;
		controller.serverPort = $multiplayermenu/port.text.to_int();
		controller.ConnectToServer();

func MultiplayerMenu_Cancel():
	$multiplayermenu.visible = false;
	$mainmenu.visible = true;

# settings
