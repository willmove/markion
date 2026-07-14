# Implementation Plan: Multi-document tabs

## Overview

Session-only tabs: open multiple `.md` files in switchable tabs within one window, each with isolated cursor/scroll/undo/dirty state. No persistence. See `design.md` for the borrow-checker strategy (migrate per-tab-only methods onto `EditorTab`) and field-split rationale.

## Tasks

- [x] 1. `EditorTab` struct + accessors
  - [x] 1.1 Define `EditorTab` with the 15 per-document fields + `last_recovery_file` + `autosave_generation`.
  - [x] 1.2 `MarkionApp`: remove the 17 fields, add `tabs: Vec<EditorTab>` + `active_tab: usize`; `active_tab()` / `active_tab_mut()` accessors; constructor builds a single initial tab.
  - [x] 1.3 Migrate per-tab-only methods onto `EditorTab` verbatim (movement, offset helpers, `cursor_offset`, `push_undo_snapshot`, `bounds_for_range`, `character_index_for_point`, layout-cache reads). `MarkionApp` delegators where the method is called from per-app contexts.

- [x] 2. Unified open paths + actions
  - [x] 2.1 Extract `open_in_new_tab(document, cx)` and `replace_active_tab(document, cx)`; rewire `new_document_confirmed`, `open_document_confirmed`, `open_tree_file_confirmed`, `check_recovery_on_startup`.
  - [x] 2.2 Add `OpenInNewTab`, `CloseTab`, `NextTab`, `PrevTab` actions + handlers; bind Ctrl+Tab / Ctrl+Shift+Tab (+ Secondary-T open-in-new-tab, Secondary-W close).
  - [x] 2.3 File→Open = replace active; file-tree click = new tab; OpenInNewTab menu item/shortcut.

- [x] 3. Dirty-guard rework
  - [x] 3.1 `confirm_discard_then`: restrict to active tab for New/Open (callback operates on active tab).
  - [x] 3.2 `request_quit` + `install_window_close_guard`: iterate all tabs ("any dirty" → prompt汇总).
  - [x] 3.3 `CloseTab`: dirty-tab confirmation; last-tab-closed → fresh untitled `EditorTab`.

- [x] 4. Autosave + recovery per-tab
  - [x] 4.1 `schedule_autosave`: capture active-tab index + per-tab `autosave_generation`; operate on `tabs[index]` on fire.
  - [x] 4.2 `check_recovery_on_startup`: restore via `open_in_new_tab`.

- [x] 5. Tab bar render + EditorElement
  - [x] 5.1 `tab_bar_view` between menu bar and search panel; shown only when `tabs.len() > 1`; per-tab title + dirty marker + × close.
  - [x] 5.2 `EditorElement` prepair/paint/request_layout: `app.<field>` → `app.active_tab().<field>` / `app.active_tab_mut().<field>`.
  - [x] 5.3 `EntityInputHandler` methods: operate on active tab; `unmark_text` clears per-tab `marked_range` + per-app `input_marked_len`.
  - [x] 5.4 Status bar title/dirty marker from `self.active_tab().document`.

- [x] 6. Tests + verification
  - [x] 6.1 `multiple_tabs_isolate_cursor_and_undo` — two tabs keep isolated cursor/undo (logic-level; full GPUI View isolation deferred — no test harness for `Context`).
  - [x] 6.2 `tab_vec_close_last_leaves_one_tab` — close-last-tab → fresh document; close-one-of-two → leaves the other. `editor_tab_new_initializes_empty_state` + `any_tab_dirty_detection`.
  - [x] 6.3 Recovery restore opens a new tab (via `open_in_new_tab`); autosave targets active tab post-switch (validated by index+generation guard in `schedule_autosave`). Not separately unit-tested (requires GPUI timers).
  - [x] 6.4 `cargo test -p markion` green (111 lib + 10 main); single-tab visual identical (tab bar hidden when `tabs.len() == 1`).
  - [x] 6.5 `openspec validate multi-document-tabs` passes.
