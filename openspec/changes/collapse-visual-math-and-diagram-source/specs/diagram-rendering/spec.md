## MODIFIED Requirements

### Requirement: Diagram blocks remain source-backed in Visual Edit
Visual Edit SHALL present a recognized diagram fence as a rendered static diagram image by default. A compact source-toggle control SHALL appear in the block's top-right corner while the pointer hovers the block. Activating that control SHALL expand the block to show the rendered diagram (or pending/error chrome) together with an editable source-backed payload editor beneath it. Clicking the rendered diagram itself SHALL NOT expand the source editor. A primary pointer click outside the expanded block SHALL collapse it back to render-only presentation, except that pending or failed diagram results SHALL keep the payload editor visible so the authored source remains correctable. The payload editor SHALL remain the only editing path: edits SHALL mutate the canonical Markdown source through the normal document mutation paths used by every other visual editor. Diagram preview state, pointer interaction, theme changes, expand/collapse, and backend completion SHALL NOT rewrite the fenced Markdown source, SHALL NOT increment the document version, SHALL NOT touch the dirty flag or undo history, and SHALL NOT add a second rendered-tree editing path. While a diagram result is pending, Visual Edit SHALL display the same localized loading placeholder used in Split Preview; on error it SHALL display the same localized error message together with the authored source. Visual Edit SHALL reuse the same `markion-diagram` backend, registry, sanitizer, rasterizer, and bounded cache as Split Preview and Read mode, so a diagram presented in one mode is the same diagram presented in any other.

#### Scenario: Valid Mermaid fence is collapsed by default in Visual Edit
- **WHEN** Visual Edit contains a valid fenced block beginning with ` ```mermaid ` and its source is not expanded
- **THEN** the block is presented as a rendered static diagram image without the editable source payload editor

#### Scenario: Hover reveals the diagram source-toggle control
- **WHEN** the pointer hovers a collapsed registered diagram fence in Visual Edit
- **THEN** a compact source-toggle control is visible in the block's top-right corner

#### Scenario: Toggle expands diagram with an editable source payload
- **WHEN** the user activates the source-toggle control on a registered diagram fence
- **THEN** the block shows the rendered static diagram image above an editable source payload editor
- **AND** the payload editor shows the authored diagram source and edits to it mutate the canonical Markdown source through the normal document mutation paths

#### Scenario: Clicking the diagram does not expand source
- **WHEN** the user primary-clicks the rendered diagram surface of a collapsed registered diagram fence
- **THEN** the source payload editor remains hidden
- **AND** the source-toggle control remains the only expand affordance

#### Scenario: Click outside collapses an expanded valid diagram
- **WHEN** a valid registered diagram fence is expanded and the user primary-clicks outside that block
- **THEN** the block returns to render-only presentation
- **AND** the source payload editor is hidden

#### Scenario: Any registered diagram backend is presented the same way in Visual Edit
- **WHEN** Visual Edit contains a fenced block whose first info-string token resolves to any registered diagram backend alias
- **THEN** Visual Edit presents the rendered diagram using the same backend, sanitizer, rasterizer, and cache as Split Preview / Read mode
- **AND** no mode-specific backend, renderer, or sanitizer is introduced
- **AND** the same collapsed/expand/collapse source affordance is used

#### Scenario: Pending diagram keeps the authored source available in Visual Edit
- **WHEN** a diagram render has been scheduled in Visual Edit but has not completed
- **THEN** the presentation slot displays the localized loading placeholder used by Split Preview without blocking the GPUI frame
- **AND** the editable source payload remains available so the user can continue editing

#### Scenario: Failed diagram shows a localized error and the authored source in Visual Edit
- **WHEN** the diagram backend rejects the authored source in Visual Edit
- **THEN** the presentation slot displays the localized error message used by Split Preview for that error category
- **AND** the editable source payload shows the exact authored diagram source so the user can correct it

#### Scenario: Wide diagram scales down without distortion in Visual Edit
- **WHEN** a rendered diagram's intrinsic width exceeds the available Visual Edit column width
- **THEN** the diagram scales down to the available width with its aspect ratio preserved and remains fully visible
- **AND** horizontal scrolling is available for diagrams wider than the column, matching Split Preview behavior

#### Scenario: Visual Edit diagram render reuses the shared cache
- **WHEN** the same backend, source, and theme has already been rendered for Split Preview, Read mode, or another Visual Edit tab
- **THEN** Visual Edit reuses the cached result and does not schedule a second backend render or rasterization

#### Scenario: Diagram completion does not create an edit
- **WHEN** a pending diagram finishes rendering while Visual Edit is active
- **THEN** the document text, dirty flag, undo history, and document version remain unchanged

#### Scenario: Theme switch does not mutate the document in Visual Edit
- **WHEN** the active Markion theme changes between light and dark while a Visual Edit diagram is displayed
- **THEN** the diagram is re-rendered for the new theme using an independent cache key
- **AND** the document text, dirty flag, undo history, and document version remain unchanged

#### Scenario: Expand and collapse do not edit the document
- **WHEN** the user expands or collapses a diagram source pane without editing the payload
- **THEN** document text, dirty flag, undo history, and document version remain unchanged
