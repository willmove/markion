## Why

Read and Visual Edit use different inline HTML semantics, so colors, HTML links, code-like tags and harmless wrappers disappear or expose source depending on context. Shared HTML blocks and cells also omit styles and collapse authored breaks, while super/subscript styling does not position glyphs.

## What Changes

- Share inline HTML semantics across mixed Markdown, HTML blocks, table cells and Visual Edit: color spans/font, anchors (including linked images), kbd/samp aliases, neutral span wrappers, and harmless attributes.
- Preserve exact source groups, literal escaped markup, malformed-element fallback, link navigation, and nested style restoration.
- Keep every authored HTML break and render super/subscripts at reduced size with a shifted baseline in all rendered surfaces.
- Add cross-context parity, projection, paint/layout and edit-history tests and update the quality matrix.

Non-goals: browser CSS/layout, script execution, external stylesheets and arbitrary DOM mutation. This extends the completed `fix-visual-inline-html-underline` change without rewriting its history.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Consistent source-backed inline HTML across rendered modes and contexts.
- `document-typography`: Actual super/subscript size and baseline geometry.
- `html-table-rendering`: Inline style and break parity in HTML cells.

## Impact

Root parser/model/visual derivation, preview and visual GPUI text layout, tests and documentation. No new dependencies or file-format changes. Derived state stays cached per document version; interaction and typography changes do not reparse or mutate source.
