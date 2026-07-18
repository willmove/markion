## Context

The interaction-core and source-mapped-model changes established exact source/display projection, semantic text-input history, stable `VisualBlockId`, and identity-aware list virtualization. Ordinary prose can therefore stay rendered while editing, but `CodeBlock`, `MathBlock`, `Image`, and `Table` still have only coarse behavior: code always shows its complete fence source, math becomes source when focused or unavailable, images have no field-level editor, and table cells only move the source caret to the table start.

Velotype demonstrates a useful ownership rule: a block owns presentation-local behavior while the editor owns document structure and cross-block mutations. Markion can adopt that rule without adopting Velotype's attributed runtime tree or Markdown regeneration boundary. Exact authored Markdown remains canonical, and direct widgets operate on proven source subranges.

The intended flow is:

```text
canonical Markdown + source-mapped preview block
  -> VisualBlockId + typed VisualBlockEditor metadata with exact subranges
  -> virtualized GPUI block widget using the shared input bridge
  -> VisualBlockEdit (one validated source replacement + selection)
  -> application history/focus boundary
  -> MarkdownDocument::replace_range
  -> incremental region derivation + stable-id reconciliation
  -> refreshed widget with current source ranges
```

## Goals / Non-Goals

**Goals:**

- Directly edit ordinary fenced-code payloads, block LaTeX, Markdown image fields, and GFM table cells while their surrounding block presentation remains visual.
- Preserve exact authored delimiters and unrelated source bytes; every widget edit is one canonical source replacement.
- Reuse the existing platform input/IME, semantic undo, stable identity, cache, virtualization, multi-tab, autosave, and recovery contracts.
- Keep registered diagrams and uncertain syntax conservative and source-backed.
- Make the source-range extraction and mutation algorithms GUI-free and differentially testable.

**Non-Goals:**

- A second rich-text tree, normalized whole-document serialization, or marker-free persisted state.
- Direct HTML/front-matter/diagram editing, nested rich blocks inside table cells, image asset management, or multi-cursor editing.
- Replacing `pulldown-cmark`, the source editor, or the existing Split/Read preview pipeline.

## Decisions

### 1. Add typed editor metadata to the source-mapped visual block

`VisualBlock` gains an optional `VisualBlockEditor` enum. Its variants contain only immutable current-version data and exact UTF-8 source ranges:

- `Code`: opening fence, optional info string, payload, and closing fence ranges.
- `Math`: authored delimiter and LaTeX payload ranges.
- `Image`: alt text, destination, optional title, and complete syntax ranges.
- `Table`: row/column coordinates plus each editable cell's authored content range.

Metadata is derived beside the visual model, cached by document version, and discarded by cache-free clones. A block receives a direct editor only when parsing proves every required range and round-trip rule. Otherwise its existing `source_island` remains authoritative.

Inferring field boundaries in GPUI render code was rejected because renderers must not become a second Markdown parser. Persisting widget state in the document was rejected because identities and ranges are derived and non-persistent.

### 2. Use one source-edit protocol for every direct widget

Add a pure `VisualBlockEdit` value containing the block ID/version expectation, replacement range, replacement text, and post-edit source selection. Constructors validate that the target range belongs to the current block metadata and apply field-specific escaping or normalization. The application revalidates version and ID immediately before taking one atomic history capture and calling the existing document replacement path.

Widgets never mutate a `PreviewBlock`, table model, image object, math cache, or code buffer independently. A temporary shadow model was rejected because source and widget state could diverge during IME, undo, or a background render completion.

### 3. Reuse the shared source-backed text input element

Each editable field is an identity projection over its exact source payload range and uses the existing `VisualEditableText`/`EntityInputHandler` bridge. Code supplies memoized token highlights, math supplies a monospaced LaTeX field next to the rendered formula, image fields use compact labeled inputs beneath the preview, and table cells use the same text element inside the grid.

Only one field owns the global source selection and caret. Clicking a field places the canonical selection in that field; Tab/Shift-Tab move between fields or cells, Enter follows field-specific rules, and arrow movement at a field edge hands control back to visual block navigation. This avoids one GPUI entity and independent IME session per cell.

### 4. Preserve authored delimiters and isolate normalization

Code and math payload edits replace only their payload ranges, leaving fence characters, fence length, info spacing, and math delimiters byte-identical. Image field edits replace only the selected field and escape characters that would terminate that field. Table cell edits may replace the complete table once, using the existing table formatter, because column widths and the separator row are coupled; the formatter returns the new cell source range so selection remains exact. Unchanged rows, cell values, and alignments are preserved semantically.

Normalizing the whole Markdown document was rejected. Editing raw delimiter characters through hidden zero-width positions was rejected because dedicated controls can express the intended fields more safely.

### 5. Gate ordinary code editing away from diagram fences

Before attaching `Code` editor metadata, the visual builder checks the existing diagram backend registry classification. Registered diagram fences keep their current source island exactly as required by `diagram-rendering`. Ordinary code uses syntax highlighting; unknown ordinary languages remain editable as plain code. Unclosed fences and byte-ambiguous constructs remain source islands.

### 6. Key widget-local state by version and stable block identity

Focus hints, field identity, measured geometry, pending Tab navigation, and code/math presentation state carry the current document version and `VisualBlockId`. Stable unchanged blocks may retain list height and scroll anchoring, but a changed block receives a new ID and cannot inherit stale field state. Highlighting and math rendering reuse their existing content-keyed application caches; focusing or moving within a widget does not reparse Markdown.

## Risks / Trade-offs

- **[A field parser accepts ambiguous Markdown]** -> Attach direct metadata only after exact boundary validation; round-trip against the source slice and fall back to the whole source island on any mismatch.
- **[Table typing causes cursor jumps when widths reflow]** -> Produce the formatted replacement and new selected cell range in one pure operation; cover every insertion/deletion/UTF-8 case with selection tests.
- **[Multiple editable fields compete for IME ownership]** -> Keep one application-level source selection and one input handler; field focus is derived from which exact range owns the selection.
- **[A widget edits stale ranges after an incremental reparse]** -> Revalidate document version, stable block ID, and field containment immediately before mutation; stale events are ignored and trigger repaint.
- **[Code/math rendering becomes expensive while typing]** -> Reuse memoized highlighting/math caches and derive only the dirty source region; pending rendered math keeps the LaTeX editor available.
- **[Image syntax variants exceed the initial parser]** -> Support byte-exact inline destination/title syntax first; reference images, angle-bracket destinations, multiline titles, and malformed syntax remain source-backed until proven.

## Migration Plan

1. Add pure typed source metadata and `VisualBlockEdit` builders behind existing source-island rendering.
2. Enable direct code and block-math editors with exact fallback and input regressions.
3. Enable image fields, then table cell editing and keyboard traversal.
4. Add rendered large-document, IME, undo, multi-tab, stale-event, and round-trip coverage; keep Split/Read unchanged.
5. Roll back any editor variant independently by declining to attach its metadata, which restores the current source-island path without persisted-data migration.

## Open Questions

- None for implementation. Reference-style images and rich inline table-cell rendering remain explicit later extensions.
