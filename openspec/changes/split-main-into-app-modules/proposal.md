## Why

`src/main.rs` has grown to more than 11,000 lines and combines application bootstrap, GPUI state, commands, custom elements, view construction, preview rendering, and tests. This makes routine changes harder to review and increases merge conflicts across otherwise independent UI capabilities.

## What Changes

- Keep `src/main.rs` as a minimal binary entry point and move GPUI application code into a dedicated `src/app/` module tree.
- Separate application state, command handlers, bootstrap wiring, and UI rendering into cohesive modules without changing observable behavior.
- Co-locate focused tests with the modules they exercise while retaining cross-module regression coverage.
- Preserve the per-document derived-state caches, syntax-highlight memoization, cached editor text handles, virtualized preview/file-tree behavior, and undo-cache invariants.
- Non-goals: redesigning the UI, changing user-facing behavior, changing persisted formats, introducing new dependencies, or moving GPUI code into workspace member crates.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `crate-architecture`: Define maintainable module boundaries for the root GPUI application while preserving the existing root-package workspace layout and GUI-free member-crate rule.

## Impact

- Affects `src/main.rs` and new modules under `src/app/`.
- Does not change public library APIs, persistence formats, dependencies, shortcuts, or user-facing behavior.
- Root `cargo build` and `cargo test` commands remain unchanged.
