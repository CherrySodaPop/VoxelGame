extends Spatial

func _ready():
	# player new chunk entered signal
	Persistant.get_node("clientPlayer").connect("enteredNewChunk", self, "HandleMeshGeneration");

func _process(delta):
	Persistant.get_node("chunkGeneration").set_block_godot(Vector3(0,90,0), 3);

func HandleMeshGeneration():
	var clientPlayer:Node =  Persistant.get_node("clientPlayer");
	Persistant.get_node("worldMeshHandler").GenerateChunkMesh(clientPlayer.currentChunk);
