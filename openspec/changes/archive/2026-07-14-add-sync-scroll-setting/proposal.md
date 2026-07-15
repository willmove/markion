## Why

In Split Preview mode the source editor and rendered preview scroll independently. When navigating a long document the two panes drift apart, so the user must scroll each pane by hand to bring the preview block for the line they are editing into view. A toggleable, persisted "Sync scroll" preference lets the two panes follow each other, while keeping the current independent behavior as the default for users who prefer it.

Non-goals: this change does not alter Markdown parsing, preview block generation, derived-state caching, view-mode switching, export, or document content. It does not add element-level (block-to-source-line) mapping; synchronization is proportional by scroll fraction, which is robust against the existing per-version caches and requires no new parse data.

## What Changes

- Add a persisted "Sync scroll" boolean preference, disabled by default, that when enabled in Split Preview mode makes the source editor and rendered preview panes scroll together proportionally.
- When Sync scroll is enabled, scrolling either pane (mouse wheel, trackpad, or dragging either pane's scrollbar) updates the other pane's scroll position to the same fraction of its scrollable range, clamped to its bounds.
- Sync scroll applies only in Split Preview mode (the only mode where both panes are visible). It has no effect in Edit or Read mode.
- Persist the new preference in the existing `config.toml`, tolerating older files that omit it; include it in preferences reset and restore-on-launch behavior.
- Add a Sync scroll toggle to the Preferences panel "Other" settings, applying immediately and persisting.
- Add localized (English + Simplified Chinese) UI strings for the panel label, status feedback, and any summary text, with the i18n exhaustiveness guard extended.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Adds the Sync scroll preference (persistence, default, reset, restore-on-launch) and the proportional scroll-coupling behavior in Split Preview mode.
- `theme-preferences`: Extends the Preferences panel "Other" settings with a Sync scroll toggle.
- `markdown-editing`: Extends the per-tab pane scroll-state requirement to describe the optional proportional coupling between the source and preview scroll positions in Split Preview mode.
- `ui-i18n`: Adds localized UI strings for the Sync scroll preference label, status feedback, and summary text.

## Impact

- Affected code: `src/model.rs` (add the preference field + default), `src/storage/preferences.rs` (TOML parse/render + legacy migration), `src/main.rs` (preference field on `MarkionApp`, toggle action, preferences panel row, reset/snapshot/`current_preferences`, and the scroll-coupling hook that observes a pane's scroll change and writes the proportional offset to the other pane), `src/i18n.rs` (new `Msg` variants + `en`/`zh` arms + exhaustiveness list).
- No new dependencies.
- The change touches scroll state only. It must preserve the cached-per-version derived Markdown state, memoized highlighting, cached text handles, and the existing per-tab scroll isolation (sync state is per-tab-scrolled, not a shared global scroll).
- Invariants to preserve: the editor reuses a cached text handle per version; the preview `ListState` is virtualized and splices only changed ranges; tab switching restores each tab's scroll positions. Sync scroll must not force a reparse or list reset.
