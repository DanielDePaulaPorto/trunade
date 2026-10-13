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
