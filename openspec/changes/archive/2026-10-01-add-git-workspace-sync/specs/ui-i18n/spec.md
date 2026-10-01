## ADDED Requirements

### Requirement: Backup, sync, and recovery chrome SHALL be fully localized by audience
All onboarding, executable checks, scope approval, identity/credential prompts, synchronization states, progress/errors, diff/history controls, conflict actions, recovery, settings and unsupported-capability messages SHALL use the existing localization catalog for every supported UI language. Ordinary surfaces SHALL use backup, synchronization, note version, this computer, other-device update, sync location, and confirmation-time language; repository, branch, remote, commit, staging, fetch, pull, push, merge, OID, and hunk terminology SHALL be reserved for Advanced Git details except where an unsupported condition cannot be explained accurately without it. Path, branch, commit author and authored commit text SHALL remain verbatim. Generated default commit messages SHALL use the configured message template independently of interface language.

Wording SHALL distinguish saved on this computer, committed locally, waiting to upload, incoming updates, confirmed delivery and uncertain delivery. It SHALL NOT describe local-only state as remotely backed up, and background-check copy SHALL say that it neither uploads nor applies changes automatically.

#### Scenario: Interface language changes
- **WHEN** the user changes language while Backup and Sync details or conflict resolution is open
- **THEN** interface labels follow the selected language without translating paths, branch names or document content

#### Scenario: Ordinary sync center opens
- **WHEN** the user opens Backup and Sync without expanding Advanced Git details
- **THEN** localized labels explain safety and next action without requiring Git transport vocabulary

#### Scenario: Offline after local commit
- **WHEN** a local commit succeeds and networking fails
- **THEN** localized feedback states that work is saved on this computer and upload is pending instead of claiming a remote backup or implying the entire operation was lost

#### Scenario: Background check setting is translated
- **WHEN** the background update-check option is displayed in any supported language
- **THEN** its localized description explicitly states that it does not automatically upload or apply changes

#### Scenario: Translation is missing
- **WHEN** a Git UI message is introduced without a required language entry
- **THEN** the existing localization completeness mechanism detects the missing translation
