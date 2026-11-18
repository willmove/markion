## 1. Semantic Table Palette Tokens

- [x] 1.1 Extend `ThemePalette` in `src/app/mod.rs` with semantic table surface, border, selection, drop-target, control, and destructive-text tokens derived with contrast and polarity guarantees for all built-in and custom `.toml` themes.

## 2. Table & Control Rendering Integration

- [x] 2.1 Replace hardcoded light hex colors in `src/app/preview.rs` for GFM tables (`PreviewBlock::Table`, `visual_table_view`), HTML tables (`html_table_grid_view`), Visual HTML collapsible editors, cell selections, drag-drop indicators, row/column grips, edge add strips, and table grip menus.

## 3. Regression Verification

- [x] 3.1 Add unit and GPUI tests in `src/app/tests.rs` verifying contrast and theme polarity across all 14 built-in themes (`Ink`, `GitHub Dark`, `Solarized Dark`, `One Dark`, `Tokyo Night`, light themes), custom full/sparse `.toml` themes, live theme switching, and `VisualEdit` / `Read` / `Split` modes.
- [x] 3.2 Run `cargo fmt --all -- --check`, `cargo clippy --workspace`, and `cargo test` (`--lib` and `--bin markion`).

## Verification Record

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace`: passed (exit 0).
- `cargo test --lib`: 779 passed, 0 failed, 1 ignored.
- `cargo test --bin markion`: 883 passed, 0 failed, 2 ignored.
