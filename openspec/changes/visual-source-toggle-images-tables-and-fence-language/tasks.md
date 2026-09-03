## 1. Model and derivation foundation

- [ ] 1.1 In `src/model.rs`, extend `VisualBlockEditor`: add `Image { payload }` (single whole-span field), add `source: VisualEditorField` to `Table`, and add `info: VisualEditorField` to `Code`; add `VisualEditorFieldKind::{ImageSource, TableSource, CodeInfo}`; keep the dormant `ImageAlt`/`ImageDestination`/`ImageTitle` kinds untouched; update `fields()` / `field_containing()` and fix exhaustive matches
- [ ] 1.2 In `src/visual.rs`, add a conservative local byte-scan span proof for `PreviewBlock::Image` (slice starts `![`, ends `)`, label/destination bounds via the existing unescaped-delimiter scan helpers; no pulldown-cmark re-parse) and construct `VisualBlockEditor::Image` only when proven — unprovable forms keep `editor: None` and today's fallback
- [ ] 1.3 In `src/visual.rs`, give the `Table` editor a `source` field covering the complete block source range (only for tables that already have proven cell editors; tables without editors keep `editor: None`)
- [ ] 1.4 In `src/visual.rs`, extend `fenced_payload_ranges`/the `CodeBlock` arm so `info` covers the first info-string token, or an empty range at `opening_fence.end` when no info string is authored (works for ordinary, diagram, and `math` fences)
- [ ] 1.5 Add derivation tests in `src/visual.rs`: proven/refused image spans (escaped labels, angle destinations, reference-style, multiline, CRLF), table source field presence/absence, info field for bare/`lang`/`lang extra` fences with LF and CRLF

## 2. Standalone image source editor

- [ ] 2.1 In `src/app/preview.rs`, route the `VisualBlockKind::Image` arm (when `editor` is present) through `visual_collapsible_source_block`: presentation stays exactly today's image + caption + caret-owner controls; payload is one monospaced field over the whole span; clicking the image never expands
- [ ] 2.2 Wire forced expand from the preview-image cache failed/unavailable state for the resolved destination (mirror the math `forced = !Ready` pattern); load failure must not mutate source/history/version
- [ ] 2.3 Add i18n strings only if the shared `</>` tooltip does not already cover the image case (EN + zh-CN) in `src/i18n.rs`
- [ ] 2.4 Add Visual Edit tests in `src/app/tests.rs`: toggle expand shows the exact authored span, one payload edit applies as one exact replacement and caption/width/alignment re-derive, caret-in-payload stays expanded, outside click collapses, unloadable destination forces the payload, toggling leaves document version/dirty/undo unchanged

## 3. Table raw-source toggle

- [ ] 3.1 In `src/app/preview.rs`, wrap the `VisualBlockKind::Table` arm in `visual_collapsible_source_block` when the editor has a `source` field: grid above, raw payload below; while expanded (or caret in payload) the grid renders read-only (no mounted cell editors) and the row/column toolbar is not interactive
- [ ] 3.2 Route the caret into the raw payload when expanding from a focused cell, and collapse + focus the clicked cell (current-version range) when a cell is activated while expanded
- [ ] 3.3 In `src/lib.rs` (`visual_editor_tab_target` / related traversal), skip cell fields for a table block whose id is expanded so Tab never enters unmounted cell editors; payload participates normally and boundary hand-off is unchanged
- [ ] 3.4 Add Visual Edit tests in `src/app/tests.rs`: expand shows complete pipe source as one field, payload edit (alignment row change, pasted rows) is one atomic replacement and re-derives the grid/exporters, cell-activation collapse routing, Tab skips cells while expanded, expand/collapse is presentation-only, ambiguous tables still take the source-backed fallback with no toggle

## 4. Fence language editing

- [ ] 4.1 Extend the code-block header in `src/app/preview.rs` with an optional editable language element mounted only when the fence owns the caret; `visual_code_editor` and `visual_diagram_editor` pass it, Read mode / Split Preview / unfocused headers keep the static label
- [ ] 4.2 Add the language field's input sanitizer in the field input path: reject whitespace, backticks, and line breaks; prevent input that would empty the token while mounted (design decision 5)
- [ ] 4.3 Route `CodeInfo` edits through the checked mutation boundary in `src/app/editing.rs`: replace the exact token range, or insert at the empty `opening_fence.end` range for bare fences; one Undo restores the prior info string
- [ ] 4.4 Add localized placeholder/aria strings for the language field (EN + zh-CN) in `src/i18n.rs`
- [ ] 4.5 Add tests in `src/app/tests.rs`: token-only replacement keeps fences/remainder/payload byte-identical, bare-fence insertion, invalid characters never reach source, retyping to `mermaid`/`math` re-dispatches via ordinary re-parse, static label on non-caret-owner surfaces, undo round-trip

## 5. Coverage matrix and docs

- [ ] 5.1 Update the Visual Edit WYSIWYG coverage matrix in `docs/visual-editing-quality.md`: standalone Markdown images row (whole-span collapsible payload, forced-expand on load failure), GFM tables row (raw-source toggle, read-only grid while expanded), fenced code row (info-token editing); keep every row's canonical-range and evidence columns accurate

## 6. Verification

- [ ] 6.1 Run `cargo test` for the root package (derivation, visual edit, mutation, i18n coverage tests) and fix regressions
- [ ] 6.2 Run `cargo test --workspace`, then manually smoke-test in Visual Edit: hover `</>` on an image and a table, expand/edit/collapse each; edit the language on ordinary, `mermaid`, and bare fences; verify an unloadable image forces its source; confirm no derived-cache recomputation on toggles in a CRLF document
