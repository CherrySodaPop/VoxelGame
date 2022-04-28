extends KinematicBody

var networkID = -1;

# with time, this stuff will moved to the server, but for its early staging, it's fine
var velocity:Vector3 = Vector3.ZERO;
var acceleration:float = 16.0
var walkSpeed:float = 4.317
var runSpeed:float = 5.612

var prevChunk:Vector2 = Vector2.ZERO;
var currentChunk:Vector2 = Vector2.ZERO;
var lookingAtBlock:Vector3 = Vector3.ZERO;
var adjacentLookingAtBlock:Vector3 = Vector3.ZERO;

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
	HandleActions(delta);
	HandleMovement(delta);
	HandleAnimation(delta);
	HandleHud(delta);
	Network();

func _input(event:InputEvent):
	if (event is InputEventMouseMotion):
		HandleCamera(event.relative);

func UpdateMiscInfo(delta):
	# chunk pos update
	currentChunk.x = floor(global_transform.origin.x / Persistant.chunkSize.x);
	currentChunk.y = floor(global_transform.origin.z / Persistant.chunkSize.x);
	if (currentChunk != prevChunk):
		emit_signal("enteredNewChunk");
		prevChunk = currentChunk;
	
	# looking at block
	lookingAtBlock = $camera/RayCast.get_collision_point();
	lookingAtBlock += (lookingAtBlock - $camera.global_transform.origin).normalized() * 0.001;
	lookingAtBlock.x = floor(lookingAtBlock.x);
	lookingAtBlock.y = ceil(lookingAtBlock.y);
	lookingAtBlock.z = floor(lookingAtBlock.z);
	
	# adjacent block
	adjacentLookingAtBlock = $camera/RayCast.get_collision_point();
	adjacentLookingAtBlock -= (adjacentLookingAtBlock - $camera.global_transform.origin).normalized() * 0.01;
	adjacentLookingAtBlock.x = floor(adjacentLookingAtBlock.x);
	adjacentLookingAtBlock.y = ceil(adjacentLookingAtBlock.y);
	adjacentLookingAtBlock.z = floor(adjacentLookingAtBlock.z);

func HandleActions(delta):
	if (Input.is_action_pressed("playerPrimaryAction")):
		pass #Persistant.get_node("chunkGeneration").set_block_gd(lookingAtBlock, 0);
	if (Input.is_action_pressed("playerSecondaryAction")):
		pass #Persistant.get_node("chunkGeneration").set_block_gd(adjacentLookingAtBlock, 23);

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

func Network():
	var network = Persistant.get_node("controllerNetwork");
	if (network.HasTicked()):
		network.rpc_id(1, "PlayerInfo", global_transform.origin, Vector2($camera.rotation.x,$camera.rotation.y));

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
