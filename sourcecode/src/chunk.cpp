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
    register_method((char*)"ConstructMesh", &chunk::ConstructMesh);
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
    parent = (worldGeneration*)get_parent();
    transformOrigin = get_transform().origin;
    Generate();
}

void chunk::_process(float delta)
{
}

Array chunk::ChunkData_BlockId()
{
    Array _chunkData;
    _chunkData.resize(CHUNK_X_SIZE);

    for (int _x = 0; _x < CHUNK_X_SIZE; _x++)
    {
        for (int _y = CHUNK_Y_SIZE - 1; _y > -1; _y++)
        {
            for (int _z = 0; _z < CHUNK_Z_SIZE; _z++)
            {
                Array _arrayY;
                Array _arrayZ;

                _arrayZ[_z] = dataBlockId[_x][_y][_z];
                _arrayY[_y] = _arrayZ;
                _chunkData[_x] = _arrayY;
            }
        }
    }

    return _chunkData;
}

int chunk::GetLocalBlockId(int _x, int _y, int _z)
{
    // if we somehow are outside the chunk, DIE
    if (_x < 0 || _y < 0 || _z < 0 ||
        _x >= CHUNK_X_SIZE || _y >= CHUNK_Y_SIZE || _z >= CHUNK_Z_SIZE)
    {
        return -1;
    }

    return dataBlockId[_x][_y][_z];
}

void chunk::Generate()
{
    OpenSimplexNoise *noise = parent->GetNoise();
    //Vector3 chunkSize = parent->get("chunkSize");
    for (int _x = 0; _x < CHUNK_X_SIZE; _x++)
    {
        for (int _z = 0; _z < CHUNK_Z_SIZE; _z++)
        {
            Vector2 trueBlockPos = Vector2(_x + transformOrigin.x, _z + transformOrigin.z);
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

// This is pretty much just a faster version of GetWorldBlock from worldGeneration.gd.
// (It does the world coords -> chunk coords conversion here, instead of in GDScript)
/*
int chunk::GetWorldBlockId(int x, int y, int z)
{
    int chunkX = x / CHUNK_X_SIZE;
    int chunkZ = z / CHUNK_Z_SIZE;
    return parent->call(
        "GetChunkBlock",
        chunkX,
        chunkZ,
        x - (chunkX * CHUNK_X_SIZE),
        y,
        z - (chunkZ * CHUNK_Z_SIZE)
    );
}

int chunk::GetWorldBlockId(Vector3 blockPos) {
    return GetWorldBlockId(blockPos.x, blockPos.y, blockPos.z);
}
*/

// Macros to avoid repetition when checking adjacent blocks
#define GetNearbyBlockId(xx, yy, zz) parent->GetWorldBlockId(trueBlockPos.x + xx, trueBlockPos.y + yy, trueBlockPos.z + zz)
#define ShouldBuildFace(x, y, z) atYMax || IsFaceVisibleBlock(GetNearbyBlockId(x, y, z))

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
                Vector3 localBlockPos = Vector3(_x, _y, _z);
                Vector3 trueBlockPos = transformOrigin + localBlockPos;
                int blockDataId = parent->GetWorldBlockId(trueBlockPos.x, trueBlockPos.y, trueBlockPos.z);

                // check if we're not air
                if (blockDataId == BLOCKDATA_MISSING || blockDataId == blockId::AIR) continue;

                bool atYMax = _y == chunkSize.y;

                // top check
                if (ShouldBuildFace(0, 1, 0))
                    BuildFace(blockFaceType::TOP, localBlockPos);

                // bottom check
                if (ShouldBuildFace(0, -1, 0))
                    BuildFace(blockFaceType::BOTTOM, localBlockPos);

                // left check
                if (ShouldBuildFace(-1, 0, 0))
                    BuildFace(blockFaceType::LEFT, localBlockPos);

                // right check
                if (ShouldBuildFace(1, 0, 0))
                    BuildFace(blockFaceType::RIGHT, localBlockPos);

                // front check
                if (ShouldBuildFace(0, 0, 1))
                    BuildFace(blockFaceType::FRONT, localBlockPos);

                // back check
                if (ShouldBuildFace(0, 0, -1))
                    BuildFace(blockFaceType::BACK, localBlockPos);
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
    surfaceToolInstance->add_uv(Vector2(0, 0));
    surfaceToolInstance->add_normal(parent->GetMeshFaceNormal()[faceType]);

    std::array< std::array<Vector3, 6> , 6 > meshPosInfoArray = parent->GetMeshFacePos();
    for (int i = 0; i < 6; i++)
    {
        surfaceToolInstance->add_vertex(meshPosInfoArray[faceType][i] + pos);
    }
}

void chunk::CommitMesh()
{
    MeshInstance *mesh = (MeshInstance*)get_node("mesh");
    mesh->set_mesh(surfaceToolInstance->commit());
}

bool chunk::IsFaceVisibleBlock(int blockDataId)
{
    return (blockDataId == BLOCKDATA_MISSING || blockDataId == blockId::AIR);
}
