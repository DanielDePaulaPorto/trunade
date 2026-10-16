---
layout: ../../../layouts/Docs.astro
title: Configuration
description: Compilers, languages, time limits and editor.
---

# Configuration

Two files, read in this order:

1. `~/.config/trunade/config.toml`, for every lesson.
2. `trunade.toml` next to the `.md`, for that folder only.

Each one overrides only what it declares. With `--config file.toml`, only that file is read.

## Example

```toml
timeout_seconds = 10
compile_timeout_seconds = 60
output_limit_bytes = 1048576
editor = ["zed", "{file}:{line}"]

[languages.cpp]
aliases = ["c++", "cxx", "cc"]
source = "main.cpp"
compile = ["g++-16", "-std=c++23", "-Wall", "-Wextra", "-fdiagnostics-color=always", "{flags}", "{source}", "-o", "{binary}"]
run = ["{binary}"]
```

## Fields

| Field | What it does |
|---|---|
| `timeout_seconds` | run time limit with fixed `stdin` |
| `compile_timeout_seconds` | compile time limit |
| `output_limit_bytes` | output limit per run |
| `editor` | command to open the `.md` at the block's line |

## Languages

Each `[languages.name]` replaces the whole language.

| Field | What it does |
|---|---|
| `aliases` | other names accepted in the fence |
| `source` | source file name |
| `compile` | compile command (optional; Python has none) |
| `run` | run command |

In the commands, `{source}` and `{binary}` become the temporary paths and `{flags}` becomes the block's flags, as separate arguments.

### Example: Rust

```toml
[languages.rust]
aliases = ["rs"]
source = "main.rs"
compile = ["rustc", "-O", "{flags}", "{source}", "-o", "{binary}"]
run = ["{binary}"]
```

## Page preferences

What you pick in the ☰ panel (fonts, theme, code colors, width, size, language) is saved by Trunade! in `~/.config/trunade/preferences.toml`. It applies to every lesson. You can edit it by hand; unknown values are ignored.

```toml
theme = "light"            # auto | light | dark
code_theme = "dark"        # auto | light | dark
locale = "en"              # pt | en
font_scale = 1.2
text_width = "narrow"      # narrow | wide
title_font = "gloock"      # gloock | ibm-plex-mono | fraunces | cormorant-garamond | young-serif
body_font = "inter"        # jetbrains-mono | fira-code | inter | open-sans
code_font = "fira-code"    # jetbrains-mono | fira-code
```

## Usage log

Every time Trunade! opens a lesson it writes a local log to `~/.local/share/trunade/logs/` (or `$XDG_DATA_HOME/trunade/logs/`), one file per session, in JSON Lines: `2026-10-16T14-02-33Z-aula14.jsonl`. The path is printed in the terminal at startup.

The log is anonymous and never leaves the machine. It keeps what happened on the page, not who was in the room:

| Event | Fields |
|---|---|
| `start`, `document` | `.md` name, version, number of blocks (`document` when the file changes) |
| `connect`, `disconnect`, `visibility` | page opened, closed, in the background |
| `section` | heading where the page stopped |
| `run` | block (`cell`, `line`, `language`), whether it was edited on the page, number of cases |
| `case` | outcome and verdict of each test case, time |
| `finished` | outcome, times, first line of the compile error |
| `quiz` | question, chosen options, whether it was right |
| `step`, `zoom`, `print`, `runAll` | step shown, zoomed block, PDF, run all |

Every event has a UTC `time`. To erase it, delete the files.
