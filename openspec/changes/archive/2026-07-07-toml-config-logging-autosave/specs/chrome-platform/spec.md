## MODIFIED Requirements

### Requirement: Narrow-scope preferences with persistence and reset
The editor SHALL provide a Preferences panel and a persisted preferences file covering: theme (and custom theme selection), focus mode, typewriter mode, code-line-numbers, sidebar visibility, and sidebar tab. The preferences file SHALL be TOML (`config.toml` in the Markion config directory) with every field optional and defaulted, and SHALL additionally carry an `[auto_save]` section (`enabled`, `delay_secs`) that is configurable only via the file, not the panel. On startup, if `config.toml` does not exist but a legacy `preferences.conf` (the retired `key=value` format) does, the editor SHALL migrate it to `config.toml` once and thereafter ignore the legacy file. The editor SHALL also offer a preference reset action and a preferences summary in the Help menu. Font family/size, code-highlight theme, extension-syntax toggles, and image-uploader credentials are **not** configurable.

#### Scenario: Supported preferences persist and restore
- **WHEN** the user changes a supported preference (theme, focus mode, typewriter mode, code line numbers, sidebar visibility, sidebar tab)
- **THEN** the change is written to `config.toml` and restored on the next launch

#### Scenario: Legacy preferences file is migrated once
- **WHEN** the editor starts with no `config.toml` but a legacy `preferences.conf` present
- **THEN** the legacy values are loaded, written out as `config.toml`, and used; subsequent launches read only `config.toml`

#### Scenario: Partial or missing config falls back to defaults
- **WHEN** `config.toml` is missing, or present but omits fields
- **THEN** missing values take their documented defaults and the editor starts normally

#### Scenario: Preferences summary and reset
- **WHEN** the user opens the Help → preferences summary or triggers the reset action
- **THEN** a summary is shown, or all preferences are reset to their defaults

## ADDED Requirements

### Requirement: Diagnostic file logging
The editor SHALL write diagnostic logs to a platform-appropriate Markion log directory (Linux `~/.cache/markion/logs`, macOS `~/Library/Logs/Markion`, Windows `%LOCALAPPDATA%\Markion\Logs`) using daily rotation and keeping at most the last 7 files. The default level SHALL be `info`, overridable via the `RUST_LOG` environment variable. Logging SHALL be initialized at startup and record at minimum: startup (with version), preference load/migration events, auto-save failures, and export-engine fallbacks. Logging failures SHALL never prevent the editor from starting.

#### Scenario: Logs rotate daily and are bounded
- **WHEN** the editor runs across multiple days
- **THEN** each day gets its own log file and no more than 7 files are retained

#### Scenario: Log level override
- **WHEN** the editor is launched with `RUST_LOG=debug`
- **THEN** debug-level events are recorded for that run

#### Scenario: Logging failure is non-fatal
- **WHEN** the log directory cannot be created or opened
- **THEN** the editor starts normally without file logging
