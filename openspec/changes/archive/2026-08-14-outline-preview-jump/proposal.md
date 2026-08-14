## Why

Clicking an outline heading always navigates the source editor, so in Read mode the visible preview does not move even though the hidden source cursor changes. Read-mode outline navigation should bring the corresponding rendered heading into view while preserving the existing document-position semantics.

## What Changes

- Make outline navigation context-aware for Read mode: clicking a heading updates the canonical source position and scrolls the rendered preview to the matching heading block.
- Keep the clicked outline item highlighted after Read-mode navigation by retaining the existing cursor-based active-section model.
- Preserve the current outline behavior in Edit, Visual Edit, and Split Preview modes.
- Resolve the preview target from the already-cached preview blocks and their source ranges, without reparsing Markdown or adding per-frame derived work.

**Non-goals**: This change does not make manual preview scrolling update the outline highlight, change Split Preview navigation or Sync scroll behavior, add smooth animated scrolling, add outline collapse/expand, or introduce keyboard focus for rendered headings.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: Extend document outline navigation so a Read-mode outline click reveals the corresponding rendered heading while retaining the existing source-position and active-highlight behavior.

## Impact

- **Affected code**: `src/app/root_view.rs` for the outline click route, `src/app/application.rs` for outline navigation orchestration, and `src/app/preview.rs` for source-offset-to-preview-heading lookup; focused coverage belongs in `src/app/tests.rs`.
- **User experience**: The outline becomes functional as navigation in Read mode, with no intended behavior change in Edit, Visual Edit, or Split Preview modes.
- **Architecture**: The change reads the existing per-version `Arc<Vec<PreviewBlock>>` and drives the existing preview `ListState`; it does not add Markdown parsing, document mutation, history entries, or a second derived-state cache.
