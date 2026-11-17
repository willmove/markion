## 1. Compact Insert Presentation

- [x] 1.1 Classify non-table block insertion and callout insertion as distinct compact groups, render insertion H1–H5 with explicit heading captions, and keep Table/Link/Formula/Image labeled using the existing shared panel/buttons. Verify the resulting flyout has two six-button block rows and one five-button callout row, retains all 21 commands in their current order, and uses the same dimensions and spacing as root controls without touching document mutation or per-version cache paths.

## 2. Interaction and Layout Verification

- [x] 2.1 Extend `context_top_level_block_controls_and_bottom_insertion_are_reachable` in `src/app/block_context_tests.rs` to verify compact row geometry and root/flyout sizing, then click the compact H2 button and verify exact insertion after the target plus one-step Undo. Run `cargo test -p markion context_top_level_block_controls_and_bottom_insertion_are_reachable` and retain its complete 21-action keyboard traversal, constrained-viewport/theme coverage, and presentation-only source/selection/history/cache assertions.
- [x] 2.2 Run `cargo fmt --all -- --check`, `cargo test -p markion block_context_tests`, and `cargo test -p markion visual_block_menu`, fixing any failures caused by this change. Verify existing insertion, stale-target, hover, table/image flyout, LF/CRLF, and Undo behavior remains covered; inspect the live flyout when available and record any unavailable manual checks. Run `openspec validate compact-context-insert-submenu --strict` and confirm the final diff is limited to this presentation change and its planning/test artifacts.

## Verification Record

- `cargo fmt --all -- --check`: passed.
- Focused compact insertion reachability test: passed, including compact row geometry, all 21 keyboard actions in Paper/Ink and normal/constrained viewports, pointer H2 insertion, exact Undo, and presentation-only version/dirty/cache checks.
- `cargo test -p markion block_context_tests`: 30 passed.
- `cargo test -p markion visual_block_menu`: 4 passed.
- `openspec validate compact-context-insert-submenu --strict` and `git diff --check`: passed.
- Native Windows check: launched the rebuilt debug application with an isolated profile and a Chinese fixture, opened the paragraph context menu and Insert flyout, confirmed compact rows and explicit H1–H5 captions, clicked H2, and confirmed one Undo restored the original document. No manual-check scope was skipped.
- Cargo emitted linker informational messages and an existing future-compatibility notice for `proc-macro-error2`; no compilation or test failures occurred. Full workspace tests were not run for this root-application presentation change.
