extends KinematicBody

var networkID:int = -1;
var username:String = "IvanTheSpaceBiker";
var camRotation:Vector2 = Vector2.ZERO;

func _ready():
	pass

func _process(delta):
	HandleMovement(delta);
	Network();

func HandleMovement(delta):
	pass

func Network():
	# todo: make it so it only networks to players in range?
	# possible but maybe expensive...
	var network:Node = Persistant.get_node("controllerNetwork");
	if (network.HasTicked()):
		network.rpc("PlayerInfo", networkID, global_transform.origin, camRotation);
