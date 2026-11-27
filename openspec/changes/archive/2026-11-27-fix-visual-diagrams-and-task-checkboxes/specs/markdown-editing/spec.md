## ADDED Requirements

### Requirement: Visual Edit task markers use consistent square dimensions

When task markers are rendered in Visual Edit, the checked and unchecked boxes SHALL use the same width and height at the configured rendered font size and align with the first line of their item. Only the checked state SHALL display a check mark.

#### Scenario: Mixed task states align in one list

- **WHEN** Visual Edit displays adjacent checked and unchecked tasks with their source prefixes hidden
- **THEN** both squares occupy the same dimensions and align with the first line of their respective items
- **AND** toggling either square keeps the marker and item text in place
