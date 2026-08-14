## Why

The status bar currently reports the document title, save state, and transient operation feedback, but it does not expose the basic document and editing context users repeatedly need while writing. Showing document counts, the current Git branch when applicable, and the caret position makes that context continuously available without opening another panel or interrupting editing.

## What Changes

- Split the status bar into a flexible message area and a compact, persistent document-context area.
- Show the active document's character count and word count using the existing per-document-version cached statistics.
- Show the caret's one-based line and column while the active editing surface has a meaningful caret.
- Show the current branch when the active document or workspace belongs to a Git repository; omit the item when no named branch is available.
- Keep Git discovery and refresh work off the render and typing paths, and cache its result independently from Markdown-derived state.
- Localize all new labels and formatting through the existing exhaustive i18n catalog.
- Preserve the existing transient status feedback and degrade the persistent context gracefully in narrow windows.
- Non-goals: Git status/actions, ahead/behind or dirty-file indicators, configurable status-bar items, selection-specific counts, or language-aware lexical analysis beyond the existing word-count semantics.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Define the persistent document context, conditional Git branch, caret semantics, responsive layout, and performance behavior of the status bar.

## Impact

- Affected application chrome and state: `src/app/root_view.rs`, active-tab/caret state, and tests in `src/app/tests.rs`.
- Reuses `MarkdownDocument::stats()` and UTF-8-safe line/column helpers without weakening the per-document-version derived-state cache invariant.
- Adds localized status-bar labels in `src/i18n.rs`; the exhaustive translation contract remains intact.
- Adds a cached, background Git-context lookup associated with the active document/workspace, using standard Git metadata without a new runtime or subprocess dependency; no workspace member gains a GPUI dependency.
