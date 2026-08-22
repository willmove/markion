## Why

Even after the fallback package gains styles, lists, and inline formatting (`improve-docx-fallback-fidelity`), the built-in DOCX writer still loses content that mainstream exports preserve: images degrade to `alt: url` text, tables ignore header rows and column alignment, math degrades to a `Math: `-prefixed Unicode approximation, footnotes are plain `[label] text` paragraphs disconnected from their references, horizontal rules are literal `----------` text, GFM alerts lose their callout semantics, and raw HTML tables disintegrate into scattered paragraphs. These are the gaps users notice most when sharing documents with Word users.

## What Changes

- Embed local images into the package: resolve relative image paths against the document's directory, copy the bytes into `word/media/`, size the drawing to fit the page width, and declare the image relationships and content types. Remote (`http(s)`) and data-URI images keep the text fallback.
- Tables: bold header row with `w:tblHeader` (repeat-on-page-break), per-column alignment (`w:jc`) from the parsed separator row, inline styles inside cells, and table width sized to the page instead of fixed 2400-dxa columns.
- Raw HTML `<table>` blocks parse into the same real table structure instead of one scattered paragraph per cell.
- Math: convert inline and display math to OMML (`m:oMath`) so equations open as native, editable Word equations; when conversion is not possible, preserve the authored LaTeX source as the math-zone text instead of the current Unicode approximation with a `Math: ` prefix.
- Real footnotes: emit `word/footnotes.xml` with `w:footnoteReference` marks in the body so references and definitions stay linked.
- Horizontal rules render as a paragraph bottom border; GFM alerts render as styled callout paragraphs (accented left border + bold kind label) instead of `> ` text.
- Switch the ZIP writer from stored-only to deflate compression.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `export`: the built-in DOCX fallback gains requirements for image embedding, table header/alignment fidelity (including raw HTML tables), OMML math, linked footnotes, horizontal-rule and GFM-alert rendering, and compressed packaging.

## Impact

- `src/export.rs`: image resolution/embedding, table rendering rework, OMML math emission, footnotes part, alert/rule styling, deflate compression.
- Image decoding: PNG/JPEG dimensions are read from file headers directly (no decode); no new image crate is needed for sizing.
- Math: the design evaluates a TeX→MathML step plus an MML→OMML transform (XSLT ported to Rust or a small direct TeX→OMML subset); the chosen approach is recorded in design.md after evaluation.
- Invariants preserved: fallback stays GPUI-free; export still consumes cached preview blocks; the pandoc engine path is untouched.

Non-goals: syntax-highlighted code shading, reference-doc/TOC on the pandoc path (tracked by `improve-docx-engine-pipeline`), export-options UI (tracked by `add-docx-export-options`), remote/data-URI image embedding, chart/diagram rendering into DOCX.
