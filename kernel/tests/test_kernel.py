import json
import sys
import time

import pytest
from jupyter_client.manager import start_new_kernel

KERNEL_NAME = "trunade-test"


@pytest.fixture(scope="module")
def kernel(tmp_path_factory):
    jupyter_path = tmp_path_factory.mktemp("jupyter")
    spec_dir = jupyter_path / "kernels" / KERNEL_NAME
    spec_dir.mkdir(parents=True)
    spec = {
        "argv": [sys.executable, "-m", "trunade", "-f", "{connection_file}"],
        "display_name": "Trunade! test",
        "language": "C++",
        "interrupt_mode": "signal",
        "env": {"TRUNADE_TIMEOUT": "30"},
    }
    (spec_dir / "kernel.json").write_text(json.dumps(spec))
    with pytest.MonkeyPatch.context() as monkeypatch:
        monkeypatch.setenv("JUPYTER_PATH", str(jupyter_path))
        manager, client = start_new_kernel(kernel_name=KERNEL_NAME)
    yield manager, client
    client.stop_channels()
    manager.shutdown_kernel(now=True)


@pytest.fixture
def client(kernel):
    return kernel[1]


def execute(client, code: str) -> tuple[dict, list[dict]]:
    streams = []

    def collect_stream(message):
        if message["msg_type"] == "stream":
            streams.append(message["content"])

    reply = client.execute_interactive(code, timeout=60, output_hook=collect_stream)
    return reply["content"], streams


def text_of(streams: list[dict], name: str) -> str:
    return "".join(s["text"] for s in streams if s["name"] == name)


def test_kernel_info_reports_cpp(client):
    reply = client.kernel_info(reply=True, timeout=10)
    language_info = reply["content"]["language_info"]
    assert language_info["name"] == "C++"
    assert "version" in language_info  # required by Zed's jupyter-protocol


def test_program_output_is_streamed(client):
    reply, streams = execute(
        client, '#include <iostream>\nint main() { std::cout << "hi"; }\n'
    )
    assert reply["status"] == "ok"
    assert streams == [{"name": "stdout", "text": "hi"}]


def test_compile_error_goes_to_stderr(client):
    _, streams = execute(client, "int main() { return x; }\n")
    assert "'x' was not declared" in text_of(streams, "stderr")
    assert text_of(streams, "stdout") == ""


def test_exit_code_is_reported(client):
    _, streams = execute(client, "int main() { return 3; }\n")
    assert "[exit code 3]" in text_of(streams, "stderr")


def test_signal_is_reported(client):
    _, streams = execute(client, "int main() { volatile int* p = nullptr; return *p; }\n")
    assert "[killed by SIGSEGV]" in text_of(streams, "stderr")


def test_interrupt_stops_running_program(kernel):
    manager, client = kernel
    client.execute("int main() { for (;;) {} }\n")
    time.sleep(3)
    manager.interrupt_kernel()
    reply = client.get_shell_msg(timeout=10)
    assert reply["msg_type"] == "execute_reply"
    assert reply["content"]["ename"] == "KeyboardInterrupt"
    follow_up, streams = execute(client, "int main() { return 4; }\n")
    assert "[exit code 4]" in text_of(streams, "stderr")
