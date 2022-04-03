#pragma once

#include "core/Godot.hpp"
#include "Node.hpp"

namespace godot
{
    class chunk : public Node
    {
        GODOT_CLASS(chunk, Node)
    private:
        chunk();
        ~chunk();
        static void _register_methods();
        void _init();
        void _process(float delta);
    }
}