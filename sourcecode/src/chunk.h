#pragma once

#include <core/Godot.hpp>
#include <Spatial.hpp>
#include <SurfaceTool.hpp>
#include <OpenSimplexNoise.hpp>
#include "worldGeneration.h"
#include "blockData.h"

namespace godot
{
    class chunk : public Spatial
    {
        GODOT_CLASS(chunk, Spatial)
    public:
        chunk();
        ~chunk();
        static void _register_methods();
        void _init();
        void _ready();
        void _process(float delta);

        // converts from C array to Godot array
        Array ChunkData_BlockId();
        
        // get block info
        int GetLocalBlockId(int _x, int _y, int _z);
        
        //int GetWorldBlockId(int x, int y, int z);
        //int GetWorldBlockId(Vector3 blockPos);

        // mesh generation
        void Generate();
        void ConstructMesh();
        void BeginMeshConstruction();
        bool IsFaceVisibleBlock(int blockDataId);
        void BuildFace(int faceType, Vector3 pos);
        void CommitMesh();

    private:
        int dataBlockId[CHUNK_X_SIZE][CHUNK_Y_SIZE][CHUNK_Z_SIZE];

        SurfaceTool *surfaceToolInstance;
        //Array meshFacePositions;
        //PoolVector3Array meshFaceNormals;

        worldGeneration *parent;
        Vector3 transformOrigin;
    };
}
