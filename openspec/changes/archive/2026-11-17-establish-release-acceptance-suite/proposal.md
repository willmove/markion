## Why

Release acceptance evidence is scattered across historical reports, application callback replays, and unfinished platform matrices. Maintainers and coding agents need one reproducible suite that identifies the candidate, supplies owned fixtures, specifies observable outcomes, and records missing evidence honestly.

## What Changes

- Add a Chinese-language release acceptance entry point, human and Codex execution guides, stable case identifiers with detailed procedures, and platform/package coverage rules under `docs/testing/`.
- Supply synthetic editable fixtures, deterministic fixture preparation and comparison utilities, and templates for per-case results, environment matrices, defects and release decisions.
- Link the canonical release process to candidate acceptance before tagging and final-artifact checks after packaging; retain existing required manual gates and historical evidence.
- Validate documentation links, case/template coverage, fixture generation, byte comparisons and OpenSpec artifacts without claiming to have run native acceptance.

Non-goals: changing application behavior, running a release campaign, publishing a version, certifying unavailable platforms, replacing existing Rust/GPUI tests, or providing a real Office-producer compatibility corpus.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `engineering-quality`: define the maintained release acceptance suite, reproducible fixture and evidence contract, and integration with release readiness.

## Impact

Documentation and standard-library-only test-material tooling under `docs/testing/`, plus references in `docs/release-process.md`. No Rust runtime, dependencies, packaging behavior, or per-document-version cache invariants change. Existing unrelated working-tree edits are preserved.
