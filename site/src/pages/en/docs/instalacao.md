---
layout: ../../../layouts/Docs.astro
title: Installation
description: Installing Trunade! on macOS and Linux.
---

# Installation

Trunade! runs on macOS and Linux.

## Requirements

- **Rust** (to build the binary).
- **Node with pnpm** (to build the interface, which is embedded in the binary).
- **A C++ compiler.** On macOS, install GCC with Homebrew:

```bash
brew install gcc
```

> The `g++` that ships with macOS is Apple Clang and has no `<bits/stdc++.h>`. That is why Trunade! defaults to `g++-16`. You can change it in the [configuration](/en/docs/configuracao/).

## Install

From the repository folder:

```bash
./install.sh
```

The script builds the web interface, embeds it in the binary and installs `trunade` in `~/.cargo/bin`.

## Use

```bash
trunade lesson.md
trunade lesson.md --port 5000
trunade lesson.md --no-open
trunade lesson.md --config other.toml
```

If port 4747 is taken, Trunade! picks a free one and prints the address in the terminal.

## Update

Run `install.sh` again and restart any running `trunade`.
