cd godot-cpp
scons platform=windows generate_bindings=yes use_custom_api_file=yes custom_api_file=../api.json bits=64 target=release -j4
cd ..
