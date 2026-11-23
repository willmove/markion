## ADDED Requirements

### Requirement: Native Markdown tables render math in their cells

Native Markdown table cells SHALL render `$...$` inline math and `$$...$$` display math through the same offline typesetting, presentation cache, and fallback rules as math in prose, in Split Preview, Read mode, and unfocused Visual Edit. Every math expression in a header or body cell SHALL be scheduled for rendering exactly like a prose expression, and an expression whose complete presentation key matches a formula elsewhere in the document SHALL reuse that cache entry instead of being typeset again.

In Split Preview and Read mode, a valid expression in a cell SHALL appear as a single baseline-aligned formula atom among the cell's text, and copying a selection that contains it SHALL preserve its complete authored syntax. In Visual Edit, an unfocused cell SHALL show each ready expression as a formula atom in the cell's rendered layout. An expression whose render is pending or has failed SHALL keep its exact authored source text in that cell without displacing the cell's other rendered content. A cell that owns the caret or a selection endpoint SHALL keep its existing whole-cell source reveal, showing the authored `$...$` or `$$...$$` syntax for editing, regardless of the Keep inline syntax hidden preference.

A formula atom wider than its cell's content width SHALL scale down proportionally to fit that width in every surface, and SHALL NOT overflow into a neighboring cell or cover its content.

Pointer interaction with a formula atom in an unfocused Visual Edit cell SHALL place the caret at the expression's authored leading or trailing source boundary according to the clicked half. Shift-click and Alt-click SHALL keep the existing rectangular cell selection, and a double-click SHALL select the complete authored expression. Rendering, completing, or clicking a cell formula SHALL NOT change document text, document version, dirty state, undo history, or per-version derived Markdown caches.

#### Scenario: Unfocused Visual Edit cell renders its formulas
- **WHEN** a GFM table cell contains `$N/4$` beside other text and the cell does not own the caret
- **THEN** the cell shows a typeset formula baseline-aligned with its neighboring text instead of the literal `$N/4$`
- **AND** focusing the cell shows its complete authored source, including `$N/4$`, for editing

#### Scenario: Read and Split Preview cells render formulas
- **WHEN** a GFM table containing math cells is displayed in Read mode or Split Preview
- **THEN** each valid expression appears as a typeset formula atom inside its cell
- **AND** a copied selection that spans the expression contains its complete authored math syntax

#### Scenario: Cell formulas join the shared render cache
- **WHEN** a document contains the same expression in a paragraph and in a table cell
- **THEN** at most one render job is scheduled for that expression's presentation key
- **AND** both locations show the same rendered result

#### Scenario: Pending or failed cell formula keeps authored source
- **WHEN** a cell expression's render is pending or has failed
- **THEN** the unfocused cell shows that expression's exact authored source text in place
- **AND** the rest of the cell stays rendered and the table remains editable

#### Scenario: Formula wider than its cell scales down
- **WHEN** a column is narrower than a formula it contains, for example a seven-column table in a narrow Split Preview pane
- **THEN** the formula scales down proportionally to the cell's content width in Split Preview, Read mode, and Visual Edit
- **AND** it does not overflow into the neighboring cell or cover that cell's text

#### Scenario: Focused cell reveals math source with inline syntax hidden
- **WHEN** Keep inline syntax hidden is on and the caret moves into a Visual Edit table cell containing `$N/4$`
- **THEN** the cell shows its authored source including the `$N/4$` delimiters
- **AND** the caret move alone leaves document text and document version unchanged

#### Scenario: Clicking a cell formula resolves to its source boundary
- **WHEN** the user clicks the trailing half of a rendered formula in an unfocused Visual Edit cell
- **THEN** the caret lands at the expression's trailing authored source boundary inside that cell
- **AND** document text, document version, dirty state, and undo history are unchanged

#### Scenario: Modified click keeps rectangular cell selection
- **WHEN** the user Shift-clicks a rendered formula in an unfocused Visual Edit cell while another cell owns the caret
- **THEN** the rectangular cell selection extends to the clicked cell exactly as it would for that cell's text
