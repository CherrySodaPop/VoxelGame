import shutil
import subprocess
from http import client
from pathlib import Path
from platform import system

ABOUT = """Convenient script for compiling the Rust libraries the project uses.
Handles copying the resulting .DLLs/.SOs to where they're required.
Also useful for testing, as it can run the client and server without you having \
to switch between many windows.
"""

OS_NAME = system().lower()
LIBRARY_NAME = "libchunkgeneration" if OS_NAME == "linux" else "chunkgeneration"
LIBRARY_SUFFIX = ".so" if OS_NAME == "linux" else ".dll"
CRATE_ROOT = Path(__file__).parent
PROJECT_ROOT = CRATE_ROOT.parent.parent
SERVER_PATH = PROJECT_ROOT.joinpath("sourcecode/VoxelGameServer")
CLIENT_PATH = PROJECT_ROOT.joinpath("sourcecode/VoxelGame")

# fmt: off
BLACK   = "\x1b[90m"
RED     = "\x1b[91m"
GREEN   = "\x1b[92m"
YELLOW  = "\x1b[93m"
BLUE    = "\x1b[94m"
MAGENTA = "\x1b[95m"
CYAN    = "\x1b[96m"
WHITE   = "\x1b[97m"
OFF     = "\x1b[0m"
# fmt: on


def build_crates() -> bool:
    print("Building crate...", end=" ", flush=True)
    build_proc = subprocess.Popen(
        [
            "cargo",
            "build",
            # We still want color even though the output is being
            # piped, as we're just going to print it out later.
            "--color",
            "always",
            "--release",
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    _, stderr = build_proc.communicate()
    if build_proc.returncode != 0:
        print(f"The {RED}build failed{OFF}, tsk tsk tsk!")
        print(stderr.decode("utf-8"))
        return False
    print(f"Build {GREEN}successful{OFF}!")
    return True


def copy_libraries() -> bool:
    print("Copying native libraries...")
    lib_path = CRATE_ROOT.joinpath("target/release/" + LIBRARY_NAME ).with_suffix(
        LIBRARY_SUFFIX
    )
    output_dirs = [
        CLIENT_PATH.joinpath("bins").resolve(),
        SERVER_PATH.joinpath("bins").resolve(),
    ]
    for output_dir in output_dirs:
        output_path = output_dir.joinpath(lib_path.with_stem("libchunkgeneration").name)
        output_path_display = output_path.relative_to(PROJECT_ROOT).as_posix()
        try:
            dst = Path(shutil.copy2(lib_path, output_path)).resolve()
        except PermissionError:
            print(
                f"\t{RED}Permission error{OFF} copying {CYAN}{lib_path.name}{OFF} to {CYAN}{output_path_display}{OFF}!"
            )
            print(f"Please ensure that the game isn't running.")
            return False
        print(
            f"\t{GREEN}Copied{OFF} {CYAN}{lib_path.name}{OFF} to {CYAN}{output_path_display}{OFF}."
        )
    print("Successfully copied native libraries.")
    return True


def run_game(godot_path: str):
    print("Running game server...", end=" ")
    server_proc = subprocess.Popen(
        [godot_path, "--no-window", "--path", SERVER_PATH],
        stderr=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
    )
    print(f"Server {GREEN}running{OFF} with PID {CYAN}{server_proc.pid}{OFF}.")
    print(f"Running client...", end=" ")
    client_proc = subprocess.Popen(
        [godot_path, "--path", CLIENT_PATH],
        stderr=subprocess.PIPE,
        stdout=subprocess.PIPE,
    )
    print(f"Client {GREEN}running{OFF}.")
    _ = client_proc.communicate()
    print(f"Client has been {RED}closed{OFF}.")
    print("Killing server...", end=" ")
    server_proc.kill()
    server_exit_code = server_proc.wait()
    print(f"Server {RED}closed{OFF}.")


def main():
    if OS_NAME not in ("windows", "linux"):
        print("This build script does not support your operating system!")
        print(f"Your OS was detected as: {OS_NAME}")
        return

    import argparse

    parser = argparse.ArgumentParser(
        description=ABOUT, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--no-build", help="Don't build the Rust crates.", action="store_true"
    )
    parser.add_argument(
        "-r",
        "--run",
        metavar="godot_path",
        dest="godot_path",
        help="Run the client & server after building.",
    )
    args = parser.parse_args()

    if not args.no_build:
        build_successful = build_crates()
        if not build_successful:
            return
        copy_successful = copy_libraries()
        if not copy_successful:
            return

    if args.godot_path:
        run_game(args.godot_path)


if __name__ == "__main__":
    main()
