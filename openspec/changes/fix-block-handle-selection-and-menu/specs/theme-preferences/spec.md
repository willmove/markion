## ADDED Requirements

### Requirement: Preferences SHALL control the automatic block-grip popup

General preferences SHALL expose a localized **Show block menu on grip click** toggle using the active theme. The toggle SHALL default to enabled, apply immediately and persist across application restarts. Disabling it SHALL close an open grip-invoked popup while retaining its selected block range. The toggle SHALL control only automatic grip-click opening; block selection, Shift extension, dragging, explicit right-click and keyboard context menus SHALL remain available. Changing this preference SHALL NOT change document source, version, dirty state, undo history or shared derived-cache identity.

#### Scenario: Enabled preference opens the popup
- **WHEN** the preference is enabled and the user completes an ordinary click on an eligible Visual Edit block grip
- **THEN** the clicked block is selected and its compact quick-settings popup opens

#### Scenario: Disabled preference retains selection-only clicks
- **WHEN** the preference is disabled and the user clicks a block grip
- **THEN** the block is selected without opening a popup
- **AND** Shift-click extension, dragging, right-click and keyboard context menus retain their documented behavior

#### Scenario: Toggling off dismisses only the automatic popup
- **WHEN** the user disables the preference while a grip popup is open
- **THEN** that popup closes immediately and its established block selection remains
- **AND** document source, version, dirty state, history and shared derived caches remain unchanged

### Requirement: Block-grip popup preference SHALL persist safely

The editor SHALL persist the automatic block-grip popup preference as the boolean `block_handle_menu` in `config.toml`. Missing or non-boolean values SHALL fall back to enabled without preventing startup or losing unrelated valid preferences. Reset Preferences SHALL restore enabled.

#### Scenario: Old configuration enables the popup
- **WHEN** an existing configuration omits `block_handle_menu`
- **THEN** automatic grip-click popups are enabled

#### Scenario: Invalid configuration falls back safely
- **WHEN** `block_handle_menu` contains a string, number or other non-boolean value
- **THEN** automatic grip-click popups default to enabled and unrelated valid preferences are honored

#### Scenario: Disabled value round-trips
- **WHEN** the user disables automatic grip-click popups and restarts the editor
- **THEN** the preference and its General-tab control remain disabled

#### Scenario: Reset restores enabled
- **WHEN** the user resets preferences after disabling automatic grip-click popups
- **THEN** automatic grip-click popups are enabled again
