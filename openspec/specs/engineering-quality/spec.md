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

### Requirement: Indexed Visual Edit identity reconciliation
Visual Edit identity reconciliation SHALL use source-range indexing or an equivalent bounded lookup strategy rather than scanning the complete new block list once for every old block. Reconciliation SHALL preserve the existing exact-source, metadata, occurrence, one-to-one successor, and fresh-identity rules. Candidate checks for an ordinary localized edit whose old and new source ranges are unique SHALL be bounded by two candidate inspections per old block. A diagnostic profile SHALL distinguish semantic update, VisualBlock construction, and identity reconciliation and SHALL report identity-candidate checks separately from elapsed time.

#### Scenario: Localized edit uses bounded candidate work
- **WHEN** a localized source edit is reconciled against an ordinary document whose old and new VisualBlock source ranges are unique
- **THEN** identity candidate checks do not exceed two inspections per old block
- **AND** every resulting VisualBlock and retained or fresh identity equals the established reconciliation contract

#### Scenario: Duplicate or empty ranges remain occurrence-safe
- **WHEN** a document contains duplicate or empty VisualBlock source ranges
- **THEN** reconciliation considers candidates in document order without overwriting them in a single-value index
- **AND** claimed or edited occurrences cannot donate their identity to an unrelated block

#### Scenario: Profile explains work without becoming a timing gate
- **WHEN** the diagnostic VisualBlock profile is requested
- **THEN** it reports cache disposition, semantic-update, VisualBlock-construction, and identity-reconciliation timings, block counts, and candidate checks
- **AND** correctness and CI performance evidence use semantic comparisons and deterministic work counts rather than a fixed elapsed-time threshold

### Requirement: Local visual edits reuse independent derived regions
After a localized edit in an independently parseable Markdown region, the editor SHALL reuse unchanged regions' visual derivation rather than rerun inline parsing and direct-editor discovery for every region. Initial region partitioning SHALL examine semantic blocks in document order with bounded linear work. Published semantic content and source ranges MUST equal complete derivation; global or ambiguous context MUST use a correctness fallback. Diagnostic counters SHALL distinguish rebuilt visual blocks from reused visual blocks.

#### Scenario: Local edit in a large independent document
- **WHEN** one paragraph changes in a document containing hundreds of independent regions
- **THEN** only affected regions rebuild visual payloads, unchanged regions reuse them, and exact current source offsets remain valid
- **AND** the result is equivalent to a fresh complete derivation

#### Scenario: Global Markdown context changes
- **WHEN** references, front matter, or uncertain region boundaries prevent safe local derivation
- **THEN** complete derivation preserves source ownership and content without publishing stale ranges

### Requirement: Performance evidence distinguishes model and desktop behavior
Editing performance verification SHALL report fixture size, build configuration, opening and edit timings, and memory attribution. Model timing MUST NOT be described as keyboard-to-screen timing. Desktop verification SHALL exercise the actual built application and record observed interaction results and any unverified platform-specific input.

#### Scenario: Contributor verifies large documents
- **WHEN** release performance is verified for 100 KB, 300 KB, 600 KB and 1 MB mixed Markdown
- **THEN** opening, local edit distributions and retained memory are recorded separately from actual desktop interaction observations

### Requirement: Large-document search has bounded context and stale-safe background results
Noncritical search refresh after editing a large document SHALL run outside synchronous input handling. Background results SHALL be published only for the same live document identity, version, query and options. Search result line/column attribution SHALL avoid rescanning the full source prefix for every ordered match, and retained snippets SHALL contain bounded UTF-8-safe context rather than a complete arbitrarily long source line per match.

#### Scenario: Dense matches on a long line
- **WHEN** a document contains thousands of matches on one long line
- **THEN** every match has the exact source range and Unicode line/column position, while each result retains a bounded snippet

#### Scenario: Old search finishes after document replacement
- **WHEN** a search refresh was scheduled for a document that is edited or replaced before publication
- **THEN** its stale results do not populate the current document's search state

### Requirement: Maintained release acceptance suite
The repository SHALL provide a single release acceptance entry point under `docs/testing/` with separate human and coding-agent execution instructions. The suite SHALL identify cases by stable identifiers and specify prerequisites, owned fixtures, ordered actions, observable expected outcomes, variants, and evidence for persistence/recovery, editing/input, rendering/tables/resources, workspace, import/export, Git, AI, and platform/package/update behavior. It MUST distinguish implemented behavior from proposals and historical verification.

#### Scenario: Maintainer starts acceptance
- **WHEN** a maintainer opens the testing entry point
- **THEN** they can select the platform/package matrix, prepare synthetic material, follow detailed cases, and create an execution record without reconstructing historical reports

#### Scenario: Coding agent cannot perform an interaction
- **WHEN** an agent lacks an OS input gesture, live input method, package, browser, viewer, or platform required by a case
- **THEN** it records the affected variant as blocked or skipped with a reason
- **AND** callback replay, simulated layout, a screenshot, or a previous build's result does not become a native pass

### Requirement: Reproducible acceptance material and evidence
Acceptance material SHALL be repository-owned synthetic content or explicitly documented external producer samples with provenance. Preparation SHALL use a separate run directory and preserve existing runs and fixture originals. The suite SHALL provide byte-sensitive text variants, structured editing and resource samples, deterministic large-document material, and a manifest identifying inputs. Records SHALL identify candidate commit and dirty state, package and executable digests, environment and external tool versions, case variants, execution method, actual outcomes, evidence, failures, and exceptions. Editing acceptance SHALL compare complete persisted output and semantic undo outcomes where applicable.

#### Scenario: Fresh fixture preparation
- **WHEN** a tester prepares material in a new run directory
- **THEN** editable working copies and independent baselines are available with input identities
- **AND** preparation refuses to overwrite an existing run

#### Scenario: Candidate or executable changes
- **WHEN** a tested source revision, packaged resource, or executable changes
- **THEN** the record identifies affected results for rerun instead of silently carrying them forward

### Requirement: Release readiness references current native evidence
The canonical release process SHALL reference the acceptance suite before tagging and require final tagged-artifact identity and smoke checks before publication is reported complete. Applicable required case variants SHALL remain distinguishable as passed, failed, blocked, skipped, not run, or not applicable. A release-specific maintainer exception SHALL record its scope and reason without reclassifying an unverified case as passed or carrying the exception to another release. Existing mandatory browser/viewer and package/signature gates SHALL remain effective.

#### Scenario: Required manual row is pending
- **WHEN** a release record contains an applicable required case that has no passing evidence
- **THEN** ordinary readiness remains incomplete unless the maintainer explicitly records an exception for that release
- **AND** automated checks and historical release exceptions do not supply that authorization

#### Scenario: Final tag packages are available
- **WHEN** the tag workflow produces installers and authenticated update metadata
- **THEN** final artifact identities, actual installed application smoke results, and publication checks are linked to the release record
- **AND** preparatory development-binary results remain separately labeled
