## MODIFIED Requirements

### Requirement: Narrow-scope preferences with persistence and reset
The editor SHALL provide a Preferences panel and a persisted preferences file covering: theme (and custom theme selection), focus mode, typewriter mode, code-line-numbers, sidebar visibility, sidebar tab, Heading menu depth (H1–H5 default, optional H1–H6), source-editor font size, rendered-document font size, rendered paragraph spacing, and Markdown auto-pair. The preferences file SHALL be TOML (`config.toml` in the Markion config directory) with every field optional and defaulted, and SHALL additionally carry an `[auto_save]` section (`enabled`, `delay_secs`) that is configurable only via the file, not the panel. On startup, if `config.toml` does not exist but a legacy `preferences.conf` (the retired `key=value` format) does, the editor SHALL migrate it to `config.toml` once and thereafter ignore the legacy file. The editor SHALL also offer a preference reset action and a preferences summary in the Help menu. Font family, code-highlight theme, extension-syntax toggles, and image-uploader credentials are **not** configurable.

#### Scenario: Supported preferences persist and restore
- **WHEN** the user changes a supported preference (theme, focus mode, typewriter mode, code line numbers, sidebar visibility, sidebar tab, Heading menu depth, source-editor font size, rendered-document font size, rendered paragraph spacing, or Markdown auto-pair)
- **THEN** the change is written to `config.toml` and restored on the next launch

#### Scenario: Legacy preferences file is migrated once
- **WHEN** the editor starts with no `config.toml` but a legacy `preferences.conf` present
- **THEN** the legacy values are loaded, written out as `config.toml`, and used; subsequent launches read only `config.toml`

#### Scenario: Partial or missing config falls back to defaults
- **WHEN** `config.toml` is missing, or present but omits fields
- **THEN** missing values take their documented defaults and the editor starts normally
- **AND** a missing `markdown_auto_pair` key defaults to enabled

#### Scenario: Preferences summary and reset
- **WHEN** the user opens the Help → preferences summary or triggers the reset action
- **THEN** a summary including supported typography values and Markdown auto-pair is shown, or all preferences including typography and Markdown auto-pair (default on) are reset to their defaults

## ADDED Requirements

### Requirement: Markdown auto-pair is a General preference
The Preferences panel General tab SHALL expose a Markdown auto-pair toggle. The choice SHALL persist as a boolean `markdown_auto_pair` in `config.toml` (default `true`; missing or non-boolean values degrade to `true`). Toggling SHALL apply to the next typed delimiter without restarting, SHALL NOT mutate document text or increment document version, and SHALL be restored by the preference reset action.

#### Scenario: General tab shows the toggle
- **WHEN** the Preferences panel is open on the General tab
- **THEN** a Markdown auto-pair control is present and reflects the persisted value

#### Scenario: Turning auto-pair off is immediate
- **WHEN** the user disables Markdown auto-pair
- **THEN** the next typed `*` in an editing surface inserts only `*`
- **AND** document version and derived Markdown caches are unchanged by the preference change
