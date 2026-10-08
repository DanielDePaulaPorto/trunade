---
layout: ../../../layouts/Docs.astro
title: Writing lessons
description: Code blocks, attributes and standard input.
---

# Writing lessons

A lesson is plain Markdown. Fenced blocks in a configured language become programs; everything else stays text.

## Code blocks

````markdown
```cpp
#include <iostream>
int main() { std::cout << "Hello, class!\n"; }
```
````

Languages and aliases configured out of the box:

| Language | Fence |
|---|---|
| C++ | `cpp`, `c++`, `cxx`, `cc` |
| C | `c` |
| Python | `python`, `py`, `python3` |

## Attributes

In braces, after the language:

````markdown
```cpp {flags="-O2 -g" timeout=5}
```
````

- `flags`: extra compiler flags, on top of the configured ones.
- `timeout`: time limit in seconds for this block.

## Standard input

There are two ways to feed a program.

### Fixed, shipped with the lesson

A `stdin` block right after the code (only blank lines in between):

````markdown
```cpp
#include <iostream>
int main() { int a, b; std::cin >> a >> b; std::cout << a + b << "\n"; }
```

```stdin
2 40
```
````

The program runs with that input, under the time limit, with stdout and stderr shown separately.

### Interactive, on the spot

Without a `stdin` block, the program runs in a real terminal: the class types the input in the page's own terminal. There is no time limit in this mode (the program may be waiting for someone to type). Use ■ to stop it.

## What shows up below

- **Compile errors:** the full compiler output, in color.
- **Non-zero exit codes**, **signals** (`Segmentation fault`), **time limit** and **output limit** appear after the program output.
- Compile and run times sit in the block's toolbar.

## Editing during class

Every block is editable in the browser. `Shift+Enter` runs the block with the cursor inside it. Edits never reach the `.md`: when the file changes, only the blocks that changed lose their edits.
