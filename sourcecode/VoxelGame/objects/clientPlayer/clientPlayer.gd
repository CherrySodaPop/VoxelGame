extends KinematicBody

# with time, this stuff will moved to the server, but for its early staging, it's fine

var velocity:Vector3 = Vector3.ZERO;
var acceleration:float = 16.0
var walkSpeed:float = 4.317
var runSpeed:float = 5.612

var prevChunk:Vector2 = Vector2.ZERO;
var currentChunk:Vector2 = Vector2.ZERO;
var lookingAtBlock:Vector3 = Vector3.ZERO;

var mouseSensitivity:float = 0.2;
var lockMouse:bool = false;

signal enteredNewChunk;

func _ready():
	global_transform.origin.y = 20; # TEMP: Prevent spawning underneath terrain
	$model/PM/Skeleton/PMMeshObj.cast_shadow = GeometryInstance.SHADOW_CASTING_SETTING_SHADOWS_ONLY;

func _process(delta):
	if (Input.is_action_just_pressed("gamePause")):
		if (Input.get_mouse_mode() == Input.MOUSE_MODE_VISIBLE):
			Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED);
		else:
			Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE);

	UpdateMiscInfo(delta);
	HandleMovement(delta);
	HandleAnimation(delta);
	HandleHud(delta);

func _input(event:InputEvent):
	if (event is InputEventMouseMotion):
		HandleCamera(event.relative);

func UpdateMiscInfo(delta):
	# chunk pos update
	var xn:int = 0;
	if (global_transform.origin.x < 0): xn = 1;
	var zn:int = 0;
	if (global_transform.origin.z < 0): zn = 1;

	var q = global_transform.origin.x;
	currentChunk.x = floor((global_transform.origin.x) / Persistant.chunkSize.x);
	currentChunk.y = floor((global_transform.origin.z) / Persistant.chunkSize.x);

	if (currentChunk != prevChunk):
		emit_signal("enteredNewChunk");
		prevChunk = currentChunk;
		
	lookingAtBlock = $camera/RayCast.get_collision_point();
	lookingAtBlock += (lookingAtBlock - $camera.global_transform.origin).normalized() * 0.001;
	lookingAtBlock.x = floor(lookingAtBlock.x);
	lookingAtBlock.y = ceil(lookingAtBlock.y);
	lookingAtBlock.z = floor(lookingAtBlock.z);

func HandleMovement(delta):
	var desiredVec2Dir:Vector2 = Vector2.ZERO;
	if (Input.is_action_pressed("playerMoveForward")):
		desiredVec2Dir.y += 1;
	if (Input.is_action_pressed("playerMoveBackward")):
		desiredVec2Dir.y -= 1;
	if (Input.is_action_pressed("playerMoveLeft")):
		desiredVec2Dir.x += 1;
	if (Input.is_action_pressed("playerMoveRight")):
		desiredVec2Dir.x -= 1;
	desiredVec2Dir = CorrectRotation(desiredVec2Dir.normalized() * 10.0);
	var velocityVec2 = Vector2(velocity.x, velocity.z);
	var storedInterpolateVelocityVec2 = velocityVec2.linear_interpolate(desiredVec2Dir * walkSpeed, acceleration * delta)
	velocity = Vector3(storedInterpolateVelocityVec2.x, velocity.y, storedInterpolateVelocityVec2.y);

	var desiredUpDownDir:float = 0.0;
	if (Input.is_action_pressed("playerJump")):
		desiredUpDownDir += 10;
	if (Input.is_action_pressed("playerCrouch")):
		desiredUpDownDir -= 10;

	velocity.y += ((desiredUpDownDir * walkSpeed) - velocity.y) * acceleration * delta;

	move_and_slide(velocity, Vector3(0, 1, 0));

func HandleAnimation(delta):
	$model.rotation.y = $camera.rotation.y + deg2rad(180);

func HandleHud(delta):
	HandleBlockHighlighting();

func HandleBlockHighlighting():
	$blockOutlineJoint.global_transform.origin = lookingAtBlock;
#	var topLeft = $camera.unproject_position(lookingAtBlock);
#	var topRight = $camera.unproject_position(lookingAtBlock + Vector3(1, 0, 0));
#	var topBottomLeft = $camera.unproject_position(lookingAtBlock + Vector3(0, 0, 1));
#	var topBottomRight = $camera.unproject_position(lookingAtBlock + Vector3(1, 0, 1));
#	var bottomLeft = $camera.unproject_position(lookingAtBlock + Vector3(0, -1, 0));
#	var bottomRight = $camera.unproject_position(lookingAtBlock + Vector3(1, -1, 0));
#	var bottomBottomLeft = $camera.unproject_position(lookingAtBlock + Vector3(0, -1, 1));
#	var bottomBottomRight = $camera.unproject_position(lookingAtBlock + Vector3(1, -1, 1));
#	$Control/blockOutline.points[0] = topLeft;
#	$Control/blockOutline.points[1] = topRight;
#	$Control/blockOutline.points[2] = topBottomLeft;
#	$Control/blockOutline.points[3] = topBottomRight;
#	$Control/blockOutline.points[4] = bottomLeft;
#	$Control/blockOutline.points[5] = bottomRight;
#	$Control/blockOutline.points[6] = bottomBottomLeft;
#	$Control/blockOutline.points[7] = bottomBottomRight;

func HandleCamera(mouseMotion:Vector2):
	if (Input.get_mouse_mode() == Input.MOUSE_MODE_CAPTURED):
		mouseMotion = -mouseMotion * mouseSensitivity;
		$camera.rotate_y(deg2rad(mouseMotion.x));

		var allowRotation:bool = true;
		if (($camera.rotation.x + deg2rad(mouseMotion.y)) >= PI/2):
			$camera.rotation.x = PI/2;
			allowRotation = false;
		if (($camera.rotation.x + deg2rad(mouseMotion.y)) <= -PI/2):
			$camera.rotation.x = -PI/2;
			allowRotation = false;
		if (allowRotation):
			$camera.rotate_object_local(Vector3.RIGHT, deg2rad(mouseMotion.y));

func CorrectRotation(direction:Vector2):
	var OffsetCalc1:Vector2 = Vector2(cos(-$camera.rotation.y), sin(-$camera.rotation.y)) * -direction.x;
	var OffsetCalc2:Vector2 = Vector2(cos(-$camera.rotation.y - deg2rad(90)), sin(-$camera.rotation.y - deg2rad(90))) * direction.y;
	var xOffsetCalc = (OffsetCalc1.x + OffsetCalc2.x);
	var zOffsetCalc = (OffsetCalc1.y + OffsetCalc2.y);
	return Vector2(xOffsetCalc,zOffsetCalc);
