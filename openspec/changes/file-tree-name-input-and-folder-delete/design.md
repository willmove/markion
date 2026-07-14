## Context

The Files sidebar context menu (added by the in-flight `file-tree-context-menu-actions` change) surfaces create / rename / delete, but:

- `FileTree::delete` uses `fs::remove_dir` (empty-only), so deleting any folder shown in the tree fails.
- create/rename apply hard-coded names (`untitled.md`, `New Folder`, `renamed.<ext>`) with no user input.

GPUI has no native text-input element, and `window.prompt` is button-only (Ok/Cancel, no free text). The app already solves free-text entry with a **redirected text-input** pattern: a focused `Div` renders `Label: <buffer>`, and `has_text_input_focus()` guards IME routing so `replace_text_in_range` / `replace_and_mark_text_in_range` write into a backing `String` returned by `active_input_text_mut()` instead of the document. The search field and file-tree filter both use this. This change reuses that pattern for the name prompt and fixes `delete`.

## Goals / Non-Goals

**Goals:**

- Delete a folder (empty or not) recursively, with a second confirm for non-empty folders.
- Let the user type a name for Create File / Create Folder / Rename via an inline prompt; Enter commits, Escape cancels.
- Keep all new strings localized.

**Non-Goals:**

- In-place tree-cell editing (the prompt is a dedicated input line, not an editable row).
- Native OS free-text dialog.
- Any change to Markdown caches, syntax highlighting, text handles, or undo.

## Decisions

### 1. Reuse the redirected-text-input pattern for the name buffer

Add a `PendingNameInput` field on `MarkionApp`:

```rust
struct PendingNameInput {
    kind: PendingNameKind,      // CreateFile | CreateFolder | Rename
    parent: PathBuf,            // dir to create in (for create); target dir (for rename, = target.parent())
    target: Option<PathBuf>,    // the entry being renamed (None for create)
    buffer: String,
}
```

Integrate it into the existing input-routing trio exactly like `file_tree_query_focused`:

- `has_text_input_focus()` -> `self.pending_name_input.is_some() || file_tree_query_focused || search_focus.is_some()`.
- `active_input_text_mut()` -> if `pending_name_input.is_some()`, return `&mut pending_name_input.buffer`.
- `after_input_changed()` -> when a name is pending, set status to a "typing name" hint (no tree re-filter).

Rationale: the search/filter inputs already prove this works with IME (composition, marked text, backspace via `pop_text_input`). Reusing it avoids a new input subsystem and keeps the prompt themeable/localizable.

Alternative considered: a separate GPUI input element. Rejected - none exists in this codebase and the redirected-input path is already IME-correct.

### 2. Commit / cancel via dedicated actions, not the context-menu handler

Register two new GPUI actions: `ConfirmPendingName` (bound to `enter` when a name is pending) and reuse `escape` to cancel. The existing `escape` binding currently maps to `ClearFileTreeSearch`; route it through a single `cancel_pending_input_or_search` dispatcher: if a name prompt is open, cancel it; else if the filter is focused, clear the filter.

On commit (`ConfirmPendingName`):

1. Read `buffer`. If empty -> localized `StatusNameRequired` warning, keep prompt open.
2. Branch on `kind`:
   - `CreateFile` -> `file_tree.create_unique_file(parent, &buffer)`; on Ok set `selected_tree_path`, `StatusCreated`.
   - `CreateFolder` -> `file_tree.create_unique_directory(parent, &buffer)`; same.
   - `Rename` -> guard dirty active doc (existing `StatusSaveBeforeRename` check); `file_tree.rename_unique(target, &buffer)`; reload any open tab whose path matched the old target (existing logic).
3. Clear `pending_name_input`; refresh tree state; `cx.notify()`.

On cancel: clear `pending_name_input`, restore prior focus, `cx.notify()`.

Rationale: the context-menu action handler (`handle_file_tree_context_action`) currently does the work inline. For create/rename we now *defer* the work to commit time, so the menu handler's job becomes "open the prompt" and a separate commit action does the FS work. This matches how `ShowFind` opens the search panel and `enter`/`FindNext` act on it later.

Alternative considered: perform the FS op in the menu handler with a default name. Rejected - that is the current broken behavior.

### 3. Pre-fill rename, default-suggest create

- Rename: pre-fill `buffer` with the current entry's file name (so the user edits in place).
- Create File: pre-fill with `untitled.md` (the current default) so Enter-without-typing preserves today's behavior.
- Create Folder: pre-fill with `New Folder`.

This keeps the prompt a convenience, not a regression: the existing defaults are still one Enter away.

### 4. Recursive folder delete + second confirm

`FileTree::delete`:

```rust
pub fn delete(&mut self, path) -> io::Result<()> {
    ensure_existing_path_within_root(&self.root, path)?;
    if path.is_dir() {
        fs::remove_dir_all(path)?;   // was fs::remove_dir
    } else {
        fs::remove_file(path)?;
    }
    self.refresh()
}
```

`delete_tree_entry` flow:

1. Always show the first confirm (existing `DialogDeleteTitle` / `DialogDeleteDetail`).
2. If `path.is_dir()`, check non-emptiness (e.g. `fs::read_dir` count > 0) before the first confirm returns, and if non-empty, chain a *second* `window.prompt` (`PromptLevel::Warning`) with a recursive-delete title/detail (`DialogDeleteFolderRecursiveTitle` / `DialogDeleteFolderRecursiveDetail`). Both must be confirmed to proceed; either cancel aborts.
3. On confirm: `FileTree::delete`, reset tabs whose path was the deleted entry (existing logic).

Rationale: recursive delete is irreversible; the second confirm specifically calls out "folder and all contents". Files and empty folders keep the single confirm so the common path is not slowed down.

Alternative considered: always one confirm with dynamic wording. Rejected - the user explicitly chose a two-step confirm for non-empty folders.

### 5. Prompt rendering and dismissal

Render the prompt as an overlay anchored in the Files panel (near the top, like the search panel) when `pending_name_input.is_some()`, reusing `search_field_view` styling (`Label: <buffer>`, blue border when active). It is not a row in the tree, so bounded-row rendering is unaffected.

Dismissal mirrors the context menu: `close_menu` / click-outside / opening another menu / Escape cancels the prompt. `toggle_menu` and `show_file_tree_context_menu` already clear related transient state; extend them to clear `pending_name_input` too.

## Risks / Trade-offs

- **Redirected-input + IME**: the name buffer goes through the same `insert_redirected_text` / `input_marked_len` path as search/filter, so IME composition is handled identically. Risk is low (proven path).
- **Recursive delete is destructive**: mitigated by the second confirm. Tests will cover empty / non-empty / file.
- **Two OpenSpec changes touch the same context-menu spec**: this change's delta composes on top of `file-tree-context-menu-actions`'s delta to the same `### Requirement: File tree panel with filename filtering`. If both archive, the sync must merge scenarios; OpenSpec handles delta merge on archive, but implementers should archive `file-tree-context-menu-actions` first if ordering is ambiguous.
- **Enter/Escape binding overlap**: `escape` already binds to `ClearFileTreeSearch`. The new dispatcher (`cancel_pending_input_or_search`) handles both in priority order, so no keybinding conflict.

## Data flow / caching note

File-tree operations (create/rename/delete) operate on the filesystem and app-level tree/tab state only. They do NOT touch:

- per-document derived Markdown state (preview blocks / outline / stats cached per version via `Arc`),
- the syntax-highlighting memo,
- the cached text handle per version,
- undo snapshots (undo is per-document-text, untouched by file-tree renames; a renamed-and-reloaded tab gets a fresh undo stack, same as today).

The redirected-input extension adds one more buffer (`pending_name_input.buffer`) to the IME routing guard; it does not change the document-editing IME path. Bounded tree-row rendering is unchanged (the prompt is an overlay).
