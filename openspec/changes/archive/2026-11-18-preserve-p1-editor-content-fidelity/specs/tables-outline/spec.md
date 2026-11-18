## ADDED Requirements

### Requirement: Native tables SHALL use the active document palette
Native Markdown and unstyled HTML table previews SHALL use readable table surfaces and borders from the active theme. Visual Edit table controls SHALL also follow that theme.

#### Scenario: Dark theme table surfaces
- **WHEN** a dark theme displays Markdown and unstyled HTML tables in Visual Edit, Read or Split Preview
- **THEN** their ordinary cell text has a readable foreground and background
- **AND** table controls in Visual Edit use the same theme

### Requirement: Native Markdown tables SHALL display embedded images in their cells

Native Markdown table previews SHALL render supported inline images among cell text. Visual Edit SHALL reveal the authored image syntax while the cell is focused for editing.

#### Scenario: A Markdown cell contains text and an inline image
- **WHEN** a Markdown table is displayed in Read, Split Preview, or an unfocused Visual Edit cell
- **THEN** the image appears among the cell's text
- **AND** focusing the cell in Visual Edit reveals its editable Markdown
