#pragma once

#include <core/Godot.hpp>
#include <PoolArrays.hpp>

// TODO: im not a massive fan of having this hardcoded here,
// though it might be necessary, so we could maybe pass this info over to godot?
// or maybe vice versa???? idk!
// https://godotengine.org/qa/115135/godot-engine-gdnative-enumerations-data-problem-gdnative
// until then keep this updated with blockData.gd

#define BLOCKDATA_ID 0
#define BLOCKDATA_META 1

#define CHUNK_X_SIZE 32
#define CHUNK_Z_SIZE 32
#define CHUNK_Y_SIZE 256

#define BLOCKDATA_MISSING -1

enum blockId {
	AIR,
	GRASS,
	DIRT,
	STONE,
};

enum blockFaceType {
	TOP,
	BOTTOM,
	LEFT,
	RIGHT,
	FRONT,
	BACK,
};

extern const char *blockNames[];