## Context

Builds on `improve-docx-fallback-fidelity` (P0), which establishes the package builder, static style/numbering parts, inline run generation from `RichText.spans`, and real lists. This change fills the remaining content-fidelity gaps: images, tables, math, footnotes, rules, alerts, and compression.

## Decisions

- **Image sizing without decoding**: only PNG and JPEG are embedded (read dimensions from headers: IHDR width/height for PNG; scan SOF0/SOF2 markers for JPEG). Other formats (gif/svg/webp) keep the text fallback — SVG especially cannot be embedded as-is in DOCX without a fallback bitmap. EMU math: 914400 EMU/inch, 96 DPI assumed; text column width = page width − margins, from the P0 page-setup constants.
- **Tables**: `PreviewBlock::Table` already carries `alignments`; cells reuse the P0 inline run builder. HTML tables reuse the existing lightweight HTML table parsing added for rowspan/colspan handling; the parsed grid feeds the same `w:tbl` renderer so both sources share one code path.
- **Math → OMML**: the realistic options were (a) a TeX→MathML crate plus a Rust port of the MML2OMML subset, or (b) a direct emitter for a supported TeX subset. **Chosen: (b).** Evaluation: the TeX stack already in the workspace (`ratex-*`, via `typune-markdown`) renders to SVG only and offers no MathML output; no maintained Rust TeX→MathML crate exists that would justify a new dependency plus an MML2OMML port. The direct emitter (`tex_to_omml` in `src/math.rs`) covers `\frac`/`\dfrac`/`\tfrac`, `\sqrt` (with optional `[n]` degree), `^`/`_` scripts, n-ary operators with limits (`\sum`, `\prod`, `\int`, `\oint`, `\bigcup`, `\bigcap`), the greek alphabet, common operator/relation symbols, `\left`/`\right`/`\big*` delimiters, and `\text`-style grouping commands. Unsupported constructs degrade to the authored LaTeX as the math-zone text (`m:oMath` containing a literal run), never to Unicode approximation.
- **Footnotes**: definitions are collected during the render pass (like hyperlink relationships in P0) and `footnotes.xml` is built after the pass. Separator/continuationSeparator footnotes with ids −1/0 precede real footnotes starting at id 1.
- **Compression**: `miniz_oxide` is already in the dependency graph (via transitive deps); adding it as a direct root dependency is the smallest path to deflate without hand-writing a compressor. Central-directory records must carry matching CRC/sizes either way.

## Invariants

- Fallback stays GPUI-free; the pandoc engine path is untouched.
- Export still consumes cached per-version preview blocks; image file reads happen once per export, not per render.
