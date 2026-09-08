## ADDED Requirements

### Requirement: Visual Edit focus presentation
In Visual Edit, focus mode SHALL keep the current caret-owning source-backed block at normal opacity and visibly dim all other blocks, including rich prose, nested list/quote leaves, tables, code, rendered atoms, and conservative source islands. Ownership SHALL follow the canonical caret endpoint of a selection and the existing unique visual caret mapping at block boundaries and document end. Disabling focus mode SHALL restore normal opacity to every row. Read and Split Preview rendered panes SHALL retain their normal presentation.

#### Scenario: Focus follows the Visual Edit caret
- **WHEN** focus mode is enabled and the caret moves between Visual Edit paragraphs by pointer or keyboard
- **THEN** the newly current paragraph renders at normal opacity and the previous paragraph becomes dimmed
- **AND** a selection uses its caret endpoint rather than keeping every selected block bright

#### Scenario: Structured and conservative content respects focus
- **WHEN** focus mode is enabled in a Visual Edit document containing quoted/list leaves, tables, code, rendered atoms, or conservative source islands
- **THEN** the caret-owning row remains at normal opacity and other rows are dimmed regardless of their rendering branch

#### Scenario: Focus toggles and context changes are presentation only
- **WHEN** focus mode is toggled or the active tab, caret, selection, or editable view changes
- **THEN** Visual Edit focus presentation reflects the current preference and active document caret
- **AND** a focus-only change does not alter source text, dirty state, undo history, document version, shared derived Markdown caches, or cached source text

#### Scenario: Disabled mode and preview are unchanged
- **WHEN** focus mode is disabled or content is displayed in Read or the rendered pane of Split Preview
- **THEN** content retains normal opacity

#### Scenario: Boundary and empty-document caret
- **WHEN** the Visual Edit caret is at a shared row boundary, the document end, or in an empty document
- **THEN** focus presentation uses the same unique row ownership as caret painting without out-of-range access
