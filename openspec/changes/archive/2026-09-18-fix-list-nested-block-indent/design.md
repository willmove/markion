## Context

The single Markdown parse pass (`MarkdownDocument::derive_preview_and_outline_inner` in `src/lib.rs`) already flattens list-nested constructs into the document-ordered `PreviewBlock` stream and truncates the owning item's range so every byte has one owner. What is missing is nesting *depth*: renderers cannot tell that a `CodeBlock`/`MathBlock`/`Table`/`Html` block sits inside one or more list containers, so both Reading mode and Visual Edit paint it at document indent.

See proposal.md — Why for the user-visible defect (`1.     Sub item 1` nested marker case).

## Goals / Non-Goals

- Goals: record list depth for nested blocks at parse time; indent those block rows to the owning item's content column in Reading mode and Visual Edit; keep source ranges, caret mapping, editing behavior, and export serialization untouched.
- Non-Goals: indenting blockquotes or thematic breaks nested in lists; changing list item/marker layout; changing HTML/DOCX/PDF export geometry; altering the derived-state caching model.

## Decisions

### Depth is captured at emit time, not re-derived by consumers

`list_stack.len()` at the moment a nested block is *emitted* (its `End` event / html push) is exact: pulldown-cmark closes container `End` tags only after nested content, so the stack still holds every enclosing list. Deriving depth later from source ranges fails: item ranges are deliberately truncated at the nested-block boundary (ownership invariant), so containment no longer identifies the innermost item; and authored indentation is not reliable (a fence indented deeper than the list content indent still nests at that list level).

### The field lives on the four already-nested variants

`PreviewBlock::CodeBlock`, `MathBlock`, `Table`, `Html` gain `list_depth: usize` (0 = top level). These are exactly the kinds that participate in the existing `record_nested_block_start` ownership plumbing; paragraphs inside items already fold into item text; `BlockQuote` and `Rule` stay flat (non-goal). Standalone display-math (`$$…$$`) cannot occur inside an item by its `standalone` predicate, so its construction stays depth 0. An accessor `PreviewBlock::list_depth()` returns 0 for all other variants so renderers need no variant-specific matching.

`VisualBlock` gains the same field, set by `visual_block_from_preview` from the preview block. Structurally synthesized rows (gaps/whitespace/reference definitions, callout titles, front matter, source islands) hardcode 0 — they own bytes outside any parsed nested block by construction.

### Render-time inset equals the owning item's content column

List rows render at `ml((level − 1) × 18)` with a `min_w(22)` marker column before content. A block nested at depth `d ≥ 1` renders at `ml((d − 1) × 18 + 22)`, aligning its left edge with the content of its owning item — the geometry GitHub uses for blocks inside list items. The inset is a presentational margin on the row container; row-internal caret painting, hit-testing, and editor geometry are unchanged because they are relative to the row. Reading mode applies the same formula around the per-block view; Visual Edit applies it around the row (inside block chrome) so context-menu affordances stay row-local.

## Risks / Trade-offs

- Width: deep nesting narrows code editors/tables by the inset; both already handle horizontal overflow (code scroll, table shrink), so no clipping regression is expected.
- CRLF/offset coalescing utilities are untouched; `list_depth` is an additive `usize` with no offset semantics, so `source_mapped` and sync-scroll are unaffected.
- Existing exhaustive pattern matches without `..` on the four variants (mostly tests) must be updated mechanically; the compiler enumerates them.

## Open Questions

None.
