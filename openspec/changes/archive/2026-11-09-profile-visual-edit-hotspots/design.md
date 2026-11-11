## Context

`MarkdownDocument::visual_blocks_shared` is the existing deep seam: callers receive a versioned `Arc<Vec<VisualBlock>>` and do not know how semantic regions or identities are derived. Keep that interface and its invariants intact. P0 is limited to measurement and the identity-reconciliation implementation behind it; broader derivation locality belongs to a later change.

The current reconciliation performs two passes over old blocks. Each pass can scan all new blocks, and the unchanged-slice pass clones and shifts an old VisualBlock even when its source range did not move.

## Goals / Non-Goals

**Goals:**

- Attribute diagnostic time to semantic update, VisualBlock construction, and identity reconciliation.
- Replace per-old-block whole-list searches with source-range indexes while retaining occurrence and stale-lineage safeguards.
- Avoid unnecessary zero-delta clones and prove output equivalence with fresh full derivation and the previous algorithm's contract.
- Keep deterministic correctness/work checks separate from informational wall-clock measurements.

**Non-Goals:**

- No new production cache, document model, dependency, or user-facing preference.
- No changes to region splitting, preview assembly, VisualBlock construction, mutation validation, or row presentation.
- No claim that P0 bounds the entire Visual Edit pipeline.

## Decisions

### 1. Profile behind the existing VisualBlock seam

The ordinary `visual_blocks_shared()` method delegates to a diagnostic implementation and drops the profile. A hidden `visual_blocks_shared_with_profile()` entry point exists for the example benchmark and tests. The profile contains cache disposition, three stage durations, block counts, and identity-candidate checks. Timing is diagnostic only; candidate checks provide deterministic work evidence.

This keeps one implementation and avoids benchmark-only derivation behavior.

### 2. Index new blocks by source range

Build one `HashMap<Range<usize>, Vec<usize>>` for new-block positions. A vector preserves duplicate or empty source ranges instead of assuming they are globally unique. Both reconciliation passes query only candidates with the mapped range; the second pass additionally filters by block kind and unclaimed state. A boolean matched-old vector replaces the second pass's repeated search for an already assigned old identity.

Candidate checks count only candidates inspected after the range lookup. Ordinary unique-range documents should require no more than two candidate checks per old block across the two passes.

### 3. Compare zero-delta unchanged blocks without cloning

When a mapped unchanged block has no byte delta and its quote-group metadata already agrees, temporarily assign the old identity to the candidate and compare the existing values directly. Restore the fresh candidate identity if comparison fails. Non-zero deltas or changed quote-group metadata retain the existing clone-and-shift proof.

This optimization does not weaken the exact-slice, exact-metadata, occurrence, or one-to-one successor checks.

### 4. Preserve existing correctness oracles

Existing randomized incremental/full comparisons remain authoritative. Add targeted duplicate-range and candidate-work tests rather than replacing semantic tests with elapsed-time assertions. The benchmark defaults stay compatible; environment overrides only shorten diagnostic runs.

## Risks / Trade-offs

- [Hash indexing changes behavior for duplicate ranges] → Keep ordered vectors and the claimed/occurrence checks; add an explicit duplicate-range regression.
- [Direct zero-delta comparison misses shifted metadata] → Use it only when quote-group ranges already agree; otherwise retain clone-and-shift.
- [Extra timing calls affect normal editing] → Keep them to three coarse stages behind one accessor and do not allocate profile strings in production rendering.
- [Indexed candidate counts do not capture all pipeline costs] → Report them as identity-reconciliation evidence only, not as proof that the full Visual Edit path is bounded.

## Migration Plan

1. Add profiling and selectable benchmark inputs; capture a short baseline.
2. Implement indexed reconciliation and zero-delta comparison with focused tests.
3. Run identical before/after benchmark shapes, full workspace tests, formatting, and OpenSpec validation.
4. Document results and remaining full-pipeline costs. Rollback is isolated to `source_mapped.rs`; profiling is read-only and can remain.

## Open Questions

None.
