## MODIFIED Requirements

### Requirement: Preferences SHALL control the automatic block-grip popup

General preferences SHALL expose a localized **Show block menu on grip click** toggle using the active theme. The toggle SHALL default to enabled, apply immediately and persist across application restarts. The enabled preference SHALL allow automatic grip-click opening and show a block-format indicator beside each visible grip's drag dots. Disabling it SHALL restore the dots-only grip and close an open grip-invoked popup while retaining its selected block range. The appearance change SHALL preserve content layout. Block selection, Shift extension, dragging, explicit right-click and keyboard context menus SHALL remain available. Changing this preference SHALL NOT change document source, version, dirty state, undo history or shared derived-cache identity.

#### Scenario: Enabled preference opens the popup

- **WHEN** the preference is enabled and the user completes an ordinary click on an eligible Visual Edit block grip
- **THEN** the clicked block is selected and its compact quick-settings popup opens

#### Scenario: Disabled preference retains selection-only clicks

- **WHEN** the preference is disabled and the user clicks a block grip
- **THEN** the grip shows only the drag dots and selects the block without opening a popup
- **AND** Shift-click extension, dragging, right-click and keyboard context menus retain their documented behavior

#### Scenario: Toggling off dismisses only the automatic popup

- **WHEN** the user disables the preference while a grip popup is open
- **THEN** that popup closes immediately and its established block selection remains
- **AND** document source, version, dirty state, history and shared derived caches remain unchanged
