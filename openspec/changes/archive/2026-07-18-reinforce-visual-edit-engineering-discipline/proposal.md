## Why

Visual Edit now has a source-mapped incremental model and dedicated block editors, but the repository has no single executable quality gate or maintained support/fallback matrix, and the `markdown-editing` capability purpose still describes WYSIWYG editing as future work. Without an explicit engineering contract, later visual changes can silently duplicate parsing, invalidate per-version caches, weaken UTF-8 range guarantees, or let documentation drift behind the implementation.

## What Changes

- Add a repository-level engineering quality capability covering local and CI gates for formatting, workspace tests, strict OpenSpec validation, and deterministic Visual Edit invariants.
- Document the current WYSIWYG-oriented support matrix, including direct editors, rendered prose, progressive marker reveal, passive whitespace, and conservative source-island fallbacks.
- Define parser ownership and source-range rules so exact boundary recognizers cannot grow into a second semantic Markdown parser.
- Keep deterministic work/reuse counters and full-derivation differential tests as the performance correctness gate; retain wall-clock benchmarks as informational diagnostics rather than flaky CI thresholds.
- Correct stale capability-purpose documentation so stable OpenSpec metadata no longer claims Visual Edit is only a future candidate.
- Add a dedicated pull-request quality workflow and a shared local PowerShell entry point.

Non-goals: no new Markdown syntax, direct editor type, persisted document model, runtime dependency, or hardware-specific wall-clock performance threshold.

## Capabilities

### New Capabilities

- `engineering-quality`: Repository quality gates, Visual Edit invariants, parser ownership, deterministic performance evidence, and CI expectations.

### Modified Capabilities

- `markdown-editing`: Require an explicit, current Visual Edit support/fallback classification and invariant coverage for every visual block/edit strategy.
- `project-documentation`: Require contributor documentation and capability metadata to reflect current Visual Edit behavior and executable verification commands.

## Impact

Affected areas are `.github/workflows/`, `scripts/`, `docs/`, OpenSpec capability metadata/deltas, and focused Rust tests around the source-mapped visual model. The change exercises but does not alter the canonical-source, per-document-version cache, shared `Arc`, stable `VisualBlockId`, memoized highlighting/math/image, virtualization, or debounced Split/Read invariants. CI gains Node/OpenSpec installation plus workspace Rust tests; no runtime API or persisted-data migration is introduced.
