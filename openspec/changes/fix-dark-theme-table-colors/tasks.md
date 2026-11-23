## 1. Semantic Table Palette Tokens

- [x] 1.1 Extend `ThemePalette` in `src/app/mod.rs` with semantic table surface, border, selection, drop-target, control, and destructive-text tokens derived with contrast and polarity guarantees for all built-in and custom `.toml` themes.

## 2. Table & Control Rendering Integration

- [x] 2.1 Replace hardcoded light hex colors in `src/app/preview.rs` for GFM tables (`PreviewBlock::Table`, `visual_table_view`), HTML tables (`html_table_grid_view`), Visual HTML collapsible editors, cell selections, drag-drop indicators, row/column grips, edge add strips, and table grip menus.

- [x] 2.2 Share the active theme's source chrome across HTML, math, front matter, and diagram editors.
- [x] 2.3 Use common control token names for colors shared with source chrome and block menus.

## 3. Review Follow-up

- [x] 3.1 Cache palette resolution and replace weight searches with fixed rules.
- [x] 3.2 Verify the original palette invariants and regenerate the built-in theme color table for the PR.

## 4. Regression Verification

- [x] 4.1 Add unit and GPUI tests in `src/app/tests.rs` verifying contrast and theme polarity across all 14 built-in themes (`Ink`, `GitHub Dark`, `Solarized Dark`, `One Dark`, `Tokyo Night`, light themes), custom full/sparse `.toml` themes, live theme switching, and `VisualEdit` / `Read` / `Split` modes.
- [x] 4.2 Run formatting, workspace lints and tests, and strict OpenSpec validation.

## Verification Record

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --locked` passed.
- `cargo test --workspace --locked` passed with 2,268 tests passed and 5 ignored.
- `openspec validate --all --strict --no-interactive` passed for all 97 items.
