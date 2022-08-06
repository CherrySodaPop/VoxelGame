extends Node3D

const NORMALS = {
	0b0110: Vector3(-1, 0, 0),
	0b0111: Vector3(1, 0, 0),
	0b1010: Vector3(0, -1, 0),
	0b1011: Vector3(0, 1, 0),
	0b1100: Vector3(0, 0, -1),
	0b1101: Vector3(0, 0, 1),
}
var VERTICES := {}

const _BASE_VERTICES = [
	[0, 0, 0, 0, 0, 0],
	[0, 1, 0, 1, 1, 0],
	[0, 0, 1, 0, 1, 1]
]

func generate_face_vertices(face: int) -> Array[Array]:
	var base = _BASE_VERTICES.duplicate()
	if face & 0b0001:
		base[0] = [1, 1, 1, 1, 1, 1]
	face >>= 1
	var shift_by = 2 - ((0b111 - face) / 2)
	for __ in range(shift_by):
		var back = base.pop_back()
		base.push_front(back)
	# Convert from [[XXXXXX], [YYYYYY], [ZZZZZZ]] to [[X, Y, Z], [X, Y, Z] ...]
	var vertices = []
	for i in range(6):
		vertices.push_back([base[0][i], base[1][i], base[2][i]])
	return vertices

func _ready():
	Persistent.get_node("controllerUI").queue_free() # TEMP
	Persistent.get_node("entityPlayer").queue_free() # TEMP
	for face in [6, 10, 12, 7, 11, 13]:
		var vertices = generate_face_vertices(face)
		if face & 0b0001:
			# Correct triangle vertices to be "clockwise," in order for it to
			# be a front face
			vertices.reverse()
		VERTICES[face] = vertices
	await get_tree().create_timer(0.5).timeout
	for x in range(-5, 6):
		for z in range(-5, 6):
			# Prevent lag spikes
			await get_tree().create_timer(0.04).timeout
			var chunk_position = Vector2i(x, z)
			# TODO: Terrain data will be sent by the server, not generated
			#       on the client like this.
			var hm = $controllerTerrain.chunk_heightmap(chunk_position)
			var mi = MeshInstance3D.new()
			mi.mesh = await create_mesh(hm)
			mi.global_position = Vector3(
				chunk_position.x * C.CHUNK_XZ, 0, chunk_position.y * C.CHUNK_XZ
			)
			mi.name = "chunk_%d_%d" % [chunk_position.x, chunk_position.y]
			add_child(mi)

func add_faces(
	root: Vector3,
	vertices: PackedVector3Array,
	normals: PackedVector3Array
):
	# TODO: Allow choosing what faces to add
	for face in [6, 10, 12, 7, 11, 13]:
		var points = VERTICES[face]
		var normal = NORMALS[face]
		for point in points:
			normals.push_back(normal)
			vertices.push_back(root + Vector3(point[0], point[1], point[2]))

func create_mesh(heightmap: Array[int]) -> ArrayMesh:
	var mesh = ArrayMesh.new()
	var vertices = PackedVector3Array()
	var normals = PackedVector3Array()
	for x in range(0, C.CHUNK_XZ):
		for z in range(0, C.CHUNK_XZ):
			var y = heightmap[(x * C.CHUNK_XZ) + z]
			# TODO: Don't render faces that can't be seen
			add_faces(Vector3i(x, y, z), vertices, normals)
	var surface_array = []
	surface_array.resize(Mesh.ARRAY_MAX)
	surface_array[Mesh.ARRAY_VERTEX] = vertices
	surface_array[Mesh.ARRAY_NORMAL] = normals
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, surface_array)
	# TODO: Actual materials system
	var mat = StandardMaterial3D.new()
#	mat.albedo_color = Color(1.0, 0.5, 0.5)
	mat.roughness = 0.75
	mat.metallic = 0.6
	mesh.surface_set_material(0, mat)
	return mesh
	
