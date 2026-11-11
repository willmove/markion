## ADDED Requirements

### Requirement: Session-only visual code folding
An ordinary code block in Visual Edit SHALL provide a localized fold/expand control and a compact folded summary. Folding SHALL be isolated per document tab and SHALL NOT change Markdown, version, dirty state or undo history. Explicit navigation or editing into a folded payload SHALL expand it before positioning input. Stale or removed code-block identities SHALL not retain unrelated folding state.

#### Scenario: Fold and expand code
- **WHEN** the user folds a code block and expands its summary
- **THEN** the exact highlighted payload becomes visible again with no source or history mutation

#### Scenario: Navigate to folded code
- **WHEN** an explicit source navigation targets a folded code payload
- **THEN** that code block expands and the canonical caret remains visible
