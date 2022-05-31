extends Label

func _process(delta):
	text = "FPS: %d (delta %4.3f)" % [Engine.get_frames_per_second(), delta]
