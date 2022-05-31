tool
extends Control

const MIN_SQUARE_WH: float = 64.0
const TRI_LENGTH: float = 6.0
const HALF_PI: float = PI / 2.0

func _notification(what):
	if what == NOTIFICATION_RESIZED:
		rect_size.y = rect_size.x

func _get_minimum_size():
	return Vector2(MIN_SQUARE_WH, MIN_SQUARE_WH)

func _physics_process(_delta):
	update()

func triangle_points(origin: Vector2, peak_distance: float, angle: float, pointyness: float) -> Array:
	var angle_fin_left = angle - HALF_PI
	var angle_fin_right = angle + HALF_PI
	var fins_distance = peak_distance / pointyness
	return [
		origin + peak_distance * Vector2(sin(angle), cos(angle)),
		origin + fins_distance * Vector2(sin(angle_fin_right), cos(angle_fin_right)),
		origin + fins_distance * Vector2(sin(angle_fin_left), cos(angle_fin_left))
	]

func get_local_position(global_position: Vector3) -> Vector3:
	var chunk = Vector2(
		global_position.x / Constants.CHUNK_SIZE.x,
		global_position.z / Constants.CHUNK_SIZE.z
	).floor()
	return Vector3(
		global_position.x - (chunk.x * Constants.CHUNK_SIZE.x),
		global_position.y,
		global_position.z - (chunk.y * Constants.CHUNK_SIZE.z)
	)

func _draw():
	var square_wh = rect_size.x
	# Base rect.
	draw_rect(
		Rect2(Vector2.ZERO, Vector2(square_wh, square_wh)),
		Color(0.0, 0.0, 0.0, 0.5),
		false,
		1.0,
		true
	)
	# Highlighted X-axis line.
	draw_line(
		Vector2.ZERO,
		Vector2(square_wh, 0),
		Color(1.0, 0.0, 0.0),
		1.5,
		false
	)
	# Highlighted Z-axis line.
	draw_line(
		Vector2.ZERO,
		Vector2(0, square_wh),
		Color(0.0, 0.0, 1.0),
		1.5,
		false
	)

	if Engine.editor_hint:
		return

	# Player triangle.
	var player_pos = owner.global_transform.origin
	var player_local_pos = get_local_position(player_pos)
	# Local position as a float between 0.0 and 1.0, like a UV:
	var player_local_offset := Vector2(
		player_local_pos.x / Constants.CHUNK_SIZE.x,
		player_local_pos.z / Constants.CHUNK_SIZE.z
	)
	var player_angle = owner.get_node("cameraJoint").rotation.y + PI
	draw_colored_polygon(
		PoolVector2Array(triangle_points(
			player_local_offset * square_wh,
			TRI_LENGTH,
			player_angle,
			3.0
		)),
		Color(0.0, 0.9, 0.0),
		PoolVector2Array(),
		null,
		null,
		true
	)
