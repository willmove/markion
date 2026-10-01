## ADDED Requirements

### Requirement: About Markion dialog exposes official project links

The About Markion dialog SHALL retain its localized title, running version, product description, and explicit confirmation control while presenting two official project links in this order: the project website `https://markion.app`, followed by the GitHub repository `https://github.com/willmove/markion`. Each URL SHALL be visibly identifiable as an interactive link and pointer activation SHALL open that exact HTTPS destination in the system default browser through the platform shell. Link activation SHALL NOT render embedded web content, stop the application, or implicitly dismiss the About dialog. User-facing labels SHALL follow the active interface language, the literal URLs SHALL remain unchanged, and the dialog SHALL derive its surface, text, border, link, hover, and control colors from the active theme palette.

#### Scenario: About dialog presents the website above GitHub

- **WHEN** the user opens About Markion from either Help-menu surface
- **THEN** the dialog shows the running version and product description
- **AND** a project-website link targeting `https://markion.app` appears above a GitHub link targeting `https://github.com/willmove/markion`
- **AND** both URLs are visually identifiable as interactive links

#### Scenario: Project website opens in the system browser

- **WHEN** the user activates the `https://markion.app` link in the About dialog
- **THEN** the system default browser opens exactly `https://markion.app`
- **AND** Markion renders no embedded web content and continues running
- **AND** the About dialog remains open until the user explicitly dismisses it

#### Scenario: GitHub repository opens in the system browser

- **WHEN** the user activates the `https://github.com/willmove/markion` link in the About dialog
- **THEN** the system default browser opens exactly `https://github.com/willmove/markion`
- **AND** Markion renders no embedded web content and continues running
- **AND** the About dialog remains open until the user explicitly dismisses it

#### Scenario: About dialog labels follow the active language

- **WHEN** the About dialog is opened after the interface language changes
- **THEN** its title, version label, product description, project-website label, GitHub label, and confirmation control render in the active language
- **AND** both HTTPS URLs are displayed verbatim rather than translated

#### Scenario: About dialog follows the active theme

- **WHEN** the About dialog is opened under a light or dark theme
- **THEN** its surface, text, border, link, hover, and confirmation-control colors remain readable and visually consistent with that active theme

#### Scenario: Confirmation dismisses the About dialog

- **WHEN** the user activates the dialog's localized confirmation control
- **THEN** the About dialog closes without changing the document, preferences, or application lifecycle
