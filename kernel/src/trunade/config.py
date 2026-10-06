import os
import shlex
from dataclasses import dataclass, field

DEFAULT_COMPILER = "g++-16"
DEFAULT_FLAGS = "-std=c++23 -Wall -Wextra"
DEFAULT_TIMEOUT_SECONDS = 10.0
DEFAULT_COMPILE_TIMEOUT_SECONDS = 60.0
DEFAULT_OUTPUT_LIMIT_BYTES = 1024 * 1024


@dataclass(frozen=True)
class Config:
    compiler: str = DEFAULT_COMPILER
    flags: list[str] = field(default_factory=lambda: shlex.split(DEFAULT_FLAGS))
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS
    compile_timeout_seconds: float = DEFAULT_COMPILE_TIMEOUT_SECONDS
    output_limit_bytes: int = DEFAULT_OUTPUT_LIMIT_BYTES

    @classmethod
    def from_env(cls) -> "Config":
        return cls(
            compiler=os.environ.get("TRUNADE_CXX", DEFAULT_COMPILER),
            flags=shlex.split(os.environ.get("TRUNADE_CXXFLAGS", DEFAULT_FLAGS)),
            timeout_seconds=float(
                os.environ.get("TRUNADE_TIMEOUT", DEFAULT_TIMEOUT_SECONDS)
            ),
            compile_timeout_seconds=float(
                os.environ.get(
                    "TRUNADE_COMPILE_TIMEOUT", DEFAULT_COMPILE_TIMEOUT_SECONDS
                )
            ),
            output_limit_bytes=int(
                os.environ.get("TRUNADE_OUTPUT_LIMIT", DEFAULT_OUTPUT_LIMIT_BYTES)
            ),
        )
