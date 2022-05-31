extends ImmediateGeometry

const DRAW_COLOR := Color(1.0, 0.0, 1.0)

onready var camera: Camera = owner.get_node("cameraJoint/camera");

func _ready():
	# Do not inherit parent position.
	set_as_toplevel(true);
	global_transform.origin = Vector3(0, 0, 0)

	# Simulate the player entering the chunk (0, 0) so that
	# we draw immediately instead of waiting for the player
	# to cross a chunk boundary.
	_on_player_entered_chunk(Vector2(0, 0))

func y_level_square(top_left: Vector2, y_level: int):
	var x_axis_adj_xminus = [
		Vector3(top_left.x, y_level, top_left.y),
		Vector3(top_left.x + Constants.CHUNK_SIZE.x, y_level, top_left.y)
	]
	for point in x_axis_adj_xminus:
			set_color(Color(1.0, 0.0, 0.0))
			add_vertex(point)

	var z_axis_adj_zminus = [
		Vector3(top_left.x, y_level, top_left.y),
		Vector3(top_left.x, y_level, top_left.y + Constants.CHUNK_SIZE.z)
	]
	for point in z_axis_adj_zminus:
		set_color(Color(0.0, 0.0, 1.0))
		add_vertex(point)

func _on_player_entered_chunk(chunk_position: Vector2):
	var top_left = Vector2(
		chunk_position.x * Constants.CHUNK_SIZE.x,
		chunk_position.y * Constants.CHUNK_SIZE.z
	);
	# (x, z) positions of the current chunk's corners.
	var chunk_corners = [
		top_left,
		Vector2(top_left.x + 32, top_left.y),
		Vector2(top_left.x, top_left.y + 32),
		Vector2(top_left.x + 32, top_left.y + 32)
	];
	clear()
	begin(Mesh.PRIMITIVE_LINES);
	for corner in chunk_corners:
		# Draw a line from the bottom of the world to the top.
		var bottom = Vector3(corner.x, 0, corner.y);
		var top = Vector3(corner.x, Constants.CHUNK_SIZE.y, corner.y);
		set_color(DRAW_COLOR)
		add_vertex(bottom);
		set_color(DRAW_COLOR)
		add_vertex(top);
	# Draw a square every 32 blocks on the y-axis.
	for y_step in range(0, Constants.CHUNK_SIZE.y / 32):
		y_level_square(top_left, y_step * 32)
	end();
