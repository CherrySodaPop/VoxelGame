extends Spatial

func _ready():
	# player new chunk entered signal
	Persistant.get_node("player").connect("enteredNewChunk", self, "HandleMeshGeneration");

func _process(delta):
	pass #Persistant.get_node("chunkGeneration").set_block_godot(Vector3(0,90,0), 3);

func HandleMeshGeneration():
	var player:Node =  Persistant.get_node("player");
	Persistant.get_node("worldMeshHandler").GenerateChunkMesh(player.currentChunk);
