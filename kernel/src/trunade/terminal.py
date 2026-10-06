"""Run Markdown C++ blocks in a real terminal, with interactive stdin.

    python -m trunade.terminal run [FILE] [ROW]  # block under ROW; menu if ROW is missing
    python -m trunade.terminal pick [PATH]       # menu; PATH may be a .md or a directory

Empty FILE/PATH (Zed task run from a view without a file) means the current directory,
where the most recently modified .md is used.
"""

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from trunade.config import Config
from trunade.executor import LocalExecutor, exit_code, signal_name
from trunade.markdown import CodeBlock, block_at_row, cpp_blocks

COLOR_FLAGS = ["-fdiagnostics-color=always"]
IGNORED_DIRS = {".git", ".venv", "node_modules", "target"}


def run_block(block: CodeBlock, source: Path) -> None:
    _print_rule(f"{source.name}:{block.start_row}  {block.heading}")
    executor = LocalExecutor(Config.from_env())
    with tempfile.TemporaryDirectory(prefix="trunade-") as workdir:
        compilation = executor.compile(block.code, Path(workdir), COLOR_FLAGS)
        sys.stderr.write(compilation.output)
        if compilation.timed_out:
            print("[compilation timed out]", file=sys.stderr)
        if compilation.succeeded:
            _run_interactively(compilation.binary)


def _run_interactively(binary: Path) -> None:
    try:
        return_code = subprocess.run([str(binary)], cwd=binary.parent).returncode
    except KeyboardInterrupt:
        print("\n[interrupted]", file=sys.stderr)
        return
    if signal_name(return_code):
        print(f"\n[killed by {signal_name(return_code)}]", file=sys.stderr)
    elif exit_code(return_code):
        print(f"\n[exit code {exit_code(return_code)}]", file=sys.stderr)


def pick_loop(markdown_file: Path) -> None:
    while True:
        blocks = cpp_blocks(markdown_file.read_text())
        if not blocks:
            print(f"No C++ blocks in {markdown_file}")
            return
        _print_menu(markdown_file, blocks)
        choice = _read_choice(len(blocks))
        if choice is None:
            return
        run_block(blocks[choice], markdown_file)


def _print_menu(markdown_file: Path, blocks: list[CodeBlock]) -> None:
    _print_rule(str(markdown_file))
    for number, block in enumerate(blocks, start=1):
        first_line = next((l for l in block.code.splitlines() if l.strip()), "")
        print(f"{number:3}) line {block.start_row:<4} {block.heading:<30.30} {first_line:.40}")


def _read_choice(count: int) -> int | None:
    while True:
        try:
            answer = input(f"\nBlock [1-{count}, Enter quits]: ").strip()
        except (EOFError, KeyboardInterrupt):
            return None
        if not answer:
            return None
        if answer.isdigit() and 1 <= int(answer) <= count:
            return int(answer) - 1
        print("Invalid choice.")


def latest_markdown(directory: Path) -> Path | None:
    candidates = [
        path
        for path in directory.rglob("*.md")
        if not IGNORED_DIRS.intersection(path.relative_to(directory).parts)
    ]
    return max(candidates, key=lambda path: path.stat().st_mtime, default=None)


def _print_rule(title: str) -> None:
    width = shutil.get_terminal_size().columns
    print(f"\n── {title} ".ljust(width, "─"), flush=True)


def _command_run(args: argparse.Namespace) -> None:
    if not args.file or args.row is None:
        _command_pick(argparse.Namespace(path=args.file))
        return
    source = Path(args.file)
    block = block_at_row(cpp_blocks(source.read_text()), args.row)
    if block is None:
        print(f"No C++ block at line {args.row}; choose one:")
        pick_loop(source)
        return
    run_block(block, source)


def _command_pick(args: argparse.Namespace) -> None:
    path = Path(args.path or ".")
    markdown_file = latest_markdown(path) if path.is_dir() else path
    if markdown_file is None:
        print(f"No .md files under {path}")
        return
    pick_loop(markdown_file)


def _optional_row(value: str) -> int | None:
    return int(value) if value.strip() else None


def main() -> None:
    parser = argparse.ArgumentParser(prog="trunade.terminal")
    commands = parser.add_subparsers(required=True)
    run = commands.add_parser("run", help="run the block under ROW")
    run.add_argument("file", nargs="?", default="")
    run.add_argument("row", nargs="?", type=_optional_row, default=None)
    run.set_defaults(handler=_command_run)
    pick = commands.add_parser("pick", help="choose a block from a menu")
    pick.add_argument("path", nargs="?", default="")
    pick.set_defaults(handler=_command_pick)
    args = parser.parse_args()
    args.handler(args)


if __name__ == "__main__":
    main()
