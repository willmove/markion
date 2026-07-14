## ADDED Requirements

### Requirement: Workspace dependency policy
The workspace SHALL use a single pulldown-cmark version, declared once in `[workspace.dependencies]` and inherited by the root package and every member via `.workspace = true` — the split-version transition state (0.11 members / 0.13 root) is retired. Member crates SHALL NOT depend on gpui or other GUI toolkits. Compute-heavy workspace members SHALL carry explicit `[profile.dev.package.<name>]` opt-level overrides, since the `"*"` wildcard does not cover workspace members.

#### Scenario: One parser version across the workspace
- **WHEN** `cargo tree -i pulldown-cmark` is inspected
- **THEN** exactly one pulldown-cmark version appears, shared by the root package and the member crates

#### Scenario: Members stay GUI-free
- **WHEN** a member crate's dependency tree is inspected
- **THEN** it contains no gpui or other GUI toolkit dependency

#### Scenario: Typing-path members keep optimized dev builds
- **WHEN** a compute-heavy member crate is added to the workspace
- **THEN** it receives an explicit dev-profile opt-level override
