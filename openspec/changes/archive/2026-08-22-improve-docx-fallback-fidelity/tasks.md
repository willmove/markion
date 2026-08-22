## 1. OOXML package builder and static parts

- [x] 1.1 Restructure `write_docx` (`src/export.rs:109-141`) into a small package builder: part list of `(path, bytes)` pairs feeding the existing stored-ZIP writer; add `word/_rels/document.xml.rels` (hyperlink relationships), `word/theme/theme1.xml`, `word/settings.xml`, `word/fontTable.xml` as embedded static templates
- [x] 1.2 Author `word/styles.xml` template: `docDefaults` (body font + `w:eastAsia`, size 10.5pt/21 half-points), Normal, Title, Heading1–6 (with eastAsia heading font, sizes, bold, spacing, `w:keepNext`), Quote (indent + italic or left border), Code/Verbatim character+paragraph style; verify every styleId referenced later exists here
- [x] 1.3 Update `[Content_Types].xml` and `_rels/.rels` for the new parts; test: package contains all parts and every `w:pStyle` in `word/document.xml` resolves to a style definition (scan both parts in one test)

## 2. Inline runs from RichText.spans

- [x] 2.1 Replace `docx_paragraph`'s single plain run (`src/export.rs:319-332`) with a run builder that walks `RichText.spans`: map each `InlineStyle` flag to `w:rPr` (`w:b`, `w:i`, `w:strike`, `w:highlight`, `w:vertAlign` superscript/subscript, monospace `w:rFonts` for code), composing multiple styles on one run
- [x] 2.2 Hyperlinks: emit `w:hyperlink` runs with `r:id` entries in `word/_rels/document.xml.rels` (deduplicated relationship table collected during render); fall back to styled text when the target is empty
- [x] 2.3 Escape audit: XML-escape run text and attribute values (URLs); tests for `<`, `&`, quotes in text and hrefs
- [x] 2.4 Tests: a paragraph exercising every inline style plus a link asserts the expected `w:rPr` properties and the hyperlink relationship target

## 3. Real lists

- [x] 3.1 Add `word/numbering.xml` template: abstract bullet numbering (•, ◦, ▪ per level) and abstract decimal numbering, each with 9 `w:lvl` entries and increasing `w:ind`
- [x] 3.2 In `render_docx_block`'s ListItem arm (`src/export.rs:215-228`), emit `w:numPr` (`w:ilvl` from the already-captured `level`, `w:numId` for bullet vs decimal) instead of literal `- `/`1. ` prefixes; task items keep their `[x]`/`[ ]` text prefix on a bullet-less paragraph or dedicated numbering
- [x] 3.3 Track list runs so consecutive items of the same kind share a `numId` and a fresh ordered list restarts numbering (new `numId` per ordered list group)
- [x] 3.4 Tests: nested two-level bullet list asserts distinct `w:ilvl` values; ordered list asserts `w:numFmt val="decimal"` and no literal `1. ` marker text

## 4. Headings, title, and page setup

- [x] 4.1 Map H1–H6 to Heading1–6 (remove the H4–H6 collapse at `src/export.rs:206-211`); emit a Title-styled paragraph from front matter title before the body
- [x] 4.2 Change `w:sectPr` (`src/export.rs:199`) to A4 (11906×16838 twips) with 1440-twip margins; keep the dimensions/margins in named constants for the later options change
- [x] 4.3 Tests: H4/H5/H6 produce Heading4/5/6 respectively; `sectPr` asserts A4 dimensions

## 5. Verification

- [x] 5.1 Extend the `src/lib.rs` write_docx tests: full part inventory, style-reference resolution, inline fidelity, lists, headings, page setup; `src/export.rs` gains its own unit tests
- [x] 5.2 Manual smoke: open a generated file (mixed CJK, all inline styles, nested lists, H1–H6) in Word and WPS and confirm headings/styles/lists render — verified 2026-08-22 in Word 16.0 via COM automation (open + PDF export, 50 paragraphs, headings/styles applied); WPS not installed on this machine. The smoke exposed a corrupt hand-rolled `theme1.xml` (under-populated `bgFillStyleLst`); fixed by embedding pandoc's Office theme verbatim, with a regression test (`theme_part_carries_complete_style_matrix`)
- [x] 5.3 `cargo test` (root) and `cargo test --workspace` pass; build warning-free under `-D warnings`
- [x] 5.4 `openspec validate improve-docx-fallback-fidelity` passes
