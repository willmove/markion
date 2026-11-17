## 1. Query recognition and exact edits

- [x] 1.1 Bound slash queries to the caret and preserve suffix text/line endings, including standalone-template separation; verify UTF-8, LF/CRLF, Chinese markers, URLs, and stale queries with block-edit unit tests.
- [x] 1.2 Gate slash state using cached Visual block eligibility; verify supported prose/container contexts, excluded payloads, selection/IME suppression, and exact confirmation/undo with GPUI tests.

## 2. Compact palette

- [x] 2.1 Replace full-width format rows with a themed compact button grid, localized tooltips/caption, and separated Table action using embedded vector icons; verify all command bounds, compact geometry, filtering, pointer, keyboard, Escape, and anchoring through GPUI tests.

## 3. Integration verification

- [x] 3.1 Run Rust formatting, scoped slash regressions, the root application test suite, a desktop build, and strict validation of this OpenSpec change; record outcomes and any verification limitations.

## 4. Expanded settings and denser controls

- [x] 4.1 Add application-level Link, Formula, and Image palette actions with localized filtering, omit H6 from slash results, and tighten button spacing; verify catalog/filter results and compact button geometry.
- [x] 4.2 Open existing settings editors against the captured query without mutating source, retain local image selection and existing resource policies, and suppress competing completion; verify confirmation, cancellation, UTF-8/CRLF suffix preservation, undo, and stale target rejection.
- [x] 4.3 Run updated slash/editor regressions, root application/library tests, formatting, strict OpenSpec validation, and an isolated native build/UI check; record results and any unrelated existing failures.

## Verification record for expanded settings (2026-11-16, Windows)

- All 13 slash regressions passed, covering H1–H5-only palette filtering in every UI language, 36×30 buttons with 2px gaps, editor open/cancel without source/cache changes, suppression of completion during settings input, exact link/formula/image insertion, UTF-8 and LF/CRLF suffix preservation, one-step undo, empty-formula retention, and stale targets both before and after opening. Output: `target/tmp/slash-panel-qa/expanded-slash-tests.log`.
- `cargo test --lib --bin markion -- --quiet`: library 745 passed / 1 ignored; application 805 passed / 2 ignored, plus the same unchanged sidebar file-path assertion failure recorded below. Output: `target/tmp/slash-panel-qa/expanded-root-tests.log`.
- Formatting, diff whitespace checks, and strict OpenSpec validation passed. Native desktop build passed using the independent output path `target/tmp/slash-panel-qa/markion.exe`.
- Native UI verification confirmed the denser H1–H5 palette, Link/Formula/Image settings, link/formula confirmation and single undo, local file chooser cancellation preserving the query, filtered `/image` selection, local image import with its configured title, and undo restoring the exact query and paragraph. The imported image's SHA-256 matched the original fixture. Native file selection was checked through Windows because GPUI's headless `prompt_for_paths` is unimplemented. The isolated QA window was closed after verification.

## Verification record for initial implementation (2026-11-16, Windows)

- Rust formatting and strict validation of this change passed.
- Root library tests: 745 passed, 1 intentionally ignored.
- Root application tests: 800 passed, 2 intentionally ignored; one unchanged sidebar-layout test fails at `src/app/sidebar_layout_tests.rs:118` while finding a file-tree row by a literal Unix-style path. It also fails in isolation. Its fixture has no slash query; this change does not modify the sidebar or its test. Full output: `target/tmp/slash-panel-qa/root-tests.log`.
- All eight application slash regressions passed, including filtering, hover/cache identity, compact geometry, pointer confirmation, keyboard/Escape, supported/restricted contexts, LF/CRLF, Chinese markers, suffix preservation, and undo. GPUI retains debug selectors across frames, so filtered Table pointer validation uses a fresh window rather than interpreting stale bounds as current UI. Scoped output: `target/tmp/slash-panel-qa/slash-tests.log`.
- Both standalone integration suites passed: 2 statistics tests and 3 Windows input-frame tests.
- The running development application locks `target/debug/markion.exe`, preventing the ordinary `cargo test` command from replacing that binary. Tested library/application suites with `cargo test --lib --bin markion`; ran the two integration suites directly with `rustc --test`.
- Native desktop build passed using `cargo rustc --bin markion -- -o target/tmp/slash-panel-qa/markion.exe`. Real Windows UI verification used isolated preferences and a CRLF fixture: open the compact palette before existing CJK/English prose, click H2, undo once, filter Table, and insert a table while retaining the original paragraph below it. The temporary QA process was closed afterward. The user's existing application and profile were left running.
