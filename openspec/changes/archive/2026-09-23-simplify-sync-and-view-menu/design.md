## Context

See `proposal.md` for motivation and the delta specs for the observable contract. Markion currently represents Source and Split Preview as separate GPUI actions, menu rows, shortcut descriptors, and shortcut-reference entries. `SetEditMode` owns the `set-edit-mode` persisted shortcut id and `Ctrl+/`; `SetSplitPreviewMode` owns `set-split-preview-mode` and `Ctrl+P`. Both handlers already converge on the central view-mode setter. The top-level repository menu and the Backup and Sync panels currently share `GitMsg::BackupAndSync`, so changing that translation in place would rename more UI than requested.

View changes are presentation state only. They must continue through the existing view-mode setter so document versions and the derived preview, outline, statistics, syntax-highlighting, visual-edit, and text-handle caches are neither invalidated nor rebuilt solely because this command ran.

## Goals / Non-Goals

**Goals:**

- Give both native and in-window menus one action and one shortcut descriptor for Source/Split Preview.
- Preserve existing customized Source-shortcut bindings without adding a preferences-file migration.
- Keep the synchronization menu title independently localizable from the broader Backup and Sync feature name.
- Route the new action through the existing view-mode transition and status-feedback path.

**Non-Goals:**

- Redesigning `ViewMode`, changing the four-mode cycle, or changing view-mode persistence.
- Changing document mutations, Markdown rendering, or cache ownership.
- General shortcut-schema versioning or retaining custom bindings for the removed Split Preview action.

## Decisions

### Introduce one source-layout toggle action

Replace the two menu-facing actions with a single `ToggleSourceSplitMode` action. Its handler selects Split Preview only when the current mode is Source; for Split Preview, Visual Edit, or Read it selects Source. It then calls the existing central view-mode setter so focus handling, status feedback, layout notification, and state persistence remain consistent with other view changes.

This rule is explicit rather than reusing the four-mode `next()` transition because `next()` includes Visual Edit and Read. Keeping a separate two-state transition also leaves the existing all-mode cycle untouched.

### Retain the Source shortcut's stable persisted id

Define the combined shortcut descriptor under a source-layout-oriented Rust constant, but retain the persisted id `set-edit-mode` and the default `secondary-/` binding. This transfers existing Source shortcut customizations to the combined command without reading or rewriting preferences. Remove `set-split-preview-mode` from the active shortcut registry; the existing unknown-id sanitizer will ignore obsolete Split Preview overrides.

The alternative was a new `toggle-source-split-mode` id plus an explicit preferences migration. That would make the storage name more literal, but adds migration and rollback surface for no user-visible benefit. The retained id should receive a compatibility comment so a future cleanup does not accidentally break saved bindings.

### Use a distinct localized message for the Sync menu title

Add a dedicated Git-menu message for “Sync” in every supported language and use it only for the native and in-window top-level menu category. Keep `GitMsg::BackupAndSync` at panel headings, setup/status/settings content, and sync-center entry points that describe the broader feature.

Reusing and shortening `GitMsg::BackupAndSync` was rejected because that message is shared by UI outside the requested menu title.

### Update both menu surfaces and the shortcut catalog from the same command contract

The native View menu and in-window View dropdown will each replace their Source and Split Preview rows with one localized Source/Split Preview row bound to `ToggleSourceSplitMode`. The shortcut registry and Help reference will expose the same combined descriptor and label. Visual Edit, Read, and the existing all-mode cycle remain separate.

Keeping both menu constructions explicit matches the existing architecture; introducing a new cross-surface menu abstraction would be disproportionate to this change.

### Preserve the existing cached-state data flow

The transition remains:

`menu row or key binding -> ToggleSourceSplitMode -> existing view-mode setter -> layout/focus/status update`

No document mutation or version increment occurs. Existing per-document-version caches stay attached to the same version and are reused according to their current rules. Tests will assert preserved document/editor state in addition to the target mode.

## Risks / Trade-offs

- [The compatibility id `set-edit-mode` no longer literally describes the combined behavior] → Keep it isolated behind the renamed descriptor constant and document that it is retained for persisted-shortcut compatibility.
- [A user-customized former Split Preview shortcut stops working] → Remove the obsolete action cleanly and rely on the established unknown-id sanitizer; document the intentional breaking shortcut change in the proposal.
- [Only one of the two menu surfaces is updated] → Add source-contract and UI/action tests covering both native and in-window menu construction.
- [Shortening the shared translation accidentally changes panel copy] → Add a dedicated message variant and tests that distinguish the Sync title from Backup and Sync content.
- [The toggle invalidates or recomputes derived Markdown state] → Use the existing view-mode setter and assert document version/state and cache invariants across transitions.

## Migration Plan

1. Add the dedicated localized Sync title and combined Source/Split Preview label.
2. Replace the two GPUI actions, bindings, menu rows, and shortcut-reference entries with the combined action while retaining the `set-edit-mode` persisted id.
3. Remove the former Split Preview descriptor and default `Ctrl+P` binding from the active registry; allow existing sanitization to ignore old overrides.
4. Run focused action/menu/localization tests, then the root and workspace test suites.

No document or preferences rewrite is required. Rolling back the code restores the former menu rows and defaults; Source customizations remain addressable because the stable id was retained.
