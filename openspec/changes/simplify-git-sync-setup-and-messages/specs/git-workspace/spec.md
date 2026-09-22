## ADDED Requirements

### Requirement: Healthy existing repositories SHALL be adopted for sync without manual setup
When synchronization is first requested for a workspace that has no saved sync policy, Markion SHALL probe the workspace and adopt an existing repository automatically instead of opening setup, when all of the following hold:

- the canonical workspace root is itself the repository worktree root (not a subdirectory of a larger worktree);
- the repository contains at least one commit;
- the repository supports write synchronization;
- the configured origin yields exactly one resolvable fetch and push target for the current branch;
- the tracked content is notes-like: hidden configuration paths (for example `.gitignore` or `.obsidian/…`) and standard repository furniture (`LICENSE`, `COPYING`, `NOTICE`, `README`) do not count as unrelated content, while visible non-notes files such as source or build files do.

Adoption SHALL create the same default personal-notes policy that manual connection creates, with background fetching following the existing global preference. Adoption SHALL NOT mutate the repository (remotes, branches, staging, and history stay untouched), SHALL NOT require a network round-trip to succeed, and SHALL announce the connection with a localized status notification. An adopted connection SHALL remain inspectable, editable, and removable in settings exactly like a manually configured one. When any criterion above fails, the existing contextual onboarding route SHALL be presented unchanged.

#### Scenario: Active dedicated notes repository is adopted
- **WHEN** the user triggers the first sync in a workspace that is the root of a repository with existing commits, a working origin, and notes-like tracked content
- **THEN** the policy is created and the requested sync proceeds without any setup dialog
- **AND** a status notification states that the existing repository was connected for sync

#### Scenario: Workspace is a subdirectory of a larger repository
- **WHEN** the discovered repository worktree root is above the workspace root
- **THEN** no automatic adoption happens and the existing advanced repository setup with both roots is offered

#### Scenario: Repository contains unrelated tracked content
- **WHEN** the repository's tracked content includes visible files outside the notes-like classes, such as source or build files
- **THEN** no automatic adoption happens and the contextual onboarding routes the user through explicit advanced review

#### Scenario: Repository furniture does not block adoption
- **WHEN** an otherwise eligible repository additionally tracks hidden configuration paths such as `.gitignore` or `.obsidian/app.json`, or standard furniture such as `LICENSE`
- **THEN** automatic adoption still proceeds without treating the repository as mixed content

#### Scenario: Repository lacks a usable remote or history
- **WHEN** the repository has no resolvable fetch/push target or no commit at all
- **THEN** no automatic adoption happens and setup opens on the applicable contextual route, requesting a sync address where required

#### Scenario: Adoption performs no Git writes
- **WHEN** adoption succeeds
- **THEN** remotes, branches, index, and commit history are unchanged
- **AND** only the application's policy file is written

#### Scenario: Adopted connection stays user-managed
- **WHEN** the user opens sync settings after an automatic adoption
- **THEN** the connection is listed with the same inspection, editing, and removal controls as a manually configured connection
