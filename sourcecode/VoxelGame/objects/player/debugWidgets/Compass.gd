tool
extends Control

const MIN_RADIUS: float = 32.0
const ORB_RADIUS: float = 3.0

const X_COLOR := Color(1.0, 0.0, 0.0)
const Z_COLOR := Color(0.0, 0.0, 1.0)

onready var camera_joint: Spatial = owner.get_node("cameraJoint")

func _notification(what):
	if what == NOTIFICATION_RESIZED:
		rect_size.y = rect_size.x

func _get_minimum_size():
	return Vector2(MIN_RADIUS, MIN_RADIUS)

func _physics_process(_delta):
	update()

func position_from_rotation(rotation: float) -> Vector2:
	return Vector2(sin(rotation), -cos(rotation))

func _draw():
	var radius = rect_size.x / 2
	var origin = Vector2(radius, radius)
	# Background circle.
	draw_arc(
		origin,
		radius,
		0,
		TAU,
		360,
		Color(0.0, 0.0, 0.0, 0.5),
		1.0,
		true
	)

	var rotation_offset: float = 0.0
	if not Engine.editor_hint:
		rotation_offset = camera_joint.rotation.y

	# Z "orb".
	var z_pos = origin + position_from_rotation(PI + rotation_offset) * radius
	draw_line(
		origin,
		z_pos,
		Z_COLOR,
		1.25,
		true
	)
	draw_circle(z_pos, ORB_RADIUS, Z_COLOR)

	# X "orb".
	var x_pos = origin + position_from_rotation((PI/2) + rotation_offset) * radius
	draw_line(
		origin,
		x_pos,
		X_COLOR,
		1.25,
		true
	)
	draw_circle(x_pos, ORB_RADIUS, X_COLOR)
