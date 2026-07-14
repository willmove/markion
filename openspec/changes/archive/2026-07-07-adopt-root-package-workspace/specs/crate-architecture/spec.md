## ADDED Requirements

### Requirement: Root-package Cargo workspace layout
The repository SHALL be a root-package Cargo workspace: the `markion` application crate lives at the repository root (manifest `Cargo.toml`, sources in `src/`), and the same manifest carries a `[workspace]` table whose members are the library crates under `crates/*`. The application crate SHALL NOT be relocated into `crates/` (no virtual-workspace conversion). `cargo build`, `cargo run`, and `cargo test` invoked at the repository root SHALL keep operating on the application crate by default, and the release pipeline (`packager.toml`, `.github/workflows/release.yml`) SHALL keep working without workspace-specific reconfiguration.

#### Scenario: Root commands unchanged after workspace adoption
- **WHEN** `cargo check`, `cargo build`, `cargo run`, or `cargo test` is invoked at the repository root
- **THEN** it targets the `markion` application crate exactly as before the `[workspace]` table existed, and all workspace members share the single root `Cargo.lock` and `target/` directory

#### Scenario: Library crate added as a member
- **WHEN** a library crate (e.g. one absorbed from Typune) is placed under `crates/<name>` with its own `Cargo.toml`
- **THEN** it is picked up by the `members = ["crates/*"]` glob without further manifest wiring, may declare its own edition and its own dependency versions (semver-incompatible duplicates with the root crate are permitted during migration), and is selectable via `cargo test -p <name>`

### Requirement: Member crates free of GUI coupling
Library crates under `crates/*` SHALL NOT depend on `gpui`. Types that implement gpui traits SHALL live in the root application crate (the orphan rule forbids implementing a foreign trait for a type from another crate); member crates expose pure data and logic only. Member crates SHALL build and pass their tests without GUI system libraries present.

#### Scenario: Member crate tested headless
- **WHEN** `cargo test -p <member>` runs in an environment without GUI system libraries (wayland/x11)
- **THEN** the member crate compiles and its tests run without pulling in `gpui`

### Requirement: Dev-profile optimization coverage for member crates
Because `[profile.dev.package."*"]` overrides do not apply to workspace members, any compute-heavy member crate on the typing path (parsing, syntax highlighting, export) SHALL get an explicit `[profile.dev.package.<name>]` opt-level override in the root manifest when it is added, so `cargo run` dev builds keep the editor responsive.

#### Scenario: Compute-heavy member joins the workspace
- **WHEN** a member crate that executes on the typing path is added under `crates/`
- **THEN** the root `Cargo.toml` gains a matching `[profile.dev.package.<name>]` override in the same change
