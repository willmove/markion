## 1. Exact Selection Targeting

- [x] 1.1 Reconcile the completed `align-visual-edit-content-and-compact-block-context-menu` prerequisite so the compact pointer/keyboard block-menu source and specification baseline are present without overlapping edits
- [x] 1.2 Add an immutable selection-format target carrying document version, exact UTF-8 range, and owning block identity, and derive it only for one safe editable run in the invoked block
- [x] 1.3 Cover selection-target construction for exact, empty, cross-run, math, conservative, source-island, different-block, and stale-version cases without changing document or cache state

## 2. Unified Context Menu

- [x] 2.1 Extend the block-menu item model and ephemeral state with optional Bold, Italic, Inline Code, and Link actions, deriving pointer rendering and keyboard traversal from the same dynamic ordered item list
- [x] 2.2 Render the selection actions as a separated localized group ahead of the existing block operations while preserving occlusion, viewport clamping, menu-local scrolling, enabled states, and pointer/keyboard activation
- [x] 2.3 Revalidate the captured version, range, and safe block ownership on dispatch, then reuse the existing formatting or link-editor commands; dismiss stale targets without mutation
- [x] 2.4 Add or reuse localized labels for every supported language and retain localization completeness coverage

## 3. Remove the Floating Toolbar

- [x] 3.1 Remove root-level automatic selection-toolbar eligibility/rendering and the obsolete toolbar view without changing Visual Edit selection painting or document layout
- [x] 3.2 Remove or replace toolbar-specific debug selectors and tests so no hidden placeholder or obsolete presentation path remains

## 4. Interaction and Invariant Coverage

- [x] 4.1 Add rendered GPUI coverage proving selection alone shows no floating toolbar, same-block right-click exposes all four formatting actions plus block operations, unrelated/unsafe targets omit formatting actions, and keyboard invocation exposes the same actions
- [x] 4.2 Add interaction coverage for Bold, Italic, and Inline Code exact source mutations with one-step Undo, plus Link opening/cancel behavior with exact selection preservation
- [x] 4.3 Add lifecycle and constrained-viewport coverage proving dynamic keyboard navigation and open/navigate/dismiss/stale paths preserve source, version, selection, history, dirty state, and shared derived-cache identity

## 5. Verification

- [x] 5.1 Run formatting checks and the focused Visual Edit/context-menu test set, fixing all failures
- [x] 5.2 Run `cargo test --workspace` and `openspec validate move-visual-selection-formatting-to-context-menu`, confirming the change is implementation-complete and spec-consistent
