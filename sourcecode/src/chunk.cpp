#include "chunk.h"
#include <Mesh.hpp>
#include <MeshInstance.hpp>
#include <ArrayMesh.hpp>
#include <string>

using namespace godot;

void chunk::_register_methods()
{
    register_method((char*)"_ready", &chunk::_ready);
    register_method((char*)"_process", &chunk::_process);
    //register_method((char*)"GetData_BlockId", &chunk::GetData_BlockId);
    register_method((char*)"GetBlockId", &chunk::GetBlockId);
}

chunk::chunk()
{
    surfaceToolInstance = SurfaceTool::_new();
}

chunk::~chunk()
{
}

void chunk::_init()
{
}

void chunk::_ready()
{
    Generate();
    ConstructMesh();
}

void chunk::_process(float delta)
{
}

int chunk::GetBlockId(int _x, int _y, int _z)
{
    return dataBlockId[_x][_y][_z];
}

/*
Array chunk::GetData_BlockId()
{
    Array godotArray;
    godotArray.resize(CHUNK_X_SIZE * CHUNK_Y_SIZE * CHUNK_Z_SIZE);
    for (int i = 0; i < (CHUNK_X_SIZE * CHUNK_Y_SIZE * CHUNK_Z_SIZE); i++)
    {
        godotArray[i] = Variant(dataBlockId[i]);
    }
    return godotArray;
}
*/

void chunk::Generate()
{
    OpenSimplexNoise *noise = get_parent()->get("simplexNoise");

    for (int _x = 0; _x < CHUNK_X_SIZE; _x++)
    {
        for (int _z = 0; _z < CHUNK_Z_SIZE; _z++)
        {
            Vector2 trueBlockPos = Vector2(_x + this->get_transform().origin.x, _z + this->get_transform().origin.z);
            float noiseHeight = noise->get_noise_2dv(trueBlockPos);
            float terrainAmp = 0.1;
            int terrainPeak = int(CHUNK_Y_SIZE * ((noiseHeight / 2.0) + 0.5) * terrainAmp);

            for (int _y = CHUNK_Y_SIZE - 1; _y > -1; _y--)
            {
                if (_y > terrainPeak)
                {
                    dataBlockId[_x][_y][_z] = blockId::AIR;
                    continue;
                }
                dataBlockId[_x][_y][_z] = blockId::STONE;
            }
        }
    }
}

void chunk::ConstructMesh()
{
    Vector3 chunkSize = get_parent()->get("chunkSize");

    BeginMeshConstruction();

    for (int _x = 0; _x < CHUNK_X_SIZE; _x++)
    {
        for (int _z = 0; _z < CHUNK_Z_SIZE; _z++)
        {
            for (int _y = 0; _y < CHUNK_Y_SIZE; _y++)
            {
                Vector3 trueBlockPos = get_transform().origin + Vector3(_x, _y, _z);
                int blockDataId = get_parent()->call("GetBlockId", trueBlockPos);
                
                // check if we're not air
                if (blockDataId == BLOCKDATA_MISSING || blockDataId == blockId::AIR) continue;

                // top check
                blockDataId = get_parent()->call("GetBlockId", trueBlockPos + Vector3(0, 1, 0));
                if (_y == CHUNK_Y_SIZE || IsFaceVisibleBlock(blockDataId))
                {
                    BuildFace(blockFaceType::TOP, Vector3(_x, _y, _z));
                }

                // bottom check
                blockDataId = get_parent()->call("GetBlockId", trueBlockPos + Vector3(0, -1, 0));
                if (_y == CHUNK_Y_SIZE || IsFaceVisibleBlock(blockDataId))
                {
                    BuildFace(blockFaceType::BOTTOM, Vector3(_x, _y, _z));
                }

                // left check
                blockDataId = get_parent()->call("GetBlockId", trueBlockPos + Vector3(-1, 0, 0));
                if (_y == CHUNK_Y_SIZE || IsFaceVisibleBlock(blockDataId))
                {
                    BuildFace(blockFaceType::LEFT, Vector3(_x, _y, _z));
                }

                // right check
                blockDataId = get_parent()->call("GetBlockId", trueBlockPos + Vector3(1, 0, 0));
                if (_y == CHUNK_Y_SIZE || IsFaceVisibleBlock(blockDataId))
                {
                    BuildFace(blockFaceType::RIGHT, Vector3(_x, _y, _z));
                }

                // front check
                blockDataId = get_parent()->call("GetBlockId", trueBlockPos + Vector3(0, 0, 1));
                if (_y == CHUNK_Y_SIZE || IsFaceVisibleBlock(blockDataId))
                {
                    BuildFace(blockFaceType::FRONT, Vector3(_x, _y, _z));
                }

                // back check
                blockDataId = get_parent()->call("GetBlockId", trueBlockPos + Vector3(0, 0, -1));
                if (_y == CHUNK_Y_SIZE || IsFaceVisibleBlock(blockDataId))
                {
                    BuildFace(blockFaceType::BACK, Vector3(_x, _y, _z));
                }
            }
        }
    }

    CommitMesh();
}

void chunk::BeginMeshConstruction()
{
    surfaceToolInstance->begin(Mesh::PrimitiveType::PRIMITIVE_TRIANGLES);
}

void chunk::BuildFace(int faceType, Vector3 pos)
{
    Dictionary meshFacePos = get_parent()->get("meshFacePos");
    Dictionary meshFaceNormal = get_parent()->get("meshFaceNormal");
    surfaceToolInstance->add_uv(Vector2(0, 0));
    surfaceToolInstance->add_normal(meshFaceNormal[faceType]);

    Array meshPosInfoArray = meshFacePos[faceType];
    for (int i = 0; i < 6; i++)
    {
        surfaceToolInstance->add_vertex((Vector3)meshPosInfoArray[i] + pos);
    }
}

void chunk::CommitMesh()
{
    MeshInstance *mesh = (MeshInstance*)get_node("mesh");
    mesh->set_mesh(surfaceToolInstance->commit());
}

bool chunk::IsFaceVisibleBlock(int blockId)
{
    return (blockId == BLOCKDATA_MISSING || blockId == blockId::AIR);
}