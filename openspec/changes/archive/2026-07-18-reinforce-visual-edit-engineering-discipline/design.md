## Context

Three Visual Edit changes established interaction fidelity, incremental source mapping, and direct code/math/image/table editors. The implementation now exceeds several repository descriptions: the Markdown capability purpose and both READMEs still describe code, math, and tables as source-only or not directly editable. Verification is also fragmented across developer memory, the release process, focused Rust tests, and an informational benchmark; pull requests have no small, explicit quality workflow that runs strict OpenSpec validation.

The engineering contract must preserve this data flow:

```text
canonical Markdown
  -> pulldown-cmark semantic blocks
  -> exact boundary proof + source-mapped VisualBlock metadata
  -> rendered/direct/source-island strategy
  -> validated canonical source edit
  -> versioned incremental derivation + stable identity reconciliation
  -> pure differential tests + rendered GPUI interaction tests
```

No quality tool may introduce a second persisted model, recompute derived state on cursor-only interaction, or turn machine-dependent timing into a flaky merge gate.

## Goals / Non-Goals

**Goals:**

- Provide one local command and one pull-request workflow for formatting, workspace tests, and strict OpenSpec validation.
- Publish a current Visual Edit support/fallback matrix and keep the English/Chinese project overviews aligned with it.
- Make source-range, edit-validation, incremental/full equivalence, stable identity, cache identity, IME, undo, multi-tab, and source round-trip expectations explicit.
- Record parser ownership so exact field recognizers remain boundary proof helpers rather than a competing Markdown parser.
- Use deterministic parsed/reused-region counters and semantic equality as CI performance evidence while retaining release-mode wall-clock benchmarks for diagnosis.

**Non-Goals:**

- Adding a visual editor, syntax feature, benchmark framework, runtime telemetry, persisted rich-text tree, or platform-specific latency promise.
- Moving existing test suites or duplicating the release workflow.

## Decisions

### 1. Use a checked-in PowerShell entry point and a dedicated CI workflow

`scripts/check-quality.ps1` runs `cargo fmt --all -- --check`, `cargo test --workspace`, and `openspec validate --all --strict --no-interactive`, failing immediately on any non-zero exit. `.github/workflows/quality.yml` invokes the equivalent pinned toolchain on pull requests and main pushes. PowerShell is already the project's documented local shell and is available as `pwsh` on GitHub runners.

Adding the commands only to release documentation was rejected because drift would still be discovered late. Folding them into packaging jobs was rejected because quality feedback should not wait for native installers.

### 2. Treat the support matrix as maintained architecture documentation

`docs/visual-editing-quality.md` records each construct's normal presentation, editable canonical range, fallback trigger, and required evidence. README summaries link to the matrix and stop claiming implemented direct editors are absent. A focused Rust test includes the document and checks that every named `VisualBlockEditor`/fallback family remains represented.

Generating user documentation from Rust enums was rejected because the matrix contains rationale and fallback semantics that are not suitable as runtime data.

### 3. Layer verification by ownership boundary

Pure model tests own UTF-8 ranges, delimiter preservation, escaping, stale edits, table reflow, and incremental/full equivalence. GPUI tests own painted projection, pointer/keyboard handoff, IME geometry, undo, mode switching, virtualization, and multi-tab isolation. Workspace tests own parser/exporter semantic compatibility. New visual strategies must add evidence at every affected layer and update the matrix.

Snapshot-only UI tests were rejected because source selection and cache/version invariants require semantic assertions.

### 4. Keep performance gates deterministic

CI asserts bounded region parsing, suffix/prefix reuse, stable `Arc` identity, and equality with a fresh full derivation. `examples/bench_large_doc.rs` remains an informational release-mode tool and is updated to describe the current incremental Visual Edit path. Fixed microsecond thresholds were rejected because shared CI hardware, link mode, and GPUI availability make them noisy and non-portable.

### 5. Declare parser ownership explicitly

`pulldown-cmark` remains the semantic Markdown parser in the root document model. `src/visual.rs` may prove exact byte boundaries only for already-classified preview blocks. `src/table.rs` is the shared GFM table mutation/range implementation. Workspace member parsers support their crate/export contracts and must not become a second Visual Edit mutation model. Any new recognizer must round-trip its source slice and fall back when uncertain.

### 6. Repair metadata only with this tracked OpenSpec change

The delta specs make current documentation and strategy classification normative. During the same completed change, stale project/OpenSpec context and the `markdown-editing` purpose metadata are corrected to match the archived requirements. This is a metadata repair, not a behavior change; the archived change remains the audit trail.

## Risks / Trade-offs

- **[CI duplicates some release compilation]** -> Keep quality jobs limited to format, tests, and specs; packaging remains in `release.yml`.
- **[The support matrix drifts]** -> Link it from both READMEs and enforce required strategy/fallback terms in a focused test.
- **[OpenSpec CLI changes break CI]** -> Pin the npm package major/minor used by the repository and update it intentionally.
- **[A timing regression passes deterministic gates]** -> Keep the release benchmark documented and use it for investigations; only promote a timing threshold after a stable dedicated runner exists.
- **[Boundary helpers accumulate semantic parsing]** -> Require parser-ownership review and fallback tests in every visual-editor proposal.

## Migration Plan

1. Add the delta specs, quality documentation, local entry point, and CI workflow.
2. Update the READMEs, OpenSpec context, and stale Visual Edit purpose metadata.
3. Add documentation/strategy coverage plus focused deterministic invariant regressions.
4. Run the local quality entry point and validate the change.
5. Archive the change so the engineering-quality capability and modified documentation requirements become stable specs.

Rollback removes the workflow/script/docs test together; runtime editing and persisted Markdown are unaffected.

## Open Questions

None. Hardware latency thresholds remain deliberately informational until dedicated benchmark infrastructure exists.
