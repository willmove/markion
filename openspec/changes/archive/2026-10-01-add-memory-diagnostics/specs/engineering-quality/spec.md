## MODIFIED Requirements

### Requirement: Deterministic incremental performance gates
Source-mapped Visual Edit performance correctness SHALL be gated with deterministic work and identity evidence rather than machine-dependent elapsed-time thresholds. Localized-edit tests SHALL bound newly parsed regions, prove reuse of unchanged regions and stable block identities, preserve shared cache identity for interaction-only state, and compare incremental blocks, outlines, and source ranges with a fresh full derivation. Retained-memory correctness SHALL be gated the same way: memory accounting tests SHALL assert machine-independent relationships — that an empty site reports zero, that a report is side-effect free and repeatable, that per-tab totals grow when a tab is opened and return to their prior value when it is closed, and that opening a tab leaves process-global render caches unchanged — and MUST NOT assert absolute byte thresholds, which vary by platform and allocator. Wall-clock large-document benchmarks and absolute memory figures SHALL be documented as informational diagnostics and MUST NOT be a required merge gate without dedicated stable benchmark hardware.

#### Scenario: Local edit occurs in a large document
- **WHEN** a UTF-8-safe localized edit is applied near the beginning or middle of a large document
- **THEN** deterministic counters show bounded new region parsing and reuse of unchanged regions
- **AND** incremental output equals a fresh full derivation

#### Scenario: Contributor runs the wall-clock benchmark
- **WHEN** a contributor invokes the release-mode large-document benchmark
- **THEN** the output is identified as diagnostic timing evidence
- **AND** ordinary CI success does not depend on a fixed microsecond threshold

#### Scenario: Memory accounting is gated
- **WHEN** memory accounting tests run in CI
- **THEN** they assert relative attribution and release relationships that hold on any platform
- **AND** they do not fail on a platform-dependent absolute byte figure
