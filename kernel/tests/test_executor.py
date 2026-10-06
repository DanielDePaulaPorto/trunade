import shutil

import pytest

from trunade.config import Config
from trunade.executor import LocalExecutor

COMPILER = Config.from_env().compiler

pytestmark = pytest.mark.skipif(
    shutil.which(COMPILER) is None, reason=f"{COMPILER} not installed"
)


@pytest.fixture
def executor() -> LocalExecutor:
    return LocalExecutor(Config(compiler=COMPILER, timeout_seconds=3))


def test_successful_program_writes_stdout_and_stderr(executor):
    result = executor.execute(
        '#include <iostream>\n'
        'int main() { std::cout << "out"; std::cerr << "err"; }\n'
    )
    assert result.compiled
    assert result.stdout == "out"
    assert result.stderr == "err"
    assert result.exit_code == 0
    assert result.signal_name is None


def test_bits_stdcpp_header_is_available(executor):
    result = executor.execute("#include <bits/stdc++.h>\nint main() {}\n")
    assert result.compiled, result.compiler_output


def test_compile_error_returns_compiler_output_verbatim(executor):
    result = executor.execute("int main() { return x; }\n")
    assert not result.compiled
    assert "'x' was not declared" in result.compiler_output
    assert result.exit_code is None


def test_warnings_are_kept_on_success(executor):
    result = executor.execute("int main() { int unused; }\n")
    assert result.compiled
    assert "unused" in result.compiler_output


def test_nonzero_exit_code_is_reported(executor):
    result = executor.execute("int main() { return 3; }\n")
    assert result.exit_code == 3


def test_segfault_is_reported_as_signal(executor):
    result = executor.execute(
        "int main() { volatile int* p = nullptr; return *p; }\n"
    )
    assert result.signal_name == "SIGSEGV"
    assert result.exit_code is None


def test_infinite_loop_times_out(executor):
    result = executor.execute("int main() { for (;;) {} }\n", extra_flags=["-O0"])
    assert result.compiled
    assert result.timed_out


def test_stdin_is_forwarded(executor):
    result = executor.execute(
        "#include <iostream>\n"
        "int main() { int a, b; std::cin >> a >> b; std::cout << a + b; }\n",
        stdin="2 40\n",
    )
    assert result.stdout == "42"


def test_missing_stdin_reads_eof_instead_of_blocking(executor):
    result = executor.execute(
        "#include <iostream>\n"
        "int main() { int a; std::cout << bool(std::cin >> a); }\n"
    )
    assert result.stdout == "0"


def test_extra_flags_reach_the_compiler(executor):
    result = executor.execute(
        "int main() { return VALUE; }\n", extra_flags=["-DVALUE=7"]
    )
    assert result.exit_code == 7


def test_output_flood_is_cut_by_size_limit():
    executor = LocalExecutor(
        Config(compiler=COMPILER, timeout_seconds=5, output_limit_bytes=4096)
    )
    result = executor.execute(
        "#include <cstdio>\nint main() { for (;;) std::puts(\"spam\"); }\n"
    )
    assert result.signal_name == "SIGXFSZ"
    assert len(result.stdout) <= 4096


def test_missing_compiler_is_reported():
    executor = LocalExecutor(Config(compiler="trunade-no-such-compiler"))
    result = executor.execute("int main() {}\n")
    assert not result.compiled
    assert "compiler not found" in result.compiler_output
