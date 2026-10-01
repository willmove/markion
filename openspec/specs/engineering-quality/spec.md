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
Every Visual Edit presentation or mutation strategy SHALL have executable evidence for each affected ownership layer: exact UTF-8 source ranges and WYSIWYG coverage classification in pure tests, canonical edit/version/id/selection behavior in document tests, rendered input/navigation/IME/history behavior in GPUI tests, and parser/export compatibility in workspace tests when semantics cross crate boundaries. A new visual block editor, a new WYSIWYG rendering for a previously gapped construct, or any change to the WYSIWYG coverage matrix MUST update the maintained coverage matrix and the `WYSIWYG coverage roadmap` (in the `markdown-editing` capability) before its change can be archived.

#### Scenario: A visual editor strategy is added or changed
- **WHEN** a change introduces or modifies a rendered editor, a progressive-reveal editor, or moves a construct out of the WYSIWYG coverage gap class
- **THEN** its proposal identifies source ownership and the resulting WYSIWYG coverage class
- **AND** its implementation updates the coverage matrix (and, if a gap was closed, the roadmap) and adds tests at every affected ownership layer

#### Scenario: A stale widget event arrives
- **WHEN** a direct-widget event targets an old document version, block identity, or field range
- **THEN** executable tests prove that the event is rejected before canonical source mutation

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

### Requirement: Markdown parser ownership
`pulldown-cmark` SHALL remain the root application's semantic Markdown parser and canonical preview-block classifier. Visual Edit boundary helpers MAY recognize exact field, payload, delimiter, and table-cell subranges only within an already-classified semantic block; they MUST round-trip the relevant authored source, MUST reuse the shared table implementation where applicable, and MUST NOT mutate an inferred rendered tree. When an exact range proof fails, the construct SHALL be classified as a WYSIWYG coverage gap under the `markdown-editing` capability's `WYSIWYG coverage roadmap` and shown as raw source only as a transitional affordance — the editor MUST NOT guess a rendered-tree mutation. Workspace member parsers and exporters MUST NOT create an independent Visual Edit mutation model.

#### Scenario: Exact boundary proof succeeds
- **WHEN** an already-classified block matches a supported byte-exact direct-editor form
- **THEN** the boundary helper returns typed UTF-8 ranges contained by that block
- **AND** the canonical semantic block and authored delimiters remain owned by the root parser/document model

#### Scenario: Boundary proof is ambiguous
- **WHEN** malformed, multiline, reference, nested, unclosed, or otherwise unsupported syntax prevents exact range proof
- **THEN** the helper returns no direct editor metadata
- **AND** Visual Edit classifies the construct as a WYSIWYG coverage gap and shows raw source as a transitional affordance
- **AND** the gap is tracked on the `WYSIWYG coverage roadmap` for closure by a future change

### Requirement: Workspace tests MUST NOT touch the developer preferences file
The workspace test suite MUST NOT read preference values from, or write preference values to, the developer machine's real preferences file (`config.toml` in the Markion config directory used by the desktop app). Tests that exercise preference persistence MUST use an isolated file. Tests that mutate in-memory preferences without an isolated file MUST leave the developer preferences file unchanged. Session-file isolation already follows this contract; preferences SHALL follow the same isolation.

#### Scenario: Preference-mutating test leaves developer config unchanged
- **WHEN** a GPUI test changes source-editor font size or any other persisted preference without redirecting preferences to an isolated file
- **THEN** the developer machine's `config.toml` is not created, overwritten, or otherwise modified

#### Scenario: Isolated preference persistence still round-trips
- **WHEN** a test points preferences at an isolated file and saves a non-default source font size
- **THEN** that isolated file records the value
- **AND** the developer machine's `config.toml` remains unchanged

#### Scenario: Tests start from documented defaults
- **WHEN** a test constructs the application without supplying an isolated preferences file
- **THEN** source font size, reading font size, and other preference fields take their documented defaults rather than whatever is stored on the developer machine

### Requirement: Cross-container source ownership has executable safety evidence
Changes to Markdown container routing SHALL include deterministic pure tests at the parser and Visual Edit projection layers. The tests MUST prove destination ownership, authored block order, non-overlapping container boundaries, in-bounds UTF-8 source ranges, complete canonical source coverage, and non-panicking fallback for malformed derived input. Regression fixtures SHALL include the smallest failing container topology and at least one realistic UTF-8 variant without depending on a developer's private document or machine state.

#### Scenario: Parser ownership regression is exercised
- **WHEN** the test suite derives preview blocks for a list item containing a blockquote that contains a list
- **THEN** assertions distinguish document-level blocks from quoted children and verify their exact ordering and range containment
- **AND** the test fails if routing is inferred from the later current container state

#### Scenario: Visual projection safety regression is exercised
- **WHEN** pure Visual Edit tests project both valid nested-container output and deliberately malformed range input
- **THEN** valid input has ordered, complete, UTF-8-safe coverage without unsupported degradation
- **AND** malformed input uses source-backed fallback without a panic

#### Scenario: Verification is independent of private application state
- **WHEN** contributors run the focused or complete workspace test suite
- **THEN** all cross-container safety fixtures are repository-contained and deterministic
- **AND** no session file, private note, WER process, window manager, or external service is required

### Requirement: Merge-blocker regressions require semantic boundary evidence
Correctness fixes for cache identity, generated structured text, or parser event reconstruction SHALL include fixtures that exercise the boundary responsible for the defect. Tests MUST compare semantic outcomes rather than only output shape: cache tests compare identities and decoded results for deliberately adversarial equal-length inputs; YAML tests parse generated text and compare typed values; extended-inline tests exercise the default parser and conflicting escape/GFM syntax. Synthetic invalid-state tests SHALL be described as invariant containment and MUST NOT be cited as proof that the same state is reachable through ordinary user input.

#### Scenario: Cache collision regression is exercised deterministically
- **WHEN** a regression test builds two equal-length valid image sources that differ only outside the former sampled regions
- **THEN** it proves their keys and decoded render results are distinct
- **AND** it does not rely on probabilistic random inputs or elapsed-time thresholds

#### Scenario: Generated YAML is validated semantically
- **WHEN** tests cover front-matter rendering or export title overrides
- **THEN** they parse the complete generated front-matter block with the production YAML type
- **AND** they compare the reparsed typed values with the inputs rather than accepting substring assertions alone

#### Scenario: Parser boundary fix protects competing syntax
- **WHEN** tests prove an extended-inline construct across adjacent text events
- **THEN** the same suite also covers the default parser, escaped delimiters, GFM strikethrough, Unicode text, and boundaries adjacent to non-text inline events

#### Scenario: Synthetic invalid state is labeled accurately
- **WHEN** a test injects a malformed visual block or invalid caret/selection range that ordinary parsing does not produce
- **THEN** the test and change report identify it as defense-in-depth invariant evidence
- **AND** they do not claim an ordinary user-triggered reproduction without a separate end-to-end fixture
