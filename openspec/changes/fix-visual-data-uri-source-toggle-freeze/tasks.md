# Tasks: fix-visual-data-uri-source-toggle-freeze

## 1. Atomic projection foundation

- [x] 1.1 Add `atomic: bool` to `VisualProjectionSegment` (src/model.rs), defaulting false in every existing constructor; compile-clean with no behavior change (`cargo test` root package).
- [x] 1.2 Make the mapping helpers in src/visual.rs snap atomic segments: `source_for_display` / `display_for_source` / `boundary_candidates` / `word_selection_range` return only segment-boundary offsets for atomic interiors, exposing the start/end pair as the upstream/downstream candidates.
- [x] 1.3 Add unit tests in src/visual.rs for atomic snapping: interior display offsets map to boundaries, double-click selects the whole atomic display range, non-atomic segments keep byte-identical mapping (differential against current expectations).

## 2. Elision policy at derivation

- [x] 2.1 Extend `VisualBlockEditor::Image` with the elision policy (`visible_head`, `token` source ranges, precomputed token display text) computed in the `PreviewBlock::Image` arm of the editor construction in src/visual.rs: data-URI destinations elide the payload after the `;base64,` comma; spans ≥ 64 KiB with other destinations keep a 48-char verbatim head; everything else stays `None` (verbatim).
- [x] 2.2 Add the localized size-label message to src/i18n.rs (with `{size}` placeholder, human-readable binary units) and use it when building the token display text (`…{size}…`), framed by ellipsis marks per the spec.
- [x] 2.3 Add derivation tests: data-URI, oversized non-data-URI, sub-threshold image, and image with title/presentation suffix each produce the exact expected ranges; the policy is computed once per document version (derivation counter unchanged across repaints).

## 3. Elided payload rendering

- [x] 3.1 In `visual_editor_field_projection` (src/app/preview.rs), emit the elided display text when the field carries an elision policy: verbatim segments for the head and tail, one atomic segment for the token mapped to the token source range, plus a token display-range span styled with the distinct chip style (background tint + dimmed text).
- [x] 3.2 Make `visual_collapsible_source_block` construct the payload editor lazily (only when `show_payload`), including the math/diagram/HTML call sites; collapsed frames must not clone the authored span (assert via the new projection-build counter staying zero).
- [x] 3.3 Wire caret/click/edit behavior end to end: caret never rests inside the token, clicks snap to boundaries by x proximity, selection covering the token replaces the whole token source range in one canonical replacement, adjacent Backspace/Delete removes the whole token with single-undo restore.
- [x] 3.4 Update/extend the toggle integration test (pattern of `visual_image_source_toggle_expands_collapses_and_edits_exactly`, src/app/tests.rs) for the scenarios in the `markdown-editing` delta: distinct-token expansion, atomic replacement, oversized-head editing, forced-visible error path with elision, presentation-only toggling, source-mode unaffected.

## 4. Line-driven navigation snapshots

- [x] 4.1 Add the `#[cfg(test)]` `visual_navigation_position_queries` counter and instrument every layout position query the snapshot path performs.
- [x] 4.2 Rewrite `visual_navigation_snapshot` (src/app/preview.rs) to enumerate wrapped lines by y and resolve each line's start/end display indices via `index_for_position` (O(W) total); drop the per-grapheme `line_ys`/`display_boundaries` collections; reshape `VisualNavigationLine` to the line-window form.
- [x] 4.3 Adapt the snapshot consumers in src/app/editing.rs (caret Up/Down via `current_visual_navigation_snapshot`, the :3046/:3099 call sites) to resolve caret x positions and candidate offsets lazily from the line window at keypress time.
- [x] 4.4 Add the snapshot-equivalence test: caret Up/Down over representative documents (wrapped paragraphs, code payloads, elided image fields) lands on the same source offsets as the per-grapheme snapshot produced before the rewrite (golden offsets captured in the test).
- [x] 4.5 Gate the bound: tests assert `visual_navigation_position_queries` per paint ≤ lines × small constant for a large fenced code payload, and that elided image fields produce constant-bounded display text length.

## 5. Fingerprint-based forced-expand

- [x] 5.1 Compute a 64-bit destination fingerprint at derivation and store it on the image editor payload; keep `PreviewImageKey` and the decode path unchanged.
- [x] 5.2 On decode completion of an `Error` result in `preview_image_cache.complete` handling (src/app/preview_image.rs), insert the URL fingerprint into a bounded `failed_image_fingerprints` set on app state (cleared with the same hygiene as `preview_probe_results`).
- [x] 5.3 Replace the per-frame `preview_image_entry` probe in `visual_image_source_editor` with an O(1) fingerprint-set lookup; add a test that a failed data URI still forces the payload editor visible (elided form) and a decode-error→fix→collapse round trip clears the forced state on re-derivation.
- [x] 5.4 Add the collapsed-frame counter test: repaints of a collapsed data-URI image perform no span clone and no span-length key derivation (per the `engineering-quality` delta scenario).

## 6. Validation and wrap-up

- [x] 6.1 Run `cargo test` (root) and `cargo test --workspace`; fix regressions; re-run the deterministic performance counters suite.
- [ ] 6.2 Manual device check: expand/collapse a multi-megabyte base64 data-URI image (hover reveal, toggle, caret entry, token replace, undo, forced-error path) stays interactive; verify the token reads clearly as a placeholder on light/dark themes.
- [x] 6.3 `openspec validate fix-visual-data-uri-source-toggle-freeze --strict` passes; update the WYSIWYG coverage matrix entry for image source editing if it references verbatim payload display.
