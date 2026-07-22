## MODIFIED Requirements

### Requirement: Read mode preview width
In Read mode and Visual Edit mode, the editor SHALL constrain rendered content to a default maximum width of 860px and center that content within the available pane. The editor SHALL provide a persisted "Preview adaptive width" preference that is disabled by default; when enabled, Read mode and Visual Edit mode rendered content SHALL use the full available pane width. This width preference SHALL NOT affect Edit mode or Split Preview mode.

#### Scenario: Read mode defaults to readable width
- **WHEN** the active view mode is Read and Preview adaptive width is disabled
- **THEN** rendered preview content is centered and constrained to a maximum width of 860px

#### Scenario: Adaptive width restores full-width Read mode
- **WHEN** the active view mode is Read and Preview adaptive width is enabled
- **THEN** rendered preview content uses the full available preview pane width

#### Scenario: Visual Edit mode defaults to readable width
- **WHEN** the active view mode is Visual Edit and Preview adaptive width is disabled
- **THEN** rendered visual edit content is centered and constrained to a maximum width of 860px

#### Scenario: Adaptive width restores full-width Visual Edit mode
- **WHEN** the active view mode is Visual Edit and Preview adaptive width is enabled
- **THEN** rendered visual edit content uses the full available pane width

#### Scenario: Split Preview mode remains full pane width
- **WHEN** the active view mode is Split Preview
- **THEN** rendered preview content uses the full preview pane width regardless of the Preview adaptive width preference

#### Scenario: Edit mode remains full pane width
- **WHEN** the active view mode is Edit
- **THEN** the source editing surface uses the full pane width regardless of the Preview adaptive width preference
