# engineering-quality Specification

## Purpose
Covers executable repository quality gates, Visual Edit invariant evidence, deterministic incremental-performance checks, and Markdown parser ownership.
## Requirements
### Requirement: Executable repository quality gate
The repository SHALL provide one documented local quality command and an automated pull-request workflow that run Rust formatting checks, the complete Cargo workspace test suite, and strict non-interactive validation of every active OpenSpec change and stable spec. Any failed command MUST fail the gate, while tests explicitly marked ignored because they require unavailable external tools or networks SHALL remain reported rather than silently reclassified as passing coverage.

#### Scenario: Contributor runs the local gate
- **WHEN** a contributor invokes the documented repository quality command from the workspace root
- **THEN** formatting, all workspace tests, and strict OpenSpec validation run in a deterministic order
- **AND** the command exits non-zero at the first failed gate

#### Scenario: Pull request changes the repository
- **WHEN** a pull request or main-branch push triggers the quality workflow
- **THEN** CI runs the same formatting, workspace-test, and strict-spec contract with pinned tool setup
- **AND** packaging remains owned by the separate release workflow

### Requirement: Visual Edit invariant evidence
Every Visual Edit presentation or mutation strategy SHALL have executable evidence for each affected ownership layer: exact UTF-8 source ranges and fallback classification in pure tests, canonical edit/version/id/selection behavior in document tests, rendered input/navigation/IME/history behavior in GPUI tests, and parser/export compatibility in workspace tests when semantics cross crate boundaries. A new visual block editor or fallback strategy MUST update the maintained support matrix and the affected evidence before its change can be archived.

#### Scenario: A visual editor strategy is added or changed
- **WHEN** a change introduces or modifies a rendered editor, direct field editor, passive source position, or conservative source island
- **THEN** its proposal identifies source ownership and fallback conditions
- **AND** its implementation updates the support matrix and adds tests at every affected ownership layer

#### Scenario: A stale widget event arrives
- **WHEN** a direct-widget event targets an old document version, block identity, or field range
- **THEN** executable tests prove that the event is rejected before canonical source mutation

### Requirement: Deterministic incremental performance gates
Source-mapped Visual Edit performance correctness SHALL be gated with deterministic work and identity evidence rather than machine-dependent elapsed-time thresholds. Localized-edit tests SHALL bound newly parsed regions, prove reuse of unchanged regions and stable block identities, preserve shared cache identity for interaction-only state, and compare incremental blocks, outlines, and source ranges with a fresh full derivation. Wall-clock large-document benchmarks SHALL be documented as informational diagnostics and MUST NOT be a required merge gate without dedicated stable benchmark hardware.

#### Scenario: Local edit occurs in a large document
- **WHEN** a UTF-8-safe localized edit is applied near the beginning or middle of a large document
- **THEN** deterministic counters show bounded new region parsing and reuse of unchanged regions
- **AND** incremental output equals a fresh full derivation

#### Scenario: Contributor runs the wall-clock benchmark
- **WHEN** a contributor invokes the release-mode large-document benchmark
- **THEN** the output is identified as diagnostic timing evidence
- **AND** ordinary CI success does not depend on a fixed microsecond threshold

### Requirement: Markdown parser ownership
`pulldown-cmark` SHALL remain the root application's semantic Markdown parser and canonical preview-block classifier. Visual Edit boundary helpers MAY recognize exact field, payload, delimiter, and table-cell subranges only within an already-classified semantic block; they MUST round-trip the relevant authored source, MUST reuse the shared table implementation where applicable, and MUST select a complete source-backed fallback instead of guessing. Workspace member parsers and exporters MUST NOT create an independent Visual Edit mutation model.

#### Scenario: Exact boundary proof succeeds
- **WHEN** an already-classified block matches a supported byte-exact direct-editor form
- **THEN** the boundary helper returns typed UTF-8 ranges contained by that block
- **AND** the canonical semantic block and authored delimiters remain owned by the root parser/document model

#### Scenario: Boundary proof is ambiguous
- **WHEN** malformed, multiline, reference, nested, unclosed, or otherwise unsupported syntax prevents exact range proof
- **THEN** the helper returns no direct editor metadata
- **AND** Visual Edit retains the complete conservative source-backed path
