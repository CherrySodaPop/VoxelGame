#pragma once

// TODO: im not a massive fan of having this hardcoded here,
// though it might be necessary, so we could maybe pass this info over to godot?
// or maybe vice versa???? idk!
// https://godotengine.org/qa/115135/godot-engine-gdnative-enumerations-data-problem-gdnative
// until then keep this updated with blockData.gd

#define BLOCKDATA_ID 0
#define BLOCKDATA_META 1

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