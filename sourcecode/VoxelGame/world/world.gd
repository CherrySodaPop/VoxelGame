extends Spatial

func _ready():
	Persistant.get_node("clientPlayer").connect("enteredNewChunk", self, "HandleMeshGeneration");

func _process(delta):
	pass #HandleMeshGeneration();

func HandleMeshGeneration():
	var clientPlayer:Node =  Persistant.get_node("clientPlayer");
	Persistant.get_node("worldMeshHandler").GenerateChunkMesh(clientPlayer.currentChunk);
