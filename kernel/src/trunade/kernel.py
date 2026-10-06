from ipykernel.kernelbase import Kernel

from trunade import __version__
from trunade.config import Config
from trunade.executor import ExecutionResult, Executor, LocalExecutor

OUTPUT_LIMIT_SIGNAL = "SIGXFSZ"


class TrunadeKernel(Kernel):
    implementation = "trunade"
    implementation_version = __version__
    language = "C++"
    language_version = "23"
    language_info = {
        "name": "C++",
        "version": language_version,
        "mimetype": "text/x-c++src",
        "file_extension": ".cpp",
    }
    banner = "Trunade! C++ kernel"

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._config = Config.from_env()
        self._executor: Executor = LocalExecutor(self._config)

    async def do_execute(
        self,
        code,
        silent,
        store_history=True,
        user_expressions=None,
        allow_stdin=False,
        *,
        cell_meta=None,
        cell_id=None,
    ):
        try:
            result = self._executor.execute(code)
        except KeyboardInterrupt:
            self._send_stream("stderr", "\n[interrupted]\n")
            return self._interrupted_reply()
        if not silent:
            self._send_result(result)
        return self._ok_reply()

    def _send_result(self, result: ExecutionResult) -> None:
        self._send_stream("stderr", result.compiler_output)
        self._send_stream("stdout", result.stdout)
        self._send_stream("stderr", result.stderr)
        self._send_stream("stderr", self._describe_termination(result))

    def _describe_termination(self, result: ExecutionResult) -> str:
        if result.timed_out:
            phase = "run" if result.compiled else "compilation"
            limit = (
                self._config.timeout_seconds
                if result.compiled
                else self._config.compile_timeout_seconds
            )
            return f"\n[{phase} timed out after {limit:g}s]\n"
        if result.signal_name == OUTPUT_LIMIT_SIGNAL:
            return f"\n[output limit of {self._config.output_limit_bytes} bytes exceeded]\n"
        if result.signal_name:
            return f"\n[killed by {result.signal_name}]\n"
        if result.exit_code:
            return f"\n[exit code {result.exit_code}]\n"
        return ""

    def _send_stream(self, name: str, text: str) -> None:
        if text:
            self.send_response(self.iopub_socket, "stream", {"name": name, "text": text})

    def _interrupted_reply(self) -> dict:
        return {
            "status": "error",
            "execution_count": self.execution_count,
            "ename": "KeyboardInterrupt",
            "evalue": "",
            "traceback": [],
        }

    def _ok_reply(self) -> dict:
        return {
            "status": "ok",
            "execution_count": self.execution_count,
            "payload": [],
            "user_expressions": {},
        }
