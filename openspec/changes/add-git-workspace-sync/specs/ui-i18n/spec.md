## ADDED Requirements

### Requirement: Git sync and recovery chrome SHALL be fully localized
All Git onboarding, executable checks, scope approval, identity/credential prompts, synchronization states, progress/errors, diff/history controls, conflict actions, recovery, settings and unsupported-capability messages SHALL use the existing localization catalog for every supported UI language. Path, branch, commit author and authored commit text SHALL remain verbatim. Generated default commit messages SHALL use the configured message template independently of interface language. Wording SHALL distinguish saved locally, committed locally, waiting to upload, incoming updates, confirmed delivery and uncertain delivery.

#### Scenario: Interface language changes
- **WHEN** the user changes language while Sync details or conflict resolution is open
- **THEN** interface labels follow the selected language without translating paths, branch names or document content

#### Scenario: Offline after local commit
- **WHEN** a local commit succeeds and networking fails
- **THEN** localized feedback states that local history was saved and upload is pending instead of implying the entire operation was lost

#### Scenario: Translation is missing
- **WHEN** a Git UI message is introduced without a required language entry
- **THEN** the existing localization completeness mechanism detects the missing translation
