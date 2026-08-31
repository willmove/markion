## Why

GFM tables in Visual Edit, Split Preview, and Read mode currently give every column equal flex weight (`flex_1()`). Short label columns therefore sit in a sea of empty space while long description columns look cramped — unlike mainstream WYSIWYG Markdown editors, which size columns from cell content. The two-column “名称 / 说明” pattern in technical documents is the typical failure case.

## What Changes

- Size GFM table columns from cell content instead of an equal split, in Visual Edit, Split Preview, and Read mode.
- Keep the table stretched to the document content column; extra width goes to columns that need it, and long cells still wrap rather than overflowing the pane.
- Share one GPUI-free width-recommendation helper so Visual Edit and the read-only preview grid stay in lockstep.
- **Non-goals:** user-draggable column handles; persisting widths in Markdown or settings; changing GFM table syntax, cell editing, toolbars, or alignment markers; DOCX/PDF/LaTeX/HTML export column grids (those stay equal/proportional as they are today); HTML `<table>` grids (CSS-grid equal tracks plus rowspan/colspan stay out of this change); measuring real GPUI glyph runs (a CJK-aware character heuristic is enough).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: GFM tables in Visual Edit, Split Preview, and Read mode SHALL allocate column widths from recommended content widths rather than an equal split, while remaining inside the document content column and wrapping overflow.

## Impact

- Shared column-weight helper (GPUI-free, unit-testable) next to `src/table.rs`, consumed by both `visual_table_view` and the Read/Split `PreviewBlock::Table` branch in `src/app/preview.rs`.
- Presentation-only layout: MUST NOT mutate document text, dirty state, history, or the per-document-version derived Markdown caches. Typography preference changes already trigger a relayout; column weights may use `DocumentTypographyMetrics::table_font_size` without bumping document version.
- Tests in `src/app/tests.rs` (and table-module tests) for the weight algorithm and for Visual Edit / preview sharing the same weights. No i18n, persistence, public API, or `crates/*` changes.
