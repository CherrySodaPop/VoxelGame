#pragma once

#include <core/Godot.hpp>
#include <Spatial.hpp>
#include <SurfaceTool.hpp>
#include <OpenSimplexNoise.hpp>
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

        Variant GetBlockId(int _x, int _y, int _z);

        //Array GetChunkData_BlockId(); - godot converters

        void Generate();
        void ConstructMesh();
        void BeginMeshConstruction();
        void BuildFace(int faceType, Vector3 pos);
        void CommitMesh();
        bool IsFaceVisibleBlock(int blockId);

    private:
        SurfaceTool *surfaceToolInstance;
        int dataBlockId[CHUNK_X_SIZE][CHUNK_Y_SIZE][CHUNK_Z_SIZE];
    };
}