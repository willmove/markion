# Design: open-documents-in-current-tab

## Context

See `proposal.md` — Why. Today's tab targeting is hard-coded per entry point: `OpenPathIntent` (`src/app/workspace.rs:3-7`) already distinguishes `ReplaceActive` from `OpenInNewTab`, and `open_supported_path(path, intent, cx)` (`workspace.rs:338-378`) is the single dispatcher that dedupes already-open files and routes images vs. documents. File-tree clicks, drag-drop, and Open Recent bypass or hard-code the "new tab" intent (`open_file_in_new_tab_from_path` at `workspace.rs:331`, `open_recent_path` at `application.rs:1421-1446`); File → Open hard-codes "replace after dirty guard" (`documents.rs:26-83`). `replace_active_with_tab` (`application.rs:799-823`) deletes the replaced tab's recovery snapshot, and `open_tree_file`'s comment (`application.rs:831-833`) records why tree clicks historically avoid both the dirty guard and replacement.

## Goals / Non-Goals

**Goals:**

- One preference-driven default-intent resolver shared by every non-explicit open entry.
- Never prompt from a gesture, never discard dirty work, never delete a recovery snapshot that belongs to a dirty tab.
- Keep all explicit new-tab affordances and already-open dedup behavior untouched.

**Non-Goals** (beyond the proposal's): no new prompts/dialogs; no changes to dormancy, session restore, or the recovery subsystem itself.

## Decisions

### D1: Resolve the default intent in one place, route everything through the dispatcher

Add a resolver on `MarkionApp`, conceptually:

```
fn default_open_intent(&self) -> OpenPathIntent {
    if !self.open_in_current_tab { return OpenInNewTab; }
    let tab = self.active_tab();
    let replaceable = tab.is_image()
        || tab.document().map_or(true, |d| d.path.is_none())   // untitled / welcome
        || tab.document().map_or(true, |d| !d.is_dirty());
    if replaceable { ReplaceActive } else { OpenInNewTab }
}
```

Every non-explicit entry calls `open_supported_path(path, self.default_open_intent(), cx)` (or an async-prefixed variant matching the existing loading styles): file-tree plain click and its context-menu "Open", drag-drop document opens, Open Recent, and File → Open. The dispatcher's existing `focus_existing_tab_for_path` dedup stays first, unchanged. Alternative considered: teaching each entry point its own conditionals — rejected because the rules would drift and the spec states them as one invariant.

*Alternative for File → Open*: when the resolved intent is `ReplaceActive`, keep the existing `confirm_discard_then` dirty guard (`documents.rs`); when it resolves to `OpenInNewTab` (preference off), skip the guard because nothing is discarded. This is the one deliberate asymmetry: pickers are low-frequency and already modal, so the guard there costs nothing, while gestures stay prompt-free.

### D2: Replace-eligibility predicate (clean / untitled / image), dirty diverts silently

A tab is replaceable iff it is an image tab, an untitled document, or a **clean** document. An untitled document with typed content is dirty, so it diverts to a new tab — this is what makes the rule safe: `replace_active_with_tab` deletes `last_recovery_file` (`application.rs:802-806`), and only dirty tabs have live recovery snapshots, and dirty tabs are never replaced by gesture opens. The dirty case falling back to "append a new tab" is exactly today's behavior for tree/drop/recent, so the dirty path needs no new code paths — only the clean path changes.

### D3: Multi-file drop replaces once, then appends

In `handle_external_drop` (`workspace.rs:393-437`), iterate the dropped supported paths: the first uses `default_open_intent()`, every subsequent one uses `OpenInNewTab` explicitly. Otherwise a 3-file drop would thrash replace→replace→replace and strand the first two files.

### D4: Ctrl/Cmd+click as the per-click escape hatch

The file-tree row click handler reads the platform modifier (Ctrl on Windows/Linux, ⌘ on macOS) from the mouse event and forces `OpenPathIntent::OpenInNewTab`. Browser/VS Code convention; makes the new default reversible without visiting Preferences. The existing context-menu "Open in New Tab" item stays as the discoverable explicit route.

### D5: Preference plumbing mirrors `show_hidden_files` exactly

`AppPreferences` field + default `true` (`src/model.rs`), `PreferencesFile` field + both `From` impls (`src/storage/preferences.rs`), mirrored `MarkionApp` field loaded in `new` and written in `current_preferences` (`src/app/application.rs`, `src/app/mod.rs`), `toggle_open_in_current_tab` in `src/app/appearance.rs`, `preference_boolean_row` in the Preferences General tab's "Other" section (`src/app/root_view.rs`), and `Msg` variants translated in all seven language blocks (`src/i18n.rs`).

For safe degradation the spec requires invalid values to fall back to the default (**on**), so add a `deserialize_bool_or_true` helper mirroring the existing `deserialize_bool_or_false` (`storage/preferences.rs:303-309`) rather than reusing the false-degrading one.

### D6: Data flow / caching impact — none new

Replacement constructs a fresh `EditorTab` through the existing `editor_tab_for_document` / `editor_tab_for_image` path; appending runs the existing dormancy pipeline (`enter_dormant`, image-claim release) on the tab being left (`application.rs:733-770`). Derived-state caching per document version, memoized highlighting, and per-version text handles are untouched — no derived state is recomposed differently than today's replace/new-tab paths already do.

## Risks / Trade-offs

- **[Default flip surprises existing users]** → release notes call it out prominently with the one-step restore (toggle off); Ctrl/Cmd+click gives per-click escape; dirty work is never affected.
- **[Replacing a clean tab drops its undo history and scroll state]** → accepted trade-off (same as VS Code preview tabs); the file's saved content is untouched and the modifier-click path preserves history when the user cares.
- **[Contradiction with the unarchived `add-drag-drop-open` delta, which hard-codes "open each dropped path … as a new tab"]** → this change rewrites that delta's requirement to the preference-driven rule before archive, so the two cannot sync into conflicting specs.
- **[Subtle regression around the recovery-snapshot delete in `replace_active_with_tab`]** → covered by D2 (dirty tabs are never gesture-replaced) plus tests asserting a dirty tab's recovery file survives a tree-click open.

## Migration Plan

Ship with the preference defaulting to on. Rollback for any user: Preferences → toggle off (persisted immediately); the previous behavior is exactly "preference off". No config migration — the field is optional and missing means on.
