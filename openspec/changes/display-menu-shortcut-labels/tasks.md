## 1. Shared Shortcut Metadata

- [x] 1.1 Add structured metadata for every menu-visible keyboard shortcut, including GPUI binding strings, Windows/Linux labels, macOS labels, and any active aliases.
- [x] 1.2 Update keybinding installation to consume the shared metadata without changing the existing bindings or action scopes.
- [x] 1.3 Add unit tests for current-platform label selection, modifier naming, aliases, and representative binding/display pairs.

## 2. In-Window Menu Rendering

- [x] 2.1 Extend the themed menu-row component to render an optional, muted, right-aligned shortcut column while preserving its full-row click target and hover state.
- [x] 2.2 Wire every File, Edit, View, Format, Export, and Help action row to its shared shortcut metadata, using no shortcut value for unbound actions.
- [x] 2.3 Preserve configured H1–H5/H1–H6 menu construction and attach shortcut labels only to heading rows that are actually visible.
- [x] 2.4 Adjust per-menu dropdown widths and spacing so the longest localized action/shortcut rows remain readable without changing language-specific menu positions.

## 3. Regression Coverage and Verification

- [x] 3.1 Add menu construction tests covering representative bound and unbound items, all six dropdowns, multiple bindings, and conditional H6 visibility.
- [x] 3.2 Run `cargo fmt --check` and `cargo test`.
- [x] 3.3 Run `openspec validate display-menu-shortcut-labels` and resolve every validation error.
