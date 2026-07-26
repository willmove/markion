# chrome-platform

## MODIFIED Requirements

### Requirement: Narrow-scope preferences with persistence and reset
The editor SHALL provide a Preferences panel and a persisted preferences file covering: theme (and custom theme selection), focus mode, typewriter mode, code-line-numbers, sidebar visibility, sidebar tab, Heading menu depth (H1–H5 default, optional H1–H6), source-editor font size, rendered-document font size, rendered paragraph spacing, and menu-action shortcut overrides. The preferences file SHALL be TOML (`config.toml` in the Markion config directory) with every field optional and defaulted, and SHALL additionally carry an `[auto_save]` section (`enabled`, `delay_secs`) that is configurable only via the file, not the panel. On startup, if `config.toml` does not exist but a legacy `preferences.conf` (the retired `key=value` format) does, the editor SHALL migrate it to `config.toml` once and thereafter ignore the legacy file. The editor SHALL also offer a preference reset action and a preferences summary in the Help menu. Font family, code-highlight theme, extension-syntax toggles, and image-uploader credentials are **not** configurable.

#### Scenario: Supported preferences persist and restore
- **WHEN** the user changes a supported preference (theme, focus mode, typewriter mode, code line numbers, sidebar visibility, sidebar tab, Heading menu depth, source-editor font size, rendered-document font size, rendered paragraph spacing, or a menu-action shortcut override)
- **THEN** the change is written to `config.toml` and restored on the next launch

#### Scenario: Legacy preferences file is migrated once
- **WHEN** the editor starts with no `config.toml` but a legacy `preferences.conf` present
- **THEN** the legacy values are loaded, written out as `config.toml`, and used; subsequent launches read only `config.toml`

#### Scenario: Partial or missing config falls back to defaults
- **WHEN** `config.toml` is missing, or present but omits fields
- **THEN** missing values take their documented defaults and the editor starts normally

#### Scenario: Preferences summary and reset
- **WHEN** the user opens the Help → preferences summary or triggers the reset action
- **THEN** a summary including supported typography values is shown, or all preferences including typography and shortcut overrides are reset to their defaults

## ADDED Requirements

### Requirement: Shortcut reference lives in the Preferences panel
The keyboard-shortcut reference SHALL be presented as a Shortcuts tab inside the Preferences panel, keeping the platform tabs and category sidebar layout. The standalone shortcut modal SHALL be removed, and the Help menu SHALL no longer contain a Keyboard Shortcuts item. The `ShowShortcuts` action (default F1) SHALL open the Preferences panel directly on the Shortcuts tab.

#### Scenario: Preferences panel exposes a Shortcuts tab
- **WHEN** the user opens the Preferences panel
- **THEN** a General tab and a Shortcuts tab are available, and the Shortcuts tab shows the categorized shortcut reference with platform tabs

#### Scenario: Help menu no longer lists Keyboard Shortcuts
- **WHEN** the user opens the Help menu
- **THEN** no Keyboard Shortcuts item is present and the About item remains

#### Scenario: F1 opens the Shortcuts tab
- **WHEN** the user invokes the ShowShortcuts action
- **THEN** the Preferences panel opens with the Shortcuts tab active

### Requirement: Menu shortcut labels reflect effective bindings
In-window menu items that display a shortcut hint SHALL render the action's effective binding — the curated default label when unmodified, or a formatted label derived from the user's override. Labels SHALL update in the same session when an override is set, reset, or cleared.

#### Scenario: Menu shows an overridden binding
- **WHEN** an action has an override and the user opens its menu
- **THEN** the item's shortcut hint shows the override binding formatted for the current platform

#### Scenario: Menu label follows a reset
- **WHEN** an override is removed via per-action reset or preferences reset
- **THEN** the menu item's shortcut hint returns to the curated default label
