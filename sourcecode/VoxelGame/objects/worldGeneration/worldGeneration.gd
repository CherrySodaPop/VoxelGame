extends Node

var lifeTimer:float = 0.0;
var tickTimer:float = 0.0;

var simplexNoise:OpenSimplexNoise = OpenSimplexNoise.new();
var textureNoise:NoiseTexture = NoiseTexture.new();

var chunkData:Dictionary = {};
var chunkSize:Vector3 = Vector3(32, 256, 32);
var chunkScene = preload("res://objects/worldGeneration/chunk.tscn"); 
var blockData = preload("res://objects/worldGeneration/blockData.gd");
const meshFaces = {
	TOP = [Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 0, 1), Vector3(1, 0, 0), Vector3(1, 0, 1), Vector3(0, 0, 1)],
	BOTTOM = [Vector3(0, -1, 1), Vector3(1, -1, 1), Vector3(0, -1, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(0, -1, 0)],
	LEFT = [Vector3(0, 0, 0), Vector3(0, 0, 1), Vector3(0, -1, 0), Vector3(0, 0, 1), Vector3(0, -1, 1), Vector3(0, -1, 0)],
	RIGHT = [Vector3(1, 0, 1), Vector3(1, 0, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(1, -1, 1), Vector3(1, 0, 0)],
	FRONT = [Vector3(0, -1, 1), Vector3(0, 0, 1), Vector3(1, 0, 1), Vector3(1, 0, 1), Vector3(1, -1, 1), Vector3(0, -1, 1)],
	BACK = [Vector3(1, -1, 0), Vector3(1, 0, 0), Vector3(0, 0, 0), Vector3(0, 0, 0), Vector3(0, -1, 0), Vector3(1, -1, 0)], }
const BLOCKDATA_ID = 0; 
const BLOCKDATA_META = 1;

func _ready():
	simplexNoise.seed = 1;
	textureNoise.noise = simplexNoise;
	textureNoise.set_flags(0);
	$TextureRect.texture = textureNoise;
	
	GenerateChunk(0,0);
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

func GetBlock(blockPos:Vector3 = Vector3.ZERO):
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
	if (is_instance_valid(_chunk) && _chunk.data.has(Vector3(_chunkBlockPosX, blockPos.y, _chunkBlockPosZ))):
		var blockData = _chunk.data[Vector3(_chunkBlockPosX, blockPos.y, _chunkBlockPosZ)]; # format: block id, directory storing meta data (example: a chest with items)
		return blockData;
	
	# if the above fails, check if it's saved on the drive instead, once that's actually implemented
	# ...
	
	# all has failed, panic!
	return null;
