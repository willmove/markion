## ADDED Requirements

### Requirement: Preferences panel SHALL expose silent save and auto-save delay
The Preferences panel General tab SHALL include an Auto-save section with (1) a boolean control for silent save-to-file mapped to `[auto_save] silent_save` (default on), and (2) a numeric control for the inactivity interval mapped to `[auto_save] delay_secs` (default 5 seconds, minimum 1). Activating either control SHALL apply immediately, persist through the existing preferences save path, and use localized labels. The panel SHALL NOT expose `[auto_save] enabled`.

#### Scenario: Silent-save toggle appears and persists
- **WHEN** the Preferences panel General tab is open
- **THEN** a silent save-to-file control is visible and reflects the current `silent_save` value
- **AND** toggling it updates subsequent autosave destination behavior and writes `silent_save` to `config.toml`

#### Scenario: Delay control appears and persists
- **WHEN** the user adjusts the auto-save delay control within the supported range
- **THEN** the new `delay_secs` value is applied to subsequent inactivity timers and persisted
- **AND** values below 1 are not stored (control disables decrement at the minimum or clamps on commit)

#### Scenario: Master enabled switch stays out of the panel
- **WHEN** the Preferences panel General tab is open
- **THEN** no control is offered for `[auto_save] enabled`
