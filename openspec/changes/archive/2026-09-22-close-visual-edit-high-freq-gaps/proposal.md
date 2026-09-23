## Why

Visual Edit still demotes four high-frequency constructs to source islands: YAML front matter is a permanent FrontMatter island; a focused reference-style image (`![alt][ref]`) loses the rendered picture; a ragged GFM table that paints as a grid while unfocused becomes an island on focus; and display math whose KaTeX state is Pending or Error islands instead of keeping the payload editor. Closing these four `WYSIWYG coverage roadmap` gaps (priorities 1, 4, 5, and 11) removes the remaining everyday “this became a gray source box” cases without expanding into indented code, unclosed fences, or definition lists.

## What Changes

- YAML `---` front matter becomes a rendered document-header row with a collapsible YAML payload editor over the complete authored header (delimiters included). Invalid YAML still edits as source in that editor and MUST NOT become an island. TOML/JSON detection stays out of scope.
- A single-line reference-style image (`![alt][ref]`, collapsed `![alt][]`, or shortcut `![alt]`) that already resolves as a block image keeps the rendered image and the collapsible whole-span source toggle when focused. Width/alignment controls stay inline-image-only so confirming them cannot rewrite a reference into `![alt](url)`. Multiline and other unprovable image forms remain islands.
- Ragged GFM tables (unequal cell counts) keep the best-effort grid while focused. Cell editors cover every logical cell, padding missing cells with empty UTF-8-safe insertion ranges. Toolbar and one-edit undo keep using the existing full-table replacement path.
- Display/fenced math whose render is Pending or Error keeps the existing payload editor (forced expand until Ready) instead of `visual_source_island_view`. Math blocks that cannot split payload from delimiters still get a Math editor over the complete authored span.

### Non-goals

Indented code blocks, unclosed/malformed fenced code, GFM definition lists, multiline/angle-bracket/malformed images, TOML/JSON front matter, Typora `[TOC]`, Read/Split Preview YAML chrome, guessing rendered-tree mutations, and changing how inline math atoms fall back to source text inside mixed prose.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Close YAML front matter, reference-style block images, ragged tables, and math render-failure islands; update the coverage matrix and roadmap.
- `ui-i18n`: Localize the YAML header label.

## Impact

- **Code:** `src/visual.rs` (front-matter block + editor; reference-image span proof; math editor fallback); `src/table.rs` (`table_cell_source_ranges` padding); `src/inline_edit.rs` (reference image span); `src/app/preview.rs` (YAML collapsible header, math Pending/Error no island, `always_source` without FrontMatter); `src/model.rs` (`VisualBlockKind` / `VisualBlockEditor` / field kind); `src/i18n.rs`; `docs/visual-editing-quality.md`.
- **Invariants:** Mutations stay one canonical `MarkdownDocument.text` edit. Showing, collapsing, or hovering YAML / image / math source chrome MUST NOT bump document version or invalidate per-version `Arc` caches. `crates/*` stay GPUI-free.
- **Tests:** Pure derivation tests for YAML editor (no FrontMatter island), reference-image payload, ragged-table cell ranges, math editor on Pending/Error; GPUI tests that focus does not paint island chrome for those four constructs.
