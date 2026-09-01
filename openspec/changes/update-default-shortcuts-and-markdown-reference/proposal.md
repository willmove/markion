## Why

Source, split, and Read mode currently jump via `Ctrl+Alt+1/2/3`, New Tab has no keyboard shortcut, and Help has no in-app Markdown syntax reference. Users coming from other Markdown editors expect faster mode switching, an explicit new-tab chord, and F1 to open syntax help rather than the Preferences Shortcuts tab.

## What Changes

- Change the factory-default shortcut for Edit (source) mode to `Ctrl+/` (`Cmd+/` on macOS).
- Change the factory-default shortcut for Split Preview mode to `Ctrl+P` (`Cmd+P` on macOS).
- Give File → New Tab a factory-default shortcut of `Ctrl+Shift+N` (`Cmd+Shift+N` on macOS).
- Change the factory-default shortcut for Read mode to `Ctrl+Shift+R` (`Cmd+Shift+R` on macOS).
- Add a Help → Markdown Reference item that opens an in-app, localized syntax cheat sheet, with factory-default shortcut `F1`.
- **BREAKING** (defaults only): Edit / Split / Read no longer default to `Ctrl+Alt+1/2/3`, and `F1` no longer opens Preferences → Shortcuts. Stored `[shortcuts]` overrides continue to win over the new factory defaults. Visual Edit (`Ctrl+Alt+4`), view-mode cycling (`Ctrl+Shift+V`), and Open in New Tab (`Ctrl+T`) stay unchanged.
- Non-goals: user-configurable shortcut schema changes; changing Visual Edit or cycle-mode bindings; opening Markdown Reference as a document tab; fetching remote help; changing Markdown parsing or per-document derived-state caches.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Pin the factory-default keystrokes for source, Split Preview, and Read mode, and add a factory-default New Tab shortcut, all through the existing customizable-shortcut registry.
- `chrome-platform`: Add Help → Markdown Reference (in-window and native menus) with `F1`, and move the `ShowShortcuts` factory default off `F1` so Preferences → Shortcuts remains reachable from the Preferences panel without occupying Help's help key.
- `ui-i18n`: Localize the new Help item, reference overlay chrome, and updated shortcut-catalog rows in every supported interface language.

## Impact

- Affected code: shared shortcut registry in `src/app/mod.rs`; keymap install in `src/app/bootstrap.rs`; File / View / Help menus in `src/app/root_view.rs` and `src/app/bootstrap.rs`; Help overlay and `ShowShortcuts` handling in `src/app/search.rs` / `src/app/root_view.rs`; localized catalog and Help strings in `src/i18n.rs`; registry, menu, catalog, and dispatch tests in `src/app/tests.rs`.
- Affected docs: `docs/keyboard-shortcuts.md` and `docs/faq.md` (and README mentions of `Ctrl+Alt+1/2/3` / `F1` if present) so published bindings match the new defaults.
- Configuration: new stable action ids `new-tab` and `show-markdown-reference` become valid `[shortcuts]` keys. Existing override tables need no schema migration. Users with no overrides receive the new defaults on next launch.
- Architecture: no change to document-version caches, syntax-highlight memoization, or cached text handles. Markdown Reference is an application overlay, not an editable document, so it MUST NOT create a tab or invalidate derived Markdown state.
