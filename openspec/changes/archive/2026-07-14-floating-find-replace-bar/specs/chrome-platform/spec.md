## MODIFIED Requirements

### Requirement: Find and replace
The editor SHALL provide a find/replace workflow supporting case-sensitive and regular-expression search, next/previous match navigation, current-match and total counts, replace current, and replace all. The Find / Replace controls SHALL render as a compact floating overlay near the upper-right of the editor workspace, above the editor/preview panes, without consuming layout height or shifting the main workspace. The overlay SHALL provide an explicit close control that hides the overlay, clears active match highlighting and search focus, and preserves the current query and replacement text for a later reopen. The overlay, fields, buttons, borders, hover states, and summary text SHALL use the active theme palette rather than hard-coded light colors.

#### Scenario: Search with options
- **WHEN** the user enters a query and toggles case-sensitive or regex
- **THEN** matches are highlighted and the current/total counts are shown

#### Scenario: Navigate, replace, and replace all
- **WHEN** the user steps to next/previous, replaces the current match, or replaces all
- **THEN** the editor navigates/replaces accordingly and updates the match state

#### Scenario: Find overlay does not shift workspace layout
- **WHEN** the user opens Find or Replace
- **THEN** the controls appear as a compact upper-right floating overlay above the editor/preview workspace
- **AND** the tab bar, editor pane, preview pane, and status bar keep their existing layout positions

#### Scenario: Closing the overlay clears active highlights
- **WHEN** the Find / Replace overlay is visible and the user activates its close control
- **THEN** the overlay is hidden
- **AND** active search focus is cleared
- **AND** active match highlighting is cleared
- **AND** the current find query and replacement text are preserved for the next time Find or Replace opens

#### Scenario: Find overlay follows active theme
- **WHEN** the active theme changes
- **THEN** the Find / Replace overlay surface, input fields, buttons, borders, hover states, and summary text render using the active theme palette
- **AND** the overlay does not use hard-coded light-only chrome colors

#### Scenario: Existing Find and Replace behavior is preserved
- **WHEN** the user invokes existing Find / Replace shortcuts or actions
- **THEN** query editing, regex and case-sensitive toggles, next/previous navigation, match counts, replace current, and replace all continue to behave as before
