## Why

Markion exposes heading levels in the Format menu and on `Ctrl/Cmd+1` through `Ctrl/Cmd+6`, but it has no matching command for returning the current heading line to an ordinary paragraph. Adding the conventional `Ctrl/Cmd+0` action completes this block-style workflow and makes the operation discoverable alongside the heading commands.

## What Changes

- Add a Paragraph action at the start of the heading section in both the native and in-window Format menus.
- Bind Paragraph to `Ctrl+0` on Windows/Linux and `Cmd+0` on macOS through the existing customizable shortcut registry, and expose its effective binding in menus, Preferences, and the localized shortcut reference.
- When invoked with a caret or selection, remove ATX heading prefixes from the intersected line or lines while preserving their content and routing edits through the existing document mutation, undo, dirty-state, and derived-cache lifecycle; intersected non-heading lines remain unchanged.
- Treat invocation when no intersected line is an ATX heading as an idempotent no-op apart from user-facing status feedback.
- Add localized menu and status text plus regression coverage for transformation behavior, shortcut dispatch/customization, menu integration, and shortcut discoverability.
- Non-goals: converting lists, blockquotes, code blocks, or Setext headings into paragraphs; changing heading-depth preferences; or redesigning Markdown parsing, rendering, or caching.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Add a source-backed Paragraph formatting action, its Format-menu placement, and its platform-aware `Ctrl/Cmd+0` shortcut contract.

## Impact

- Affected code: the GPUI action and shared shortcut registry in `src/app/mod.rs`, key binding and native menu construction in `src/app/bootstrap.rs`, the in-window Format menu in `src/app/root_view.rs`, formatting dispatch in `src/app/editing.rs`, Markdown transformation support in `src/model.rs`, localization and shortcut-catalog data in `src/i18n.rs`, and focused tests.
- No new dependencies, persistence schema changes, or public API changes are expected; the new stable shortcut id participates in the existing preferences override format.
- The action uses the canonical `MarkdownDocument.text` mutation path. It changes the document version only when source text changes, preserving the existing per-version derived Markdown caches, syntax-highlight memoization, and cached text-handle invariants.
