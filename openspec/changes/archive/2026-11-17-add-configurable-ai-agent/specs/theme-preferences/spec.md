## MODIFIED Requirements

### Requirement: Preferences panel SHALL expose an Appearance tab
The Preferences panel SHALL provide an Appearance tab at the same tab-strip level as General, Shortcuts, Export, and AI. General, Appearance, Shortcuts, and Export SHALL retain that relative order; additional capability-owned tabs such as Images SHALL retain their existing placement, and AI SHALL be appended after Export. Opening the Preferences panel from File → Preferences SHALL land on General. The Appearance tab SHALL host the theme swatch grid and the document typography controls (Source font size, Reading font size, Paragraph spacing, and the source/rendered/code font-family slots). The Appearance tab SHALL NOT host language, display/workspace toggles, auto-save, shortcut, export, or AI controls.

#### Scenario: Appearance tab sits beside the other Preferences tabs
- **WHEN** the user opens the Preferences panel
- **THEN** General, Appearance, Shortcuts, and Export appear in their existing relative order, other existing tabs retain their placement, and AI is appended after Export

#### Scenario: Opening Preferences lands on General
- **WHEN** the user opens the Preferences panel from File → Preferences
- **THEN** the General tab is active

#### Scenario: Appearance tab contains theme and typography
- **WHEN** the Appearance tab is active
- **THEN** the tab body shows the theme swatch grid and the typography controls
- **AND** language, display/workspace, auto-save, shortcut, export, and AI controls are not rendered in that body

#### Scenario: General tab does not host appearance controls
- **WHEN** the Preferences panel General tab is open
- **THEN** the theme swatch grid is not rendered in that tab
- **AND** Source font size, Reading font size, Paragraph spacing, and the three font-family slots are not rendered in that tab

## ADDED Requirements

### Requirement: Preferences SHALL expose accessible AI setup
The Preferences panel SHALL provide an AI tab even while AI is disabled. It SHALL expose basic enablement/provider/key/model/test controls and collapsed advanced settings using editable controls, visible validation/status, active-theme colors, and the current language. Content and the tab strip SHALL remain reachable when viewport width/height or font scaling prevents fitting all controls. Explicit AI setup entry points SHALL land on AI without changing the default File → Preferences landing tab. Invalid draft values SHALL not replace a working stored profile until committed validly.

#### Scenario: Disabled feature can be configured
- **WHEN** AI is disabled and the user opens Preferences → AI
- **THEN** setup controls remain usable and no AI request is sent merely by opening the tab

#### Scenario: Invalid endpoint draft
- **WHEN** the user types an invalid advanced endpoint
- **THEN** inline feedback appears and the last valid profile remains intact
