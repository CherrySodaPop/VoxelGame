extends Label

onready var player: Spatial = owner

func _physics_process(_delta):
	var player_pos = player.global_transform.origin
	text = "Position: (%.2f, %.2f, %.2f)" % [player_pos.x, player_pos.y, player_pos.z]
