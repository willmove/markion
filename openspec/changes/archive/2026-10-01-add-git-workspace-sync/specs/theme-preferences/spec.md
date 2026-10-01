## ADDED Requirements

### Requirement: Backup and Sync preferences SHALL separate ordinary choices from advanced Git settings
Preferences SHALL expose an ordinary Backup and Sync section for connection state, human-readable sync location, opt-in checks for updates from other devices, disconnect, and recovery access. The background-check label and description SHALL state that checking does not automatically upload or apply changes. Missing settings SHALL preserve existing editing behavior and keep background checking off; background write synchronization SHALL not be offered in this change.

Git executable detection/override, branch/endpoints, approved roots, message template, commit author identity, raw diagnostics, and transport details SHALL remain accessible under Advanced Git Settings or troubleshooting rather than appearing as ordinary sync choices. Repository-specific settings SHALL be persisted atomically in a versioned local policy file separate from session data and Git-owned remote/upstream configuration. Global and per-repository background settings SHALL identify their respective default/current-workspace scope.

#### Scenario: Existing installation upgrades
- **WHEN** the app loads configuration without Git keys or a repository policy file
- **THEN** it retains existing preferences and does not connect or synchronize any repository automatically

#### Scenario: Git override is invalid
- **WHEN** the user selects an unusable executable path
- **THEN** Advanced Git Settings report the failed detection without making ordinary document operations unavailable

#### Scenario: Ordinary settings are opened
- **WHEN** the user opens the General preferences category
- **THEN** it shows one Backup and Sync section using note/device language and keeps executable selection behind an Advanced Git disclosure

#### Scenario: Background checking is offered
- **WHEN** the user reviews the background update-check option
- **THEN** the interface states that it checks for updates from other devices and does not automatically upload or apply them

#### Scenario: Repository-specific background setting is shown
- **WHEN** the current workspace has a per-repository background-check override
- **THEN** preferences distinguish that workspace setting from the global default rather than presenting two unlabeled equivalent toggles

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
