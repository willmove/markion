# crate-architecture Specification

## Purpose
Define the structural invariants of the repository's Cargo workspace: the root-package layout (application crate at the repo root, absorbed library crates under `crates/*`), the GUI-free constraint on member crates, dev-profile optimization for typing-path members, and the single-version dependency policy for shared parsers.
## Requirements
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

### Requirement: Absorbed-crate de-inventory policy
Absorbed library crates (under `crates/*`) SHALL NOT retain modules whose keep/delete verdict has been settled by an evaluation (e.g. `docs/typune-integration-audit.md` §3 P2a/P3, or the `unify-pulldown-cmark` design.md AST-adoption decision) and which are verified to have no live reference from the application crate. Once the decision point resolves to "delete", the module SHALL be removed in that same change — alongside its `pub mod`/`pub use` declarations and any test file that imports it directly — rather than persisting as permanent dead code. Modules serving as inventory for a *recorded, gated* future decision (e.g. `incremental.rs` pending the AST-adoption gate) MAY be retained, but the gate and its re-open triggers SHALL be documented in the corresponding change's design.md.

#### Scenario: Decided-unused module is removed, not retained
- **WHEN** an evaluation concludes an absorbed-crate module is unused and will not be adopted (e.g. the Typune `html.rs`/`latex.rs`/`image.rs` exporters after the P2a comparison, or the `SyntaxHighlighter`/`AsyncHighlighter` facade after Phase 2 and the AST-adoption deferral)
- **THEN** the module file, its `pub mod` / `pub use` declarations in the crate `lib.rs`, and any test file importing it directly are all removed in one change

#### Scenario: Gated inventory is retained with a documented trigger
- **WHEN** a module is unused at present but serves a recorded future decision (e.g. `incremental.rs` pending the AST-adoption gate whose re-open triggers are in `unify-pulldown-cmark/design.md`)
- **THEN** it MAY remain in the crate, and the change that last touched the decision SHALL have a `design.md` naming the module and the conditions under which it will be adopted or deleted

#### Scenario: Live export-parse-path modules are not deleted
- **WHEN** a module is transitively reachable from a code path the application uses (e.g. `extended_inline` / `emoji` reached from `parser.rs` feeding the export engine, even though `src/` does not name them directly)
- **THEN** it SHALL NOT be removed without first verifying the reachability and recording the analysis in the change proposal

### Requirement: Root GPUI application source modularity
The root application crate SHALL keep its binary entry point thin and organize GPUI-dependent implementation under cohesive root-package application modules. Application bootstrap, per-document state, command handling, custom editor elements, and rendering SHALL remain in the root package and SHALL NOT be moved into GUI-free workspace member crates.

#### Scenario: Binary entry point delegates to application module
- **WHEN** the root binary source is inspected after the refactor
- **THEN** `src/main.rs` contains only crate-level binary configuration, the application module declaration, and delegation to the application runner

#### Scenario: GPUI responsibilities have explicit module ownership
- **WHEN** a maintainer locates bootstrap wiring, editor state, command handlers, custom elements, or preview and panel rendering
- **THEN** each responsibility is implemented in a cohesive module under `src/app/` rather than accumulated in the binary entry-point file

#### Scenario: Application modularization preserves runtime invariants
- **WHEN** the modularized root application is built and tested
- **THEN** existing document-version caches, shared derived Markdown state, syntax-highlight memoization, cached editor text handles, virtualized lists, and undo behavior remain covered and operate without behavioral changes

### Requirement: Diagram rendering has a GUI-free workspace boundary
The workspace SHALL contain a `crates/diagram` member whose Cargo package name is `markion-diagram`. The crate SHALL own the diagram backend trait, registry, pure render request/result/error types, output safety checks, and optional built-in backend adapters without depending on `gpui` or any other GUI toolkit. GPUI image conversion, application scheduling, and presentation state SHALL remain in the root `markion` crate. Because diagram rendering executes on the live preview path, the root manifest SHALL include an explicit `[profile.dev.package.markion-diagram]` optimization override.

#### Scenario: Diagram crate builds without GUI dependencies
- **WHEN** the dependency tree and tests for `markion-diagram` are inspected
- **THEN** the crate contains no GPUI or GUI toolkit dependency and can render/sanitize enabled backend output in a headless environment

#### Scenario: GPUI adaptation stays in the root crate
- **WHEN** sanitized diagram SVG is converted into a GPUI image or presented in a preview element
- **THEN** that implementation resides in the root application crate rather than `markion-diagram`

#### Scenario: Diagram member receives a dev optimization override
- **WHEN** `markion-diagram` is added to the workspace and used by live preview
- **THEN** the root `Cargo.toml` includes an explicit development-profile package override for `markion-diagram`

### Requirement: New diagram backends use explicit compile-time registration
A new in-tree or external diagram backend SHALL be able to depend on the GUI-free `markion-diagram` contract, implement the backend trait for its own backend type, and join Markion through explicit compile-time registry construction. The contract SHALL NOT require backend code to depend on Markdown parser internals, GPUI application state, or HTML export internals, and SHALL NOT claim runtime dynamic-plugin ABI compatibility.

#### Scenario: Contributor backend implements one core contract
- **WHEN** a contributor adds a new statically linked diagram format
- **THEN** format-specific parsing/rendering implements the diagram backend trait and is connected by explicit registry registration without adding a format-specific preview-block variant

#### Scenario: Runtime binary plugins remain out of scope
- **WHEN** the diagram registry is initialized
- **THEN** it uses statically linked backend instances and performs no dynamic-library discovery or loading

