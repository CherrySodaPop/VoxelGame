class_name npc_base
extends KinematicBody

var health:int = 20;
var velocity:Vector3 = Vector3.ZERO;
var acceleration:float = 16.0;
var baseSpeed:float = 2.5;
var gravity:float = 36.0;

var camRotation:Vector2 = Vector2.ZERO;

var DEBUG_movementTriggered:bool = false;
var DEBUG_movementTimer:float = 0.0;
var DEBUG_movementPosition:Vector2 = Vector2.ZERO;

func _ready():
	Persistant.get_node("controllerNPCS").AddNPC(self.get_instance_id());

func _process(delta):
	HandleTempMovement(delta);
	HandleMovement(delta);
	Network();

func HandleTempMovement(delta):
	# TODO: proper movement, this is for demonstration purposes ONLY
	if (!DEBUG_movementTriggered):
		DEBUG_movementTimer += delta;
		if (DEBUG_movementTimer >= 1.0):
			DEBUG_movementTriggered = true;
			DEBUG_movementTimer = 0.0;
			var randDir = rand_range(0.0, PI);
			var randRange = rand_range(1.0, 40.0);
			DEBUG_movementPosition.x = global_transform.origin.x + sin(randDir) * randRange;
			DEBUG_movementPosition.y = global_transform.origin.z + cos(randDir) * randRange;
	else:
		DEBUG_movementTimer += delta;
		if (DEBUG_movementTimer >= 8.0):
			DEBUG_movementTriggered = false;
			DEBUG_movementTimer = 0.0;
			DEBUG_movementPosition.x = global_transform.origin.x;
			DEBUG_movementPosition.y = global_transform.origin.z;

func HandleMovement(delta):
	var posVec2 = Vector2(global_transform.origin.x, global_transform.origin.z);
	var desiredVec2Dir = (DEBUG_movementPosition - posVec2).normalized();
	if (DEBUG_movementPosition.distance_to(posVec2) < 0.25):
		desiredVec2Dir = Vector2.ZERO;
	
	var velocityVec2 = Vector2(velocity.x, velocity.z);
	var storedInterpolateVelocityVec2 = velocityVec2.linear_interpolate(desiredVec2Dir * baseSpeed, acceleration * delta);
	
	velocity = Vector3(storedInterpolateVelocityVec2.x, velocity.y, storedInterpolateVelocityVec2.y);
	velocity.y -= gravity * delta;
	
	move_and_slide(velocity, Vector3(0, 1, 0));
	if (is_on_floor() || is_on_ceiling()):
		velocity.y = 0.0;

func Network():
	var network = Persistant.get_node("controllerNetwork");
	if (network.HasTicked()):
		network.rpc("NPCBaseInfo", self.get_instance_id(), global_transform.origin, camRotation);

func _exit_tree():
	Persistant.get_node("controllerNPCS").RemoveNPCWithID(self.get_instance_id());
