from trunade.markdown import block_at_row, cpp_blocks

LESSON = """# Aula

## Saída

```cpp
int main() {}
```

```python
print("ignored")
```

## Segundo

~~~c++
int a;
int main() { return a; }
~~~
"""


def test_only_cpp_blocks_are_extracted_with_heading_and_rows():
    blocks = cpp_blocks(LESSON)
    assert [(b.start_row, b.end_row, b.heading) for b in blocks] == [
        (5, 7, "Saída"),
        (15, 18, "Segundo"),
    ]
    assert blocks[1].code == "int a;\nint main() { return a; }\n"


def test_block_at_row_includes_fence_lines():
    blocks = cpp_blocks(LESSON)
    assert block_at_row(blocks, 5) is blocks[0]
    assert block_at_row(blocks, 7) is blocks[0]
    assert block_at_row(blocks, 8) is None


def test_hash_inside_code_is_not_a_heading():
    blocks = cpp_blocks("## Real\n```cpp\n#include <x>\n```\n```cpp\n```\n")
    assert [b.heading for b in blocks] == ["Real", "Real"]


def test_longer_fence_needs_matching_close():
    blocks = cpp_blocks("````cpp\n```\n````\n")
    assert blocks[0].code == "```\n"
