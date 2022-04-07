# block data!

enum id {
	AIR,
	GRASS,
	DIRT,
	STONE,
}

const info = {
	id.AIR : {}, # no data dummy!
	id.GRASS : {
		# note: you should specify this data to other blocks EVEN if it's super obvious stuff (except air xd)
		# and also add checks in your functions if the key exists before reading it, just to avoid crashes
		"texture" : "", # texture.
		"breakTime" : 1.0, # the amount of time, in seconds, to break it!
		"fullBlock" : true, # if it's a full 1x1x1 block it'll hide/skip other full block's faces
		"isSceneBlock" : false, # scene block, instead of creating a block thats part of the chunk mesh, a scene is made instead, example: doors
		"sceneBlock" : "",
	},
	id.DIRT : {
		"texture" : "",
		"breakTime" : 1.0,
		"fullBlock" : true,
		"isSceneBlock" : false,
		"sceneBlock" : "",
	},
	id.STONE : {
		"texture" : "",
		"breakTime" : 10.0,
		"fullBlock" : true,
		"isSceneBlock" : false,
		"sceneBlock" : "",
	},
}
