## ADDED Requirements

### Requirement: About Markion dialog invites a GitHub star

The About Markion dialog SHALL show a short localized invitation after the product description asking users who find Markion useful to star the project, followed immediately by a clickable GitHub link targeting `https://github.com/willmove/markion`. The invitation SHALL NOT replace the existing title, version, product description, project-website link, GitHub repository link, or confirmation control. The star link SHALL be visually identifiable as an interactive link; pointer activation SHALL open that exact HTTPS destination in the system default browser through the platform shell. Link activation SHALL NOT render embedded web content, stop the application, or dismiss the About dialog. User-facing invitation and star-link labels SHALL follow the active interface language; the literal repository URL SHALL remain unchanged and SHALL be displayed verbatim.

#### Scenario: About dialog shows the star invitation above the official project links

- **WHEN** the user opens About Markion from either Help-menu surface
- **THEN** the dialog shows the product description, then a localized star invitation, then a GitHub star link targeting `https://github.com/willmove/markion`
- **AND** the existing project-website and GitHub repository links remain below that star link
- **AND** the star URL is visually identifiable as an interactive link

#### Scenario: Star link opens the repository in the system browser

- **WHEN** the user activates the GitHub star link in the About dialog
- **THEN** the system default browser opens exactly `https://github.com/willmove/markion`
- **AND** Markion renders no embedded web content and continues running
- **AND** the About dialog remains open until the user explicitly dismisses it

#### Scenario: Star invitation follows the active language

- **WHEN** the About dialog is opened after the interface language changes
- **THEN** the star invitation and star-link label render in the active language
- **AND** the star-link HTTPS URL is displayed verbatim rather than translated
