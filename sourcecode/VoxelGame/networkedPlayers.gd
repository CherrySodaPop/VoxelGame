extends Node

var instances: Dictionary = {}

func new_player(id: int):
	var instance = preload("res://objects/clientPlayer/clientPlayer.tscn").instance()
	instances[id] = instance
	add_child(instance)

func remove_player(id: int):
	instances[id].queue_free()
	# warning-ignore:RETURN_VALUE_DISCARDED
	instances.erase(id)

func update_player_appearance(id: int, skin: ImageTexture):
	var mesh = instances[id].get_node("model/PM/Skeleton/PMMeshObj")
	mesh.get("material/0").albedo_texture = skin
	mesh.get("material/1").albedo_texture = skin

func update_player(id: int, position: Vector3, camera_rot: Vector3):
	pass

func has(id: int):
	return instances.has(id)
