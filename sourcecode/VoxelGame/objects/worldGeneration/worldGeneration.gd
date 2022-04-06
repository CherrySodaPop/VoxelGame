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
const meshFacePos = {
	meshFaceType.TOP : [Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 0, 1), Vector3(1, 0, 0), Vector3(1, 0, 1), Vector3(0, 0, 1)],
	meshFaceType.BOTTOM : [Vector3(0, -1, 1), Vector3(1, -1, 1), Vector3(0, -1, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(0, -1, 0)],
	meshFaceType.LEFT : [Vector3(0, 0, 0), Vector3(0, 0, 1), Vector3(0, -1, 0), Vector3(0, 0, 1), Vector3(0, -1, 1), Vector3(0, -1, 0)],
	meshFaceType.RIGHT : [Vector3(1, 0, 1), Vector3(1, 0, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(1, -1, 1), Vector3(1, 0, 0)],
	meshFaceType.FRONT : [Vector3(0, -1, 1), Vector3(0, 0, 1), Vector3(1, 0, 1), Vector3(1, 0, 1), Vector3(1, -1, 1), Vector3(0, -1, 1)],
	meshFaceType.BACK : [Vector3(1, -1, 0), Vector3(1, 0, 0), Vector3(0, 0, 0), Vector3(0, 0, 0), Vector3(0, -1, 0), Vector3(1, -1, 0)], }
const meshFaceNormal = {
	meshFaceType.TOP : Vector3(0, 1, 0),
	meshFaceType.BOTTOM : Vector3(0, -1, 0),
	meshFaceType.LEFT : Vector3(-1, 0, 0),
	meshFaceType.RIGHT : Vector3(1, 0, 0),
	meshFaceType.FRONT : Vector3(0, 0, 1),
	meshFaceType.BACK : Vector3(0, 0, -1), }
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
	
	for _x in range(1):
		for _y in range(1):
			GenerateChunk(_x, _y);
	#for _x in range(1):
	#	for _z in range(1):
	#		var objChunk = chunkScene.instance();
	#		objChunk.transform.origin.x = _x * chunkSize.x;
	#		objChunk.transform.origin.z = _z * chunkSize.z;
	#		add_child(objChunk);

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
	add_child(objChunk);

func GetBlockId(blockPos:Vector3 = Vector3.ZERO):
	blockPos.x = int(floor(blockPos.x));
	blockPos.y = int(floor(blockPos.y));
	blockPos.z = int(floor(blockPos.z));
	
	var _chunkX = floor(blockPos.x / chunkSize.x)
	var _chunkZ = floor(blockPos.z / chunkSize.z)
	var _chunk = null;
	if (chunkData.has(Vector2(_chunkX, _chunkZ))):
		_chunk = chunkData[Vector2(_chunkX, _chunkZ)];
	var _chunkBlockPosX = (blockPos.x - (_chunkX * chunkSize.x));
	var _chunkBlockPosZ = (blockPos.z - (_chunkZ * chunkSize.z));
	
	# check if the chunk is already loaded in-game before reading the disk
	# todo: when threads are added, add a check here making sure the chunk has actually finished generating
	if (is_instance_valid(_chunk)):
		return _chunk.GetBlockId();
		#return _chunkBlockIdData[ _chunkBlockPosX + (_chunkBlockPosZ * CHUNK_X_SIZE) + (blockPos.y * CHUNK_X_SIZE * CHUNK_Z_SIZE) ];
	
	# if the above fails, check if it's saved on the drive instead, once that's actually implemented
	# ...
	
	# all has failed, panic!
	return -1;
