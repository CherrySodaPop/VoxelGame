extends Node

var lifeTimer:float = 0.0;
var tickTimer:float = 0.0;

var simplexNoise:OpenSimplexNoise = OpenSimplexNoise.new();
var textureNoise:NoiseTexture = NoiseTexture.new();

var chunkData:Dictionary = {};
var chunkSize:Vector3 = Vector3(32, 256, 32);
var chunkScene = preload("res://objects/worldGeneration/chunk.tscn");
var blockData = preload("res://objects/worldGeneration/blockData.gd");
enum meshFaceType {
	TOP,
	BOTTOM,
	LEFT,
	RIGHT,
	FRONT,
	BACK, }
# These arrays will be indexed with the meshFaceType enum,
# so the first element in meshFacePositions represents meshFaceType.TOP,
# the second represents meshFaceTop.BOTTOM, etc.
const meshFacePositions = [
	PoolVector3Array([Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 0, 1), Vector3(1, 0, 0), Vector3(1, 0, 1), Vector3(0, 0, 1)]),
	PoolVector3Array([Vector3(0, -1, 1), Vector3(1, -1, 1), Vector3(0, -1, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(0, -1, 0)]),
	PoolVector3Array([Vector3(0, 0, 0), Vector3(0, 0, 1), Vector3(0, -1, 0), Vector3(0, 0, 1), Vector3(0, -1, 1), Vector3(0, -1, 0)]),
	PoolVector3Array([Vector3(1, 0, 1), Vector3(1, 0, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(1, -1, 1), Vector3(1, 0, 0)]),
	PoolVector3Array([Vector3(0, -1, 1), Vector3(0, 0, 1), Vector3(1, 0, 1), Vector3(1, 0, 1), Vector3(1, -1, 1), Vector3(0, -1, 1)]),
	PoolVector3Array([Vector3(1, -1, 0), Vector3(1, 0, 0), Vector3(0, 0, 0), Vector3(0, 0, 0), Vector3(0, -1, 0), Vector3(1, -1, 0)]) ];
const meshFaceNormals = PoolVector3Array([
	Vector3(0, 1, 0),
	Vector3(0, -1, 0),
	Vector3(-1, 0, 0),
	Vector3(1, 0, 0),
	Vector3(0, 0, 1),
	Vector3(0, 0, -1) ]);
const BLOCKDATA_ID = 0;
const BLOCKDATA_META = 1;
const CHUNK_X_SIZE = 32;
const CHUNK_Y_SIZE = 256;
const CHUNK_Z_SIZE = 32;

func _ready():
	simplexNoise.seed = 1;
	textureNoise.noise = simplexNoise;
	textureNoise.set_flags(0);
	$TextureRect.texture = textureNoise;

	for _x in range(2):
		for _y in range(2):
			GenerateChunk(_x, _y);

	# Generate meshes once all chunk data is available
	for child in get_children():
		if child is Spatial:
			print("Constructing mesh for ", child.name);
			child.ConstructMesh();
	# ^ This will have to get smarter once threaded chunk generation is implemented

func _process(delta):
	lifeTimer += delta;
	tickTimer += delta;
	#simplexNoise.octaves = 3;
	#simplexNoise.period = 64;
	#textureNoise.noise_offset.x = lifeTimer * 50;

func GenerateChunk(_x:int, _z:int):
	var objChunk = chunkScene.instance();
	objChunk.transform.origin.x = _x * chunkSize.x;
	objChunk.transform.origin.z = _z * chunkSize.z;
	chunkData[Vector2(_x, _z)] = objChunk;
	print("Generating ", objChunk.name);
	add_child(objChunk);

func GetWorldBlock(blockPos:Vector3 = Vector3.ZERO):
	blockPos.x = int(floor(blockPos.x));
	blockPos.y = int(floor(blockPos.y));
	blockPos.z = int(floor(blockPos.z));

	var _chunkX = floor(blockPos.x / chunkSize.x)
	var _chunkZ = floor(blockPos.z / chunkSize.z)
	var _chunkBlockPosX = (blockPos.x - (_chunkX * chunkSize.x));
	var _chunkBlockPosZ = (blockPos.z - (_chunkZ * chunkSize.z));

	return GetChunkBlock(
		_chunkX,
		_chunkZ,
		_chunkBlockPosX,
		blockPos.y,
		_chunkBlockPosZ
	)

func GetChunkBlock(
	chunkX: int,
	chunkZ: int,
	chunkBlockX: int,
	chunkBlockY: int,
	chunkBlockZ: int
):
	var chunkKey = Vector2(chunkX, chunkZ);
	var _chunk = null;
	if (chunkData.has(chunkKey)):
		_chunk = chunkData[chunkKey];

	if (is_instance_valid(_chunk)):
		return _chunk.GetLocalBlockId(chunkBlockX, chunkBlockY, chunkBlockZ);

	# if the above fails, check if it's saved on the drive instead, once that's actually implemented
	# ...

	# all has failed, panic!
	return -1;
