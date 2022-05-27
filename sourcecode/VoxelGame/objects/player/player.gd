extends KinematicBody

var animationTimer:float = 0.0;

var networkID = -1;

# with time, this stuff will moved to the server, but for its early staging, it's fine
var velocity:Vector3 = Vector3.ZERO;
var acceleration:float = 16.0;
var walkSpeed:float = 4.0;
var runSpeed:float = 6.0;
var jumpForce:float = 10.0;
var gravity:float = 36.0;

onready var prevPos:Vector3 = global_transform.origin;
var prevChunk:Vector2 = Vector2.ZERO;
var currentChunk:Vector2 = Vector2.ZERO;
var lookingAtBlock:Vector3 = Vector3.ZERO;
var adjacentLookingAtBlock:Vector3 = Vector3.ZERO;

var bodyRotation:float = 0.0;
var armLeftRotation:float = 0.0;
var armRightRotation:float = 0.0;
var legLeftRotation:float = 0.0;
var legRightRotation:float = 0.0;

var mouseSensitivity:float = 0.2;
var lockMouse:bool = false;
var thirdPerson:bool = false;
var wireframeView:bool = false;

signal enteredNewChunk;

func _ready():
	global_transform.origin.y = 100; # TEMP: Prevent spawning underneath terrain
	$model/PM/Skeleton/PMMeshObj.cast_shadow = GeometryInstance.SHADOW_CASTING_SETTING_SHADOWS_ONLY;

	# update skin
	var skinFile = File.new();
	var skinImage = Image.new();
	var skinImageTexture = ImageTexture.new();
	if (skinFile.file_exists("user://skin.png")):
		skinFile.open("user://skin.png", File.READ);
		skinImage.load_png_from_buffer(skinFile.get_buffer(skinFile.get_len()));
		skinImageTexture.create_from_image(skinImage, 0);
		var mesh:MeshInstance = $model/PM/Skeleton/PMMeshObj;
		var under:SpatialMaterial = mesh.get("material/0");
		var top:SpatialMaterial = mesh.get("material/1");
		under.albedo_texture = skinImageTexture;
		top.albedo_texture = skinImageTexture;

func _process(delta):
	animationTimer += (delta * 2.0) + ((GetVelocityDif(delta) / walkSpeed) * delta * 10.0);

	UpdateMiscInfo(delta);
	HandleActions(delta);
	HandleMovement(delta);
	HandleAnimation(delta);
	HandleHud(delta);
	Network();

func TogglePerspective():
	thirdPerson = not thirdPerson
	if thirdPerson:
		$cameraJoint.get_node("camera").translation.z = 2;
		$model/PM/Skeleton/PMMeshObj.cast_shadow = GeometryInstance.SHADOW_CASTING_SETTING_ON;
	else:
		$cameraJoint.get_node("camera").translation.z = 0;
		$model/PM/Skeleton/PMMeshObj.cast_shadow = GeometryInstance.SHADOW_CASTING_SETTING_SHADOWS_ONLY;

func ToggleWireframeView():
	wireframeView = not wireframeView;
	VisualServer.set_debug_generate_wireframes(wireframeView);
	get_viewport().debug_draw = (
		Viewport.DEBUG_DRAW_WIREFRAME
		if wireframeView
		else Viewport.DEBUG_DRAW_DISABLED
	);

func _input(event:InputEvent):
	if (event is InputEventMouseMotion):
		HandleCamera(event.relative);
	elif event.is_action_pressed("playerTogglePerspective"):
		TogglePerspective();
	elif event.is_action_pressed("debugToggleWireframe"):
		ToggleWireframeView();
	elif event.is_action_pressed("gamePause"):
		if (Input.get_mouse_mode() == Input.MOUSE_MODE_VISIBLE):
			Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED);
		else:
			Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE);

func UpdateMiscInfo(delta):
	# chunk pos update
	currentChunk.x = floor(global_transform.origin.x / Persistent.chunkSize.x);
	currentChunk.y = floor(global_transform.origin.z / Persistent.chunkSize.x);
	if (currentChunk != prevChunk):
		emit_signal("enteredNewChunk");
		prevChunk = currentChunk;

	$RayCast.rotation = $cameraJoint.rotation;

	# looking at block
	lookingAtBlock = $RayCast.get_collision_point();
	lookingAtBlock += (lookingAtBlock - $cameraJoint.global_transform.origin).normalized() * 0.001;
	lookingAtBlock.x = floor(lookingAtBlock.x);
	lookingAtBlock.y = ceil(lookingAtBlock.y);
	lookingAtBlock.z = floor(lookingAtBlock.z);

	# adjacent block
	adjacentLookingAtBlock = $RayCast.get_collision_point();
	adjacentLookingAtBlock -= (adjacentLookingAtBlock - $cameraJoint.global_transform.origin).normalized() * 0.01;
	adjacentLookingAtBlock.x = floor(adjacentLookingAtBlock.x);
	adjacentLookingAtBlock.y = ceil(adjacentLookingAtBlock.y);
	adjacentLookingAtBlock.z = floor(adjacentLookingAtBlock.z);

func HandleActions(_delta):
	if (Input.is_action_pressed("playerPrimaryAction")):
		Persistent.controllerNetwork.rpc_unreliable_id(1, "SetBlock", lookingAtBlock, 0);
	if (Input.is_action_pressed("playerSecondaryAction")):
		Persistent.controllerNetwork.rpc_unreliable_id(1, "SetBlock", adjacentLookingAtBlock, 23);
		pass #Persistent.get_node("chunkGeneration").set_block_gd(adjacentLookingAtBlock, 23);

func HandleMovement(delta):
	prevPos = global_transform.origin;

	var desiredVec2Dir:Vector2 = Vector2.ZERO;
	if (Input.is_action_pressed("playerMoveForward")):
		desiredVec2Dir.y += 1;
	if (Input.is_action_pressed("playerMoveBackward")):
		desiredVec2Dir.y -= 1;
	if (Input.is_action_pressed("playerMoveLeft")):
		desiredVec2Dir.x += 1;
	if (Input.is_action_pressed("playerMoveRight")):
		desiredVec2Dir.x -= 1;
	desiredVec2Dir = CorrectRotation(desiredVec2Dir.normalized());
	var velocityVec2 = Vector2(velocity.x, velocity.z);
	var movementSpeed = runSpeed if Input.is_action_pressed("playerSprint") else walkSpeed;
	var storedInterpolateVelocityVec2 = velocityVec2.linear_interpolate(
		desiredVec2Dir * movementSpeed, acceleration * delta
	);

	if (Input.is_action_pressed("playerJump") && is_on_floor()):
		velocity.y += jumpForce;

	velocity = Vector3(storedInterpolateVelocityVec2.x, velocity.y, storedInterpolateVelocityVec2.y);
	velocity.y -= gravity * delta;

	move_and_slide(velocity, Vector3(0, 1, 0));
	if (is_on_floor() || is_on_ceiling()):
		velocity.y = 0.0;

func HandleAnimation(delta):
	var fixedCamRotationY = $cameraJoint.rotation.y;
	var headBodyDif = fmod(fixedCamRotationY - bodyRotation, TAU);
	var headBodyDifShort = fmod(2 * headBodyDif, TAU) - headBodyDif;
	var bodyToHeadRotation = fixedCamRotationY + (headBodyDifShort * 1.0);

	var headTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
	headTransform = headTransform.rotated(Vector3.LEFT, $cameraJoint.rotation.x);
	headTransform = headTransform.rotated(Vector3.FORWARD, fixedCamRotationY);
	var armLeftTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
	var armRightTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
	var legLeftTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
	var legRightTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);

	var skeleton:Skeleton = get_node("model").get_node("PM/Skeleton");

	if (is_zero_approx(round(velocity.x)) && is_zero_approx(round(velocity.z))):
		if (abs(headBodyDifShort) > deg2rad(25)):
			bodyRotation = lerp_angle(bodyRotation, fixedCamRotationY + deg2rad(-sign(headBodyDifShort) * 25.0), 8.0 * delta);
		var bodyTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
		bodyTransform = bodyTransform.rotated(Vector3.UP, bodyRotation);
		skeleton.set_bone_pose(skeleton.find_bone("core"), bodyTransform);
	else:
		var vec2velocity = Vector2(velocity.x, velocity.z).normalized();
		var desiredBodyAngle = -Vector2.ZERO.angle_to_point(vec2velocity) + deg2rad(90);

		var headMovementBodyDif = fmod(fixedCamRotationY - desiredBodyAngle, TAU);
		var headMovementBodyDifShort = fmod(2 * headMovementBodyDif, TAU) - headMovementBodyDif;
		if (abs(headMovementBodyDifShort) > deg2rad(25)):
			desiredBodyAngle = fixedCamRotationY + deg2rad(-sign(headMovementBodyDifShort) * 25.0);

		bodyRotation = lerp_angle(bodyRotation, desiredBodyAngle, 8.0 * delta);
		var bodyTransform = Transform(Vector3.RIGHT, Vector3.UP, Vector3.BACK, Vector3.ZERO);
		bodyTransform = bodyTransform.rotated(Vector3.UP, bodyRotation);
		skeleton.set_bone_pose(skeleton.find_bone("core"), bodyTransform);

	var speedAmp = GetVelocityDif(delta) / walkSpeed;
	armLeftRotation = sin(animationTimer) * (0.06 + (speedAmp * 0.5));
	armRightRotation = -sin(animationTimer) * (0.06 + (speedAmp * 0.5));
	legLeftRotation = -sin(animationTimer) * (speedAmp);
	legRightRotation = sin(animationTimer) * (speedAmp);
	armLeftTransform = armLeftTransform.rotated(Vector3.FORWARD, armLeftRotation);
	armRightTransform = armRightTransform.rotated(Vector3.BACK, armRightRotation);
	legLeftTransform = legLeftTransform.rotated(Vector3.FORWARD, legLeftRotation);
	legRightTransform = legRightTransform.rotated(Vector3.BACK, legRightRotation);
	skeleton.set_bone_pose(skeleton.find_bone("arm_L"), armLeftTransform);
	skeleton.set_bone_pose(skeleton.find_bone("arm_R"), armRightTransform);
	skeleton.set_bone_pose(skeleton.find_bone("leg_L"), legLeftTransform);
	skeleton.set_bone_pose(skeleton.find_bone("leg_R"), legRightTransform);

	skeleton.set_bone_pose(skeleton.find_bone("head"), headTransform);

func HandleHud(_delta):
	HandleBlockHighlighting();

func HandleBlockHighlighting():
	$blockOutlineJoint.global_transform.origin = lookingAtBlock;

func Network():
	if (Persistent.controllerNetwork.HasTicked()):
		Persistent.controllerNetwork.rpc_id(
			1,
			"PlayerInfo",
			global_transform.origin,
			Vector2($cameraJoint.rotation.x, $cameraJoint.rotation.y)
		);

func HandleCamera(mouseMotion:Vector2):
	if (Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED):
		mouseMotion = -mouseMotion * mouseSensitivity;
		mouseMotion = Vector2(deg2rad(mouseMotion.x), deg2rad(mouseMotion.y));
		$cameraJoint.rotate_y(mouseMotion.x);
		# Prevent the player from snapping their neck
		$cameraJoint.rotation.x = clamp($cameraJoint.rotation.x + mouseMotion.y, -PI/2, PI/2);

func CorrectRotation(direction:Vector2):
	return direction.rotated(-$cameraJoint.rotation.y + deg2rad(180)).normalized();

func GetVelocityDif(delta) -> float:
	var vec2prevpos = Vector2(prevPos.x, prevPos.z);
	var vec2currentpos = Vector2(global_transform.origin.x, global_transform.origin.z);
	var posDistance = vec2prevpos.distance_to(vec2currentpos);
	return posDistance / delta;
