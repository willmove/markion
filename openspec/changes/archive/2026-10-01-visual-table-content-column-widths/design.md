## Context

GFM pipe tables render as stacked flex rows in two places:

- Read / Split Preview: `PreviewBlock::Table` in `src/app/preview.rs` (~5971)
- Visual Edit: `visual_table_view` in the same file (~5171)

Every cell uses `.flex_1().min_w_0()`, so Taffy splits the document content column equally. The table chrome (borders, header shading, padding, alignment, cell editing, toolbar) is otherwise fine. Cell text already lives on the per-document-version derived cache (`RichText` rows); this change only reallocates layout.

GPUI's public `flex_grow()` helper always sets `1.0`. Custom grow weights are set on the style refinement the same way other preview views already poke `style.size`.

## Goals / Non-Goals

**Goals:**

- Allocate GFM table column widths from cell content so a short label column stays narrow and a long description column takes the remaining space.
- Keep Visual Edit and Read/Split Preview visually aligned.
- Keep the table stretched to the document content column; long cells wrap instead of overflowing.
- Cap any column at three times the equal share (`3 / n`) and keep short header parenthesis units on one line, with a three-line header wrap budget.
- Compute weights in a GPUI-free helper so the algorithm is unit-testable and does not touch derived Markdown caches.

**Non-Goals:**

- User-resizable columns, persisted widths, or Markdown/HTML width syntax.
- Real GPUI glyph measurement (no extra layout pass, no font-dependent cache).
- HTML `<table>` CSS-grid tracks (`html_table_grid_view` / `grid_cols`), which need unequal grid templates GPUI does not expose.
- Export writers (DOCX/PDF/LaTeX/HTML) — they keep today's equal or page-fitted grids.
- Changing cell editing, toolbars, alignment, or source rewriting.

## Decisions

### Decision 1: Content weights as proportional flex grow, still filling the table

**Choice:** Replace `.flex_1()` with the same flex recipe (`flex_basis = 0`, `flex_shrink = 1`, `min_w_0`) but `flex_grow = weight[column]`. The table remains full width of the document content column. Extra space and shrink share follow the weights, so columns stay aligned across rows.

**Rationale:** This is the smallest change that matches CSS `table-layout: auto` stretched to 100%: preferred widths become shares of the available row. `min_w_0` already allows wrapping, which we keep.

**Alternative considered:** Size the table to the sum of preferred widths and leave unused pane space empty (GitHub's unstretched tables). Rejected — Markion tables already fill the content column; shrinking them would look like a regression next to headings and paragraphs.

**Alternative considered:** Per-cell `w(relative(share))` percentages. Equivalent when shares sum to 1, but custom `flex_grow` is closer to the current `flex_1()` contract and does not fight flex wrapping.

### Decision 2: GPUI-free CJK-aware character heuristic

**Choice:** Estimate a cell's preferred width from its `RichText.text` (rendered plain text, not focused source markup):

- ASCII / Latin: `0.55 * table_font_size`
- All other scalars (CJK, fullwidth, emoji): `1.0 * table_font_size`
- Plus the existing cell padding (`p_2` = 8px per side → 16px)

A column's preferred width is the max over its cells, floored at `padding + 2 * table_font_size` so empty or single-glyph columns do not collapse. Weights are those preferred widths (or a normalized copy). Missing cells in ragged rows count as empty.

**Rationale:** Matches the file-tree estimator (`estimate_file_tree_text_width`) and scales with `DocumentTypographyMetrics::table_font_size`. No new crate, no window/cx, no per-keystroke derived-cache invalidation. Focusing a Visual Edit cell that reveals `**bold**` MUST NOT reflow columns; source markup is longer than rendered text.

**Alternative considered:** Measure real GPUI shaped runs. Rejected — needs a window, is font-family sensitive, and cannot live in `src/table.rs`. The heuristic is good enough for the 名称/说明 class of tables.

**Alternative considered:** Count Markdown source pipe-column character widths (the GFM separator / padding the formatter already uses). Rejected — those widths describe source alignment, not rendered CJK vs ASCII, and would ignore inline formatting collapse.

### Decision 3: Compute weights at paint time from cached cell text

**Choice:** Call the helper from both table views while building the GPUI tree. Inputs are the already-cached `rows: &[Vec<RichText>]` plus `table_font_size`. Do **not** store weights on `PreviewBlock` / `VisualBlock` or bump document version.

**Rationale:** Weight math is O(cells) and cheaper than the text shaping that follows. Caching it on the derived Markdown snapshot would force a version bump (or a parallel presentation cache) whenever typography changes; typography already relayouts without mutating document version, which is the invariant we must keep.

**Data flow:**

```
MarkdownDocument (version N, cached Arc preview/visual rows)
        │
        ▼  paint (Visual Edit / Read / Split)
table_column_flex_weights(rows, table_font_size)  // GPUI-free
        │
        ▼
each cell: flex_grow = weights[col], flex_basis 0, min_w_0
```

No write to document text, dirty flag, undo, or per-version caches.

### Decision 4: One helper, two render paths

**Choice:** Put `table_column_flex_weights` (name bikesheddable) in `src/table.rs` next to the other GPUI-free table algorithms. `visual_table_view` and the `PreviewBlock::Table` branch both call it so a 名称/说明 table looks the same in Visual Edit and Read mode.

**Rationale:** The WYSIWYG contract already requires Visual Edit to match the reading surface. Duplicating the heuristic would drift.

### Decision 5: Cap any column at three equal-shares

**Choice:** After preferred widths and header minima, clamp each column to `3 / n` of the current weight sum (`n` = column count). Never clamp below that column’s header minimum. One-column tables skip the cap. A 6-column table therefore never gives one column more than half the row.

**Rationale:** Uncapped content weights let a “关键配置” / “备注” paragraph take most of a six-column row, so a unit header like `实际功率（W）` wraps onto four lines. Three times the equal share is enough for a true description column and still leaves room for the other headers. A hard 80% ceiling on two-column 名称/说明 tables does not bite (`3/2 = 150%`).

**Alternative considered:** Cap at `2 / n`. Rejected — two legitimate description columns in a six-column grid would both want ~33% and would fight the cap instead of wrapping their body text.

### Decision 6: Header minima, then compress leftover body width

**Choice:** Row 0 is the GFM header. Each column’s minimum is the max of (a) the empty-column floor, (b) the width of a `(`…`)` / `（`…`）` unit whose inner non-whitespace span is 1–3 characters, (c) unwrapped header content width ÷ 3 plus padding. Preferred width is `max(content, min)`. Extra width above the min is compressed with `sqrt(extra × floor)` so a paragraph-sized body cell grows faster than a header but not linearly. Then apply Decision 5.

**Rationale:** The 3-line and short-paren rules are “as far as possible”: they raise the column’s requested share so `（W）` stays on one line and `实际功率（W）` wraps in at most three lines. Linear extras still starve those mins when two body columns are long; geometric compression makes the mins occupy a real fraction of the flex sum without flattening 名称/说明 back to 50/50.

**Alternative considered:** Real GPUI line-count measurement against the laid-out column. Rejected — needs a second layout pass and window/cx; the helper stays GPUI-free.

## Risks / Trade-offs

- **[Risk] Heuristic vs real glyphs** (narrow fonts, tabs, inline code) → Mitigation: floor plus wrapping; accept small slack. Do not add a second layout pass in this change.
- **[Risk] Focused Visual Edit source is wider than the rendered weight** → Mitigation: size from `RichText.text` only; the focused field wraps inside the column like today.
- **[Risk] Custom `flex_grow` via style refinement is slightly off the public Styled helpers** → Mitigation: isolate in one cell-style helper used by both views; if GPUI later grows a `flex_grow(f32)` API, swap the poke.
- **[Risk] HTML tables stay equal-width** → Mitigation: documented non-goal; they use CSS grid for rowspan/colspan and cannot take flex weights without a separate layout rewrite.
- **[Trade-off] Table still stretches full width** — a two-cell table of short words will have leftover space inside the columns (distributed by weight, not 50/50). That leftover is still better than equal split when weights differ.
- **[Trade-off] Header minima can exceed `3 / n` when every header is long** — mins win; the cap is best-effort. Geometric extra compression is a heuristic, not glyph-accurate wrapping.

## Migration Plan

Presentation-only. No document, setting, or export migration. Rollback is reverting the two table views and the helper.

## Open Questions

None blocking implementation. Optional follow-up: HTML `<table>` unequal tracks once GPUI exposes non-equal `grid-template-columns`, and export writers reusing the same weights.
