## ADDED Requirements

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
