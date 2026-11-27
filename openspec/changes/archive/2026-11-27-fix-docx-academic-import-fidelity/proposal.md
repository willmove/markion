## Why

Word imports of academic documents can expose bookmark markup, lose mathematical structure, and break emphasis when Word splits a styled phrase into multiple runs. This makes the imported document harder to read and edit.

## What Changes

- Map heading bookmarks to heading identifiers and retain additional bookmark targets.
- Preserve emphasis across Word runs and at Markdown delimiter boundaries.
- Convert the math structures and notation used in the reference document to renderable LaTeX, with explicit diagnostics for unsupported equations.
- Retain quote styling, thematic breaks, and consistent table alignment. Only report endnote loss when the document has endnote references.
- Keep formula-heavy documents stable by scheduling native formula rendering near the current viewport.
- Include a readable summary of heading formulas in the outline.

Non-goals include reproducing Word pagination and changing the Markdown derivation cache.

## Capabilities

### New Capabilities

### Modified Capabilities

- `docx-import` covers semantic headings, emphasis, and math fidelity.
- `code-and-math` covers viewport-scoped formula rendering.
- `tables-outline` covers math in heading labels and alignment in formula cells.

## Impact

The native DOCX importer and its end-to-end editor regressions. The importer remains independent of GPUI and uses the existing Markdown rendering pipeline.
