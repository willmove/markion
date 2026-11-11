## Why

The release large-document diagnostic shows strongly non-linear growth in localized Visual Edit reads even though semantic regions are reused. The current benchmark cannot separate semantic derivation, VisualBlock construction, and identity reconciliation, while identity reconciliation repeatedly scans the new block list for every old block.

## What Changes

- Add a diagnostic-only profile around the existing `visual_blocks_shared` seam, reporting cache disposition, per-stage elapsed time, block counts, and identity-candidate checks without changing normal editing behavior.
- Make the existing large-document benchmark selectable by document size and stroke count, and include the profile fields in its output so short before/after runs are reproducible.
- Replace repeated whole-list identity lookups with source-range indexes and occurrence-safe candidate tracking; avoid cloning unchanged zero-delta blocks merely to compare their identities.
- Add deterministic regression evidence that identity reconciliation produces exactly the prior blocks and bounds candidate checks for ordinary unique-range documents.
- Record baseline and optimized diagnostics in a P0 analysis report.

**Non-goals:** no rope migration, no full VisualBlock-region redesign, no asynchronous Visual Edit derivation, no editing or overlay behavior changes, and no machine-dependent CI latency threshold.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `engineering-quality`: extend deterministic incremental performance evidence to cover indexed visual-block identity reconciliation and diagnostic profiling.

## Impact

- `src/source_mapped.rs`: identity reconciliation implementation and unit tests.
- `src/lib.rs`: a diagnostic wrapper around the existing VisualBlock cache seam.
- `examples/bench_large_doc.rs`: selectable benchmark inputs and per-stage profile output.
- `docs/visual-p0-performance.md`: baseline, implementation explanation, verification, and remaining work.
- The canonical Markdown representation, per-version `Arc` caches, VisualBlock identities, editing mutations, and all user-visible editing behavior remain unchanged.
