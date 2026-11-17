## ADDED Requirements

### Requirement: First-run setup SHALL complete without circular dependencies
Model discovery SHALL be usable before a model is selected when the address and required credentials are valid. After testing a complete draft, an explicit Save and Enable action SHALL persist that draft and its chosen credential policy before enabling AI. Failed persistence SHALL preserve the draft without claiming completion. Tool verification SHALL remain associated with the tested profile, endpoint, protocol and model identity and SHALL NOT authorize another identity.

#### Scenario: Discover an unknown model
- **WHEN** a user enters a valid service address and required key but leaves the model blank and requests model discovery
- **THEN** discovery runs and the returned models can populate the draft

#### Scenario: Save and enable a tested draft
- **WHEN** a disabled user successfully tests and explicitly saves and enables a draft
- **THEN** the persisted draft is the active request configuration and its matching verified capabilities remain available
- **AND** no document content is sent by enablement

### Requirement: AI settings SHALL organize common tasks and expose accurate state
Settings SHALL separate service configuration, writing preferences, local history/privacy and advanced limits. Save controls SHALL remain reachable outside the settings body scroll. Key state SHALL distinguish absent keys, registered secure credential references and session-only credentials without revealing secrets or treating a reference as proof of current OS-store access. Limits SHALL have visible units. Model lists SHALL be bounded and verification feedback SHALL identify when a changed profile, endpoint, protocol or model requires retesting. Destructive history clearing SHALL require explicit confirmation.

#### Scenario: Review settings on a small window
- **WHEN** AI settings are opened and the body is scrolled
- **THEN** saving remains accessible and unrelated history and technical limits are collapsed into named sections

#### Scenario: Change a tested model
- **WHEN** the user edits the model or endpoint after a successful test
- **THEN** the changed configuration is not presented as verified
