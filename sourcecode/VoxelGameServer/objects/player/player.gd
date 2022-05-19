extends KinematicBody

var networkID:int = -1;
var username:String = "IvanTheSpaceBiker";
var camRotation:Vector2 = Vector2.ZERO;

var currentChunk = Vector2(0, 0);

func _ready():
	pass

func _process(delta):
	HandleMovement(delta);
	UpdateMiscInfo(delta);
	Network();

func HandleMovement(delta):
	pass

func UpdateMiscInfo(delta):
	currentChunk.x = floor(global_transform.origin.x / 32.0);
	currentChunk.y = floor(global_transform.origin.z / 32.0);

func Network():
	# todo: make it so it only networks to players in range?
	# possible but maybe expensive...
	var network:Node = Persistant.get_node("controllerNetwork");
	if (network.HasTicked()):
		# DEBUG
		#var senderID = get_tree().get_rpc_sender_id();
		#var chunkData = network.chunkLoader.terrain_encoded(currentChunk);
		#if (chunkData != null):
		#	chunkData = chunkData.compress();
		#	if chunkData != null:
		#		network.rpc_unreliable_id(senderID, "ChunkData", chunkData, currentChunk);
			
		network.rpc("PlayerInfo", networkID, global_transform.origin, camRotation);
