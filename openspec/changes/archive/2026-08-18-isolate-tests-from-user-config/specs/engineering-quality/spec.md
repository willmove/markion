## ADDED Requirements

### Requirement: Workspace tests MUST NOT touch the developer preferences file
The workspace test suite MUST NOT read preference values from, or write preference values to, the developer machine's real preferences file (`config.toml` in the Markion config directory used by the desktop app). Tests that exercise preference persistence MUST use an isolated file. Tests that mutate in-memory preferences without an isolated file MUST leave the developer preferences file unchanged. Session-file isolation already follows this contract; preferences SHALL follow the same isolation.

#### Scenario: Preference-mutating test leaves developer config unchanged
- **WHEN** a GPUI test changes source-editor font size or any other persisted preference without redirecting preferences to an isolated file
- **THEN** the developer machine's `config.toml` is not created, overwritten, or otherwise modified

#### Scenario: Isolated preference persistence still round-trips
- **WHEN** a test points preferences at an isolated file and saves a non-default source font size
- **THEN** that isolated file records the value
- **AND** the developer machine's `config.toml` remains unchanged

#### Scenario: Tests start from documented defaults
- **WHEN** a test constructs the application without supplying an isolated preferences file
- **THEN** source font size, reading font size, and other preference fields take their documented defaults rather than whatever is stored on the developer machine
