cd godot-cpp
scons platform=windows generate_bindings=yes use_custom_api_file=yes custom_api_file=../api.json bits=64 -j4
cd ..
