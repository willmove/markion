## Context

The binary application is implemented almost entirely in `src/main.rs`. The file owns GPUI action declarations, application and per-tab state, command handlers, custom elements, rendering helpers, platform integration, and tests. These concerns share `MarkionApp`, but they do not need to share one source file.

The refactor must preserve this typing/rendering data flow:

```text
command or IME input
        |
        v
MarkdownDocument mutation -> text version increment/cache invalidation
        |
        v
EditorTab per-version text/layout/preview state
        |
        +-> debounced background preview parse -> Arc<Vec<PreviewBlock>> -> ListState splice
        +-> cached editor SharedString/layout data -> EditorElement paint
```

The root-package workspace layout and the rule that `crates/*` remain free of GPUI dependencies also constrain the destination: application modules belong under the root package's `src/app/`, not in a new workspace member.

## Goals / Non-Goals

**Goals:**

- Reduce `src/main.rs` to a thin binary entry point.
- Give bootstrap, state, commands, custom editor elements, root views, preview views, and tests clear source-file ownership.
- Preserve all existing behavior and performance invariants.
- Keep implementation visibility internal to the binary crate.

**Non-Goals:**

- Redesigning `MarkionApp` or replacing it with a new state-management architecture.
- Changing UI behavior, shortcuts, localization, persistence, exports, or workspace semantics.
- Moving GPUI-dependent code into `crates/*`.
- Adding dependencies or changing public library APIs.

## Decisions

### Keep a single `app` parent module

`src/main.rs` will declare `mod app` and forward to `app::run()`. `MarkionApp` and all GPUI implementation modules remain descendants of `app`, allowing crate-internal coordination without exposing application internals publicly.

Alternative considered: move application code into `src/lib.rs`. Rejected because it would expand the public library surface and mix GPUI application wiring with the Markdown library API.

### Split by cohesive responsibility without redesigning state

The first refactor will use focused modules for state/history, core app accessors, document commands, workspace/view commands, appearance/preferences, search, editing commands, custom editor elements, root/panel views, preview/visual rendering, and bootstrap wiring. Existing fields and algorithms move mechanically before any deeper state grouping is considered.

Alternative considered: simultaneously replace the 52 `MarkionApp` fields with nested state objects. Rejected for this change because it would combine module movement with borrow-structure and behavioral changes, making regressions harder to isolate.

### Use parent-scoped visibility for cross-module handlers

Items referenced across sibling application modules will use `pub(super)` rather than `pub` or `pub(crate)`. Child modules may continue to access the parent-owned application state; no new external API is created.

Alternative considered: introduce controller traits for every command family. Rejected as unnecessary indirection for a single GPUI entity.

### Move tests with their owning concerns

Focused unit tests will live in each implementation module when practical. Cross-module regression tests may remain in `app/tests.rs`. Tests will continue to exercise the same algorithms and application invariants.

## Risks / Trade-offs

- [Large mechanical diff conflicts with active UI changes] -> Keep the refactor behavior-preserving, avoid formatting unrelated library files, and validate after each extraction stage.
- [Rust privacy errors appear when methods move into sibling modules] -> Use `pub(super)` only for items required across the `app` module tree and let `cargo check` identify missing edges.
- [A cache reset or render path is accidentally changed] -> Move method bodies without rewriting them and retain the existing cache/preview regression tests.
- [Too many tiny files reduce discoverability] -> Split at feature-level responsibilities and keep closely coupled helpers together; do not create one file per type or action.

## Migration Plan

1. Create `src/app/`, move the application implementation behind `app::run()`, and leave the crate-level Windows subsystem attribute in `src/main.rs`.
2. Extract contiguous responsibility groups into `app` child modules, adjusting only module imports and visibility.
3. Relocate tests to the application test module and run formatting, build, and root-package tests.
4. If validation fails, revert the latest extraction group rather than changing algorithms to fit the new layout.

## Open Questions

None. Further grouping of `MarkionApp` fields can be evaluated as a separate change after these module boundaries settle.
