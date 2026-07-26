## Context

Today every menu shortcut is a compile-time constant: `menu_shortcuts` in `src/app/mod.rs` holds `MenuShortcut { binding, windows_linux, macos }` triples, `src/app/bootstrap.rs` binds them once via `cx.bind_keys([...])` at startup (alongside fixed core-editing and file-tree keys), and `src/i18n.rs` duplicates the same key combos as display strings inside `shortcut_catalog`. The Help -> Keyboard Shortcuts item opens a read-only modal (`shortcut_panel_view`). Menu dropdown rows render the curated `windows_linux` / `macos` label next to each item.

Constraints:
- GPUI keystroke strings use the `secondary` modifier alias (Cmd on macOS, Ctrl elsewhere); overrides must be stored in the same vocabulary so one value works per platform rendering.
- GPUI exposes `App::clear_key_bindings()` + `bind_keys()`, so a full rebind at runtime is possible as long as the entire binding set (core editing keys, file-tree keys, menu shortcuts) is rebuilt by one function.
- The unarchived `add-tabbed-shortcut-panel` change introduced the current modal; this change supersedes its presentation (modal -> Preferences tab) while keeping its structured catalog model.
- Project invariant: derived Markdown state caching is untouched; this change never re-parses documents.

## Goals / Non-Goals

Goals:
- One registry of customizable actions with stable ids, default GPUI bindings, and curated default display labels.
- Effective binding = valid override else default, used by the keymap, menu labels, and the reference list alike.
- Preferences panel gains a Shortcuts tab with the existing platform tabs + category sidebar; each row is editable via key capture, with conflict rejection and per-action reset.
- Overrides persist in `config.toml` as a `[shortcuts]` table and are cleared by the global preferences reset.

Non-goals:
- Remapping core editing keys (arrows, backspace/delete, home/end, enter, tab/shift-tab, shift-selection), file-tree keys (F2/F5/secondary-alt-*), or unbinding actions.
- Multiple bindings per action, chorded sequences, search/filter in the tab.
- Changing native OS menu key equivalents beyond what GPUI derives from the keymap.

## Decisions

### 1. Single shortcut registry keyed by stable action id
Introduce a registry (in `src/app/mod.rs`, replacing `menu_shortcuts`) where each entry is `{ id: &'static str, default_binding: &'static str, label_win: &'static str, label_mac: &'static str }`. Ids are kebab-case (`"new-document"`, `"toggle-sidebar"`, ...) and are the `config.toml` keys. `src/i18n.rs` `ShortcutAction` gains the same `id` so the catalog, menus, and settings rows all resolve effective bindings through one lookup.
- Alternative considered: keep `menu_shortcuts` consts and bolt on an override map keyed by `Msg` variant — rejected: `Msg` variants are presentation keys, not stable config identity, and would couple config.toml to localization refactors.

### 2. Overrides in `config.toml` `[shortcuts]` table
`AppPreferences` gains `shortcut_overrides: BTreeMap<String, String>` (id -> GPUI keystroke string); `PreferencesFile` mirrors it as `shortcuts`, omitted when empty. On load, entries with unknown ids or keystroke strings that fail GPUI parsing are dropped with a log line and the default applies. Reset-preferences writes a default `AppPreferences`, which clears overrides for free.
- Alternative: separate `keymap.toml` — rejected: the repo convention is a single TOML config with optional tables (`[auto_save]`, `[export]`).

### 3. Display labels: curated for defaults, derived for overrides
Default bindings keep their curated labels (existing tests assert them). Overridden bindings are formatted by a keystroke-string formatter (`secondary-shift-s` -> `Ctrl+Shift+S` / `Cmd+Shift+S`, function keys and named keys like `enter`/`tab`/`comma` handled). One formatter serves both menu labels and the reference list, for both platform previews.
- Alternative: derive labels for defaults too — rejected: risk of regressing pixel-checked labels (e.g. `Ctrl+,`).

### 4. Live rebinding via full rebuild
Extract bootstrap's entire `bind_keys([...])` into `bind_app_keys(cx, &overrides)` which binds (a) the fixed core-editing set, (b) the fixed file-tree set, (c) every registry action at its effective binding. On any shortcut change: `clear_key_bindings()` then `bind_app_keys(...)`. Startup uses the same function with loaded preferences, so there is exactly one binding code path.
- Risk: `clear_key_bindings` wipes something GPUI-internal — mitigated by rebinding the complete set the app needs and by manual verification of typing/navigation keys after a remap.

### 5. Capture-based editing with strict validation
Clicking a binding chip puts the row into capture mode (panel focus handle + `on_key_down`): the next key press becomes the candidate. Validation rejects: unparseable keystrokes, bare printable keys without modifiers (except function keys F1-F12), the fixed reserved core bindings, and any binding equal to another action's effective binding — with a localized inline message naming the conflicting action. Esc cancels. Accepting writes the override, persists `config.toml`, rebinds, and updates menu labels in the same frame.
- Alternative: free-text input of keystroke strings — rejected: error-prone and unfriendly; capture matches user expectation from other editors.

### 6. Preferences panel becomes tabbed
`preferences_panel_view` gains a tab strip (General / Shortcuts). The Shortcuts tab reuses the modal's platform tab + category sidebar + scrollable action list layout, widened to the former modal's 720px shell; the General tab keeps today's content. The standalone modal, its `shortcut_panel_open` state, and the Help -> Keyboard Shortcuts items (in-window and native) are removed; Help keeps About. `ShowShortcuts` (F1) opens the Preferences panel directly on the Shortcuts tab. Shortcut panel selection state (platform/category) is retained as Preferences-tab state.

## Risks / Trade-offs

- [Captured keystrokes serialize as physical modifier names (`ctrl`, `cmd`), not `secondary`, so a config written on macOS may not map 1:1 on Windows] → Documented behavior; overrides are per-machine config. The formatter renders whatever is stored correctly per platform.
- [User remaps a shortcut onto an OS-reserved combo that GPUI never delivers] → Out of app control; the reference list still shows it, and reset restores defaults.
- [Archive-order clash with unarchived `add-tabbed-shortcut-panel`, whose delta adds the modal requirement this change removes] → This change supersedes that presentation; archive `add-tabbed-shortcut-panel` first (it is functionally complete) so this delta applies cleanly, or reconcile at archive time.
- [Conflict detection misses fixed bindings (file-tree F2/F5, core keys)] → Validation checks a reserved set that includes every fixed binding string, not just registry actions.

## Migration Plan

No user migration: missing `[shortcuts]` means all defaults, which matches current behavior. Rollback = delete the `[shortcuts]` table.

## Open Questions

- None blocking; the exact reserved-key list is finalized during implementation from the bootstrap binding set.
