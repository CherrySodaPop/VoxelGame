extends Node

const SEA_LEVEL: int = 60
const NOISE_SCALE: float = 0.2
const NOISE_AMP: float = 100.0
@export var world_seed: int = 12345678
@onready var noise := FastNoiseLite.new()

func _ready():
	noise.seed = world_seed
	
func get_height(position: Vector2i) -> int:
	return SEA_LEVEL + int(
		noise.get_noise_2d(position.x * NOISE_SCALE, position.y * NOISE_SCALE)
		* NOISE_AMP
	)
	
func chunk_heightmap(chunk_position: Vector2i) -> Array[int]:
	var heightmap = []
	var root = chunk_position * C.CHUNK_XZ
	for x_offset in range(0, C.CHUNK_XZ):
		for z_offset in range(0, C.CHUNK_XZ):
			var position = Vector2i(
				root.x + x_offset, root.y + z_offset
			)
			var height = get_height(position)
			heightmap.push_back(height)
	return heightmap
