import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from platform import system
from typing import NamedTuple, Optional

ABOUT = """Convenient script for building the project, \
and copying the resulting .DLLs/.SOs to where they're required.
\
Also useful for testing, as it can run the client and server without you having \
to switch between many windows.
"""

OS_NAME = system().lower()
LIBRARY_PREFIX = "lib" if OS_NAME == "linux" else ""
LIBRARY_SUFFIX = ".so" if OS_NAME == "linux" else ".dll"

CRATES_ROOT = Path(__file__).parent
TARGET_PATH = CRATES_ROOT.joinpath("target")

PROJECT_ROOT = CRATES_ROOT.parent.parent
SERVER_PATH = PROJECT_ROOT.joinpath("sourcecode/VoxelGame/server")
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


# Crate building


def build_crates(debug: bool = False) -> bool:
    print("Building crate...", end=" ", flush=True)
    args = [
        "cargo",
        "build",
        # We still want color even though the output is being
        # piped, as we're just going to print it out later.
        "--color",
        "always",
    ]
    if not debug:
        args.append("--release")
    build_proc = subprocess.Popen(
        args,
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


# File copying


def pretty_path(path: Path, relative_to: Optional[Path] = None) -> str:
    """Formats a `Path` using pretty colors."""
    if relative_to is not None:
        path = path.relative_to(relative_to)
    return f"{CYAN}{path.as_posix()}{OFF}"


@dataclass
class FileCopy:
    src: Path
    dst: Path

    def execute(self) -> Path:
        """Performs the file copy operation.

        Returns the path of the resulting file."""
        return Path(shutil.copy2(self.src, self.dst)).resolve()

    def pretty(self, relative_to: Path) -> str:
        """Formats the file copy using pretty colors."""
        return f"{pretty_path(self.src, relative_to)} to {pretty_path(self.dst, relative_to)}"


def library_filename(library_name: str) -> str:
    return f"{LIBRARY_PREFIX}{library_name}{LIBRARY_SUFFIX}"


def copy_libraries(debug: bool = False) -> bool:
    print("Copying native libraries...")
    binary_dir = TARGET_PATH.joinpath("debug" if debug else "release")
    client_lib_path = binary_dir.joinpath(library_filename("chunkclient"))
    server_lib_path = binary_dir.joinpath(library_filename("chunkserver"))
    copies = [
        FileCopy(client_lib_path, CLIENT_PATH.joinpath("bins")),
        FileCopy(server_lib_path, SERVER_PATH.joinpath("bins")),
    ]
    for copy in copies:
        copy_pretty = copy.pretty(PROJECT_ROOT)
        try:
            copy.execute()
        except PermissionError:
            print(f"\t{RED}Permission error{OFF} copying {copy_pretty}!")
            print(f"Please ensure that the game isn't running.")
            return False
        except FileNotFoundError:
            print(f"\t{RED}Missing file{OFF} {pretty_path(copy.src, PROJECT_ROOT)}!")
            print(
                "You might need to build the crates first, which this script can do for you."
            )
            return False
        print(f"\t{GREEN}Copied{OFF} {copy_pretty}")
    print("Successfully copied native libraries.")
    return True


# Game running


def run_game(godot_path: str, stdout: str):
    server_pipe = subprocess.DEVNULL
    client_pipe = subprocess.DEVNULL
    if stdout == "client":
        client_pipe = None
    elif stdout == "server":
        server_pipe = None
    print("Running game server...", end=" ")
    server_proc = subprocess.Popen(
        [godot_path, "--no-window", "--path", SERVER_PATH],
        stderr=server_pipe,
        stdout=server_pipe,
    )
    print(f"Server {GREEN}running{OFF} with PID {CYAN}{server_proc.pid}{OFF}.")
    print(f"Running client...", end=" ")
    client_proc = subprocess.Popen(
        [godot_path, "--path", CLIENT_PATH], stderr=client_pipe, stdout=client_pipe
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
        "--debug", help="Build/copy the crates in debug mode.", action="store_true"
    )
    parser.add_argument(
        "--no-build",
        help="Don't build the Rust crates.",
        action="store_true",
    )
    parser.add_argument(
        "-r",
        "--run",
        metavar="godot_path",
        dest="godot_path",
        help="Run the client & server after building.",
    )
    parser.add_argument(
        "--stdout",
        "-s",
        help="Specifies whether the server, client, or neither should have its stdout+stderr printed.",
        choices=["server", "s", "client", "c", "neither"],
        default="neither",
    )
    args = parser.parse_args()

    if args.stdout == "s":
        args.stdout = "server"
    if args.stdout == "c":
        args.stdout = "client"

    if not args.no_build:
        build_successful = build_crates(args.debug)
        if not build_successful:
            return
    copy_successful = copy_libraries(args.debug)
    if not copy_successful:
        return

    if args.godot_path:
        run_game(args.godot_path, args.stdout)


if __name__ == "__main__":
    main()
