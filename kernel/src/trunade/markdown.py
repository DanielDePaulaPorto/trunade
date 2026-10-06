from dataclasses import dataclass

CPP_FENCE_LANGUAGES = {"cpp", "c++", "cxx", "cc"}
FENCE_CHARS = ("`", "~")


@dataclass(frozen=True)
class CodeBlock:
    start_row: int  # 1-based row of the opening fence
    end_row: int  # 1-based row of the closing fence
    heading: str  # nearest heading above the block, "" if none
    code: str

    def contains(self, row: int) -> bool:
        return self.start_row <= row <= self.end_row


@dataclass(frozen=True)
class _Fence:
    marker: str
    language: str


def cpp_blocks(markdown: str) -> list[CodeBlock]:
    blocks: list[CodeBlock] = []
    heading = ""
    open_fence: _Fence | None = None
    start_row = 0
    body: list[str] = []
    for row, line in enumerate(markdown.splitlines(), start=1):
        if open_fence is None:
            if line.startswith("#"):
                heading = line.lstrip("#").strip()
            open_fence = _parse_opening_fence(line)
            start_row, body = row, []
        elif _closes(line, open_fence):
            if open_fence.language in CPP_FENCE_LANGUAGES:
                blocks.append(CodeBlock(start_row, row, heading, "\n".join(body) + "\n"))
            open_fence = None
        else:
            body.append(line)
    return blocks


def block_at_row(blocks: list[CodeBlock], row: int) -> CodeBlock | None:
    return next((block for block in blocks if block.contains(row)), None)


def _parse_opening_fence(line: str) -> _Fence | None:
    stripped = line.lstrip()
    for char in FENCE_CHARS:
        marker_length = len(stripped) - len(stripped.lstrip(char))
        if marker_length >= 3:
            info = stripped[marker_length:].strip().split()
            language = info[0].lower() if info else ""
            return _Fence(char * marker_length, language)
    return None


def _closes(line: str, fence: _Fence) -> bool:
    stripped = line.strip()
    return stripped.startswith(fence.marker) and stripped.strip(fence.marker[0]) == ""
