#include "chunk.h"

using namespace godot;

void chunk::_register_methods()
{
    register_method((char*)"_process", &chunk::_process);
}

chunk::chunk()
{
}

chunk::~chunk()
{
}