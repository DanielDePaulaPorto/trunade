# Rascunho: pedido ao Zed (GitHub Discussions)

Não publicado. Categoria sugerida: Feature Requests.

---

**Title:** Provide task variables (`ZED_FILE`, `ZED_ROW`, …) when the Markdown preview is focused

**Problem**

When a Markdown preview is the active item, spawned tasks receive no file context.
A task such as

```json
{ "label": "probe", "command": "echo", "args": ["FILE=${ZED_FILE:none}", "ROW=${ZED_ROW:none}"] }
```

prints `FILE=none ROW=none` when run from the preview, while the same task run from
the source editor prints the path and row.

**Use case**

I teach programming from Markdown files shown in the preview. I'd like to click a
fenced code block in the preview and press a key bound to `task::Spawn` that
compiles and runs that block in the terminal. The preview already moves the source
editor's cursor on a single click (`on_source_click` →
`change_selection_to_source_index`), so the row is known; it just doesn't reach
the task context.

**Why it happens (as far as I can tell)**

- `MarkdownPreviewView::act_as_type` already returns the source `Editor`.
- But `tasks_ui::task_contexts` only uses that editor when the active item has a
  project path (`active_project_editor` depends on `active_item_worktree`), and
  `MarkdownPreviewView` does not implement `for_each_project_item`, so
  `active_project_path` is `None`.

**Possible fix**

Implement `for_each_project_item` (or `active_project_path`) on
`MarkdownPreviewView` by delegating to the previewed editor's buffer. Then the
existing code path would compute `ZED_FILE` and `ZED_ROW` from the source editor's
selection, which already follows clicks in the preview.

Zed version: <preencher com `zed: about`>
