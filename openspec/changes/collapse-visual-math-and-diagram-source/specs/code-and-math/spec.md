## MODIFIED Requirements

### Requirement: Direct block-math editing in Visual Edit
Visual Edit SHALL present an exactly ranged block-math construct as a rendered formula by default. A compact source-toggle control SHALL appear in the block's top-right corner while the pointer hovers the block. Activating that control SHALL expand the block to show the rendered formula (or readable error/pending state) together with a direct monospaced LaTeX editor for its authored payload. Clicking the rendered formula itself SHALL NOT expand the source editor. A primary pointer click outside the expanded block SHALL collapse it back to render-only presentation, except that pending or invalid formulas SHALL keep the payload editor visible so the authored source remains correctable. Editing the payload SHALL preserve its complete delimiters and unrelated whitespace and SHALL use the same validation, source selection, IME, and undo paths as other text input. Expand/collapse state SHALL be presentation-only and SHALL NOT mutate document text, dirty state, undo history, or derived Markdown caches.

#### Scenario: Valid block math is collapsed by default
- **WHEN** an exactly ranged valid block-math construct appears in Visual Edit and its source is not expanded
- **THEN** only the rendered formula (or a non-editor pending placeholder) is shown
- **AND** the LaTeX payload editor is not visible

#### Scenario: Hover reveals the source-toggle control
- **WHEN** the pointer hovers a collapsed exactly ranged block-math construct
- **THEN** a compact source-toggle control is visible in the block's top-right corner

#### Scenario: Toggle expands formula and LaTeX editor together
- **WHEN** the user activates the source-toggle control on a block-math construct
- **THEN** its rendered formula or readable error/pending state is shown together with the editable LaTeX payload
- **AND** focusing the payload does not replace the whole block with raw delimiter source

#### Scenario: Clicking the formula does not expand source
- **WHEN** the user primary-clicks the rendered formula surface of a collapsed block-math construct
- **THEN** the LaTeX payload editor remains hidden
- **AND** the source-toggle control remains the only expand affordance

#### Scenario: Click outside collapses an expanded valid formula
- **WHEN** a valid block-math construct is expanded and the user primary-clicks outside that block
- **THEN** the block returns to render-only presentation
- **AND** the LaTeX payload editor is hidden

#### Scenario: Invalid or pending LaTeX remains directly editable
- **WHEN** a payload edit makes the formula invalid, rendering is pending, or the render cache reports an error
- **THEN** the editor shows the validation, pending, or render state without discarding the payload editor
- **AND** the authored source remains available for correction even if the block would otherwise be collapsed

#### Scenario: Math payload edit is atomic and lossless
- **WHEN** the user replaces text within the LaTeX payload
- **THEN** one canonical source replacement changes only that payload range
- **AND** one Undo restores the prior formula source, selection, and delimiters

#### Scenario: Expand and collapse do not edit the document
- **WHEN** the user expands or collapses a block-math source pane without editing the payload
- **THEN** document text, dirty flag, undo history, and document version remain unchanged
