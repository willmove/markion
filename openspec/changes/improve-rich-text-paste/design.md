## Context

See proposal.md for motivation. `MarkionApp::paste` already prefers clipboard HTML via `markion_html_import::html_to_markdown`, then Unicode text. Table rendering in `crates/html-import/src/render.rs` emits an empty GFM header when no `<th>` exists and serializes any `colspan`/`rowspan` table as a single-line raw HTML island. TSV is never inspected. Word lists that are not real `<ul>`/`<ol>` become paragraphs. Windows `extract_cf_html_fragment` requires the entire CF_HTML payload to be UTF-8 or the HTML flavor is dropped. `Ctrl+Shift+V` is Toggle View Mode.

Paste inserts Markdown through the existing `replace_text_in_range` + atomic undo path; derived preview/outline/stats caches invalidate by document version as for any other insertion. This change must not add per-keystroke conversion.

```
clipboard item
    │
    ├─ image & not a text field ──► existing image import
    ├─ text field focused ────────► raw Unicode text
    ├─ Paste as Plain Text ───────► raw Unicode text (no HTML, no TSV)
    └─ Paste
          ├─ HTML (UTF-8) ──► html_to_markdown
          │                     (tables flatten; Word fake lists)
          ├─ else Unicode TSV (rectangular, ≥2 rows) ──► tsv_to_markdown
          └─ else Unicode text verbatim
```

## Goals / Non-Goals

**Goals:**
- One paste orchestration function chooses HTML conversion, TSV conversion, or verbatim text without extra document-version churn.
- Table and list policy lives in `markion-html-import` so app tests and crate tests share the same functions.
- Default Paste as Plain Text binding avoids the existing Toggle View Mode chord.

**Non-Goals:**
- New crate or new GPUI clipboard entry variant.
- Encoding libraries for GBK CF_HTML (Unicode text + TSV path is the fallback).
- Changing Visual Edit table chrome; GFM output reuses it.

## Decisions

### 1. First row is the header for headerless HTML tables and for TSV

Excel/Sheets emit `<td>` for every cell, including the visual header. An empty GFM header row is worse than occasionally promoting a data row. Nested tables stay raw HTML. 1×1 TSV/HTML grids paste as text so a single Excel cell does not become a one-column table.

Alternative considered: detect `Excel.Sheet` / `google-sheets-html-origin` and only then promote the first row. Rejected for v1 — a headerless HTML table from a browser is still usually a grid with a first-row header, and one heuristic keeps tests small. Cover-sheet-style merged HTML that must stay HTML is the nested-table / later P2 path.

### 2. Flatten merged cells instead of raw HTML

`html-table-rendering` can paint rowspan/colspan HTML, but Visual Edit treats those as HTML islands without the GFM cell toolbar. Flattening matches Excel's own TSV (value in the origin, blanks elsewhere) and keeps the table editable. Nested `<table>` still serializes as raw HTML because GFM cannot represent it.

Implementation: expand each row to the max grid width, writing the origin cell text into the first covered slot and `""` into the rest; then run the existing GFM emitter with first-row-as-header when there is no `<th>`.

### 3. TSV conversion is a crate function called from paste, not a second HTML pass

`pub fn tsv_to_markdown(text: &str) -> Option<String>` returns `None` when the payload is not a rectangular ≥2-row TSV. `paste` calls it only when HTML is absent, empty after conversion, or unused because UTF-8 failed. Paste as Plain Text never calls it.

Do not parse CSV: commas in prose are common; tabs in prose are not.

### 4. Word fake lists: reconstruct from `MsoListParagraph` / `mso-list` before generic paragraph rendering

Word list HTML is a sequence of `<p class=MsoListParagraph>` (or `MsoListParagraphCxSp*`) with `style` containing `mso-list: lN levelK ...` and often a `mso-list:Ignore` span holding `1.` / `•`. Detect those paragraphs, strip the ignore-marker text, choose `-` vs `1.` from the marker (digit → ordered), and nest by `levelN` or `margin-left` buckets of 18pt/36pt. Adjacent fake-list paragraphs become one list; a normal paragraph breaks it.

Google Docs real `<ul>` is unchanged. Unknown list dialects stay paragraphs.

### 5. Invalid UTF-8 HTML is omitted, not decoded

Windows `read_html_from_clipboard` already returns `None` when `String::from_utf8` fails. Keep that. Paste then sees Unicode text (UTF-16 `CF_UNICODETEXT`, always correct for CJK) and TSV conversion. No `encoding_rs` in gpui.

### 6. Paste as Plain Text uses `Ctrl+Alt+V`, not `Ctrl+Shift+V`

`TOGGLE_VIEW_MODE` already owns `secondary-shift-v`. New `PastePlainText` action, `menu_shortcuts::PASTE_PLAIN` = `secondary-alt-v` / `Ctrl+Alt+V` / `Cmd+Option+V`, Edit menu item immediately under Paste, i18n `Msg::ItemPastePlain` in every language, shortcut catalog entry. Users who want Typora's chord can rebind in Preferences.

## Risks / Trade-offs

- [First-row-as-header on a true headerless grid] → Users get a data row as header. Mitigation: Paste as Plain Text; later P2 can add an Excel-fingerprint-only heuristic if this shows up.
- [Flattened merges lose visual span] → Editable GFM is preferred over an HTML island. Nested tables remain HTML.
- [Word list false positives] → Require `MsoListParagraph` class or `mso-list:` in style, not merely `margin-left`.
- [CF_HTML fragment offsets vs UTF-8] → Unchanged; invalid UTF-8 drops HTML entirely.
- [Live Excel not available in CI] → Fixtures reconstructed from documented Excel/Sheets/WPS HTML dialects; task notes if a GUI session cannot be run.

## Migration Plan

No preference-file migration. New shortcut id `paste-plain` defaults to `secondary-alt-v`. Existing `paste` and `toggle-view-mode` bindings are untouched. Rollback is reverting the change; documents already pasted as GFM or HTML islands stay as the user saved them.
