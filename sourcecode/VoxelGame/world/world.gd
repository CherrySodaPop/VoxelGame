extends Spatial

func _ready():
	get_tree().paused = false
	# player new chunk entered signal
	Persistent.get_node("player").connect("enteredNewChunk", self, "HandleMeshGeneration");

func _process(delta):
	pass #Persistent.get_node("chunkGeneration").set_block_godot(Vector3(0,90,0), 3);

func HandleMeshGeneration():
	var player:Node =  Persistent.get_node("player");
	Persistent.get_node("worldMeshHandler").GenerateChunkMesh(player.currentChunk);
