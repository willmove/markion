## Why

Markion is single-document-per-window: `MarkionApp.document` is one `MarkdownDocument`, and opening a new file discards (after a dirty-guard) the current one. For an editor meant for daily note-taking and multi-file projects, this forces users to launch multiple windows or lose their place. Tabs are the canonical fix — the file-tree sidebar already lists many files, but clicking a second one replaces the first.

The approved scope (see `docs/typune-integration-plan-glm-zcode.md` decision log) is session-only tabs with no persistence: opening multiple `.md` files into switchable tabs within one window, each with isolated cursor/scroll/undo state, and per-tab dirty tracking. Restart returns to a single document. This avoids entangling with the crash-recovery flow and the preferences schema in this change.

## What Changes

- **New `EditorTab` struct** (`src/main.rs`) holding the 15 per-document fields currently on `MarkionApp` (`document`, `undo_stack`, `redo_stack`, `editor_scroll`, `preview_scroll`, `selected_range`, `selection_reversed`, `marked_range`, `last_lines`, `line_offsets`, `line_heights`, `last_bounds`, `line_height`, `is_selecting`, `display_text_cache`) plus `last_recovery_file` and a per-tab `autosave_generation` (see design.md — these must move per-tab to keep autosave correct across tab switches).
- **`MarkionApp` refactor**: the 15 fields above leave `MarkionApp`, replaced by `tabs: Vec<EditorTab>` + `active_tab: usize`. Accessor methods `active_tab()` / `active_tab_mut()`. `highlight_cache` stays PER-APP (cross-tab, keyed on content); `view_mode`, search panel state, sidebar/file-tree state, theme/prefs, menus, status all stay PER-APP.
- **Methods that touch only per-tab fields migrate onto `EditorTab`** (the borrow-checker-safe strategy — see design.md): cursor movement, offset helpers, layout-cache reads, `cursor_offset`, `push_undo_snapshot`, `replace_text_in_range` body, etc. `MarkionApp` delegates via `self.active_tab_mut()`. Methods mixing per-tab and per-app state (e.g. `after_document_changed` which calls `schedule_autosave` + `refresh_search_matches`) stay on `MarkionApp` and sequence their borrows.
- **Unified open paths**: `open_in_new_tab(document, cx)` and `replace_active_tab(document, cx)` extract the 5-line reset block duplicated at lines 603/723/1093/1136.
- **New actions**: `OpenInNewTab`, `CloseTab`, `NextTab` (Ctrl+Tab), `PrevTab` (Ctrl+Shift+Tab). File→Open stays replace-current (behavior continuity); file-tree click opens in a new tab; "Open in New Tab" menu item/shortcut for explicit new-tab open.
- **`confirm_discard_then` rework**: the `fn(&mut Self, &mut Context)` callback can't carry a tab index, so the dirty-guard is restricted to the **active tab** for new/open (matching single-doc behavior), and the window-close / quit guards iterate all tabs ("any tab dirty" → prompt). Closing a tab with unsaved changes prompts per-tab.
- **Tab bar render**: a `tab_bar_view` inserted between the menu bar and the search panel (line ~3712), shown only when `tabs.len() > 1`. Each tab is a `div()` with the file name + `*` dirty marker + an X close button (GPUI 0.2.2 has no TabBar element; custom `div()` following the existing `menu_title_button` style).
- **`EditorElement` / `EntityInputHandler`**: mechanical — they already go through `Entity<MarkionApp>`, so only `app.<field>` → `app.active_tab().<field>` / `app.active_tab_mut().<field>` substitutions.
- **Closing the last tab** creates a fresh untitled document rather than closing the window.

## Capabilities

### Modified Capabilities
- `markdown-editing`: the editor holds multiple documents in tabs within one window; each tab has isolated cursor, selection, scroll, undo history, and dirty state. File→Open replaces the active tab; file-tree open and "Open in New Tab" open a new tab.

## Impact

- Edited: `src/main.rs` (large — ~195 per-tab field accesses rewritten, ~30 methods migrated onto `EditorTab`, new `EditorTab` struct + accessors, 4 new actions + handlers, tab bar render, `confirm_discard_then` / quit / close-guard rework, autosave/recovery per-tab).
- New tests: open 3 files → switch → per-tab cursor/scroll/undo isolation; close-unsaved-tab confirmation; close-last-tab → fresh document; recovery-restore opens a new tab.
- `cargo test --workspace` must stay green; single-tab behavior (the common case) must be visually identical (tab bar hidden when `tabs.len() == 1`).
- Risk: LARGE. The borrow-checker strategy (migrate per-tab-only methods onto `EditorTab`, operate via a single `active_tab_mut()` binding) is the key mitigation. See `design.md`.
