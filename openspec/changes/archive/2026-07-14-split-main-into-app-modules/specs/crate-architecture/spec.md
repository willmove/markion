## ADDED Requirements

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
