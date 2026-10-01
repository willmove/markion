## Why

Rich-text paste already converts clipboard HTML to Markdown, but spreadsheet copies (Excel, Google Sheets, WPS, Calc) still land as a blank GFM header row, a raw HTML island when any cell is merged, or tab-separated plain text when HTML is missing or not UTF-8. Word list paste is similarly incomplete: Word often emits indented `MsoListParagraph` runs instead of real `<ul>`/`<ol>`, so structure is lost, and there is no "paste as plain text" escape hatch when conversion is wrong.

## What Changes

- Treat spreadsheet-like clipboard payloads as grids: convert rectangular TSV (`CF_UNICODETEXT` / `text/plain` with tabs) and headerless HTML tables into GFM tables whose first row is the header; a 1×1 grid pastes as ordinary text.
- Flatten `colspan`/`rowspan` into a rectangular GFM table (value in the origin cell, remaining spanned cells empty) instead of serializing the whole table as raw HTML, so Visual Edit keeps cell editing and row/column toolbars.
- Add Excel / Google Sheets / WPS-shaped HTML+TSV fixtures so conversion is locked to real clipboard dialects, not only hand-written `<th>` tables.
- Reconstruct Word-style fake lists (`MsoListParagraph`, `mso-list` markers and indent levels) into nested Markdown lists.
- When clipboard HTML is not valid UTF-8 (typical GBK CF_HTML), drop the HTML flavor and continue through the Unicode plain-text path, including TSV→GFM.
- Add Edit → Paste as Plain Text (`Ctrl+Alt+V` / `Cmd+Option+V`, customizable) that inserts the clipboard's Unicode text with no HTML/TSV conversion. `Ctrl+Shift+V` stays Toggle View Mode.

**Non-goals:** Excel `<style>`/class formatting (bold via `xl*`), GFM column alignment from number formats, XML Spreadsheet / BIFF / CSV flavors, downloading pasted HTML images, RTF, stealing `Ctrl+Shift+V` from Toggle View Mode, MarkNice import, or writing an HTML clipboard flavor on copy.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Spreadsheet paste becomes a GFM table; merged HTML tables flatten to GFM; Word fake lists become Markdown lists; invalid UTF-8 HTML falls back to Unicode text; Paste as Plain Text inserts raw clipboard text.

## Impact

- `crates/html-import` (`markion-html-import`): table rendering, TSV conversion, Word list reconstruction; crate stays GUI-free with no new dependency. Conversion still happens once per paste; inserted Markdown goes through the existing atomic undo + per-version derived-cache path — no extra preview/outline recompute beyond a normal insertion.
- `src/app/editing.rs`: paste orchestration (HTML vs TSV vs plain); new `PastePlainText` action.
- Vendored Windows CF_HTML reader: invalid UTF-8 no longer poisons the text path (HTML omitted, Unicode text kept).
- Edit menu, shortcut registry (`menu_shortcuts`), and `src/i18n.rs` (all languages + shortcut catalog) for Paste as Plain Text.
- App-level paste tests plus html-import fixtures. Live Excel/Sheets/WPS verification is recorded if a GUI session is available; otherwise fixtures stand in and the gap is explicit.
