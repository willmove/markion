## ADDED Requirements

### Requirement: Square-corner primary document surfaces
The application chrome SHALL render the primary source editor, visual editor, and rendered preview surfaces as square-corner rectangles with zero corner radius in every view mode where those surfaces appear. The surfaces SHALL retain their active-theme background fill, border, padding, scrollbar behavior, and existing input interactions. Rounded styling on secondary controls and content elements is outside this requirement.

#### Scenario: Source editor uses square corners
- **WHEN** the active view mode is Edit or Split Preview
- **THEN** the source editor surface is rendered with square, zero-radius corners

#### Scenario: Visual editor uses square corners
- **WHEN** the active view mode is Visual Edit
- **THEN** the visual editor surface is rendered with square, zero-radius corners

#### Scenario: Preview uses square corners
- **WHEN** the active view mode is Split Preview or Read
- **THEN** the rendered preview surface is rendered with square, zero-radius corners

#### Scenario: Existing surface chrome and behavior are preserved
- **WHEN** a square-corner primary document surface is rendered
- **THEN** its active-theme background fill, border, padding, scrolling, resizing, drag-and-drop handling, and mode-specific visibility behave as before
