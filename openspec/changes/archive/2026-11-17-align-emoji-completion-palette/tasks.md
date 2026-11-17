## 1. Shared completion presentation

- [x] 1.1 Share slash/emoji popup styling and compact button metrics; anchor emoji completion to the active surface and add glyph tooltips, hover selection, and shortcode caption. Verify rendered geometry against slash completion, including source mode and viewport edges, without changing per-version caches.

## 2. Integration verification

- [x] 2.1 Add focused GPUI coverage for placement, compact candidate reachability, filtering, hover/cache stability, keyboard/pointer confirmation, Escape, and undo; run these and existing slash/emoji regressions.
- [x] 2.2 Run formatting, root library/application tests, a native desktop build, and strict OpenSpec validation; record outcomes and any pre-existing failures or verification limitations.

## 3. Follow-up: completed emoji display

- [x] 3.1 Reproduce the reported post-insertion rendering failure, fix its source/render mapping within the existing shortcode contract, and cover the affected candidates and post-confirmation projections with regression tests, preserving exact source edits and one-step undo.
- [x] 3.2 Build the repaired application and verify actual Windows pointer/keyboard insertion, immediate glyph display, continued typing, undo/redo, and preview/source consistency; record the results and remaining limitations.

### Final follow-up verification (2026-11-16, Windows)

- Reproduced the supplied screenshot: consecutive line-leading shortcodes were incorrectly consumed as definition markers. The pre-fix library regression failed and the native window displayed the same `bug:` / `+1:` / `book:` text.
- Guarded complete recognized shortcode openers during block parsing while retaining byte offsets and restoring textual payloads. Regression coverage includes consecutive/quoted/indented lines, LF/CRLF, real definitions, code, smart punctuation, entities and literal controls.
- Expanded the shared catalog to 257 shortcodes, preserving every legacy alias/glyph. All 514 immediate rendered-projection combinations passed; keyboard and wheel navigation past the first twelve candidates retain compact geometry and exact pointer/undo behavior.
- Final root suites: library 755 passed / 1 ignored; application 821 passed / 2 ignored; zero failures. Markdown member unit/integration/doc suites: 223 passed. Formatting, whitespace and strict OpenSpec validation passed.
- Final native build and Windows OS input validation passed: same-block confirmation/typing/caret movement, Undo/Redo, reading preview, real definition-list preservation, mouse-wheel browsing and confirming an off-first-page candidate. Saved source exactly matched the expected canonical shortcodes, CJK and CRLF.
- Detailed evidence and screenshot: `docs/verification/emoji-completion-windows-2026-11-16.md` and `.jpg`. The isolated QA process was closed. The configured explicit progressive-reveal behavior on entering an authored token remains available; the reported definition-list collision is fixed.
- Additional ordinary `cargo build` compiled/linked successfully but could not replace the running `target/debug/markion.exe` (Windows error 5). The currently running old executable was not updated. The independently built and OS-verified repaired executable remains available at `target/tmp/emoji-insertion-qa/repaired/markion.exe`.
- [x] 3.3 Reuse the existing Markdown emoji catalog while preserving legacy aliases, make all matches keyboard-reachable in the twelve-button popup, and verify catalog/render consistency, navigation past the first window, filtering and exact undo.

## 4. Taller emoji palette

- [x] 4.1 Increase the unfiltered candidate window from 12 to 36 buttons in six rows, keeping the width and controls unchanged; update existing geometry/navigation/viewport regressions, run scoped emoji tests and formatting/OpenSpec checks, and verify the larger popup in a native Windows build.

### Taller palette verification (2026-11-16, Windows)

- Increased the visible window from 12 to 36 candidates: six rows of six 36×30px buttons with 2px gaps. The grid is 190px tall; the whole popup is about 268px tall versus about 140px previously. Width remains 248px.
- All nine scoped emoji tests passed, including distinct bounds for all 36 buttons, six-row geometry, navigation and wheel paging across 36-result windows, pointer confirmation/undo, filtering, source/Visual Edit anchoring and a constrained viewport with physical-pixel rounding tolerance. Output: `target/tmp/emoji-height-qa/final-tests.log`.
- Native build, Rust formatting, whitespace and strict OpenSpec validation passed. Output: `target/tmp/emoji-height-qa/build.log`.
- Windows UI verification confirmed all six rows, bottom-right candidate insertion as exact `:brown_heart:`, Undo back to the colon query, and wheel browsing to the following 36-result window. Screenshot: `docs/verification/emoji-palette-height-windows-2026-11-16.jpg`.
- With no Markion instance running, copied the verified executable to `target/debug/markion.exe` and the previously supplied repaired executable path, verifying SHA-256 equality. The native check used the updated canonical debug executable with an isolated profile; the QA process was closed afterward.

### Initial follow-up investigation before screenshot (2026-11-16, Windows)

- Added a GPUI regression checking all 32 completion candidates immediately after confirmation, both alone in an empty block and before CJK text with CRLF. It checks the actual painted Visual Edit projection as well as canonical shortcode source. All 64 combinations passed against the current working tree. Output: `target/tmp/emoji-insertion-qa/candidate-tests.log`.
- Built current application code to `target/tmp/emoji-insertion-qa/markion.exe`. Windows UI testing with an isolated profile reproduced correct pointer insertion of thumbs-up before prose, keyboard insertion of a heart in an empty block, continuing text after the heart, and pointer insertion of `100` in the latest build. Each displayed its colored glyph. Undo restored the colon query and Redo restored the `100` glyph; the isolated process was closed afterward.
- The reported failure is not yet reproduced. No production behavior has been changed by this follow-up. Requested the specific symptom and candidate from the user; repair tasks remain open pending that evidence. Existing progressive inline reveal can show an authored shortcode when the caret is moved inside it; this is distinct from the immediate post-confirmation display being checked here.

## Initial palette verification (2026-11-16, Windows)

- All four new GPUI regressions passed: shared slash/emoji geometry at paragraph, heading, list, quote and later-block starts; twelve reachable 36×30px candidates in two six-button rows with 2px gaps; UTF-8/CRLF suffix preservation and one-step pointer undo; hover/cache stability; filtered keyboard navigation, Enter/Tab confirmation, Escape; source-caret placement despite stale Visual Edit bounds; source scrolling; and constrained-window fitting. Output: `target/tmp/emoji-palette-qa/final-geometry-tests.log`.
- Final `cargo test --lib --bin markion -- --quiet`: library 749 passed / 1 ignored; application 816 passed / 2 ignored; zero failures. This includes existing slash, emoji, and inline-overlay regressions. Output: `target/tmp/emoji-palette-qa/final-root-tests.log`. Earlier sidebar and block-grip failures in the shared working tree were resolved by concurrent work before this final run; this change did not edit those tests.
- `cargo fmt --check`, `git diff --check`, and `openspec validate align-emoji-completion-palette --strict` passed.
- Native Windows UI verification used an isolated profile and test document. Typing `:` and `/` before existing prose showed the same compact shell width, styling, button metrics, below-caret offset and top alignment. The emoji popup displayed two rows of colored glyphs and a shortcode caption. Undo restored the original paragraph. The temporary QA process was closed after inspection.
- Final native build passed with `cargo rustc --bin markion -- -o target/tmp/emoji-palette-qa/markion.exe`, using an independent executable so the running development application remained undisturbed. Output: `target/tmp/emoji-palette-qa/final-build.log`.
