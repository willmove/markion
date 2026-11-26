## Context

The quote serializer already inserts a marker on each table line and uses the checked block edit transaction. The parser currently emits a table outside its quote, and the cell mapper expects a contiguous pipe-table slice. On Windows the source-island font request can panic before GPUI's higher-level fallback runs.

## Goals / Non-Goals

Preserve direct cell editing, quote presentation and the one-step undo transaction. Whole-table structural edits inside quotes require separate source-rewrite semantics for quote prefixes and remain outside this change.

## Decisions

- Resolve primary families in the Windows backend, then try explicit fallback families in order and the system UI family. Preserve glyph fallback behavior and cache only resolved selections.
- Route parsed tables through the existing quote children. Reuse the table grid in Read and Split Preview, with source lookup that identifies the quote child and cell.
- Derive a logical pipe-table source from the proven quote marker ranges. Reuse the existing parser on that source and map each cell and hard break back to canonical byte offsets. Use the same conversion to revalidate cell edits. Apply mutations to the authored byte range through the checked editing path.
- Keep the existing block-menu transaction and retain the per-version cache boundary. Projection derives field ranges when visual rows are rebuilt, and direct edits revalidate only their owning table.

## Risks / Trade-offs

LF and CRLF, UTF-8, inherited quote markers and stored column-width metadata affect offset translation. Cover these in focused mapping tests. Keep structural mutation affordances scoped to tables their existing source path can rewrite safely.
