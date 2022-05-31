extends Label

func _on_player_entered_chunk(chunk_position: Vector2):
	text = "Chunk: (%d, %d)" % [chunk_position.x, chunk_position.y]
