## ADDED Requirements

### Requirement: Visual Edit code folding preference
The Appearance tab's Code blocks section SHALL offer a localized switch that enables manual folding of ordinary code blocks in Visual Edit. The switch SHALL default to off for first launches and configurations without a saved value. Changes SHALL apply immediately and persist across launches. Resetting preferences SHALL restore the off state.

#### Scenario: Enable folding from preferences
- **WHEN** the user turns on the switch in Appearance → Code blocks
- **THEN** the current Visual Edit code blocks show their manual fold controls and stay expanded until folded by the user
- **AND** the switch remains on after restarting the application

#### Scenario: Reset the folding preference
- **WHEN** the user resets preferences with the switch on
- **THEN** the switch returns to off and Visual Edit shows expanded code blocks with their fold controls hidden
