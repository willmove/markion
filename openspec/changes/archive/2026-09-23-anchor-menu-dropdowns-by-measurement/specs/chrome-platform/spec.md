## ADDED Requirements

### Requirement: In-window dropdowns align with their menu buttons in every language
The in-window menu-bar dropdown panels SHALL open horizontally aligned with their title buttons for every interface language and under the machine's actual system UI font. The horizontal anchor SHALL be derived at runtime from measurements of the rendered menu-bar label widths in the active system UI font, combined with the menu bar's fixed layout paddings, rather than from fixed per-language offset constants. When the interface language changes, the dropdown anchors SHALL follow the new labels' measurements on the next menu open, and re-measurement SHALL be cached so steady-state rendering performs no additional text layout work.

#### Scenario: Every language aligns its dropdowns
- **WHEN** the user opens any top-level menu dropdown in any of the supported interface languages
- **THEN** the dropdown panel's left edge aligns with its title button's left edge

#### Scenario: Alignment holds on machines with different system UI fonts
- **WHEN** the application runs on a machine whose system UI font or font rendering differs from the development machine
- **THEN** the dropdown panels still align with their title buttons, because the anchor is measured with that machine's resolved UI font

#### Scenario: Language switch realigns immediately
- **WHEN** the interface language changes while the application is running
- **AND** the user opens a menu dropdown afterwards
- **THEN** the dropdown aligns with the newly rendered title button for the new language
