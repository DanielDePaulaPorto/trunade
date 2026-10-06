import resource
import signal
import subprocess
import tempfile
import time
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from trunade.config import Config

SOURCE_NAME = "main.cpp"
BINARY_NAME = "main"


@dataclass(frozen=True)
class ExecutionResult:
    compiled: bool
    compiler_output: str = ""
    stdout: str = ""
    stderr: str = ""
    exit_code: int | None = None
    signal_name: str | None = None
    timed_out: bool = False
    compile_seconds: float = 0.0
    run_seconds: float = 0.0


class Executor(Protocol):
    def execute(
        self, code: str, stdin: str = "", extra_flags: Sequence[str] = ()
    ) -> ExecutionResult: ...


@dataclass(frozen=True)
class _ProcessOutcome:
    stdout: str
    stderr: str
    return_code: int | None
    timed_out: bool
    seconds: float


@dataclass(frozen=True)
class Compilation:
    succeeded: bool
    output: str
    timed_out: bool
    seconds: float
    binary: Path


class LocalExecutor:
    def __init__(self, config: Config) -> None:
        self._config = config

    def execute(
        self, code: str, stdin: str = "", extra_flags: Sequence[str] = ()
    ) -> ExecutionResult:
        with tempfile.TemporaryDirectory(prefix="trunade-") as workdir:
            compilation = self.compile(code, Path(workdir), extra_flags)
            if not compilation.succeeded:
                return ExecutionResult(
                    compiled=False,
                    compiler_output=compilation.output,
                    timed_out=compilation.timed_out,
                    compile_seconds=compilation.seconds,
                )
            run = self._run(compilation.binary, stdin)
            return ExecutionResult(
                compiled=True,
                compiler_output=compilation.output,
                stdout=run.stdout,
                stderr=run.stderr,
                exit_code=exit_code(run.return_code),
                signal_name=signal_name(run.return_code),
                timed_out=run.timed_out,
                compile_seconds=compilation.seconds,
                run_seconds=run.seconds,
            )

    def compile(
        self, code: str, workdir: Path, extra_flags: Sequence[str] = ()
    ) -> Compilation:
        (workdir / SOURCE_NAME).write_text(code)
        binary = workdir / BINARY_NAME
        command = [
            self._config.compiler,
            *self._config.flags,
            *extra_flags,
            SOURCE_NAME,
            "-o",
            BINARY_NAME,
        ]
        try:
            outcome = self._run_process(
                "compile",
                command,
                workdir,
                stdin="",
                timeout_seconds=self._config.compile_timeout_seconds,
            )
        except FileNotFoundError:
            message = f"trunade: compiler not found: {self._config.compiler}\n"
            return Compilation(False, message, False, 0.0, binary)
        return Compilation(
            succeeded=outcome.return_code == 0,
            output=outcome.stdout + outcome.stderr,
            timed_out=outcome.timed_out,
            seconds=outcome.seconds,
            binary=binary,
        )

    def _run(self, binary: Path, stdin: str) -> _ProcessOutcome:
        return self._run_process(
            "run",
            [str(binary)],
            binary.parent,
            stdin=stdin,
            timeout_seconds=self._config.timeout_seconds,
            preexec_fn=self._limit_output_size,
        )

    def _limit_output_size(self) -> None:
        limit = self._config.output_limit_bytes
        resource.setrlimit(resource.RLIMIT_FSIZE, (limit, limit))

    def _run_process(
        self,
        phase: str,
        command: list[str],
        workdir: Path,
        stdin: str,
        timeout_seconds: float,
        preexec_fn: Callable[[], None] | None = None,
    ) -> _ProcessOutcome:
        stdout_path = workdir / f"{phase}.stdout"
        stderr_path = workdir / f"{phase}.stderr"
        started = time.perf_counter()
        with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
            try:
                completed = subprocess.run(
                    command,
                    cwd=workdir,
                    input=stdin.encode(),
                    stdout=stdout,
                    stderr=stderr,
                    timeout=timeout_seconds,
                    preexec_fn=preexec_fn,
                )
                return_code, timed_out = completed.returncode, False
            except subprocess.TimeoutExpired:
                return_code, timed_out = None, True
        return _ProcessOutcome(
            stdout=self._read_limited(stdout_path),
            stderr=self._read_limited(stderr_path),
            return_code=return_code,
            timed_out=timed_out,
            seconds=time.perf_counter() - started,
        )

    def _read_limited(self, path: Path) -> str:
        with path.open("rb") as file:
            return file.read(self._config.output_limit_bytes).decode(errors="replace")


def exit_code(return_code: int | None) -> int | None:
    if return_code is None or return_code < 0:
        return None
    return return_code


def signal_name(return_code: int | None) -> str | None:
    if return_code is None or return_code >= 0:
        return None
    try:
        return signal.Signals(-return_code).name
    except ValueError:
        return f"signal {-return_code}"
