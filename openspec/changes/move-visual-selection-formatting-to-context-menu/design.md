## Context

See `proposal.md` for motivation. Visual Edit currently derives `show_selection_toolbar` in the application root whenever `visual_selection_supports_contextual_format` accepts the active selection, then renders `visual_selection_toolbar` as a fixed top-right overlay with Bold, Italic, Inline Code, and Link buttons. Separately, every eligible Visual Edit row opens the compact block-operation overlay with an immutable `BlockTarget`; the menu already supports pointer and keyboard invocation, viewport anchoring, localized labels, keyboard navigation, stale-target validation, and presentation-only dismissal.

The completed active change `align-visual-edit-content-and-compact-block-context-menu` supplies the compact menu implementation on which this design depends. It should be archived or reconciled before application so the implementation and delta-spec baselines agree.

The relevant state flow becomes:

`exact selected_range + visual blocks + invoked BlockTarget -> optional SelectionFormatTarget -> one dynamic context-menu model -> revalidation -> existing formatting/link command -> canonical Markdown mutation`

Opening or dismissing the menu changes only ephemeral UI state. Formatting continues to mutate `MarkdownDocument.text` through existing command paths; no selection-only interaction changes the document version, shared per-version `Arc` derivations, memoized syntax highlighting, or cached text handles.

## Goals / Non-Goals

**Goals:**

- Consolidate selection formatting and block operations into one contextual overlay and one dismissal/navigation lifecycle.
- Keep the four current selection actions available by pointer and keyboard without automatically covering document content.
- Bind formatting actions to an immutable, exactly mapped selection target and reject stale or unsafe dispatch.
- Preserve existing block targeting, link editing, localization, one-command/one-undo semantics, and cache invariants.

**Non-Goals:**

- Expanding the set of inline formatting commands or adding formatting-state checkmarks.
- Changing how Markdown markers or links are serialized.
- Changing selection creation, block transformation/reordering, the Format menu, or Source/Read/Split interactions.
- Persisting context-menu or selection-formatting UI state.

## Decisions

### 1. Extend the compact block menu instead of creating a second selection menu

`BlockMenuState` gains an optional exact selection-format target. When present, the menu model prepends localized Bold, Italic, Inline Code, and Link items followed by a separator; the existing block groups follow unchanged. The root item collection and keyboard indexes are derived from one dynamic menu model rather than maintained as separate hard-coded pointer and keyboard lists.

This keeps overlay precedence, viewport clamping, click-away behavior, keyboard invocation, and menu-local scrolling in one place. A separate `SelectionContextMenu` was rejected because it would duplicate lifecycle state and require arbitration with the block menu whenever selected prose is also a transformable block.

### 2. Capture formatting eligibility as an immutable selection target

At menu invocation, the current selection is eligible only when it is non-empty, lies completely within one safe editable run of the invoked block, and that run is neither conservative fallback nor math/source-island content. The state records the document version, exact UTF-8 source range, and owning block identity alongside the existing `BlockTarget`. The current `visual_selection_supports_contextual_format` predicate becomes or delegates to a target-producing helper so rendering and dispatch share the same ownership rules.

Right-clicking another block continues to preserve the existing selection as required by the block-menu contract, but the unrelated menu omits the formatting group. Keyboard context invocation uses the same target builder against the caret/selection-owning block. Offering selection actions for every right-click while any selection exists was rejected because an unrelated block menu could then mutate text outside its visible target.

### 3. Revalidate before dispatch and reuse existing commands

A formatting menu item first verifies that the active tab is still the same document version, the exact selected range is unchanged, the owning block target is valid, and the range still belongs to one safe editable run. Failure dismisses the stale menu and performs no action. Success closes the menu and calls the existing Bold, Italic, or Inline Code action, or opens the existing exact source-backed link editor.

Reimplementing marker wrapping inside the menu was rejected because it would bypass established semantic undo, UTF-8 selection restoration, autosave/recovery, dirty-state, and status-feedback behavior.

### 4. Remove automatic toolbar composition without replacing it with hidden layout

The root no longer computes `show_selection_toolbar` or appends `visual_selection_toolbar`; the fixed overlay function and toolbar-specific debug selectors are removed. Selection paint remains unchanged. No invisible placeholder or reserved space is retained because the toolbar is absolutely positioned and has no document-layout responsibility.

### 5. Keep formatting labels localized and menu behavior consistent

The menu reuses the existing localized Bold, Italic, Inline Code, and Link messages where their wording fits. Any group heading or accessibility label that is newly visible must be added through the localization tables for every supported language. Selection actions use the same hover, disabled, keyboard-selected, occlusion, and viewport-overflow presentation as existing block items.

### 6. Verify behavior at the state and rendered-overlay seams

Unit tests cover target construction and stale rejection for empty, exact, cross-run, math, conservative, source-island, different-block, and changed-version selections. Rendered GPUI tests prove that selection alone does not create the old toolbar, pointer and keyboard context invocation show all four items, block actions remain present, and constrained menus stay reachable. Interaction tests invoke each command, verify exact source plus one-step Undo, verify Link cancel, and assert that open/navigation/dismiss paths preserve source, version, selection, history, dirty state, and derived `Arc` identity.

## Risks / Trade-offs

- **[Adding four items makes the compact menu taller]** -> Keep them in one separated group and retain the existing viewport clamp and menu-local scrolling tests.
- **[Selection can change while the menu is open]** -> Store an immutable selection target and revalidate every field immediately before dispatch.
- **[Right-click targeting can confuse selection and block ownership]** -> Show formatting only when the invoked block completely and safely owns the selection; preserve, but do not act on, unrelated selections.
- **[Dynamic menu items can desynchronize keyboard indexes]** -> Generate pointer rendering, enabled-state traversal, and keyboard activation from the same ordered item model.
- **[Removing the toolbar reduces discoverability]** -> Retain Format-menu commands and shortcuts, and expose the same four actions through both pointer and keyboard context-menu paths.

## Migration Plan

No persisted data or Markdown migration is required. Reconcile the compact block-menu prerequisite, add selection target/model support, add the four dynamic menu actions and tests, then remove the floating-toolbar composition and its obsolete tests. Rollback restores the toolbar render path and removes the optional menu prefix without affecting saved documents, preferences, or Markdown serialization.
