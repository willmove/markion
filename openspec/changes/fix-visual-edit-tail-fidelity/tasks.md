# Tasks: fix-visual-edit-tail-fidelity

## 1. Whitespace row height fidelity

- [x] 1.1 Extract the whitespace row height into a pure helper (blank-line count → height: 12px per line, minimum one line, generous ~4096-line bound instead of the 72px clamp) and use it from the `Whitespace` render arm in `src/app/preview.rs`
- [x] 1.2 Unit-test the helper: height grows with line count without the old cap, shrinks back on undo/shrink paths, floors at one line, and respects the sanity bound
- [x] 1.3 Confirm the caret-bearing whitespace row keeps the thin-caret passive presentation (no island chrome) at any height; adjust the existing whitespace-activation tests if heights changed their fixtures

## 2. Height-aware splice identity

- [x] 2.1 Add `height_signature: Option<u32>` to `VisualBlock`; stamp the covered newline count for Whitespace blocks in `build_visual_blocks` (`src/visual.rs`), `None` for every other kind; update `document_memory` byte accounting and its tests for the new field
- [x] 2.2 Change `visual_block_splice` (`src/app/preview.rs`) to compare `(id, height_signature)` pairs; keep the common prefix/suffix algorithm unchanged
- [x] 2.3 Unit-test the splice: a whitespace row whose covered newline count changed with a stable ID lands inside the spliced range (forces re-measure), while an unchanged whitespace row and all non-whitespace identity-preserved rows stay outside it (cached heights and scroll anchoring remain reusable)
- [x] 2.4 Verify the incremental visual cache still assigns stable IDs to the grown/shrunk tail block so the signature is what forces re-measure, not incidental ID churn; add a regression test if ID stability is not already covered

## 3. Scroll extent covers list padding

- [x] 3.1 Move `.pt`/`.pb` off the Visual Edit `list` element onto a wrapping padded container in `visual_edit_surface_view` (`src/app/root_view.rs`); apply the same restructure to the preview list
- [x] 3.2 Re-run pane-scroll and Sync scroll tests; fix any scrollbar-thumb or sync-anchor offset math in `list_pane_scrollbar_view`/`pane_scrollbar_view` that shifted with the padding move
- [x] 3.3 Manually verify in a debug build: the last text line scrolls fully into view, the scrollbar thumb reaches the bottom, repeated Enter at the tail visibly grows the blank region and stays revealed, and scrolling back up shows no layout jump
- [x] 3.4 Decide the two design open questions with test evidence (12px constant vs typography-derived line height; whether the preview list also needs the height signature) and record the outcome in the change folder notes

## 4. Mutation diagnostics for in-memory duplication

- [ ] 4.1 Add `tracing::debug!` at the canonical choke points `MarkdownDocument::replace_source_range` and `set_text` (`src/lib.rs`): document version, edit range, old/new text lengths — never content
- [ ] 4.2 Add one tagged `tracing::debug!` per high-level mutation entry point (structural Enter `insert_newline`, `replace_text_in_range` and both IME mark paths, `apply_markdown_format`, `apply_exact_block_edit`, table edits, undo, redo, `reload_from_disk`) with an `op` tag and selection range
- [ ] 4.3 Verify the installed log setup (`src/app/bootstrap.rs` / subscriber init) records `debug` for the `markion` target in the default build; enable debug for that target if the filter drops it, and confirm a test edit produces log lines in `%LOCALAPPDATA%/Markion/Logs`
- [ ] 4.4 Document the repro procedure for the outline-duplication incident (keep the corrupted in-memory document — copy text out before undoing; collect the log window) in the change folder so the next repro identifies the writing path

## 5. Verification

- [ ] 5.1 `cargo test --workspace` passes with the new tests
- [ ] 5.2 `openspec validate fix-visual-edit-tail-fidelity` passes

## 6. Tail caret follow (remaining UX hole after row-height fidelity)

- [x] 6.1 Paint the whitespace-row caret on the insertion line that matches newlines before the source caret, and map clicks by Y onto those newlines
- [x] 6.2 After a caret-moving edit, pixel-follow `visual_caret_bounds` into the Visual Edit list viewport so last-line typing and tail Enter cannot grow below the clip
- [x] 6.3 Tests: caret Y increases with each tail Enter; typed tail text stays inside the laid-out visual viewport
