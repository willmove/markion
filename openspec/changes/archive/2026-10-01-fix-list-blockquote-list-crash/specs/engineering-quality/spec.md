## ADDED Requirements

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
