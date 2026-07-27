## ADDED Requirements

### Requirement: P1 recovery and block-authoring workflows SHALL be localized
Every user-visible recovery-manager and block-authoring string SHALL be provided for every supported language, including recovery states and bulk/per-entry actions, slash command names and empty results, block transform/duplicate/delete/move labels, stale/unsupported feedback, and drag/reorder status. Catalog completeness SHALL be enforced by exhaustive compilation or an explicit all-language catalog test.

#### Scenario: Recovery manager follows active language
- **WHEN** recovery snapshots are available and the interface language changes
- **THEN** manager title, entry states, Restore, Discard, Restore All, Discard All, and status feedback render in the active language

#### Scenario: Block command chrome follows active language
- **WHEN** the slash palette or block menu is visible
- **THEN** every command and operation label renders through the active language catalog
- **AND** no newly introduced workflow text is hard-coded in English

#### Scenario: P1 catalog remains complete
- **WHEN** a P1 workflow message is added
- **THEN** tests fail until every supported language contains a non-empty translation
