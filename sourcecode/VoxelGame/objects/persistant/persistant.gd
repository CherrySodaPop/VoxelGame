extends Node

var chunkSize:Vector3 = Vector3(32, 256, 32);

func get_block(position: Vector3):
	return $chunkGeneration.get_block_gd(position);
