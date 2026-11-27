## MODIFIED Requirements

### Requirement: Session-only visual code folding
When the Visual Edit code folding preference is enabled, an ordinary code block in Visual Edit SHALL provide a localized fold/expand control and a compact folded summary. With the preference disabled, ordinary code blocks SHALL show their full editable payload with the fold control hidden. Folding SHALL be isolated per document tab and SHALL NOT change Markdown, version, dirty state or undo history. Explicit navigation or editing into a folded payload SHALL expand it before positioning input. Stale or removed code-block identities SHALL not retain unrelated folding state. Disabling the preference SHALL clear folded states across tabs.

#### Scenario: Fold and expand code
- **WHEN** the folding preference is enabled and the user folds a code block and expands its summary
- **THEN** the exact highlighted payload becomes visible again with no source or history mutation

#### Scenario: Navigate to folded code
- **WHEN** an explicit source navigation targets a folded code payload
- **THEN** that code block expands and the canonical caret remains visible

#### Scenario: Disable folding while blocks are collapsed
- **WHEN** the user disables the preference with folded blocks in open tabs
- **THEN** the visible blocks expand immediately and the fold controls disappear
- **AND** previously folded blocks in inactive tabs appear expanded when revisited, including if the preference was re-enabled first
