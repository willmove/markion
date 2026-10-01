## Context

The preview pane is driven by `MarkdownDocument::derive_preview_and_outline` (`src/lib.rs`), a single streaming pass over `pulldown-cmark` events that produces a flat `Vec<PreviewBlock>` cached per document version and shared via `Arc`. Blockquotes are accumulated as one flat `RichText` (`quote: Vec<InlineSpan>`) delimited by `quote_depth`; list items are accumulated as `ListItemDraft` and flushed via `flush_list_item` (`src/parse.rs`) directly into the top-level block vector.

Two interactions cause the bug:

1. In `push_preview_rich`, routing priority is heading → list item → quote → paragraph, so text inside a list item inside a quote lands in the list-item draft, not the quote.
2. `flush_list_item` always pushes a top-level `PreviewBlock::ListItem`, regardless of `quote_depth`.

Net effect: `> 1. foo` produces a top-level `ListItem` block plus a `BlockQuote` block containing only the quote's paragraph text. The preview (`src/app/preview.rs`) renders each block independently, so the list visually escapes the quote container (rendered above it, unindented, without the quote's left border).

## Goals / Non-Goals

**Goals:**
- A blockquote's nested content (paragraphs and ordered/unordered/task list items) stays attached to the quote in the derived preview model.
- The preview renders nested list items inside the quote container with correct markers: ordered numbers honoring the list start index, unordered bullets, and task checkboxes, using quote typography.
- All `PreviewBlock::BlockQuote` consumers (text extraction, export, memory accounting, selection/copy, math collection) keep working with the extended shape.
- The per-version derive-then-cache architecture is unchanged; the fix is confined to the existing derive pass.

**Non-Goals:**
- No recursive rendering of arbitrary block types inside quotes (code blocks, tables, images, nested quotes) beyond what is needed to not regress: they may keep their current flattened behavior.
- No Visual Edit (WYSIWYG) structural changes; Visual Edit already keys off its own projection.
- No changes to source-editor syntax highlighting or the outline.

## Decisions

### Decision 1: Give `PreviewBlock::BlockQuote` a children list instead of only flat text

Change the model to:

```rust
BlockQuote {
    text: RichText,                 // paragraph text (existing behavior)
    children: Vec<PreviewBlock>,    // nested list items (and future nested blocks)
    source_range: Range<usize>,
}
```

Rationale: list items already carry everything the preview needs to render them (`level`, `ordered`, `index`, `checked`, `text`, `source_range`), so reusing `PreviewBlock::ListItem` as the child type avoids a parallel "quoted list item" type and keeps rendering code shared. Keeping `text` alongside `children` minimizes churn: quotes without lists are byte-identical to today, and text-only consumers (word counts, export) keep reading `text` and additionally fold in child text.

Alternative considered: flatten list items into the quote's `RichText` with literal `"1. "` marker prefixes. Rejected — it loses real markers/indent, breaks selection/copy fidelity less gracefully, and cannot render task checkboxes or honor continuation numbering without ad-hoc re-parsing of the synthesized text.

Alternative considered: fully recursive `children: Vec<PreviewBlock>` for all block types (mini-DOM). Rejected as over-scope; the derive pass is a streaming single-pass state machine and generalizing it risks regressions across every consumer.

### Decision 2: Route list items into the open quote during derivation

In `derive_preview_and_outline`:
- When `quote_depth > 0` and a list item flushes, push the resulting `PreviewBlock::ListItem` into a per-quote `children: Vec<PreviewBlock>` draft instead of the top-level `blocks`.
- `Event::Start(Tag::Item)` currently flushes the open item eagerly; that flush must also target the quote's children when a quote is open.
- Ordered numbering continues to come from `list_stack`, so a `> 3. a` start index is honored unchanged; nested list levels inside the quote keep their relative `level`.
- Paragraph spacing: existing behavior appends `"\n"` between sibling paragraphs in a quote; list items interleaved with paragraphs keep document order by flushing into `children` in event order, with paragraph text remaining in `text` (paragraphs stay dominant, matching today's flattening for quotes without lists).

Note the ordering subtlety: today, a quote's paragraph text and its list items appear in one flattened stream. With `text` + `children`, a quote like `> intro\n> 1. a\n> outro` renders intro/outro paragraph text followed by the list — a minor reordering within the quote. This is accepted for this fix (paragraphs-then-children) since the common case (quote containing only a list, or list after an intro line) renders correctly, and full interleaving requires the recursive model explicitly deferred above. If interleaving proves simple during implementation (e.g. by promoting paragraph chunks into `children` as `Paragraph` blocks), prefer that; the spec scenarios are written against containment and numbering, not paragraph/list interleave order.

### Decision 3: Render children inside the existing quote container

In `src/app/preview.rs`, the `BlockQuote` arm keeps its current container (left border, quote typography) and appends one row per child list item using the same marker logic as top-level list items (number from `index`, bullet, or checkbox), reusing the existing list-item row layout where practical so styling stays consistent.

### Decision 4: Update text-consuming helpers to fold in children

Consumers that read quote text (`document_memory.rs`, `export.rs`, `app/math_render.rs`, selection/copy in `app/preview.rs`, stats extraction in `src/lib.rs`) gain a small fold over `children` so quoted list text still counts toward stats, exports, copies, and math rendering. A helper like `PreviewBlock::plain_text()` (or extending the existing extraction sites) keeps this in one place.

## Risks / Trade-offs

- [Pattern-match breakage across the crate: `BlockQuote { text, .. }` sites silently ignore `children`] → Compile errors are impossible (struct update adds a field with `..` still legal), so each site is audited in tasks; tests assert stats/export/selection include quoted list text.
- [Paragraph/list interleaving inside a quote renders paragraphs first, then children] → Accepted limitation, documented above; spec scenarios avoid asserting interleave order.
- [Incremental source-mapped derivation (`src/source_mapped.rs`) diverges from the full parse for quotes] → The change lives in the shared derive path; the existing "incremental output equals full parse" requirement and its tests cover equivalence.
- [Visual Edit projection (`src/visual.rs`) maps `PreviewBlock` variants] → Audit its `BlockQuote` handling; it flattens to `VisualBlockKind::BlockQuote` today and should continue to do so without consuming `children`.
