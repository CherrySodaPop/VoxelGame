#include "worldGeneration.h"
#include "blockData.h"
#include <Vector2.hpp>
#include <PoolArrays.hpp>
#include <ResourceLoader.hpp>

using namespace godot;

void worldGeneration::_register_methods()
{
    register_method((char*)"_ready", &worldGeneration::_ready);
    register_method((char*)"_process", &worldGeneration::_process);
    register_method((char*)"_ready", &worldGeneration::_ready);
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
}

worldGeneration::~worldGeneration()
{
}

void worldGeneration::_ready()
{
    for (int _x = 0; _x < 2; _x++)
    {
        for (int _z = 0; _z < 2; _z++)
        {
            GenerateChunk(_x, _z);
        }
    }
}

void worldGeneration::GenerateChunk(int _x, int _z)
{
    Node *funy = chunkScene.instance();
}