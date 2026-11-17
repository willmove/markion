## ADDED Requirements

### Requirement: Preferences tab strip SHALL stay on one row
The Preferences panel tab strip SHALL lay out every preferences tab on a single non-wrapping row, in the existing tab order. Tabs SHALL size to their labels instead of using a minimum width that forces a second row. When the panel is narrower than the labels, the strip SHALL scroll horizontally and every tab SHALL remain reachable. The strip SHALL NOT wrap a tab onto a second row. Opening File → Preferences SHALL still land on General.

#### Scenario: Six tabs stay on one row
- **WHEN** the Preferences panel is open at its normal width in Simplified Chinese
- **THEN** General, Appearance, Images, Shortcuts, Export, and AI Agent appear on one row
- **AND** no tab is placed on a second row

#### Scenario: Narrow panel scrolls the strip
- **WHEN** the preferences panel is narrower than the combined tab labels
- **THEN** the tab strip scrolls horizontally
- **AND** the user can still activate every tab, including AI Agent

#### Scenario: Opening Preferences lands on General
- **WHEN** the user opens the Preferences panel from File → Preferences
- **THEN** the General tab is active
