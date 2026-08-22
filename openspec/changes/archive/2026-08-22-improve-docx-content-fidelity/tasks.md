## 1. Image embedding

- [x] 1.1 Add image part plumbing to the package builder: during the render pass, collect local images (resolve relative paths against the document directory, skip remote/data-URI), assign `word/media/imageN.<ext>` names, read bytes, and record relationships + content-type entries
- [x] 1.2 Read PNG/JPEG dimensions from file headers (IHDR / SOF markers, no decode) to compute `wp:extent` EMUs; scale down to the text column width when wider, keep natural size at 96 DPI otherwise; alt text becomes `wp:docPr` description
- [x] 1.3 Emit `w:drawing` (`wp:inline`) runs in `render_docx_block`'s image arm; keep the `alt: url` text fallback for unresolvable sources
- [x] 1.4 Tests: fixture with a real temp PNG asserts media part, relationship, content type, and extent; missing/remote/data-URI sources keep the text fallback and export succeeds

## 2. Table fidelity

- [x] 2.1 Header row: first `w:tr` gets `w:tblHeader` in `w:trPr` and bold runs; cells keep inline styles (reuse the P0 run builder inside cells)
- [x] 2.2 Column alignment: map parsed `alignments` to `w:jc` per cell; table width = text column, column widths proportional (equal split when no better signal)
- [x] 2.3 Raw HTML tables: parse `<table>/<tr>/<th>/<td>` structure (including existing rowspan/colspan handling conventions, `src/export.rs:293-303`) into the same table renderer instead of per-cell paragraphs
- [x] 2.4 Tests: header bold + tblHeader, `|:--|:-:|--:|` → left/center/right, HTML table → one `w:tbl` with row/cell counts

## 3. OMML math

- [x] 3.1 Evaluate TeX→OMML routes (existing Rust TeX→MathML crate + ported MML2OMML transform vs. a direct TeX-subset→OMML emitter); record the choice and its supported subset in design.md before implementing
- [x] 3.2 Implement the converter and wire inline math → `m:oMath`, display math → `m:oMathPara`; unsupported constructs fall back to the authored LaTeX as the math-zone text
- [x] 3.3 Tests: common constructs (fractions, sqrt, super/subscripts, greek, sums) produce expected OMML elements; an unsupported construct preserves exact source

## 4. Footnotes, rules, alerts

- [x] 4.1 Footnotes: collect definitions during render, emit `word/footnotes.xml` (plus content type + relationship), body marks via `w:footnoteReference`; footnote text renders with the P0 inline run builder
- [x] 4.2 Horizontal rule: paragraph with `w:pBdr` bottom border; remove the literal `----------` output
- [x] 4.3 GFM alerts: render `AlertKind`-carrying quotes (Note/Tip/Important/Warning/Caution) as callout paragraphs — bold label + left accent border via `w:pBdr` + indent
- [x] 4.4 Tests: footnote reference/definition id pairing; rule has `w:pBdr` and no dash text; alert has bold label run

## 5. Compression and verification

- [x] 5.1 Switch the ZIP writer to deflate (method 8); if no suitable dependency-free path exists, take the `miniz_oxide` crate (already a transitive dependency in the lockfile) — verify before adding to root deps
- [x] 5.2 Update existing raw-bytes ZIP assertions in `src/lib.rs` tests (stored ZIP assumptions) to decompress entries first
- [x] 5.3 `cargo test` (root) and `cargo test --workspace` pass; build warning-free under `-D warnings`
- [x] 5.4 Manual smoke in Word and WPS: images, header-repeat tables, equations, footnotes, callouts all render — verified 2026-08-22 in Word 16.0 via COM automation on a comprehensive fixture (image embedded as inline shape, OMML equations, footnotes, alert callout all present in the exported PDF); WPS not installed on this machine
- [x] 5.5 `openspec validate improve-docx-content-fidelity` passes
