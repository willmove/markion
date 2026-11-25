## Why

DOCX runs whose bold or italic text begins or ends with whitespace currently produce non-flanking CommonMark delimiters, so Markion displays literal asterisks instead of the authored formatting. This is a reproducible import-fidelity regression affecting ordinary Word and LibreOffice documents (GitHub Issue #39).

## What Changes

- Serialize leading and trailing run-edge whitespace outside the complete inline formatting wrapper while preserving the source characters and their order.
- Apply the boundary handling consistently to bold, italic, bold-italic, and other active inline wrappers.
- Preserve the existing source-aware bold compatibility-space behavior and add parser-level regression coverage.

Non-goals: changing DOCX style resolution, normalizing interior whitespace, or redesigning the import API and preview pipeline.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `docx-import`: require imported inline formatting to keep edge whitespace outside Markdown emphasis delimiters so the authored style remains interoperable.

## Impact

Affected code is limited to run serialization in `crates/docx-import/src/render.rs` and its tests in `crates/docx-import/src/tests.rs`. The public `import_docx` and `PreparedImport` APIs, dependencies, UI, persistence, and cached-per-document-version Markdown architecture remain unchanged.
