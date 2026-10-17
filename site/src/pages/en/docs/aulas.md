---
layout: ../../../layouts/Docs.astro
title: Writing lessons
description: Code blocks, attributes and standard input.
---

# Writing lessons

A lesson is plain Markdown. Fenced blocks in a configured language become programs; everything else stays text. This page is a tour; the full reference is in [Syntax](/en/docs/sintaxe/) and ready-made recipes in [Examples](/en/docs/exemplos/).

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

## Template: the code that repeats

To avoid repeating `#include`, `using namespace std` and fast input in every block, declare a template once. The `{code}` line (it may sit behind a comment: `//{code}`, `# {code}`) marks where each block goes:

````markdown
```cpp template
#include <bits/stdc++.h>
using namespace std;
//{code}
```

```cpp
int main() { cout << gcd(12, 18) << "\n"; }
```
````

- The template applies to the following blocks of the same language (aliases included), until the next template. An empty template turns it off.
- Without a `{code}` line, the whole template goes before the block.
- The template is not shown on the page. Each block shows a discreet strip, "2 template lines above", that opens the hidden part dimmed and read-only.
- Line numbers are those of the whole program (the block above starts at line 3), so compiler errors match what you see.

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

### Test cases and expected output

After the `stdin`, an `expected` block says what the program should print. Several `stdin`/`expected` pairs in a row become several cases: the program is compiled once and runs on each.

````markdown
```stdin
4 6
```

```expected
12
```

```stdin
3 5
```

```expected
15
```
````

Each case gets ✓ or ✗ next to its input, and the terminal shows each output followed by the verdict, like a judge: `✗ wrong answer on line 1: expected «16», got «15»`. Trailing spaces and trailing blank lines do not count. A program that ends with an error (nonzero exit code, signal, time limit) gets ✗ without comparing the output.

An `expected` with no `stdin` before it runs with empty input. A `stdin` with no `expected` just runs, with no verdict. Input and expected output can also be edited on the page.

### Interactive, on the spot

Without a `stdin` block, the program runs in a real terminal: the class types the input in the page's own terminal. There is no time limit in this mode (the program may be waiting for someone to type). Use ■ to stop it.

## What shows up below

- **Compile errors:** the full compiler output, in color.
- **Non-zero exit codes**, **signals** (`Segmentation fault`), **time limit** and **output limit** appear after the program output.
- Compile and run times sit in the block's toolbar.

## Editing during class

Every block is editable in the browser. `Shift+Enter` runs the block with the cursor inside it. Edits never reach the `.md`: when the file changes, only the blocks that changed lose their edits.
