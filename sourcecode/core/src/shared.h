#ifndef VG_SHARED_H
#define VG_SHARED_H

#define CHUNK_WIDTH_LENGTH 32
#define CHUNK_HEIGHT 256

enum BLOCK_IDS {
    PLACEHOLDER_0, // these are just for scenarios where we need "logic" blocks for generation, just in case
    PLACEHOLDER_1,
    PLACEHOLDER_2,
    PLACEHOLDER_3,
    PLACEHOLDER_4,
    PLACEHOLDER_5,
    PLACEHOLDER_6,
    PLACEHOLDER_7,
    PLACEHOLDER_8,
    PLACEHOLDER_9,
    PLACEHOLDER_10,
    PLACEHOLDER_11,
    PLACEHOLDER_12,
    PLACEHOLDER_13,
    PLACEHOLDER_14,
    AIR,
    DIRT,
    GRASS,
    LIMESTONE,
    SHALE,
    OAK_WOOD_LOG,
    OAK_LEAVES,
    BLOCK_LAST,
};

#endif // VG_SHARED_H