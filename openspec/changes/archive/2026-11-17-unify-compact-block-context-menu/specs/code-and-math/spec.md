## ADDED Requirements

### Requirement: Code backgrounds respect the document column across tabs
Code-block backgrounds, headers and editable payloads SHALL remain within the document content column across tab activation, new/close/reopen transitions and viewport reflow. Long payload lines SHALL follow the existing wrapping/overflow policy without expanding the background beyond the column. Presentation changes SHALL not mutate canonical source or history, or introduce extra derivation invalidation beyond the established dormant-tab policy.

#### Scenario: Return to a tab containing code
- **WHEN** the user opens or closes a new tab and returns to an existing document containing a code block
- **THEN** the code background remains bounded by the same content column as adjacent blocks without needing fold/unfold
- **AND** source and undo history remain unchanged, and ordinary per-version caching and dormant-tab eviction are preserved
