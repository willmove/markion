## ADDED Requirements

### Requirement: Git synchronization SHALL have a GUI-free workspace boundary
The workspace SHALL contain `crates/git-sync` with package name `markion-git-sync`, owning pure repository/plan/state models, system-Git adaptation, execution policy, and recovery reconciliation without depending on GPUI. Root-app modules SHALL own live document coordination, GPUI tasks, credential/conflict dialogs and rendering. Git snapshots SHALL be cached/shared and process/network/filesystem work SHALL not run on the typing/render path. No additional Git backend SHALL be required for initial delivery.

#### Scenario: Core tests run headlessly
- **WHEN** `cargo test -p markion-git-sync` is run in a headless environment
- **THEN** its dependency graph does not require GPUI or GUI system libraries and real local Git fixtures can exercise its operations

#### Scenario: Status updates without document changes
- **WHEN** repeated repository status snapshots arrive while source text is unchanged
- **THEN** the root app updates Git presentation from cached state without incrementing document versions or rebuilding Markdown-derived caches

#### Scenario: Large repository is scanned
- **WHEN** an asynchronous status/diff/history operation processes a large repository
- **THEN** the UI consumes bounded snapshots/pages, avoids unbounded concurrent scans, and performs no synchronous Git calls during input/rendering
