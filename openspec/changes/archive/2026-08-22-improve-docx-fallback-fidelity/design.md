## Context

The built-in DOCX fallback (`src/export.rs`) hand-writes a stored-only ZIP with four parts. `render_docx_block` walks cached `PreviewBlock`s; `docx_paragraph` emits one unstyled run per paragraph, ignoring `RichText.spans` (which already carries `InlineStyle` flags and link targets) and `ListItem.level`. The writer has no `styles.xml`, so its `Heading1..4` `w:pStyle` references are dangling.

## Approach

Keep the dependency-free hand-written OOXML approach (no new crates) but restructure it into a tiny package builder:

```
write_docx(blocks, metadata)
  → DocxPackage::new()
      .part("[Content_Types].xml", content_types(...))
      .part("_rels/.rels", ROOT_RELS)
      .part("docProps/core.xml", core_props(metadata))
      .part("word/styles.xml", STYLES_XML)          // static template
      .part("word/numbering.xml", NUMBERING_XML)    // static template
      .part("word/settings.xml", SETTINGS_XML)
      .part("word/fontTable.xml", FONT_TABLE_XML)
      .part("word/theme/theme1.xml", THEME_XML)
      .part("word/document.xml", document_xml)      // built by render pass
      .part("word/_rels/document.xml.rels", doc_rels) // from render pass
  → stored ZIP (unchanged writer + CRC32)
```

Key decisions:

- **Static XML templates** for styles/numbering/theme/settings/fontTable as `const &str` — they are fixed boilerplate; only `document.xml`, `document.xml.rels`, and `[Content_Types].xml` vary per document. This keeps the change small and reviewable.
- **Relationship collection during render**: the document render pass owns a `Vec<(rId, target)>` for external hyperlinks; `document.xml.rels` is built after the pass. rIds start after the implicit style/numbering relationships.
- **Run generation from spans**: `RichText.spans` is authoritative; the plain `text` field is only a fallback when spans are empty. Style flags compose into one `w:rPr` per run.
- **Numbering**: two abstract definitions (bullet, decimal) with 9 levels each. Each contiguous ordered-list group gets a fresh concrete `w:num` so numbering restarts per list; bullets share one `numId` (levels carry the depth).
- **Fonts**: docDefaults carry `w:ascii/hAnsi` (e.g. Calibri) plus `w:eastAsia` (e.g. DengXian/等线); headings get an eastAsia heading face (e.g. Microsoft YaHei/微软雅黑); code gets Consolas + eastAsia fallback. Chosen faces are Windows-standard, degrading gracefully elsewhere.
- **Page setup**: A4 (11906×16838 twips), 1440-twip margins, in named constants so `add-docx-export-options` can later parameterize without another refactor.

## Invariants

- Export consumes the cached per-version preview blocks; nothing recomputes Markdown state.
- No new dependencies; the ZIP writer stays stored-only (deflate is deferred to the content-fidelity change).
- The pandoc engine path is untouched; both backends keep the disclosure behavior already specced.
