#pragma once

#include <core/Godot.hpp>
#include <Spatial.hpp>
#include <SurfaceTool.hpp>
#include <OpenSimplexNoise.hpp>

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

        void Generate();
        void ConstructMesh();
        void BeginMeshConstruction();
        void BuildFace(int faceType, Vector3 pos);
        void CommitMesh();
        bool IsFaceVisibleBlock(Array blockData);
        Dictionary GetData() { return data; }

    private:
        Dictionary data;
        SurfaceTool *surfaceToolInstance;
    };
}