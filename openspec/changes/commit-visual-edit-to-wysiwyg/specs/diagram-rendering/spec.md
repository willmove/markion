## RENAMED Requirements

### FROM: Diagram blocks remain source-backed in Visual Edit
### TO: Diagram blocks render WYSIWYG in Visual Edit

## MODIFIED Requirements

### Requirement: Diagram blocks render WYSIWYG in Visual Edit
Visual Edit SHALL present a recognized diagram fence as a rendered static diagram image on top of an editable source payload editor — the rendered form of a diagram block IS the diagram image with its source one keystroke away, so this is rendered WYSIWYG. The payload editor SHALL be the only editing path: edits SHALL mutate the canonical Markdown source through the normal document mutation paths used by every other visual editor. Diagram preview state, pointer interaction, theme changes, and backend completion SHALL NOT rewrite the fenced Markdown source, SHALL NOT increment the document version, SHALL NOT touch the dirty flag or undo history, and SHALL NOT add a second rendered-tree editing path. While a diagram result is pending, Visual Edit SHALL display the same localized loading placeholder used in Split Preview; on error it SHALL display the same localized error message together with the authored source. Visual Edit SHALL reuse the same `markion-diagram` backend, registry, sanitizer, rasterizer, and bounded cache as Split Preview and Read mode, so a diagram presented in one mode is the same diagram presented in any other.

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
