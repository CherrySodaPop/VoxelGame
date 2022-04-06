#pragma once

#include <core/Godot.hpp>
#include <Node.hpp>
#include <array>
#include <OpenSimplexNoise.hpp>
#include <NoiseTexture.hpp>
#include <PackedScene.hpp>

namespace godot
{
    class worldGeneration : public Node
    {
        GODOT_CLASS(worldGeneration, Node)
    public:
        worldGeneration();
        ~worldGeneration();
        static void _register_methods();
        void _init();
        void _ready();
        void _process();
        
        void GenerateChunk(int _x, int _z);

        int GetWorldBlockId(int _x, int _y, int _z);
        int GetChunkBlockId(int chunkX, int chunkZ, int chunkBlockX, int chunkBlockY, int chunkBlockZ);

        // world generation info - no touchy
        OpenSimplexNoise *GetNoise() { return pSimplexNoise; }
        std::array< std::array<Vector3, 6> , 6 > GetMeshFacePos() { return meshFacePositions; }
        std::array< Vector3 , 6 > GetMeshFaceNormal() { return meshFaceNormals; }

    private:
        std::array< std::array<Vector3, 6> , 6 > meshFacePositions;
        std::array< Vector3, 6 > meshFaceNormals;

        Ref<PackedScene> chunkScene;
        OpenSimplexNoise *pSimplexNoise;
        NoiseTexture *pNoiseTexture;

        // why still use a dictionary for storing the chunk instance?
        // its super easy and fast enough :P
        Dictionary chunckData;
    };
}