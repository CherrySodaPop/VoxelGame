# Voxel Game

An endless voxel sandbox.

``gdport-refactor``, as the name implies, is a complete refactor of VoxelGame, previously the goal was to have all the componenets of world generation be written in GDScript until Rust bindings were released for Godot 4.0

The problem with this approach was the amount of wait time till the bindings would release and the requirement of re-writing everything once they did release. GDExtension on the other hand is available now and after some research revealed its tight connection with the engine allowing for a much cleaner implementation of world generation.

This does not mean a Rust implementation is off the table, down the road if the Rust bindings prove themselves to be just as close knit, then a world generation port will be considered if not experimented with.
