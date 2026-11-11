## Why

P0 removed quadratic identity matching, but P2 is still unfinished: each keystroke rebuilds every visual block and initial region partitioning scans every block for every region. Large-document opening and typing need bounded work and measured application-level evidence.

## What Changes

- Cache visual derivation per independently parseable region and relocate reusable results after local edits.
- Partition initial semantic results in document order, retain full-parse fallback and executable equivalence checks.
- Describe precise mutation effects once and use them to retain unaffected geometry and schedule noncritical search work.
- Measure opening, local edits, retained memory, and actual Windows interaction with the same fixtures.
- Preserve canonical Markdown, per-version shared caches, memoized highlighting, and cache-free undo snapshots.

Non-goals: replacing the text buffer with a rope or claiming universal latency/memory guarantees from one machine.

## Capabilities

### New Capabilities

### Modified Capabilities
- `engineering-quality`: deterministic work bounds for region-level visual reuse and measured performance evidence.

## Impact

`src/source_mapped.rs`, document derivation/mutation receipts, application geometry/search scheduling, benchmarks and verification documentation. No new runtime dependency.
