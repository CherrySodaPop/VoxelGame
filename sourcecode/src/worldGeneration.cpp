#include "worldGeneration.h"
#include "chunk.h"
#include "blockData.h"
#include <Vector2.hpp>
#include <PoolArrays.hpp>
#include <ResourceLoader.hpp>

using namespace godot;

void worldGeneration::_register_methods()
{
    register_method((char*)"_ready", &worldGeneration::_ready);
    register_method((char*)"_process", &worldGeneration::_process);
    register_method((char*)"GetWorldBlockId", &worldGeneration::GetWorldBlockId);
}

worldGeneration::worldGeneration()
{
	meshFacePositions[blockFaceType::TOP] = {Vector3(0, 0, 0), Vector3(1, 0, 0), Vector3(0, 0, 1), Vector3(1, 0, 0), Vector3(1, 0, 1), Vector3(0, 0, 1)};
	meshFacePositions[blockFaceType::BOTTOM] = {Vector3(0, -1, 1), Vector3(1, -1, 1), Vector3(0, -1, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(0, -1, 0)};
	meshFacePositions[blockFaceType::LEFT] = {Vector3(0, 0, 0), Vector3(0, 0, 1), Vector3(0, -1, 0), Vector3(0, 0, 1), Vector3(0, -1, 1), Vector3(0, -1, 0)};
	meshFacePositions[blockFaceType::RIGHT] = {Vector3(1, 0, 1), Vector3(1, 0, 0), Vector3(1, -1, 1), Vector3(1, -1, 0), Vector3(1, -1, 1), Vector3(1, 0, 0)};
	meshFacePositions[blockFaceType::FRONT] = {Vector3(0, -1, 1), Vector3(0, 0, 1), Vector3(1, 0, 1), Vector3(1, 0, 1), Vector3(1, -1, 1), Vector3(0, -1, 1)};
	meshFacePositions[blockFaceType::BACK] = {Vector3(1, -1, 0), Vector3(1, 0, 0), Vector3(0, 0, 0), Vector3(0, 0, 0), Vector3(0, -1, 0), Vector3(1, -1, 0)};

    meshFaceNormals[blockFaceType::TOP] = Vector3(0, 1, 0),
	meshFaceNormals[blockFaceType::BOTTOM] = Vector3(0, -1, 0);
	meshFaceNormals[blockFaceType::LEFT] = Vector3(-1, 0, 0);
	meshFaceNormals[blockFaceType::RIGHT] = Vector3(1, 0, 0);
	meshFaceNormals[blockFaceType::FRONT] = Vector3(0, 0, 1);
	meshFaceNormals[blockFaceType::BACK] = Vector3(0, 0, -1);

    chunkScene = ResourceLoader::get_singleton()->load("res://objects/worldGeneration/chunk.tscn");
    pSimplexNoise = OpenSimplexNoise::_new();
    pNoiseTexture = NoiseTexture::_new();
}

worldGeneration::~worldGeneration()
{
}

void worldGeneration::_init()
{
}

void worldGeneration::_ready()
{
    GenerateChunk(0, 0);
    //for (int _x = 0; _x < 2; _x++)
    //{
    //     for (int _z = 0; _z < 2; _z++)
    //     {
    //         GenerateChunk(_x, _z);
    //     }
    // }
}

void worldGeneration::_process()
{
}

void worldGeneration::GenerateChunk(int _x, int _z)
{
    //char info[256];
    //snprintf(info, sizeof(info), "Generating: %d, %d", _x, _z);
    //Godot::print(info);

    chunk *objChunk = (chunk *)chunkScene->instance();
    int _xChunkPos = _x * CHUNK_X_SIZE;
    int _zChunkPos = _z * CHUNK_X_SIZE;
    objChunk->set_translation(Vector3(_xChunkPos, 0.0, _zChunkPos));
    this->add_child(objChunk);
}

int worldGeneration::GetWorldBlockId(int _x, int _y, int _z)
{
    int _chunkX = int(floor( _x / CHUNK_X_SIZE ));
    int _chunkZ = int(floor( _z / CHUNK_Z_SIZE ));
    int _chunkBlockPosX = int(_x - (_chunkX * CHUNK_X_SIZE));
    int _chunkBlockPosZ = int(_z - (_chunkZ * CHUNK_Z_SIZE));

    return GetChunkBlockId(_chunkX, _chunkZ, _chunkBlockPosX, _y, _chunkBlockPosZ);
}

int worldGeneration::GetChunkBlockId(int chunkX, int chunkZ, int chunkBlockX, int chunkBlockY, int chunkBlockZ)
{
    Vector2 chunkKey = Vector2(chunkX, chunkZ);
    chunk *chunkNode = nullptr;

    // read the loaded memory check data
    if (chunckData.has(chunkKey))
    {
        chunkNode = chunckData[chunkKey];
        // core_1_1_api ... neat i guess
        if (godot::core_1_1_api->godot_is_instance_valid(chunkNode)) // TODO: add a check here for threads to make sure the chunk is generated!
        {
            return chunkNode->GetLocalBlockId(chunkBlockX, chunkBlockY, chunkBlockZ);
        }
    }

    // if we cant find and loaded chunk info,
    // check if any is saved on the disk
    // ...

    // all has failed, panic!
    return BLOCKDATA_MISSING;
}