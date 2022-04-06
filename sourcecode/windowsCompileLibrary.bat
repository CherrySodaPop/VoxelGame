@ECHO off
echo If using MSVC, make sure you're in the x64 Native Tools command prompt!

REM Using multiple threads for compilation on Windows seems
REM to cause link errors, for some reason.
scons platform=windows bits=64 target=release -j1
