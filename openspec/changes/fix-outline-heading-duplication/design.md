## Context

See proposal.md for motivation. Canonical source edits invalidate versioned derived state. Visual Edit seeds a source-mapped region cache from a full parse, reparses changed regions, then assembles preview blocks and headings into shared document caches. Debug builds compare assembled output against a full parse and silently repair mismatches; release builds omit that oracle.

## Goals / Non-Goals

**Goals:** Reproduce duplicated outline entries through the production region-cache path and restore exact full-parse equivalence without losing region reuse.

**Non-Goals:** UI redesign, title-based deduplication, or always enabling the debug full-parse oracle in release builds.

## Decisions

- Assert region assembly equivalence directly and assert no debug-oracle fallback after document edits. This makes a release-only mismatch observable in ordinary debug tests.
- Check ownership of heading offsets at region boundaries and use disjoint source intervals if confirmed by the regression. Canonical source offsets, rather than equal title strings, distinguish headings; authored duplicate titles must remain valid.
- Keep source -> full seed -> local region update -> assembled cached outline as the existing data flow; do not change versioning or Arc sharing.

## Risks / Trade-offs

- Debug repair can mask production corruption -> verify fallback counters as well as resulting headings.
- Boundary changes could lose a heading -> cover document start, region starts, the final heading, and legitimate duplicate titles against a full parse.

## Diagnosis and Validation

The minimal source `# One\n\n## Two\n` yielded headings at offsets `[0, 7, 7]` after assembling the full-seeded region cache, while the full parse yielded `[0, 7]`. The synchronous model test excludes UI row reuse and asynchronous parse installation as causes of this reproduction. Region partitioning used `start <= heading.offset <= end`, so the heading at offset 7 was owned by both neighboring regions. Reusing those regions after a rename republishes duplicated or obsolete entries. Debug builds repaired this mismatch with a full parse, masking the production symptom.

The repair uses `start <= heading.offset < end`; all other cache/version behavior is unchanged. Before the fix, both the assembly regression and rename/fallback regression failed deterministically. After the fix, `cargo test --lib source_mapped::tests -- --nocapture` passed all 24 tests in 0.07 seconds. Added coverage includes both full-cache constructors, Chinese UTF-8 headings, Setext headings, empty headings, CRLF, a final heading without a newline, insertion/deletion, repeated renaming, and authored duplicate titles. Fallback-counter assertions ensure the debug oracle does not hide this bug again. Formatting and `openspec validate fix-outline-heading-duplication` also passed.

The complete root library suite (`cargo test --lib`) passed: 710 passed, 0 failed, 1 pre-existing ignored test, in 5.31 seconds. No interactive GUI verification or release packaging was performed.
