## Why

Issue #51 shows that built-in LaTeX export flattens nested lists into one environment and restarts every ordered list at 1, even though the cached preview model already carries each item's level and resolved index. This changes the document's structure and numbering in a format intended to provide full-fidelity text export.

## What Changes

- Preserve nested ordered, unordered, and task-list hierarchy in built-in LaTeX output.
- Start each ordered LaTeX environment at the resolved first item index, including lists separated by prose or another list kind.
- Continue using the existing rich-text item renderer so inline styles and checked/unchecked task markers remain intact.
- Add real `.tex` export regressions for nesting, mixed list kinds, non-1 starts, and separated lists.

Non-goals: do not change HTML, DOCX, PDF, or Markdown export behavior; do not compile LaTeX or redesign the preview/list model; do not add dependencies.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `export`: extend the native LaTeX fidelity requirement so list environments preserve resolved nesting and ordered starting values in addition to inline styles and task markers.

## Impact

- Affects the built-in LaTeX body/item rendering in `src/lib.rs` and `src/render.rs`, plus root-crate export tests.
- Public document/export APIs and the persisted Markdown representation remain unchanged.
- The change consumes the existing per-document-version cached `PreviewBlock::ListItem` values and does not add recomputation, mutable derived state, or cache invalidation.
