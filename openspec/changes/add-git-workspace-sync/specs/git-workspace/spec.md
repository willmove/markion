## ADDED Requirements

### Requirement: Git availability and repository capabilities SHALL be explicit
Markion SHALL detect the configured or system Git executable, report its version and required capabilities, and preserve ordinary editing when unavailable. It SHALL support write synchronization for ordinary complete non-bare worktrees, including empty repositories, and SHALL detect and disable unsupported write synchronization for detached HEAD, linked worktrees, submodule-containing or LFS-managed repositories, bare, sparse, partial, and shallow repositories. Detection SHALL run outside rendering and text input.

#### Scenario: Git is missing
- **WHEN** a user opens Git setup without a usable Git executable
- **THEN** setup reports the missing prerequisite and offers executable configuration and installation guidance
- **AND** local opening, editing, saving, and recovery remain available

#### Scenario: Repository requires unsupported capabilities
- **WHEN** discovery identifies a linked worktree, submodule, LFS, or another unsupported repository shape
- **THEN** the app identifies the limitation and disables one-click write synchronization
- **AND** it neither converts the repository nor reports incomplete content as synchronized

### Requirement: Repository onboarding SHALL support clone connect and initialize
Markion SHALL support cloning an HTTPS or SSH remote, connecting an existing local repository, and initializing a local notes folder against an empty remote. It SHALL discover advertised default branches or existing upstreams, collect an explicit branch choice when discovery is ambiguous, and reject destructive destination replacement. Nonempty unrelated remote history SHALL lead to clone-and-copy guidance rather than an automatic unrelated-history merge.

#### Scenario: Clone succeeds
- **WHEN** the user selects a remote, branch, and absent or empty destination and cloning succeeds
- **THEN** the complete clone opens through the existing workspace-switching flow
- **AND** dirty buffers from the previous workspace retain their existing protection

#### Scenario: Clone is cancelled or destination changes
- **WHEN** cloning is cancelled or another process populates the chosen destination
- **THEN** the app preserves user-owned paths and the previous active workspace
- **AND** it cleans only confirmed operation-owned temporary content

#### Scenario: Empty remote receives first notes
- **WHEN** the user connects an empty remote and approves an initial branch, identity, and file policy
- **THEN** the first nonempty local commit can establish that branch and upstream without assuming an advertised default exists

#### Scenario: Local folder meets unrelated remote history
- **WHEN** an initialization flow discovers existing unrelated remote history
- **THEN** the app stops automatic integration and offers cloning to a separate directory and reviewing copied notes
- **AND** it preserves the original folder and any already-created local repository

### Requirement: Sync binding SHALL identify exactly one repository and destination
A connection SHALL bind a canonical worktree identity, named local branch, remote, resolved fetch and push endpoints, and one full destination branch ref. Setup SHALL explain whole-repository checkout and branch-history transfer even for a workspace subdirectory. Multiple push destinations, mirror configuration, ambiguous target mappings, and unreviewed changes of identity, branch, upstream or endpoint SHALL block one-click writes until explicitly resolved. Existing remote configuration SHALL NOT be replaced implicitly.

#### Scenario: Notes folder is inside a larger repository
- **WHEN** the workspace root is a subdirectory of a Git worktree
- **THEN** setup displays both roots and explains that integration can change files outside the workspace and push includes required branch history

#### Scenario: Bound destination changes externally
- **WHEN** the configured branch or resolved push endpoint changes outside Markion
- **THEN** an existing connection requires review before another write operation
- **AND** a running operation does not silently adopt the new target

#### Scenario: Selected existing remote branch disappears
- **WHEN** a previously bound remote branch is missing on a subsequent check
- **THEN** automatic synchronization stops and reports the missing target instead of silently recreating it

### Requirement: Personal notes policy SHALL authorize routine automatic commits once
Setup SHALL persist explicit repository-relative roots for tracked changes and explicit roots/classes for new notes and attachments. Routine Sync Now SHALL include eligible modifications, additions, and deletions without requiring repeated file selection or a commit message. New hidden files, dotpaths, symlinks, nested repositories, arbitrary binaries, and changes outside approved policy SHALL require attention rather than automatic inclusion. Git ignore rules SHALL apply to untracked candidates, not remove already-tracked paths from status. Scope approval SHALL NOT be described as limiting transferred branch history.

#### Scenario: Normal new note and image fall within policy
- **WHEN** the user adds a supported note and attachment in approved directories and clicks Sync Now
- **THEN** both are included with eligible tracked changes and an automatically generated message without another selection dialog

#### Scenario: Unexpected file appears
- **WHEN** an unignored new private configuration file or a tracked change outside the approved roots appears
- **THEN** the app shows the exception and allows explicit inclusion, policy adjustment, or exclusion
- **AND** the ordinary automatic commit does not silently include it

#### Scenario: Ignore rule matches an already-tracked note
- **WHEN** an already-tracked note changes and a new ignore rule matches its path
- **THEN** its tracked change remains visible and follows the tracked-path policy

#### Scenario: Rename crosses approved roots
- **WHEN** a renamed tracked file moves across a scope boundary
- **THEN** the app requests review of both paths and does not commit only the deletion or only the addition as an unnoticed scope effect

### Requirement: Git authentication and commit identity SHALL remain separate and protected
Explicit operations SHALL support HTTPS credential helpers and SSH user configuration, agents and host verification. The app SHALL keep author name/email separate from login identity, default author edits to repository-local scope, and respect signing/hook failures. App-acquired secrets SHALL remain session-only or be stored through a detected secure helper, never in repository files, session/policy files, command arguments, remote URLs, or logs. The app SHALL NOT disable certificate/host verification or silently configure plaintext credential storage.

#### Scenario: Missing HTTPS credential is requested
- **WHEN** an explicit operation needs a credential unavailable from the configured helper
- **THEN** the app exposes a controlled input flow with session-only use or supported secure-helper storage
- **AND** cancellation stops the operation without discarding local changes

#### Scenario: SSH host identity changes
- **WHEN** SSH reports a changed host key
- **THEN** the app reports a verification failure and does not automatically trust the new identity

#### Scenario: Author or signing information is unavailable
- **WHEN** a commit cannot obtain an author identity or required signing operation fails
- **THEN** the app requests identity configuration or reports signing failure without pretending the commit succeeded or bypassing the requirement

#### Scenario: Diagnostics contain secrets
- **WHEN** a URL, credential helper, or process error includes authentication material
- **THEN** displayed/stored diagnostics are redacted and no secret enters the operation journal
