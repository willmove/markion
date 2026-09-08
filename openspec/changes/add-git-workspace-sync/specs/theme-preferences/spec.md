## ADDED Requirements

### Requirement: Git preferences SHALL be additive and background writes disabled
Preferences SHALL expose Git executable detection/override and an opt-in background remote-check setting with safe defaults. Missing settings SHALL preserve existing editing behavior and keep background checking off. Background write synchronization SHALL not be offered in this change. Repository-specific connection, scope and message settings SHALL be accessible from Sync setup/details and persisted atomically in a versioned local policy file separate from session data and Git-owned remote/upstream configuration.

#### Scenario: Existing installation upgrades
- **WHEN** the app loads configuration without Git keys or a repository policy file
- **THEN** it retains existing preferences and does not connect or synchronize any repository automatically

#### Scenario: Git override is invalid
- **WHEN** the user selects an unusable executable path
- **THEN** the preferences report the failed detection without making ordinary document operations unavailable

#### Scenario: Global Git settings are grouped
- **WHEN** the user opens the General preferences category
- **THEN** it shows one Git Sync section at the end containing both executable selection and background remote checking, with informational lines using the same text size as other General settings

### Requirement: Repository policies SHALL outlive recent sessions and fail closed
Recent-workspace eviction and general Preferences reset SHALL NOT delete repository policies, Git history, credentials or unresolved recovery. Disconnect SHALL remove the sync binding/scheduling without deleting files, `.git` or remotes. Invalid/unsupported policy data SHALL disable writes with actionable feedback rather than broaden allowed scope. Replaced/moved repository identities SHALL require reconnection before using prior authorization.

#### Scenario: Workspace falls out of recent list
- **WHEN** a connected workspace is evicted from the bounded recent-workspace list
- **THEN** its separate repository policy remains available when that same repository is opened again

#### Scenario: Preferences are reset
- **WHEN** the user resets general preferences
- **THEN** global Git settings return to defaults and existing repository policies and recovery remain intact

#### Scenario: Policy cannot be parsed
- **WHEN** the repository policy is corrupted or has an unsupported version
- **THEN** one-click writes are disabled with repair feedback and no permissive replacement policy is silently generated
