# Design — Tab bar context menu

## Context

The app has two structurally identical right-click menus: `FileTreeContextMenu`
(state `src/app/mod.rs:610`, show `src/app/workspace.rs:474`, render
`src/app/root_view.rs:2626`) and `PreviewContextMenu` (`src/app/mod.rs:1210`,
render `src/app/root_view.rs:2691`). This change adds a third, `TabContextMenu`,
following that pattern exactly: a small state struct on `MarkionApp`, an action
enum, a label→`Msg` mapping, an `anchored()` + `occlude()` popup view, and
clearing of all other menus on open.

## Decisions

### D1 — Target semantics: switch-then-operate (user decision)

Menu actions act on the **right-clicked** tab, implemented by calling
`switch_active_tab(index)` before running the action — the same idiom the tab
`×` button already uses (`src/app/editing.rs:2174`). We do NOT refactor to
`close_tab_at(index)`; all existing active-tab-oriented handlers
(`close_tab`, `confirm_discard_then`, rename dirty guard) stay untouched.

Staleness: the `×` listener guards with `index < app.tabs.len()`. The menu can
stay open longer, so the stored target is re-resolved at dispatch time: keep the
index but validate it against the tab's identity captured when the menu opened
(`path()` for file-backed tabs, plus title for untitled tabs). If resolution
fails, drop the menu with a cancel status instead of operating on the wrong tab.

### D2 — Batch close policy: dirty tabs kept + summary dialog (user decision)

`Close Others` / `Close to the Right`:

1. Close every in-scope **clean** tab immediately (silently, via the
   `close_tab_confirmed` cleanup path — recovery files only exist for dirty
   tabs, but keep the shared removal helper so image-claim release and session
   persistence run identically).
2. If any in-scope **dirty** tabs remain, show one summary dialog:
   `PromptLevel::Warning`, message "N tab(s) have unsaved changes", with two
   buttons — **Discard all** (closes the kept dirty tabs, discarding recovery
   snapshots like `request_quit` does) and **Cancel** (keeps them open).
   GPUI prompts are not re-entrant; this is a single prompt, matching the
   `request_quit` precedent (`src/app/editing.rs:1163`), not sequential
   per-tab prompts.
3. Closing the last remaining tab leaves a fresh untitled document (existing
   `close_tab_confirmed` rule) — the batch helper reuses that guarantee.
4. The clicked tab itself is never in scope.

### D3 — Rename: reuse the file-tree pipeline verbatim

The inline rename prompt is NOT a tree row — `pending_name_prompt_view`
(`src/app/root_view.rs:1612`) is a floating line rendered at the top of the
file-tree panel whenever `pending_name_input` is set. `rename_unique`
(`src/storage/file_tree.rs:162`) does a direct `fs::rename` on any path with a
parent; it does not require the target to be listed in the tree, and the commit
path already re-points all open tabs to the renamed file
(`src/app/workspace.rs:780-825`).

Tab-level Rename therefore reuses `PendingNameKind::Rename` unchanged. One gap:
if the file-tree panel is hidden (sidebar closed), the prompt line would be
invisible while keystrokes are still redirected to it. Fix: render
`pending_name_prompt_view` in the tab bar row as well when
`pending_name_input` is open, so the prompt is always reachable. The view is
cheap and stateless; rendering it in two places is mutually exclusive in
practice (prompt is a singleton).

Dirty guard: reuse the existing "save before rename" rule. The current check is
written against the active tab's path (`src/app/workspace.rs:788`); with D1's
switch-then-operate the target IS the active tab at dispatch time, so the check
works unchanged.

Untitled tabs: Rename is disabled (gray, no dispatch). Mapping it to Save As
was considered and dropped — Save As is already one menu away and conflating
the two hides a destructive distinction.

### D4 — Menu contents and rendering

```
Close Tab
Close Others
Close to the Right
─────────────────        ← first separator in an app context menu
Rename…
Copy File Path            (disabled for untitled)
Reveal in File Manager    (disabled for untitled)
```

- Item enable/disable reuses the `PreviewContextMenu` pattern (enabled flag →
  muted text, no hover, no handler).
- Separator support: add a thin hairline variant to the menu item list (the
  action enum grows a `Separator`-like structural entry, or the view builder
  takes grouped slices; prefer grouped slices to keep the action enum pure).
- No shortcut hints in v1 (neither existing context menu shows them).
- Middle-click close: add `on_mouse_up(MouseButton::Middle, …)` to the tab
  element = switch + `close_tab`, identical to `×`.
- Copy path writes via `cx.write_to_clipboard(ClipboardItem::new_string)` with
  a status-bar confirmation message (`StatusCopiedPath`).

### D5 — i18n

New `Msg` entries following the `ItemTab*` / `Dialog*` / `Status*` conventions,
English + Simplified Chinese: six item labels, the summary dialog
title/detail/buttons, and statuses (copied path, N tabs kept). All through
`src/i18n.rs`; no user-facing literals in views.

## Non-goals (reaffirmed)

Pin, drag-reorder, split panes, reopen-closed history, shortcut hints.

## Testing strategy

Unit/integration tests in `src/app/tests.rs` mirroring the existing
file-tree/preview context-menu test style (right-click event constructors
around `tests.rs:7530-8108`):

- Menu opens on right-click; dispatch performs switch-then-operate (assert
  active tab + tab vector).
- Stale-target: tab vector mutates while menu open → dispatch is a no-op
  cancel.
- Close Others with 0/1/2 dirty tabs: silent close, kept-and-reported,
  discard-all path.
- Untitled tab: file-backed items disabled (no dispatch).
- Rename: prompt opens with prefilled name; dirty tab refuses; renamed file
  re-points tabs (may reuse existing rename tests' fixtures).
- Copy path / reveal dispatch (reveal mocked or asserted at the action
  boundary).
