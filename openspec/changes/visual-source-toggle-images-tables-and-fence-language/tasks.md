## 1. Model and derivation foundation

- [x] 1.1 In `src/model.rs`, extend `VisualBlockEditor`: add `Image { payload }` (single whole-span field), add `source: VisualEditorField` to `Table`, and add `info: VisualEditorField` to `Code`; add `VisualEditorFieldKind::{ImageSource, TableSource, CodeInfo}`; keep the dormant `ImageAlt`/`ImageDestination`/`ImageTitle` kinds untouched; update `fields()` / `field_containing()` and fix exhaustive matches
- [x] 1.2 In `src/visual.rs`, add a conservative local byte-scan span proof for `PreviewBlock::Image` (slice starts `![`, ends `)`, label/destination bounds via the existing unescaped-delimiter scan helpers; no pulldown-cmark re-parse) and construct `VisualBlockEditor::Image` only when proven — unprovable forms keep `editor: None` and today's fallback
- [x] 1.3 In `src/visual.rs`, give the `Table` editor a `source` field covering the complete block source range (only for tables that already have proven cell editors; tables without editors keep `editor: None`)
- [x] 1.4 In `src/visual.rs`, extend `fenced_payload_ranges`/the `CodeBlock` arm so `info` covers the first info-string token, or an empty range at `opening_fence.end` when no info string is authored (works for ordinary, diagram, and `math` fences)
- [x] 1.5 Add derivation tests in `src/visual.rs`: proven/refused image spans (escaped labels, angle destinations, reference-style, multiline, CRLF), table source field presence/absence, info field for bare/`lang`/`lang extra` fences with LF and CRLF

## 2. Standalone image source editor

- [x] 2.1 In `src/app/preview.rs`, route the `VisualBlockKind::Image` arm (when `editor` is present) through `visual_collapsible_source_block` with `payload_first = true` so the source payload renders **above** the image (per user testing feedback): presentation stays exactly today's image + caption + caret-owner controls below; payload is one monospaced field over the whole span; clicking the image never expands
- [x] 2.2 Wire forced expand from the preview-image cache failed/unavailable state for the resolved destination (mirror the math `forced = !Ready` pattern); load failure must not mutate source/history/version
- [x] 2.3 Add i18n strings only if the shared `</>` tooltip does not already cover the image case (EN + zh-CN) in `src/i18n.rs` — _the shared `</>` toggle carries no localized tooltip, so nothing to add_
- [x] 2.4 Add Visual Edit tests in `src/app/tests.rs`: toggle expand shows the exact authored span, one payload edit applies as one exact replacement and caption/width/alignment re-derive, caret-in-payload stays expanded, outside click collapses, unloadable destination forces the payload, toggling leaves document version/dirty/undo unchanged

## 3. Table raw-source toggle — REVERTED (user testing feedback)

The dual-editor table mode was implemented, tested, and then reverted after real-machine testing found the demoted grid hurt the editing experience. GFM tables keep their always-editable grid exactly as before this change; all table code, tests, and the `tables-outline` delta were removed.

- [x] 3.1 ~~Wrap the `VisualBlockKind::Table` arm in `visual_collapsible_source_block`~~ — reverted: `visual_table_view` restored to the original always-editable grid and interaction-gated toolbar
- [x] 3.2 ~~Route the caret between payload and cells~~ — reverted: cell click/edit behavior unchanged
- [x] 3.3 ~~Skip cell fields for an expanded table in `visual_editor_tab_target`~~ — reverted: `visual_editor_tab_target` restored to its original table-cell traversal; the transient `expanded_visual_source_mirror` on `MarkdownDocument` was removed with it
- [x] 3.4 ~~Add Visual Edit table tests~~ — the two raw-source tests were removed; the original toolbar/cell test suite passes unchanged

## 4. Fence language editing

- [x] 4.1 In `src/app/preview.rs`, hide the code-fence language label while the pointer is outside the fence and the fence does not own the caret; reveal it as an explicit click-target chip (`CodeHeaderLanguage::Field`) while the fence is hovered (`tab.hovered_visual_code_block`) or owns the caret; `visual_diagram_editor` mounts the same chip whenever its payload editor is visible; Read mode / Split Preview keep the static label
- [x] 4.2 Add the language field's input sanitizer in the field input path: reject whitespace, backticks, and line breaks; Enter in the field commits instead of inserting a newline
- [x] 4.3 Route `CodeInfo` edits through the checked mutation boundary: replace the exact token range, or insert at the empty `opening_fence.end` range for bare fences; code-payload edge handoff filters the unmounted info field
- [x] 4.4 Add localized placeholder strings for the language chip (`Msg::CodeLanguage`, all seven languages) in `src/i18n.rs`
- [x] 4.5 Add tests in `src/app/tests.rs`: token-only replacement keeps fences/remainder/payload byte-identical, bare-fence insertion, invalid characters never reach source, retyping to `mermaid`/`math` re-dispatches via ordinary re-parse, chip reveal on hover/caret and click-to-place-caret, bare-fence placeholder chip click lands in the insertion range

## 5. Coverage matrix and docs

- [x] 5.1 Update the Visual Edit WYSIWYG coverage matrix in `docs/visual-editing-quality.md`: standalone Markdown images row (whole-span collapsible payload above the image, forced-expand on load failure), fenced code row (hover/caret-revealed info-token chip); GFM tables row stays untouched; keep every row's canonical-range and evidence columns accurate

## 6. Verification

- [x] 6.1 Run `cargo test` for the root package (derivation, visual edit, mutation, i18n coverage tests) and fix regressions
- [x] 6.2 Run `cargo test --workspace` (all green: 518 lib + 453 app + member crates, 0 failures); manual smoke-test deferred to the user's real-machine pass — covered behaviors: hover `</>` on an image expands the source above the image, unloadable image forces its source, fence language chip reveals on hover/caret and edits the token, tables keep the always-editable grid
