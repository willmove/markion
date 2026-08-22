## Why

When pandoc is not installed, DOCX export falls back to the built-in writer in `src/export.rs`, which emits a structurally broken OOXML package: it references `Heading1..4` paragraph styles but ships no `styles.xml`, so Word renders headings as plain body text; all inline formatting (bold, italic, strikethrough, code, highlight, links) is flattened away even though `RichText.spans` carries it; lists are literal `- `/`1. ` text with nesting discarded; headings H4–H6 collapse into Heading4; there is no CJK font declaration; and the page setup is hard-coded US Letter. The result is a document with no skeleton and no formatting — far below what mainstream Markdown editors (Typora, Obsidian, VS Code extensions) produce without external dependencies.

## What Changes

- Emit a complete OOXML package from the built-in writer: add `word/styles.xml` (docDefaults plus Normal, Title, Heading1–6, Quote, and code styles), `word/theme/theme1.xml`, `word/settings.xml`, and `word/fontTable.xml` alongside the existing parts, so every `w:pStyle` reference resolves.
- Consume `RichText.spans` (which already carries `InlineStyle` and link targets) to emit per-run `w:rPr`: bold, italic, strikethrough, inline code (monospace font), highlight (`w:highlight`), superscript/subscript (`w:vertAlign`), and real `w:hyperlink` relationships for links instead of dropping URLs.
- Emit `word/numbering.xml` with abstract bullet and decimal numbering definitions, and render list items as real numbered/bulleted paragraphs (`w:numPr` with `w:ilvl`) so nested lists keep their depth and stay editable in Word.
- Give H1–H6 six distinct heading styles (no more H4–H6 collapse) and a document Title paragraph when front matter provides a title.
- Declare CJK-capable fonts via `w:rFonts` `w:eastAsia` in docDefaults and heading styles (body, heading, and code each get a sensible east-asian default), so Chinese text no longer depends on Word's fallback heuristics.
- Change the default page setup from hard-coded US Letter to A4 with configurable margins (default 2.54 cm), matching CJK-region printing conventions.
- Strengthen tests: assert the package contains all required parts, that style references resolve, and that inline styles, hyperlinks, and nested lists appear in `word/document.xml`.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `export`: the built-in DOCX fallback gains a requirement covering package completeness (styles/numbering/theme/settings parts), inline-style and hyperlink fidelity, structural nested lists, six heading levels, CJK font declarations, and A4-default page setup.

## Impact

- `src/export.rs`: the `write_docx`/`render_docx_block`/`docx_paragraph` path is reworked into a small OOXML package builder; inline runs are generated from `RichText.spans`; numbering and style parts are embedded as static XML templates.
- `src/model.rs`: no shape changes expected — the existing `PreviewBlock`/`RichText`/`InlineStyle` data is sufficient (verified: `ListItem.level` and link targets are already captured but currently unused).
- `src/lib.rs`: existing `write_docx` tests extended; new package-structure and inline-fidelity tests.
- Invariants preserved: the fallback stays GPUI-free and dependency-free (still hand-written stored ZIP; no new crates), and export continues to consume the cached per-version preview blocks without recomputation.

Non-goals: image embedding, table header/alignment styling, OMML math, real footnotes (tracked by `improve-docx-content-fidelity`); pandoc engine options (tracked by `improve-docx-engine-pipeline`); export-options UI (tracked by `add-docx-export-options`); ZIP deflate compression.
