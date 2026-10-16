---
layout: ../../../layouts/Docs.astro
title: In class
description: Keyboard shortcuts, theme, font size and text width.
---

# In class

The lesson page is built for the projector: white background in the light theme, no fixed toolbars, and everything on the keyboard.

## Shortcuts

| Key | Action |
|---|---|
| <kbd>T</kbd> | section outline (<kbd>→</kbd> opens subsections) |
| <kbd>Space</kbd>, <kbd>↓</kbd> or <kbd>Page Down</kbd> | next section, with smooth scrolling |
| <kbd>⇧ Space</kbd>, <kbd>↑</kbd> or <kbd>Page Up</kbd> | previous section |
| <kbd>+</kbd> <kbd>−</kbd> <kbd>0</kbd> | bigger, smaller, reset font |
| <kbd>W</kbd> | text width: narrow or full page |
| <kbd>F</kbd> | full screen (<kbd>Esc</kbd> or <kbd>F</kbd> leaves) |
| <kbd>Z</kbd> | zooms the code block in use (or the one nearest the middle of the screen) to fill the screen; <kbd>Esc</kbd> returns |
| <kbd>R</kbd> | run all: every block with fixed input, in order |
| <kbd>D</kbd> | theme: automatic, light, dark |
| <kbd>⌘P</kbd> or <kbd>Ctrl+P</kbd> | export PDF |
| <kbd>O</kbd> | settings (also under ☰) |
| <kbd>⇧ Enter</kbd> | run the block, with the cursor in the code |
| <kbd>Esc</kbd> | leave the code or the terminal |

Shortcuts are ignored inside code, the terminal and text fields.

## Settings

The ☰ in the corner opens the panel with:

- **Font size.** Changes only the page content. Browser zoom still works and multiplies with it.
- **Theme.** Automatic (follows the system), light or dark.
- **Text width.** Narrow, more comfortable to read, or full page, for wide code.
- **Code colors.** Monokai dark (default), Monokai light, or follow the page theme.
- **Fonts.** Titles: Gloock (default), IBM Plex Mono, Fraunces, Cormorant Garamond, Young Serif. Text: JetBrains Mono (default), Fira Code, Inter, Open Sans. Code: JetBrains Mono (default) or Fira Code.
- **Language.** Portuguese or English.

Choices are saved in `~/.config/trunade/preferences.toml` and apply to every lesson, on any port. You can edit the file by hand.

## Run all

In ☰, **Run all** (or <kbd>R</kbd>) runs in order every block with `stdin` or `expected`, to check the lesson before class or before exporting the PDF. Interactive blocks are skipped, since they would wait for someone to type. A bar at the bottom shows the progress and, at the end, how many passed and which failed (compilation, runtime error, output other than expected); clicking a line takes you to the block, opening its step if it is inside one.

## Export PDF

In ☰, **Export PDF** (or <kbd>⌘P</kbd>) opens the browser print dialog; pick "Save as PDF". The file looks like the lesson on screen, with colors, edited code and the outputs already run, plus every step open and quiz answers checked. The suggested name is the `.md` name.

## The symbol in the corner

The symbol in the header shows whether a program is running: the rotor turns and the eyes light up while compiling or running, and it spins back once when it finishes.
