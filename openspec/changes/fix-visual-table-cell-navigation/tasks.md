## 1. Cell geometry and pointer placement

- [x] 1.1 Add display-only source-backed empty-cell geometry and cell-padding pointer placement while preserving precise text clicks and control event isolation; verify GPUI tests cover empty header/body cells, padding, caret bounds, text input/IME and undo without pointer-created edits.

## 2. Same-column navigation

- [x] 2.1 Register all table cell layouts, scope active-cell line navigation, and resolve adjacent logical rows in the same column; verify Up/Down, selection, wrapping, blank rows, table boundary handoff, LF/CRLF and unchanged per-version cache identity in GPUI regressions.

## 3. Integration verification

- [x] 3.1 Update the visual editing coverage document and validate the change with `openspec validate fix-visual-table-cell-navigation --strict`; verify formatting and diff whitespace checks pass.
- [x] 3.2 Run focused table regressions and `cargo test --workspace`, repair regressions, and record the results without claiming a manual app smoke test.

## Verification results (2026-10-08)

- Focused table suite: 16 passed. The final workspace run also passes the added unchanged-repaint geometry check.
- `cargo test --workspace`: 1877 passed, 0 failed, 5 pre-existing ignored tests, including 685 root-library and 668 application tests.
- `cargo fmt --all -- --check`, `git diff --check`, `cargo clippy --workspace`, `cargo build`, and the pinned MarkNice bundle verifier passed. Clippy still reports existing repository warnings.
- `openspec validate fix-visual-table-cell-navigation --strict` passed.
- Global `openspec validate --all --strict --no-interactive` reports 55 passed and 33 failed items in existing, untouched changes/specs (missing preserved scenarios or placeholder Purpose sections). This change passes; unrelated artifacts were not edited.
- No manual desktop-app smoke test was performed. The development executable is updated at `target/debug/markion.exe`.
