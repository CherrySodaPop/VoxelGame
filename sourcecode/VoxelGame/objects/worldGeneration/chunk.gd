extends Spatial

var data:Dictionary = {};
var surfaceToolInstance:SurfaceTool = SurfaceTool.new();

func _ready():
	# is chunk generated?
	#	LoadChunkData();
	# else
	
	var beforeGeneration = OS.get_system_time_msecs();
	Generate();
	var afterGeneration = OS.get_system_time_msecs();
	print("ChunkGenerationTime: %d" % [afterGeneration - beforeGeneration]);
	
	var beforeConstruct = OS.get_system_time_msecs();
	ConstructMesh();
	var afterConstruct = OS.get_system_time_msecs();
	print("MeshContructionTime: %d" % [afterConstruct - beforeConstruct]);
	
	#BegimMeshConstruction();
	#BuildFace(get_parent().meshFaces.TOP, Vector3(0,0,0));
	#BuildFace(get_parent().meshFaces.TOP, Vector3(1,0,0));
	#BuildFace(get_parent().meshFaces.TOP, Vector3(2,0,0));
	#BuildFace(get_parent().meshFaces.TOP, Vector3(3,0,0));
	#BuildFace(get_parent().meshFaces.BOTTOM, Vector3(1,0,0));
	#CommitMesh();
	#var packer = PackedScene.new();
	#packer.pack(self);
	#ResourceSaver.save("res://assets/debug/chunk.tscn", packer);

func Generate():
	var noise:OpenSimplexNoise = get_parent().simplexNoise;
	var chunkSize = get_parent().chunkSize;
	
	#BegimMeshConstruction();
	
	# terrain generation
	for _x in range(chunkSize.x):
		for _z in range(chunkSize.z):
			var noiseHeight:float = noise.get_noise_2d(_x + transform.origin.x, _z + transform.origin.z);
			var terrainAmp:float = 0.1;
			var terrainPeak:int = int(chunkSize.y * ((noiseHeight / 2) + 0.5) * terrainAmp);
			var blockData = get_parent().blockData;
			
			for _y in range(chunkSize.y, -1, -1):
				if (_y > terrainPeak):
					data[Vector3(_x,_y,_z)] = [blockData.id.AIR, {}]; # format: block id, directory storing meta data (example: a chest with items)
					continue;
				if (_y == terrainPeak):
					data[Vector3(_x,_y,_z)] = [blockData.id.STONE, {}];
					continue;
				data[Vector3(_x,_y,_z)] = [blockData.id.STONE, {}];
			#BuildFace(get_parent().meshFaces.TOP, Vector3(_x, terrainPeak, _z));
	#CommitMesh();

func ConstructMesh():
	var chunkSize = get_parent().chunkSize;
	var blockData = get_parent().blockData;
	BegimMeshConstruction();
	# TODO: gee i sure love for loops!
	for _x in range(chunkSize.x):
		for _z in range(chunkSize.z):
			for _y in range(chunkSize.y):
				var trueBlockPos:Vector3 = transform.origin + Vector3(_x, _y, _z);
				# check if im not an air block
				if (get_parent().GetBlock(trueBlockPos) == null || get_parent().GetBlock(trueBlockPos)[get_parent().BLOCKDATA_ID] == blockData.id.AIR):
					continue;
				# top check
				if (_y == chunkSize.y || IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, 1, 0)) )):
					BuildFace(get_parent().meshFaceType.TOP, Vector3(_x, _y, _z));
				# bottom check
				if (_y == 0 || IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, -1, 0)) )):
					BuildFace(get_parent().meshFaceType.BOTTOM, Vector3(_x, _y, _z));
				# left check
				if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(-1, 0, 0)) )):
					BuildFace(get_parent().meshFaceType.LEFT, Vector3(_x, _y, _z));
				# right check
				if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(1, 0, 0)) )):
					BuildFace(get_parent().meshFaceType.RIGHT, Vector3(_x, _y, _z));
				# front check
				if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, 0, 1)) )):
					BuildFace(get_parent().meshFaceType.FRONT, Vector3(_x, _y, _z));
				# back check
				if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, 0, -1)) )):
					BuildFace(get_parent().meshFaceType.BACK, Vector3(_x, _y, _z));
	CommitMesh();

func BegimMeshConstruction():
	surfaceToolInstance.begin(Mesh.PRIMITIVE_TRIANGLES);

func BuildFace(faceType:int, pos:Vector3 = Vector3.ZERO):
	surfaceToolInstance.add_uv(Vector2(0, 0));
	surfaceToolInstance.add_normal(get_parent().meshFaceNormal[faceType]);
	for i in range(6):
		surfaceToolInstance.add_vertex(get_parent().meshFacePos[faceType][i] + Vector3(pos));

func CommitMesh():
	$mesh.mesh = surfaceToolInstance.commit();

func IsFaceVisibleBlock(pickedBlockData) -> bool:
	var blockData = get_parent().blockData;
	return (pickedBlockData == null || pickedBlockData[get_parent().BLOCKDATA_ID] == blockData.id.AIR);
	# || !pickedBlockData[get_parent().BLOCKDATA_META].has("fullBlock") || pickedBlockData[get_parent().BLOCKDATA_META]["fullBlock"] == false);

func _process(delta):
	return;
	BegimMeshConstruction();
	BuildFace(get_parent().meshFaceType.TOP);
	$mesh1.mesh = surfaceToolInstance.commit();
	
	BegimMeshConstruction();
	BuildFace(get_parent().meshFaceType.BOTTOM);
	$mesh2.mesh = surfaceToolInstance.commit();
	
	BegimMeshConstruction();
	BuildFace(get_parent().meshFaceType.LEFT);
	$mesh3.mesh = surfaceToolInstance.commit();
	
	BegimMeshConstruction();
	BuildFace(get_parent().meshFaceType.RIGHT);
	$mesh4.mesh = surfaceToolInstance.commit();
	
	BegimMeshConstruction();
	BuildFace(get_parent().meshFaceType.FRONT);
	$mesh5.mesh = surfaceToolInstance.commit();
	
	BegimMeshConstruction();
	BuildFace(get_parent().meshFaceType.BACK);
	$mesh6.mesh = surfaceToolInstance.commit();
