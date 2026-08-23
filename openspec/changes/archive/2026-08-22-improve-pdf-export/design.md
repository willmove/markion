# Design — improve-pdf-export

## Context

PDF export currently has two degraded paths. The pandoc engine (`crates/export/src/pdf.rs`) assumes a XeLaTeX install, configures no CJK fonts (Chinese glyphs silently missing), omits `--resource-path` (relative images lost), and passes an inert `--katex` flag. The built-in fallback (`write_pdf`, `src/export.rs:226`) writes a single hand-rolled page: Helvetica base-14 only, every non-ASCII byte replaced by `?` (`plain_pdf_text`), the document truncated to 52 wrapped lines of `plain_text_preview()` — no structure, no metadata, no fonts. The `export` spec encodes this as "deliberately limited".

Research (2025–2026 ecosystem) found three viable architectures for a Rust app without a bundled Chromium: embed the Typst compiler as crates, shell out to a bundled `typst` CLI, or build layout on `krilla` + `cosmic-text`. **The project owner selected the third**: binary size (+2–5 MB vs. +20–40 MB and a ~1M-SLoC dependency tree for Typst) and preview consistency (math and code highlighting can reuse the exact renderers the preview uses, rather than a foreign typesetter's interpretation) outweigh the cost of owning block layout and pagination. genpdf (dormant, rusttype cannot parse CFF/TTC CJK fonts) and printpdf (API churn, shipped regressions) remain rejected regardless of this decision.

## Goals / Non-Goals

**Goals:**
- A built-in PDF writer that produces paginated, font-embedded, structurally rich PDFs with zero external tools, for Latin and CJK documents alike.
- Reuse the cached `preview_blocks()` pipeline (same source the built-in DOCX writer consumes) so PDF fidelity tracks the preview; reuse the preview's math SVG renderer and the editor's memoized syntax highlighting so the PDF *looks like* the preview.
- Keep the pandoc engine as an optional alternative backend, with its CJK/image/dead-flag defects fixed.
- Config-driven PDF options (`[export.pdf]`) honored by the built-in writer and mapped onto pandoc variables.

**Non-Goals:**
- No graphical PDF options dialog (follow-up change; this one is config-driven).
- No bundled Chromium / HTML-print pipeline; PNG/JPEG snapshot fidelity unchanged.
- No remote-image fetching (remote/data-URI images keep the text fallback, as DOCX does).
- Diagram fences keep their code-block behavior in PDF (the SVG-embed mechanism built for math makes graphical diagrams an easy follow-up, but it is out of scope here).
- No advanced CJK book typography (vertical writing, punctuation hanging); pan-CJK coverage beyond the bundled SC subset + system fonts.
- No tagged-PDF/PDF-A conformance claims (krilla supports them; we do not commit to them in this change).

## Decisions

### D1 — Self-built layout on `krilla` + `cosmic-text` (owner decision)

- **krilla** (by the author of typst-pdf, which is built on it) provides the PDF layer: CFF+TTF font embedding with per-document subsetting, images, vector fills/strokes, link annotations, document outline (bookmarks), and document metadata. It explicitly excludes text layout, pagination, tables, and headers/footers — those are ours.
- **cosmic-text** (System76; also GPUI's Linux text stack, already in our dependency closure via vendored GPUI) provides the text layer: `fontdb` system-font discovery, `harfrust` shaping, bidi, and UAX#14-aware wrapping with per-span font fallback. We do not hand-roll shaping or line breaking; our code is block layout + pagination + emission only.

Alternatives rejected: embedded Typst crate and bundled typst CLI (owner decision — binary size and preview consistency; recorded here so the trade-off is not re-litigated: Typst would have given free TOC/footnotes/tagged PDF at the cost of +20–40 MB and a second typesetting system whose defaults differ from Markion's preview). genpdf/printpdf rejected per Context.

### D2 — Crate split: root renders an IR, member crate lays out and emits

New GPUI-free member crate `crates/pdf` (package `markion-pdf`) owns fonts, layout, pagination, and krilla emission. The root crate converts the cached `preview_blocks()` into a **layout IR** — plain data (block kinds, styled runs, image references, table grids, link targets) with no gpui types — mirroring how `render_docx_document_xml` walks the same blocks. Unlike the previously considered Typst-markup plan, the IR is structured data, so there is no markup-escaping/injection surface at all. Both sides are unit-testable: IR construction as pure functions in the root crate; layout/emission against fixture IRs in the member crate. Public entry: `markion_pdf::render(ir, options) -> Result<Vec<u8>, PdfError>`.

### D3 — Font strategy: bundled fallbacks + one shared `fontdb`

- Bundle via `include_bytes!` in the member crate: **Noto Sans SC subset** (OFL; common-use Han + punctuation, ≈1–2 MB) and **Liberation Serif/Sans/Mono** (OFL; full Latin coverage) as guaranteed fallbacks, registered into the `fontdb::Database` at init. `include_bytes!` (not `assets/` resource lookup) keeps dev and packaged builds identical and avoids packager changes.
- System fonts are discovered once per process into the same fontdb (behind a `OnceLock`; export-path only, no typing-path cost), so per-OS CJK faces (Microsoft YaHei, PingFang SC, Noto Sans CJK SC) are preferred when present.
- cosmic-text performs per-glyph fallback through the fontdb automatically; krilla subsets and embeds exactly the glyphs used, so output PDFs stay small.
- Alternatives considered: full Noto CJK (~16 MB/weight — too large); system fonts only (fails on minimal Linux); rusttype/genpdf (cannot parse CFF/TTC CJK fonts).

### D4 — Math: reuse the preview SVG renderer, embed via `krilla-svg`

Inline/display math spans carry authored LaTeX and the preview already renders them through a GPUI-free math renderer to sanitized, self-contained SVG (the same one HTML export uses). The PDF writer embeds that SVG as vector graphics via `krilla-svg`, giving byte-for-byte preview consistency with zero new conversion dependency. On renderer failure the writer emits the byte-identical authored LaTeX in a code-styled block and export succeeds — parity with the DOCX OMML fallback contract. Rejected alternative: LaTeX→typesetter-native math conversion (e.g. tex2typst under the Typst plan) — selectable text, but a second math interpretation that can diverge from what the user sees, plus a spike risk.

### D5 — Options model and document "theme"

`PdfExportOptions { page_size: A4|Letter|Legal, margin_mm: u32 (default 25), toc: bool (default false), page_numbers: bool (default true) }` persisted under `[export.pdf]`, following the `DocxExportOptions` pattern. The built-in writer applies all four (TOC is generated from the headings collected during layout, rendered as an opening contents page with dot leaders and page numbers; the footer draws page numbers per page). The pandoc path maps them to `--variable=geometry:`/`--toc`. Visual styling (heading scale, spacing, table borders, alert accent colors, code highlight palette) lives in a constants module in the member crate, seeded from the editor's light-theme palette so exports are print-friendly regardless of the editing theme.

### D6 — Export flow: pandoc engine first (fixed), then the built-in writer

Order is unchanged (pandoc attempt → built-in) so `[export] pdf_engine` keeps its meaning. What changes: the built-in is now the rich writer, so status-bar copy is reworked — the built-in PDF path is disclosed neutrally (no "installing pandoc yields richer output" hint for PDF; DOCX keeps its hint), and the engine-failure category disclosure is retained for diagnosability. The hand-rolled `write_pdf`, `plain_pdf_text`, `wrap_text`, and the PDF use of `plain_text_preview()` are deleted.

Pandoc invocation fixes (`crates/export/src/pdf.rs`): add `-V CJKmainfont=<platform default>` when the document contains CJK (with config overrides), add `--resource-path <document dir>` (parity with DOCX), remove `--katex` (HTML-only flag), and honor the configured page size and TOC.

### D7 — Layout and pagination algorithm (the part we own)

Single-pass measure-as-you-place over the IR with explicit keep rules:

- **Text**: one cosmic-text `Buffer` per paragraph/run sequence (spans → attrs: weight, style, family, color, background for highlight, baseline offset for super/subscript, underline+color for links); wrapping and mixed CJK/Latin breaking come from cosmic-text.
- **Blocks**: headings keep-with-next; list items carry generated markers (bullets, auto numbers, task checkboxes) at computed indents; quotes/alerts render as indented blocks with an accent rule; code blocks keep-together when they fit one page, else split at line boundaries; rules are vector strokes.
- **Tables**: column widths from the separator-row proportions fitted to the text column; the header row repeats on page continuation; rows split only between rows.
- **Footnotes**: a per-page footnote area at the bottom; notes whose reference lands on a page are placed there, overflow carries to the next page's area; references render as superscript numbers linked to the note.
- **Images**: local PNG/JPEG embedded via krilla, SVG via krilla-svg; wider than the text column → scaled down proportionally; alt text as the accessibility description.
- **Links/bookmarks/metadata**: krilla link annotations per link run, an outline entry per heading with hierarchy, and front-matter title/author/date into document properties.

### Data flow and caching (per project invariants)

```
Export PDF action → save dialog → MarkdownDocument::export_to_with(Pdf, prefs)
  ├─ pandoc engine (fixed args; unchanged trait) → bytes
  └─ built-in: preview_blocks_shared()          // cached per version — no re-parse
       → build_pdf_ir(blocks, options)          // root crate, pure data
         · math spans → preview math SVG renderer (same as HTML export)
         · code blocks → memoized syntax-highlight colors
       → markion_pdf::render(ir, options)
           fonts: OnceLock fontdb (system scan + bundled subsets)
           text: cosmic-text buffers → shaped glyph runs
           emit: krilla (glyphs, images, vectors, links, outline, metadata)
       → bytes → atomic write
```

Export is a one-shot user action: it reads the existing per-version `Arc` block cache and adds no per-keystroke or per-frame work; the memoized-highlighting and text-handle invariants are untouched. Image bytes and the fontdb scan are read once per export / once per process respectively.

## Risks / Trade-offs

- **We own layout correctness**: pagination edge cases (keep-with-next loops, footnote overflow, table splitting) are our bugs. → Staged implementation (text+pagination first, tables/footnotes last), golden-fixture tests per block kind, and manual viewer verification on all three OSes before release.
- **krilla/cosmic-text API specifics** (glyph emission, link annotations, outline API, SVG fidelity for the math renderer's output) are validated but not yet exercised by us. → Implementation-start spike task; both crates are pinned to exact versions; krilla is battle-tested through typst-pdf and maintains the best cross-viewer test matrix in the Rust PDF space.
- **Footnote area overflow** (many long notes referenced on one page) is the classic hard case. → Carry-forward algorithm (D7); if it proves unstable in testing, fall back to rendering notes as an endnotes section and amend the spec scenario before archive.
- **Pan-CJK gaps**: the bundled subset covers SC; TC/JP/KR-only glyphs on a font-poor system could still tofu. → System fonts are ordered before bundled subsets; documented non-goal.
- **Pandoc flag changes** alter output for existing xelatex users. → CJKmainfont only applied when the document contains CJK or the user configured a font; additive and documented in release notes.
- **Effort vs. Typst** (the rejected option): more of our own code, longer to first rich output. → Accepted by the owner for binary size and preview consistency; scope is protected by the non-goals list.

## Migration Plan

Purely additive for users: new `[export.pdf]` section with defaults; no config migration, no behavior change for non-PDF formats. Rollback = revert the change; the pandoc path remains functional throughout. Release notes must state the new built-in engine, the removed 52-line/`?`-substitution writer, and the bundled-font license notices.

## Open Questions

1. krilla API spike results → **Settled** (task 2.2, `crates/pdf/tests/spike.rs`, passing): at krilla 0.8.2, pre-shaped cosmic-text glyphs emit via `Surface::draw_glyphs` + `KrillaGlyph` (advances normalized per em; TTC face index comes from fontdb, not cosmic's font id); links via `Page::add_annotation(Annotation::new_link(LinkAnnotation::new(rect, Target::Action(…))))`; bookmarks via `Document::set_outline(Outline/OutlineNode)`; metadata via `Document::set_metadata(Metadata builder)`. krilla-svg 0.8.1 renders path-only SVG — the math renderer's only allowed shape (`is_self_contained_svg` rejects `<text`) — with no fonts; SVG `<text>` requires a populated `usvg::Options::fontdb` and `SvgSettings::embed_text` toggles embedded text vs. outlined paths.
2. Footnote overflow stability (D7 risk) — settled by fixture tests; endnote contingency only if needed.
3. Exact glyph set for the bundled SC subset (common-use set vs. larger 通用规范汉字表 subset) — decided in the font task by measured size.
4. Whether to advertise `pdf_engine = "typst"` as the recommended pandoc engine for users who have pandoc but not TeX (docs-only decision).
