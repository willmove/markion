## MODIFIED Requirements

### Requirement: Diagram blocks remain source-backed in Visual Edit
Visual Edit SHALL present a recognized diagram fence as a rendered static diagram image on top of an editable source-backed payload editor. The payload editor SHALL be the only editing path: edits SHALL mutate the canonical Markdown source through the normal document mutation paths used by every other visual editor. Diagram preview state, pointer interaction, theme changes, and backend completion SHALL NOT rewrite the fenced Markdown source, SHALL NOT increment the document version, SHALL NOT touch the dirty flag or undo history, and SHALL NOT add a second rendered-tree editing path. While a diagram result is pending, Visual Edit SHALL display the same localized loading placeholder used in Split Preview; on error it SHALL display the same localized error message together with the authored source. Visual Edit SHALL reuse the same `markion-diagram` backend, registry, sanitizer, rasterizer, and bounded cache as Split Preview and Read mode, so a diagram presented in one mode is the same diagram presented in any other.

#### Scenario: Mermaid fence renders as a diagram with an editable source payload in Visual Edit
- **WHEN** Visual Edit contains a valid fenced block beginning with ` ```mermaid `
- **THEN** the block is presented as a rendered static diagram image above an editable source payload editor
- **AND** the payload editor shows the authored diagram source and edits to it mutate the canonical Markdown source through the normal document mutation paths

#### Scenario: Any registered diagram backend is presented the same way in Visual Edit
- **WHEN** Visual Edit contains a fenced block whose first info-string token resolves to any registered diagram backend alias
- **THEN** Visual Edit presents the rendered diagram using the same backend, sanitizer, rasterizer, and cache as Split Preview / Read mode
- **AND** no mode-specific backend, renderer, or sanitizer is introduced

#### Scenario: Pending diagram shows a localized loading placeholder in Visual Edit
- **WHEN** a diagram render has been scheduled in Visual Edit but has not completed
- **THEN** the presentation slot displays the localized loading placeholder used by Split Preview without blocking the GPUI frame
- **AND** the editable source payload remains available below the placeholder

#### Scenario: Failed diagram shows a localized error and the authored source in Visual Edit
- **WHEN** the diagram backend rejects the authored source in Visual Edit
- **THEN** the presentation slot displays the localized error message used by Split Preview for that error category
- **AND** the editable source payload below shows the exact authored diagram source so the user can correct it

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
