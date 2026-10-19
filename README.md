<h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="site/public/brand/trunade-logo-escuro.svg">
    <img src="site/public/brand/trunade-logo.svg" alt="Trunade!" height="64">
  </picture>
</h1>

Programming lessons written in Markdown, with code blocks that compile and run in the browser.

```bash
trunade lesson.md
```

The command opens the lesson in the browser. Every ` ```cpp ` block becomes an editable box with ▶; the output shows right below, in a terminal. You edit the `.md` in your own editor (Zed, VS Code, Vim…) and the page updates when you save.

Full documentation (syntax, examples, lesson features): [trunade.org](https://www.trunade.org), or `site/src/pages/en/docs/` in this repository.

## Why not Jupyter?

Jupyter is great for data analysis, where each cell continues the state of the previous one and results (tables, plots) are stored in the notebook. A C++ programming lesson asks for something else.

| | Trunade! | Jupyter |
|---|---|---|
| **Lesson file** | Plain Markdown. Readable git diffs, opens in any editor, no stored output or metadata. | `.ipynb` JSON, with outputs and metadata mixed into the text. Hard merge conflicts. |
| **What a block is** | A whole program, with `main()`, compiled the way students compile in an exam or a contest. | A cell that continues earlier state. For C++ (xeus-cling), interpreted code with no `main` and its own semantics. |
| **Compiler** | The real one (`g++-16` by default, configurable), with your flags. Full, colored errors. | The kernel's. |
| **Runtime errors** | Exit code, signals (`Segmentation fault`), time limit and output limit are shown. | Kernel-dependent; an infinite loop hangs the cell. |
| **Standard input** | Interactive in a real terminal, or fixed in a ` ```stdin ` block kept with the lesson, with expected output and a judge-like verdict. | Limited `input()`; usually no interactive `cin` in C++. |
| **Projecting in class** | Light theme, adjustable font, section navigation with Space and an outline with `T`, no toolbars. | A notebook UI, built for editing. |
| **Installation** | One binary, works offline. | Python, Jupyter and a C++ kernel. |

Use Jupyter when state across cells is the point (data exploration, plots). Use Trunade! when every example is a program.

## Installation

Requirements: Rust, Node with pnpm, and a C++ compiler (on macOS, `brew install gcc` installs `g++-16`; Apple's `g++` is Clang and has no `<bits/stdc++.h>`).

```bash
./install.sh
```

The script builds the web interface, embeds it in the binary and installs `trunade` into `~/.cargo/bin`.

## Writing a lesson

Fenced blocks in a configured language become runnable cells. Everything else stays text.

````markdown
```cpp {flags="-O2" timeout=5}
#include <iostream>
int main() { int a, b; std::cin >> a >> b; std::cout << a + b << "\n"; }
```

```stdin
2 40
```

```expected
42
```
````

- **Attributes** on the fence: `flags` (extra compiler flags) and `timeout` (seconds).
- **Test cases:** ` ```stdin ` and ` ```expected ` blocks right after the code. Several pairs make several cases, each with ✓ or ✗. Without them, the program runs in a terminal and you type the input on the spot.
- **Templates:** declare the repeated code once (` ```cpp template=contest `, with a `//{code}` line) and use it only where you want (` ```cpp-contest `).
- **Formulas** (KaTeX), **diagrams** (` ```mermaid `), **questions** (` ```quiz `), **exercises in steps** (`<!-- step: Title -->`), callouts (`> [!TIP]`) and HTML, including `data-reveal` to keep an element hidden until its section comes up.

See the [syntax reference](https://www.trunade.org/en/docs/sintaxe/) and the [examples](https://www.trunade.org/en/docs/exemplos/).

## In class

| Key | Action |
|---|---|
| `T` | section outline (→ opens subsections) |
| `Space` / `↓` / `Page Down` | next section, smooth scroll |
| `Shift+Space` / `↑` / `Page Up` | previous section |
| `+` `-` `0` | bigger, smaller, reset font |
| `D` | switch theme |
| `W` | text width: narrow or full page |
| `F` | full screen |
| `Z` | zoom into a code block (`Esc` returns) |
| `R` | run every block |
| `⌘P` / `Ctrl+P` | export PDF |
| `O` | settings (also ☰) |
| `Shift+Enter` | run the block, with the cursor in the code |

The ☰ panel also exports a self-contained HTML file and copies the lesson as plain HTML for a Moodle page. Choices (fonts, theme, width, language, code colors) are kept in `~/.config/trunade/preferences.toml` for every lesson.

## Configuration

`~/.config/trunade/config.toml` applies to every lesson; a `trunade.toml` next to the `.md` overrides only what it declares.

```toml
timeout_seconds = 10
editor = ["zed", "{file}:{line}"]

[languages.cpp]
aliases = ["c++", "cxx", "cc"]
source = "main.cpp"
compile = ["g++-16", "-std=c++23", "-Wall", "{flags}", "{source}", "-o", "{binary}"]
run = ["{binary}"]
```

C++, C and Python come configured.
