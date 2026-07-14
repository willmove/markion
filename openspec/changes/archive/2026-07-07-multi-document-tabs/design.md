# Design: Multi-document tabs

## Context

Markion is single-document-per-window. `MarkionApp` (src/main.rs:359-440) mixes per-document state (`document`, `undo_stack`, `selected_range`, scroll handles, layout caches, …) with per-window state (menus, themes, sidebar, search panel). The refactor introduces an `EditorTab` struct for the per-document fields and changes `MarkionApp` to hold `tabs: Vec<EditorTab>` + `active_tab: usize`. Scope is session-only (no persistence) per the approved decision log.

This document records the structural decisions, the borrow-checker strategy, and the known simplifications. It is informed by a full read of `src/main.rs` (5898 lines) — field access counts, method boundaries, and the open/close/quit handler chain.

## Part 1 — Field split (confirmed)

**Move to `EditorTab`** (15 fields + 2 autosave):
`document`, `undo_stack`, `redo_stack`, `editor_scroll`, `preview_scroll`, `selected_range`, `selection_reversed`, `marked_range`, `last_lines`, `line_offsets`, `line_heights`, `last_bounds`, `line_height`, `is_selecting`, `display_text_cache`, plus `last_recovery_file` and `autosave_generation`.

**Stay on `MarkionApp`** (per-app):
`focus_handle`, `active_menu`, `status`, `confirming_close`, `allow_close`, `preferences_path`, `theme`/`custom_theme`/`custom_themes`/`themes_dir`/`selected_theme_name`, `preferences_panel_open`, `focus_mode`/`typewriter_mode`/`code_line_numbers`, `view_mode`, `workspace_root`, `editor_split_ratio`, `sidebar_width`, `file_tree`, `sidebar_visible`/`sidebar_tab`/`file_tree_query`/`file_tree_query_focused`/`input_marked_len`/`selected_tree_path`, the search-panel fields (`search_visible`/`replace_visible`/`search_query`/`replace_text`/`search_case_sensitive`/`search_regex`/`search_focus`/`search_matches`/`current_search_index`), `auto_save_preferences`/`export_preferences`, `recovery_dir`, `highlight_cache`, `language`.

### Why `last_recovery_file` and `autosave_generation` move per-tab
The autosave timer (`schedule_autosave`, line 739) captures `autosave_generation` and, when it fires, autosaves `self.document` and writes `self.last_recovery_file`. If both stay per-app, switching tabs while a timer is pending autosaves the **now-active** tab and may delete a recovery file belonging to a different tab. Moving them per-tab and having `schedule_autosave` operate on `self.active_tab_mut()` (capturing the active index at schedule time) keeps the autosave target unambiguous. See Part 4.

### Why `highlight_cache` stays per-app
It is keyed on `(Option<String>, String)` = `(language, code)` (line 549). Two tabs showing the same code block share the entry — that is a deliberate cache, not per-document state.

### Why `view_mode` stays per-app
It is a window layout toggle (Source/Split/Preview), used only in `render` and `toggle_view_mode`. Keeping it per-app is a deliberate simplification (the plan records per-tab view mode as a possible later enhancement).

### Why search-panel state stays per-app
The Find/Replace panel is a single shared UI. Keeping its state per-app means find query/matches are shared across tabs; switching tabs calls `refresh_search_matches()` against the active tab's document. The plan records per-tab search as a later enhancement.

## Part 2 — Borrow-checker strategy (the key decision)

~195 direct per-tab field accesses must change form. The naive rewrite (`self.active_tab().X` for reads, `self.active_tab_mut().X` for writes) hits the borrow checker wherever a method reads one per-tab field and mutates another in the same scope — e.g. `scroll_editor_to_offset` reads `self.document` and mutates `self.editor_scroll`; `replace_text_in_range` mutates `self.document`, `self.selected_range`, `self.marked_range`, and calls `self.push_undo_snapshot()` (which reads `self.document`/`self.undo_stack`).

**Strategy: migrate per-tab-only methods onto `EditorTab` verbatim, and have `MarkionApp` delegate through a single `let tab = self.active_tab_mut();` binding.**

- A method that touches **only** per-tab fields (e.g. `cursor_offset`, `range_from_utf16`, `push_undo_snapshot`, the movement methods `left`/`right`/`up`/`down`/`home`/`end`, `bounds_for_range`, `character_index_for_point`) becomes `impl EditorTab { fn ...(&self) / (&mut self) }`. Inside it, `self` *is* the tab, so all field accesses stay `self.document` etc. — no rewrite of the method body.
- `MarkionApp` keeps a thin delegator: `fn cursor_offset(&self) -> usize { self.active_tab().cursor_offset() }` — or, for the input handlers, obtains `let tab = self.active_tab_mut();` once and operates on `tab`.
- Methods that mix per-tab and per-app state (e.g. `after_document_changed`, which touches the tab's document AND calls `schedule_autosave` + `refresh_search_matches` on the app) stay on `MarkionApp` and sequence their borrows: read per-tab into locals first, drop the tab borrow, then mutate per-app.

This keeps the method-body churn minimal (only the delegator wrappers are new) and sidesteps the aliasing problem entirely. The cost is an `impl EditorTab` block of ~30 migrated methods — but they are verbatim moves, not rewrites.

## Part 3 — Open / close / quit handlers

### Unified open paths
The 5-line reset block (`selected_range=0..0; selection_reversed=false; marked_range=None;` + `document = <new>; push_undo_snapshot();`) is duplicated at lines 603, 723, 1093, 1136. Extract:
- `open_in_new_tab(document, cx)`: construct a fresh `EditorTab`, push it, set `active_tab` to the new index, `refresh_search_matches()`, `cx.notify()`.
- `replace_active_tab(document, cx)`: `discard_current_recovery_file()` on the active tab, replace its `document`, reset selection/scroll/undo on the tab, `refresh_search_matches()`, `cx.notify()`.

Mapping: file-tree click → `open_in_new_tab`; File→Open → `replace_active_tab` (behavior continuity); recovery restore → `open_in_new_tab`; New → `replace_active_tab` (or new tab — see Part 5 decision).

### `confirm_discard_then` rework
Current signature (`fn(&mut Self, &mut Context)`) can't carry a tab index, so:
- For **File→New / File→Open**, the guard checks the **active tab only** (matches single-doc behavior — the user is replacing what they're looking at). `on_confirm` operates on the active tab.
- For **window close / quit**, the guard iterates all tabs; if any is dirty, prompt once per dirty tab (or a single "N unsaved documents" prompt — see Part 5 decision). `request_quit` (2414) and `install_window_close_guard` (5507) currently check `self.document.is_dirty()`; they become `self.tabs.iter().any(|t| t.document.is_dirty())`.

### Close tab
New `CloseTab` action: if the active tab is dirty, prompt; on confirm, remove the tab. If it was the last tab, push a fresh untitled `EditorTab` (window stays open). The closed tab's recovery file is discarded.

## Part 4 — Autosave + recovery per-tab

`schedule_autosave` captures `autosave_generation` (now per-tab) and the active tab index at schedule time. When the timer fires, it validates the index still exists and the generation matches, then operates on `tabs[index]`. `last_recovery_file` is read/written on the same tab.

`check_recovery_on_startup` (567) restores into a new tab via `open_in_new_tab(MarkdownDocument::recovered(...), cx)` instead of replacing the single document.

## Part 5 — Decisions to make during implementation

- **New document: new tab or replace active?** Default plan: replace active (matches File→New in single-doc editors). Alternative: always new tab. Recommend replace-active for continuity; the user explicitly opens new tabs via the tree or OpenInNewTab.
- **Window-close with N unsaved tabs: one prompt per tab, or one汇总 prompt?** Recommend汇总 ("You have N unsaved documents — Discard all / Cancel") for usability; per-tab if汇总 proves awkward. Decision deferable to implementation.
- **Ctrl+Tab cycle order:** by opening order (the `tabs` Vec order). No reordering on focus (VSCode-style "recently used" is a later enhancement).

## Part 6 — EditorElement / EntityInputHandler

Both already go through `Entity<MarkionApp>`, so the refactor is mechanical substitution:
- `EditorElement::prepaint`: `let app = self.app.read(cx); let tab = app.active_tab();` then `tab.document.text()`, `tab.marked_range`, `tab.selected_range`, `tab.cursor_offset()`. Per-app reads (`app.focus_mode`, `app.theme`) stay on `app`.
- `EditorElement::paint`: `self.app.update(cx, |app, _| { let tab = app.active_tab_mut(); tab.last_lines = ...; ... })`.
- `EntityInputHandler` methods: obtain `let tab = self.active_tab_mut();` (for mutating methods) or `let tab = self.active_tab();` (for read methods) and operate on `tab`. `unmark_text` clears both `tab.marked_range` and `self.input_marked_len` (per-app) — sequence as two writes.

## Part 7 — Tab bar render

Insert `tab_bar_view(self, cx)` between the menu bar (ends ~3711) and the search panel (~3712). Shown only when `tabs.len() > 1`. Each tab `div()` shows `title_from_path(tab.document.path())` + `*` if dirty; the active tab is highlighted (background/border). An `×` button on each tab triggers `CloseTab`. Styling follows the existing `menu_title_button` / `toolbar_button` `div()` patterns (GPUI 0.2.2 has no native TabBar). The status bar's title/dirty marker (3561-3576) is computed from `self.active_tab().document`.

## Part 8 — Verification

- New tests: open 3 files → switch → assert per-tab cursor/scroll/undo isolation; close an unsaved tab → confirmation; close the last tab → fresh untitled document appears; recovery restore opens a new tab; autosave targets the active tab after a switch.
- `cargo test --workspace` green.
- Single-tab case (the common path) is visually identical: tab bar hidden, behavior matches pre-refactor.
