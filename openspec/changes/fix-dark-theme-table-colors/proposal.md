## Why

Markdown GFM tables, HTML `<table>` grids, and Visual Edit table controls in `src/app/preview.rs` used hardcoded light-theme colors (`0xe2e8f0`, `0xcbd5e1`, `0xf1f5f9`, `0xffffff`, and bright hover/selection washes). On dark themes (such as Ink, GitHub Dark, Solarized Dark, One Dark, and Tokyo Night), those light backgrounds and borders produced unreadable text contrast and broken table chrome (GitHub issue #27).

## What Changes

- Extend `ThemePalette` in `src/app/mod.rs` with semantic table tokens (`table_header_bg`, `table_body_bg`, `table_border`, `table_selected_bg`, `table_drop_target_bg`, `table_control_bg`, `table_control_hover_bg`, `table_control_text`, `table_control_hover_text`, and `destructive_text`) resolved from each built-in or custom theme's polarity and surface/accent colors with WCAG AA (>= 4.5:1) contrast invariants.
- Replace hardcoded light hex table colors in `src/app/preview.rs` across Markdown GFM tables, HTML tables, Visual Edit collapsible HTML source chrome, cell selection, drag-and-drop row/column indicators, row/column grips, edge add strips, and table grip context menus (including destructive actions).
- Keep light themes (`Paper`, `One Light`, `GitHub Light`, `Solarized Light`, `Tokyo Night Light`, etc.) legible and derive matching colors for full and sparse custom `.toml` themes without scattering `if dark` color literals through rendering code.
- **Non-goals:** changing Markdown or HTML table parsing, table column-width sizing, cell editing/selection semantics, or adding new user preference keys.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: refine `Native tables SHALL use the active document palette` to cover semantic table surface/border/interactive-control tokens, light/dark polarity preservation, custom themes, and live theme switching across Visual Edit, Read, and Split Preview.

## Impact

- `src/app/mod.rs`: semantic table palette fields and contrast-checked resolver helpers on `ThemePalette`.
- `src/app/preview.rs`: table, HTML table, collapsible HTML card, grip, edge strip, and table menu styling wired through `palette`.
- `src/app/tests.rs`: unit and GPUI tests across all 14 built-in themes, custom full/sparse `.toml` themes, live theme switching, and all three rendered modes (`VisualEdit`, `Read`, `Split`).
- Preserves all document-version, `Arc`-shared derived Markdown caches, memoized highlighting, text-handle caching, and undo invariants; theme switching affects presentation only.
