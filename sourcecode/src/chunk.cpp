#include "chunk.h"
#include <Mesh.hpp>
#include <MeshInstance.hpp>
#include <ArrayMesh.hpp>
#include "blockData.h"

using namespace godot;

void chunk::_register_methods()
{
    register_method((char*)"_ready", &chunk::_ready);
    register_method((char*)"_process", &chunk::_process);
    register_method((char*)"GetData", &chunk::GetData);
    register_property("data", &chunk::data, {});
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

void chunk::Generate()
{
    OpenSimplexNoise *noise = get_parent()->get("simplexNoise");
    Vector3 chunkSize = get_parent()->get("chunkSize");
    //Dictionary blockData = (get_parent()->get("blockData/info"));

    for (int _x = 0; _x < chunkSize.x; _x++)
    {
        for (int _z = 0; _z < chunkSize.z; _z++)
        {
            Vector2 trueBlockPos = Vector2(_x + this->get_transform().origin.x, _z + this->get_transform().origin.z);
            float noiseHeight = noise->get_noise_2dv(trueBlockPos);
            float terrainAmp = 0.1;
            int terrainPeak = int(chunkSize.y * ((noiseHeight / 2.0) + 0.5) * terrainAmp);

            for (int _y = chunkSize.y; _y > -1; _y--)
            {
                if (_y > terrainPeak)
                {
                    Array blockData;
                    Dictionary d;
                    blockData.append(blockId::AIR);
                    blockData.append(d);
                    data[Vector3(_x, _y, _z)] = blockData;
                    continue;
                }
                if (_y <= terrainPeak)
                {
                    Array blockData;
                    Dictionary d;
                    blockData.append(blockId::STONE);
                    blockData.append(d);
                    data[Vector3(_x, _y, _z)] = blockData;
                    continue;
                }
            }
        }
    }
}

void chunk::ConstructMesh()
{
    Vector3 chunkSize = get_parent()->get("chunkSize");

    BeginMeshConstruction();

    for (int _x = 0; _x < chunkSize.x; _x++)
    {
        for (int _z = 0; _z < chunkSize.z; _z++)
        {
            for (int _y = 0; _y < chunkSize.y; _y++)
            {
                Vector3 trueBlockPos = get_transform().origin + Vector3(_x, _y, _z);
                Array blockData = get_parent()->call("GetBlock", trueBlockPos);
                
                int blockDataId = (int)blockData[BLOCKDATA_ID];

                // check if we're not air
                if (blockDataId == BLOCKDATA_MISSING || blockDataId == blockId::AIR) continue;

                // top check
                blockData = get_parent()->call("GetBlock", trueBlockPos + Vector3(0, 1, 0));
                if (_y == chunkSize.y || IsFaceVisibleBlock(blockData))
                {
                    BuildFace(blockFaceType::TOP, Vector3(_x, _y, _z));
                }

                // bottom check
                blockData = get_parent()->call("GetBlock", trueBlockPos + Vector3(0, -1, 0));
                if (_y == chunkSize.y || IsFaceVisibleBlock(blockData))
                {
                    BuildFace(blockFaceType::BOTTOM, Vector3(_x, _y, _z));
                }

                // left check
                blockData = get_parent()->call("GetBlock", trueBlockPos + Vector3(-1, 0, 0));
                if (_y == chunkSize.y || IsFaceVisibleBlock(blockData))
                {
                    BuildFace(blockFaceType::LEFT, Vector3(_x, _y, _z));
                }

                // right check
                blockData = get_parent()->call("GetBlock", trueBlockPos + Vector3(1, 0, 0));
                if (_y == chunkSize.y || IsFaceVisibleBlock(blockData))
                {
                    BuildFace(blockFaceType::RIGHT, Vector3(_x, _y, _z));
                }

                // front check
                blockData = get_parent()->call("GetBlock", trueBlockPos + Vector3(0, 0, 1));
                if (_y == chunkSize.y || IsFaceVisibleBlock(blockData))
                {
                    BuildFace(blockFaceType::FRONT, Vector3(_x, _y, _z));
                }

                // back check
                blockData = get_parent()->call("GetBlock", trueBlockPos + Vector3(0, 0, -1));
                if (_y == chunkSize.y || IsFaceVisibleBlock(blockData))
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

bool chunk::IsFaceVisibleBlock(Array blockData)
{
    int blockDataId = (int)blockData[BLOCKDATA_ID];
    return (blockDataId == BLOCKDATA_MISSING || blockDataId == blockId::AIR);
}