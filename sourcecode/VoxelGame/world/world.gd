extends Spatial

func _ready():
	get_tree().paused = false
	# player new chunk entered signal

func _process(delta):
	pass #Persistent.get_node("chunkGeneration").set_block_godot(Vector3(0,90,0), 3);

