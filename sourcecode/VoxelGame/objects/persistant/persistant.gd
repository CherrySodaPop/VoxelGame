extends Node

var chunkSize:Vector3 = Vector3(32, 512, 32);
onready var chunkLoader = $chunkLoader;

func get_block(position: Vector3):
	return chunkLoader.get_block_gd(position);
