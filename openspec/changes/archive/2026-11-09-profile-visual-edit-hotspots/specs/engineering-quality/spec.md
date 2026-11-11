## ADDED Requirements

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
