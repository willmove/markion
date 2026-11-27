## ADDED Requirements

### Requirement: Rendered modes agree on the parsed diagram body

Visual Edit, Split Preview, and Read mode SHALL share the rendering result of the same diagram fence for the same theme. Visual Edit SHALL use the Markdown-parsed fence body for both scheduling and displaying a diagram. The editable payload SHALL retain the exact authored source.

#### Scenario: A valid fence finishes rendering in Visual Edit

- **WHEN** a supported diagram fence with LF or CRLF line endings or Markdown container indentation finishes rendering
- **THEN** Visual Edit shows the resulting diagram instead of the loading placeholder
- **AND** switching between rendered modes reuses the same completed result for that fence and theme
