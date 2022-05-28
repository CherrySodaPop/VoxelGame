extends Node

const OTHER_PLAYER: PackedScene = preload("res://objects/otherPlayer/otherPlayer.tscn")

var instances: Dictionary = {}

func add_player(id: int):
	var instance = OTHER_PLAYER.instance()
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

func update_player(id: int, position: Vector3, camera_rot: Vector2):
	instances[id].global_transform.origin = position
	instances[id].camRotation = camera_rot

func has(id: int):
	return instances.has(id)
