## Why

In Visual Edit, the default Notes sample exposes broken reference-link / footnote rendering: footnote references stay literal (`[^links]`), footnote definitions split into a gray source island plus an orphan paragraph, and link reference definition lines render as Unsupported source islands. Split Preview / HTML already treat these as resolved footnotes and omit link definitions — Visual Edit breaks the WYSIWYG contract for document-scoped link metadata.

## What Changes

- Visual Edit resolves footnote references during per-block inline parsing the same way it already resolves reference-style links (document-scoped definition suffix).
- Preview derivation tracks footnote definition wrappers so definition bodies are not emitted as ordinary paragraphs; Visual Edit presents a single footnote-definition block covering the full authored range.
- Gaps that contain only link reference definition lines are no longer Unsupported source islands; they remain source-backed and editable without the gray island chrome (matching the fact that HTML/preview omit them from the rendered narrative).
- Spec deltas under `markdown-editing` lock the Notes-sample behavior and move footnote definitions out of the transitional gap framing for the happy path.

Non-goals: changing Split Preview/Read HTML exporters beyond sharing the corrected preview-block derivation; multiline complex footnote nesting UX polish; reference-style images; editing footnote numbering UI.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `markdown-editing`: Clarify Visual Edit footnote-reference fidelity, footnote-definition presentation, and link-reference-definition gap handling so the Notes sample matches rendered preview semantics without source-island chrome.

## Impact

- **Code:** `src/visual.rs` (definition collection + gap classification + footnote visual blocks), `src/lib.rs` / preview derivation (`derive_preview_and_outline`), `src/model.rs` (`PreviewBlock` / `VisualBlockKind`), `src/app/preview.rs` (render arms). Cached-per-version visual/preview derivation paths only — no keystroke-path recomputation beyond existing invalidation.
- **Invariants:** `MarkdownDocument.text` remains canonical; definition collection stays inside the already-cached `build_visual_blocks` / preview derive; per-block source ranges stay byte-exact.
- **Tests:** regression covering the Notes fixture’s footnote ref, footnote definition, and link definition visual blocks.
