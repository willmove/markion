## MODIFIED Requirements

### Requirement: Narrow-scope preferences with persistence and reset
The editor SHALL provide a Preferences panel and a persisted preferences file covering: theme (and custom theme selection), focus mode, typewriter mode, code-line-numbers, sidebar visibility, sidebar tab, Heading menu depth (H1–H5 default, optional H1–H6), source-editor font size, rendered-document font size, and rendered paragraph spacing. The preferences file SHALL be TOML (`config.toml` in the Markion config directory) with every field optional and defaulted, and SHALL additionally carry an `[auto_save]` section with `enabled` (default true), `silent_save` (default true), and `delay_secs` (default 5). The Preferences panel SHALL expose `silent_save` and `delay_secs`; `enabled` SHALL remain configurable only via the config file, not the panel. On startup, if `config.toml` does not exist but a legacy `preferences.conf` (the retired `key=value` format) does, the editor SHALL migrate it to `config.toml` once and thereafter ignore the legacy file. The editor SHALL also offer a preference reset action and a preferences summary in the Help menu. Font family, code-highlight theme, extension-syntax toggles, and image-uploader credentials are **not** configurable through this requirement's historical scope (font-family controls elsewhere do not expand this list).

#### Scenario: Supported preferences persist and restore
- **WHEN** the user changes a supported preference (theme, focus mode, typewriter mode, code line numbers, sidebar visibility, sidebar tab, Heading menu depth, source-editor font size, rendered-document font size, rendered paragraph spacing, silent save, or auto-save delay)
- **THEN** the change is written to `config.toml` and restored on the next launch

#### Scenario: Legacy preferences file is migrated once
- **WHEN** the editor starts with no `config.toml` but a legacy `preferences.conf` present
- **THEN** the legacy values are loaded, written out as `config.toml`, and used; subsequent launches read only `config.toml`
- **AND** missing `[auto_save] silent_save` defaults to true

#### Scenario: Partial or missing config falls back to defaults
- **WHEN** `config.toml` is missing, or present but omits fields (including `silent_save`)
- **THEN** missing values take their documented defaults and the editor starts normally

#### Scenario: Preferences summary and reset
- **WHEN** the user opens the Help → preferences summary or triggers the reset action
- **THEN** a summary including supported typography values and auto-save panel settings is shown, or all preferences including typography and `[auto_save]` defaults (`enabled` true, `silent_save` true, `delay_secs` 5) are reset
