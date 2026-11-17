## ADDED Requirements

### Requirement: Preferences SHALL control automatic update downloads

General preferences SHALL expose a localized Updates section with an actionable Automatically download updates boolean, defaulting to enabled and using active-theme colors. It SHALL explain the five-hour check cadence and explicit restart confirmation, and SHALL describe browser-download fallback when the current installation cannot self-update. The value SHALL apply to subsequent downloads immediately and persist as `automatically_download_updates` in `config.toml`. Missing or non-boolean values SHALL default to true without losing unrelated valid preferences. Reset Preferences SHALL restore true. Disabling the preference SHALL not cancel an already-started download or discard a ready update, and SHALL not disable version checks or manual downloads. Changing it SHALL preserve documents and derived caches.

#### Scenario: Default and backward compatibility
- **WHEN** preferences are new or the automatic-download key is absent or invalid
- **THEN** automatic downloads default to enabled and other valid preferences load

#### Scenario: Disable and restart
- **WHEN** the user disables automatic update downloads and restarts Markion
- **THEN** the control remains disabled and later release discovery waits for download consent

#### Scenario: Reset and immediate application
- **WHEN** the user resets preferences after disabling automatic update downloads
- **THEN** automatic downloads become enabled and are persisted
- **AND** no document text, version, selection, dirty state, history or derived cache is changed
