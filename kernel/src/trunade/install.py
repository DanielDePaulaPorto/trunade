import argparse
import json
import shutil
import sys
from pathlib import Path

from jupyter_client.kernelspec import KernelSpecManager

from trunade.config import (
    DEFAULT_COMPILE_TIMEOUT_SECONDS,
    DEFAULT_COMPILER,
    DEFAULT_FLAGS,
    DEFAULT_OUTPUT_LIMIT_BYTES,
    DEFAULT_TIMEOUT_SECONDS,
)

# Zed shows the kernelspec directory name, not display_name.
KERNEL_NAME = "Trunade"


def resolve_compiler(compiler: str) -> str:
    """Store an absolute path: Zed may launch kernels with a PATH that lacks Homebrew."""
    resolved = shutil.which(compiler)
    if resolved is None:
        print(f"warning: compiler '{compiler}' not found in PATH", file=sys.stderr)
        return compiler
    return resolved


def build_kernel_spec(args: argparse.Namespace) -> dict:
    return {
        "argv": [sys.executable, "-m", "trunade", "-f", "{connection_file}"],
        "display_name": "Trunade! (C++)",
        "language": "C++",
        "interrupt_mode": "signal",
        "env": {
            "TRUNADE_CXX": resolve_compiler(args.cxx),
            "TRUNADE_CXXFLAGS": args.flags,
            "TRUNADE_TIMEOUT": str(args.timeout),
            "TRUNADE_COMPILE_TIMEOUT": str(args.compile_timeout),
            "TRUNADE_OUTPUT_LIMIT": str(args.output_limit),
        },
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Install the Trunade! kernelspec.")
    parser.add_argument("--cxx", default=DEFAULT_COMPILER)
    parser.add_argument("--flags", default=DEFAULT_FLAGS)
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_SECONDS)
    parser.add_argument(
        "--compile-timeout", type=float, default=DEFAULT_COMPILE_TIMEOUT_SECONDS
    )
    parser.add_argument("--output-limit", type=int, default=DEFAULT_OUTPUT_LIMIT_BYTES)
    return parser.parse_args()


def remove_previous_installs(kernels_dir: Path) -> None:
    """Drop any spec whose name differs only in case (e.g. the old 'trunade')."""
    for existing in kernels_dir.glob("*"):
        if existing.name.lower() == KERNEL_NAME.lower():
            shutil.rmtree(existing)


def main() -> None:
    spec = build_kernel_spec(parse_args())
    # install_kernel_spec() lowercases the name, so write the directory ourselves.
    kernels_dir = Path(KernelSpecManager().user_kernel_dir)
    kernels_dir.mkdir(parents=True, exist_ok=True)
    remove_previous_installs(kernels_dir)
    destination = kernels_dir / KERNEL_NAME
    destination.mkdir()
    (destination / "kernel.json").write_text(json.dumps(spec, indent=2))
    print(f"Installed kernelspec '{KERNEL_NAME}' at {destination}")


if __name__ == "__main__":
    main()
