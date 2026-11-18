## 1. Formatter Registry & Safe Subprocess Runner

- [x] 1.1 Implement `CodeFormatter` and `CodeFormatterRegistry` in `src/code_format.rs` supporting built-in JSON (`serde_json`), JS/TS (`biome`/`prettier`), Java (`google-java-format` binary and `.jar`), Rust (`rustfmt`), and Python (`ruff format`), plus fence boundary/CRLF/indent-preserving patching helpers.
- [x] 1.2 Extend `src/external_command.rs` with process-tree timeout cleanup and safe direct-argument invocation (no shell interpolation, bounded input/output, Windows `.ps1`/space-path/JAR support).

## 2. Preferences, Shortcuts, Menus & i18n

- [x] 2.1 Add `CodeBlockFormatPreferences` in `src/model.rs`, `[code_blocks]` TOML persistence and reset handling in `src/storage/preferences.rs`, and the `Preferences -> Editor -> Code Blocks` tab in `src/app/code_format_prefs.rs`.
- [x] 2.2 Register `FormatCodeBlock` (`format-code-block`, default `secondary-alt-l`) in `src/app/mod.rs` and `src/app/bootstrap.rs`, add Visual Edit & Split Preview header format buttons, editable source/visual context menu items, and 8-language localization in `src/code_format_i18n.rs`.

## 3. Editor Integration, Async Safety & Verification

- [x] 3.1 Wire opt-in `format_on_paste`, manual formatting dispatch, generation/version/tab stale checks, single-step Undo/Redo, and caret preservation in `src/app/editing.rs`, `src/app/preview.rs`, and `src/app/root_view.rs`.
- [x] 3.2 Add unit tests (`src/code_format.rs`, `src/storage/preferences.rs`) and GPUI end-to-end tests (`src/app/tests.rs`), and verify `./scripts/check-quality.ps1` Rust/workspace/bundle gates.

## Verification Record

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace`: passed (exit 0).
- `cargo test --workspace`: all crates passed (`markion` lib 789 passed / 1 ignored; `markion` bin 886 passed / 2 ignored; integration suites and all member crates 0 failed).
- `cargo run -p wechat-workspace --bin verify-bundle -- assets/marknice-workspace`: passed (`verified 23 files (3140387 bytes)`).
