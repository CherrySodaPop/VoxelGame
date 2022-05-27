extends Spatial

var networkID:int = -1;
var username:String = "IvanTheSpaceBiker"

var bodyRotation:float = 0.0;
var camRotation:Vector2 = Vector2.ZERO;

var timeoutDestroy:float = 0.0;
var timeoutDestroyMax:float = 100.0;

func _ready():
	pass

func _process(delta):
	HandleAnimations(delta);
	HandleDestroyTimeout(delta);

func HandleAnimations(delta):
	var fixedCamRotationY = camRotation.y + deg2rad(180);
	var headBodyDif = fmod(fixedCamRotationY - bodyRotation, TAU);
	var headBodyDifShort = fmod(2 * headBodyDif, TAU) - headBodyDif;
	var bodyToHeadyRotation = fixedCamRotationY + (headBodyDifShort * 1.0);
	if (abs(headBodyDifShort) > deg2rad(25)):
		bodyRotation = lerp_angle(bodyRotation, fixedCamRotationY + deg2rad(-sign(headBodyDifShort) * 25.0), 8.0 * delta);

	var skeleton:Skeleton = get_node("model").get_node("PM/Skeleton");
	var bodyTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
	bodyTransform = bodyTransform.rotated(Vector3.UP, bodyRotation);

	var headTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
	headTransform = headTransform.rotated(Vector3.LEFT, camRotation.x);
	headTransform = headTransform.rotated(Vector3.FORWARD, fixedCamRotationY);

	skeleton.set_bone_pose(skeleton.find_bone("core"), bodyTransform);
	skeleton.set_bone_pose(skeleton.find_bone("head"), headTransform);

func HandleDestroyTimeout(delta):
	timeoutDestroy += delta;
	if (timeoutDestroy >= timeoutDestroyMax):
		if (Persistent.controllerNetwork.playerInstances.has(networkID)):
			Persistent.controllerNetwork.playerInstances.erase(networkID);
			queue_free();
