## Context

See proposal.md. `SourceMappedCache` already stores independent semantic regions, but flattens and rebuilds all VisualBlocks. Initial partitioning is quadratic. Published source offsets are absolute; cached region offsets can remain local. Full derivation is the correctness oracle.

## Goals / Non-Goals

Goals: reuse unchanged visual payload derivation, linear initial partitioning, accurate memory accounting, precise effect routing, reproducible release measurements.

Non-goals: changing Markdown ownership, persisting derived caches, replacing the buffer, or hiding unsupported Markdown in a guessed editable tree.

## Decisions

1. Keep region-local semantic and visual results shared. Reuse regions by edit lineage and exact text; only changed regions derive inline/editor metadata. Assemble absolute-range views and reconcile changed-region identities under the existing rules. Reference definitions, front matter, fences and ambiguous boundaries retain a full fallback.
2. Partition sorted semantic blocks/headings with advancing cursors. Cross-region ownership rejects partitioning instead of duplicating a block.
3. Mutation receipts supply old range/new length and conservative effect flags. Geometry is retained only for proven unchanged IDs and relocated ranges; search may debounce with document identity/version checks. The active editing surface remains synchronous.
4. Add deterministic derived/reused work counters and differential tests. Timing is diagnostic, not a machine-dependent CI threshold.
5. Share immutable rich text, visual text and visual block payloads with copy-on-write relocation. Materialize the flat Preview snapshot only when requested. Stable front matter is an indivisible reusable region; header/context changes retain a full fallback.
6. Large-document search is debounced and computed on a background executor. Results carry identity/version/query generations. Ordered matches advance line/column positions once and retain at most a bounded UTF-8-safe context per result, including on pathological single-line documents.

Data flow: checked source mutation -> receipt/effects -> versioned region update -> changed-region visual build + cached-region relocation -> shared visual snapshot -> list splice/geometry consumers. Undo stores authored state only.

Native verification uses an explicit environment-gated replay in the production GPUI window, synchronized after real frames. The launcher isolates preferences/recovery, uses repository fixtures, records content-free paint timing and OS process counters, writes a JSON result, and closes only its own test process. Application input callbacks and the native clipboard are exercised; unavailable OS mouse/key injection and the system IME candidate UI remain separate evidence boundaries.

## Risks / Trade-offs

- Flat public vectors still require linear relocation/allocation -> report this residual cost and bound retained caches.
- Context-sensitive syntax invalidates independent parsing -> conservative full fallback and randomized equivalence tests.
- Delayed work can finish after a tab switch -> identity/version validation before publication.
