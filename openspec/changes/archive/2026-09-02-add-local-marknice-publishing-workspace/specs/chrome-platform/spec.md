## ADDED Requirements

### Requirement: Local WeChat publishing workspace action
The application SHALL expose a localized action in its Export menu for opening the active document in the local WeChat publishing workspace. The action SHALL use the platform default browser, SHALL report successful launch and launch failures through localized in-app status feedback, and SHALL remain available for untitled and empty documents. Invoking it SHALL take a snapshot only and SHALL NOT save or mutate the document, change its active tab or view mode, or disturb document selection and versioned derived-state cache identity.

#### Scenario: Export menu launches the local workspace
- **WHEN** the user activates the WeChat publishing workspace item in the Export menu
- **THEN** Markion creates a publishing snapshot and asks the operating system to open its local session URL in the default browser
- **AND** shows localized in-app feedback that the publishing workspace was opened

#### Scenario: Launch preserves editor state
- **WHEN** the workspace action is invoked for an active document
- **THEN** the active tab, view mode, selection, text, dirty state, document version, and already-derived cache identities remain unchanged

#### Scenario: Browser launch failure is visible
- **WHEN** a publishing session is created but the operating system cannot open its URL
- **THEN** Markion revokes that unused session
- **AND** shows a localized status explaining that the browser could not be opened

#### Scenario: Session setup failure is visible
- **WHEN** the local workspace assets are missing or the loopback service cannot start securely
- **THEN** Markion does not open a partial or unauthenticated workspace
- **AND** shows a localized actionable error while the editor remains usable

