extends Spatial

var data:Dictionary = {};
var surfaceToolInstance:SurfaceTool = SurfaceTool.new();

func _ready():
	# is chunk generated?
	#	LoadChunkData();
	# else
	Generate();
	#ConstructMesh();

func Generate():
	var noise:OpenSimplexNoise = get_parent().simplexNoise;
	var chunkSize = get_parent().chunkSize;
	
	BegimMeshConstruction();
	
	# terrain generation
	for _x in range(chunkSize.x):
		for _z in range(chunkSize.z):
			print(noise.get_noise_2d(_x + transform.origin.x, _z + transform.origin.z));
			var noiseHeight:int = noise.get_noise_2d(_x + transform.origin.x, _z + transform.origin.z);
			print(str(noiseHeight) + " " + str(Vector2(_x,_z)) + "\n")
			var terrainAmp:int = 1.0;
			var terrainPeak:int = int(chunkSize.y * ((noiseHeight / 2) + 0.5));
			var blockData = get_parent().blockData;
			
			for _y in range(chunkSize.y, -1, -1):
				if (_y > terrainPeak):
					data[Vector3(_x,_y,_z)] = [blockData.id.AIR, {}]; # format: block id, directory storing meta data (example: a chest with items)
					continue;
				if (_y == terrainPeak):
					data[Vector3(_x,_y,_z)] = [blockData.id.STONE, {}];
					continue;
				data[Vector3(_x,_y,_z)] = [blockData.id.AIR, {}];
			BuildFace(get_parent().meshFaces.TOP, Vector3(_x, terrainPeak, _z));
	CommitMesh();

func ConstructMesh():
	var chunkSize = get_parent().chunkSize;
	var blockData = get_parent().blockData;
	BegimMeshConstruction();
	# TODO: gee i sure love for loops!
	for _x in range(chunkSize.x):
		for _z in range(chunkSize.z):
			for _y in range(chunkSize.y):
				var trueBlockPos:Vector3 = transform.origin + Vector3(_x, _y, _z);
				#var pickedBlockData = get_parent().GetBlock(trueBlockPos + Vector3(0, -1, 0);
				# top check
				if (_y == chunkSize.y || IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, 1, 0)) )):
					BuildFace(get_parent().meshFaces.TOP, Vector3(_x, _y, _z));
				# bottom check
				#if (_y == 0 || IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, -1, 0)) )):
				#	BuildFace(get_parent().meshFaces.BOTTOM, Vector3(_x, _y, _z));
				# left check
				#if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(-1, 0, 0)) )):
				#	BuildFace(get_parent().meshFaces.LEFT, Vector3(_x, _y, _z));
				# right check
				#if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(1, 0, 0)) )):
				#	BuildFace(get_parent().meshFaces.RIGHT, Vector3(_x, _y, _z));
				# front check
				#if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, 0, 1)) )):
				#	BuildFace(get_parent().meshFaces.FRONT, Vector3(_x, _y, _z));
				# back check
				#if (IsFaceVisibleBlock( get_parent().GetBlock(trueBlockPos + Vector3(0, 0, -1)) )):
				#	BuildFace(get_parent().meshFaces.BACK, Vector3(_x, _y, _z));
	CommitMesh();

func BegimMeshConstruction():
	surfaceToolInstance.begin(Mesh.PRIMITIVE_TRIANGLES);

func BuildFace(faceType:Array, pos:Vector3 = Vector3.ZERO):
	surfaceToolInstance.add_color(Color(1, 1, 1));
	surfaceToolInstance.add_uv(Vector2(0, 0));
	for i in range(6):
		surfaceToolInstance.add_vertex(faceType[i] + Vector3(pos));

func CommitMesh():
	$mesh.mesh = surfaceToolInstance.commit();

func IsFaceVisibleBlock(pickedBlockData) -> bool:
	var blockData = get_parent().blockData;
	return !(pickedBlockData == null || pickedBlockData[get_parent().BLOCKDATA_ID] == blockData.id.AIR);
	# || !pickedBlockData[get_parent().BLOCKDATA_META].has("fullBlock") || pickedBlockData[get_parent().BLOCKDATA_META]["fullBlock"] == false);

func _process(delta):
	pass
#	var returnMesh:ArrayMesh = BuildFace(get_parent().meshFaces.TOP);
#	$mesh1.mesh = returnMesh;
#	returnMesh = BuildFace(get_parent().meshFaces.BOTTOM);
#	$mesh2.mesh = returnMesh;
#	returnMesh = BuildFace(get_parent().meshFaces.LEFT);
#	$mesh3.mesh = returnMesh;
#	returnMesh = BuildFace(get_parent().meshFaces.RIGHT);
#	$mesh4.mesh = returnMesh;
#	returnMesh = BuildFace(get_parent().meshFaces.FRONT);
#	$mesh5.mesh = returnMesh;
#	returnMesh = BuildFace(get_parent().meshFaces.BACK);
#	$mesh6.mesh = returnMesh;
