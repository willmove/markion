## Why

The `WYSIWYG coverage roadmap` tracks GFM-style definition lists as an open gap: the parser option is off, so `Term` followed by `: Definition` renders as one line of literal prose ("Term : Definition") in every surface. Definition lists are common in glossaries, API docs, and notes imported from other Markdown tools (Obsidian, Pandoc, PHP Markdown Extra). This change closes the gap, as the roadmap requires.

## What Changes

- Enable pulldown-cmark's definition-list extension for preview, Read, Visual Edit, and built-in HTML export.
- New preview blocks `DefinitionTerm` and `DefinitionDetail`. A definition's inline content, direct or from its paragraphs, is flattened into one detail. A nested non-paragraph block ends the detail and renders as an ordinary block after it, so block order and byte ownership stay exact.
- Split Preview and Read mode render terms in bold and definitions indented.
- Visual Edit rows of the same kinds. The `:` marker is a block prefix (hidden, then revealed with the caret), the same mechanism as list markers.
- Built-in exports: HTML emits `<dl>/<dt>/<dd>`; DOCX, PDF, and LaTeX render bold terms and following definitions (indented in DOCX and LaTeX).
- The coverage matrix and roadmap move definition lists out of the gap class.

Compatibility: a line starting with `:` directly after a paragraph line now forms a definition list where it used to stay prose. This matches Pandoc, PHP Markdown Extra, and Obsidian. No stored data changes.

Non-goals: definition lists nested inside list items (the list item keeps owning that text, as before); a dedicated "insert definition list" command; Pandoc-engine exports, which pass the Markdown to pandoc unchanged.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: adds "Definition lists SHALL render in preview, Read, and Visual Edit"; updates "Maintained Visual Edit support classification" and "WYSIWYG coverage roadmap".
- `export`: adds "Built-in exports render definition lists".

## Impact

- Code: `src/parse.rs` (option, span helper), `src/lib.rs` (preview-block builder, LaTeX), `src/model.rs` (preview/visual kinds, `DefinitionMarker` prefix, `RichText::emboldened`), `src/visual.rs` (kind mapping, prefix), `src/source_mapped.rs`, `src/document_memory.rs`, `src/export.rs` (PDF, DOCX), `src/app/preview.rs` (preview and Visual Edit rows), `docs/visual-editing-quality.md`, tests.
- Invariants: blocks are produced in the same single parse pass and cached per version like other blocks. Visual Edit edits stay single exact source mutations.
