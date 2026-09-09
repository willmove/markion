# Proposal: fix-list-item-block-roundtrip

## Why

`render_to_markdown` in `crates/markdown` corrupts loose list items — a list item containing additional child blocks (e.g. a second paragraph) is re-rendered without the blank line that CommonMark requires between the item's content line and its child blocks. Re-parsing the output merges the child paragraph into the item's inline text via lazy continuation (`- first paragraph\n\n  second paragraph\n` renders to `- first paragraph\n  second paragraph\n`, which re-parses as a single paragraph `first paragraph second paragraph`). This is a real, user-visible data-integrity defect: the export engine feeds `render_to_markdown` output to pandoc (`crates/export/src/engine.rs`), so exporting a document with multi-paragraph list items silently flattens their structure. The defect was caught by the pre-existing `roundtrip_property_test` proptest suite and confirmed against a realistic hand-authored document.

## What Changes

- Fix `render_list` in `crates/markdown/src/renderer.rs` to emit a blank (continuation-indented) line between a list item's inline content line and its first child block, and blank-line separation between sibling child blocks, so child blocks survive re-parsing as separate blocks.
- Add focused renderer unit tests covering loose list items, multi-paragraph items, and items mixing paragraph/code children.
- Persist the previously failing proptest seed (`8b1f2303…`, list item `?` + paragraph `  .`) into `roundtrip_property_test.proptest-regressions` so the property suite replays it permanently, and confirm the full property suite passes from the repository root.

Non-goals: no overhaul of verbatim `Inline::Text` escaping beyond the existing `escape_leading_block_marker` coverage; no AST changes (the `sub_items` model that cannot represent ordered nested lists stays as-is); no changes to parser behavior.

## Capabilities

- **New Capabilities**: none.
- **Modified Capabilities**:
  - `markdown-editing` — add a serialization round-trip fidelity requirement for the Markdown AST renderer (`render_to_markdown`), which this spec's "parsing and formatting" scope currently does not cover.

## Impact

- **Code**: `crates/markdown/src/renderer.rs` (`render_list`), `crates/markdown/tests/roundtrip_property_test.proptest-regressions`, renderer unit tests.
- **Consumers**: `crates/export` pandoc input path stops flattening loose list items; preview/editing paths are unaffected (they do not consume `render_to_markdown`).
- **Invariants**: no project-context invariants touched — the change is confined to a GUI-free workspace member and adds no dependencies.
