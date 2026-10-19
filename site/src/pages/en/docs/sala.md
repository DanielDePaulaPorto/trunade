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
| <kbd>R</kbd> | run all: every block, in order |
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

In ☰ › Run and export, **Run all** (or <kbd>R</kbd>) runs every block in order, to check the lesson before class or before exporting. Blocks without `stdin` run once with empty input and a time limit: one that reads nothing runs as usual, and one that would wait for someone to type ends instead of blocking the queue. A bar at the bottom shows the progress and, at the end, how many passed and which failed (compilation, runtime error, output other than expected); clicking a line takes you to the block, opening its step if it is inside one.

## Export PDF

In ☰, **Export PDF** (or <kbd>⌘P</kbd>) opens the browser print dialog; pick "Save as PDF". The file looks like the lesson on screen, with colors, edited code and the outputs already run, plus every step open and quiz answers checked. The suggested name is the `.md` name.

## Export HTML

In ☰ › Run and export, **Export HTML** writes a single `.html` file next to the `.md` (with a `-1`, `-2`… suffix if the name is taken), without asking anything. It opens in any browser, without Trunade! and offline: the page, the fonts in use, formulas, diagrams and images travel inside the file (about 2 MB).

- ▶ replays the last run of each block, with colors and verdicts, as if it ran. A block that was not run has no output and says so when clicked.
- The file's theme is **light** by default; dark is an option next to the button.
- With **Run every block first** (checked by default), every block runs before writing, so none is left without output. Unchecked, the last run of each block goes in, and a block never run has no output.
- The file stays as it was on screen, like a PDF: same theme, fonts and width, code with your edits, no settings panel and no editing. Navigation (<kbd>T</kbd>, <kbd>Space</kbd>, arrows), <kbd>F</kbd>, <kbd>Z</kbd> and <kbd>R</kbd> still work.
- Quiz answers stay hidden until clicked, as in class.

## Copy for Moodle

In ☰ › Run and export, **Copy for Moodle** puts the whole lesson on the clipboard as plain HTML, to paste into a Moodle **Page** (in the normal editor or in source view). Moodle removes scripts and stylesheets, so this copy is made to survive that:

- no effects, no scripts and no custom fonts: every style is written on each element;
- code goes whole (template included, if any), colored in Monokai, ready for students to select and copy;
- input, expected output and the output on screen go below each block: run everything first (<kbd>R</kbd>);
- formulas go back to `\( … \)` and `\[ … \]`, for Moodle's MathJax filter to draw;
- the lesson's SVG figures, diagrams and images go as embedded images, in the light theme's colors;
- questions become a list with a **Show answer** that opens without JavaScript; steps become sections in sequence; `data-reveal` shows everything.

Large GIFs make the copy heavy, as in the exported HTML.

## The symbol in the corner

The symbol in the header shows whether a program is running: the rotor turns and the eyes light up while compiling or running, and it spins back once when it finishes.
