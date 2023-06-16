# IMPORTANT (June 2023)
**This repository is dead!**

But the project lives on and is being replaced by something cooler in silence.
The latest version of this codebase is ``gd4-refactor``.

# Voxel Game
This is a very early development of a "voxel based Animal Crossing" sandbox game in Godot.

![game](https://user-images.githubusercontent.com/42105283/164111917-8e5a0663-3d5a-431d-b551-37a4cbf5d352.png)

# Things of Importance
- Godot Project version: 3.4.4
- Required Rust edition is 2021, the MSRV is currently undetermined.

  You will have to compile the Rust crates for world generation, a convenient build script is provided!
  
  LLVM is required for the first build (see [godot-rust](https://godot-rust.github.io/book/getting-started/setup.html)'s documentation).

# Todo
- ⬜ Threaded mesh generation
  - ✅ Basic threading (currently creates unlimited threads)
  - ⬜ Better threading (a thread pool, or a single thread using channels)
- ✅ ~~Server and client (multiplayer)~~
- ⬜ Proper player (inventory, health, etc.)
- ⬜ NPCs (see branch `npcs`)
