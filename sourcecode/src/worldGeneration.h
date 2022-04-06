#pragma once

#include <core/Godot.hpp>
#include <Node.hpp>
#include <array>
#include <OpenSimplexNoise.hpp>
#include <NoiseTexture.hpp>

namespace godot
{
    class worldGeneration : public Node
    {
        GODOT_CLASS(worldGeneration, Node)
    public:
        worldGeneration();
        ~worldGeneration();
        static void _register_methods();
        void _ready();
        void _process();
        
        void GenerateChunk(int _x, int _z);
    private:
        std::array< std::array<Vector3, 6> , 6 > meshFacePositions;
        std::array< Vector3, 6 > meshFaceNormals;

        Ref<Resource> chunkScene;
        OpenSimplexNoise pSimplexNoise;
        NoiseTexture pNoiseTexture;

        // why still use a dictionary for storing the chunk instance?
        // its super easy and fast enough :P
        Dictionary chunckData;
    };
}