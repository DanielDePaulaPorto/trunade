---
layout: ../../../layouts/Docs.astro
title: Overview
description: What Trunade! is and how it works.
---

# Overview

Trunade! shows a lesson written in Markdown in the browser. Every code block becomes an editable box with a ▶: click it and the program is compiled with your machine's compiler, with the output in a terminal right below.

```bash
trunade lesson.md
```

The command serves the lesson at `http://localhost:4747` and opens the browser. Keep editing the `.md` in your editor; the page updates when you save.

## How it works

- **One binary.** `trunade` is a local server written in Rust with the interface built in. It only listens on your machine and works offline.
- **Every block is a program.** There is no state between blocks: each has its own `main()` and is compiled from scratch, the way students do it in an exam.
- **The lesson does not change.** Editing code in the browser never writes to the file. Reload and you are back to what the `.md` says.
- **A real compiler.** C++, C and Python come configured. Other languages go in the configuration file.

## Who it is for

For people teaching programming, algorithms and competitive programming who want to run examples on the projector without juggling slides, editor and terminal.

If state across cells is the point of the lesson (data analysis, plots), Jupyter is still the right tool.
