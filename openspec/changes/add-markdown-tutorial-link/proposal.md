## Why

Help → Markdown Reference currently offers only an in-app syntax cheat sheet. Users who want a fuller tutorial have no path from that overlay to the existing Kenhuang Markdown tutorial, even though that is the natural place to discover it.

## What Changes

- Add a Kenhuang Markdown tutorial link at the top of the Markdown Reference overlay, below the overlay title and above the scrollable syntax sections, so it stays visible without scrolling.
- Open `https://kenhuang.com/markdown/` when the interface language is Simplified or Traditional Chinese, and `https://kenhuang.com/en/markdown/` for every other supported language.
- Activate the link through the system default browser via the existing platform-shell `open_url` path. The overlay itself does not fetch remote content or embed a web view.
- Localize the tutorial-link label for every supported interface language; display the destination URL verbatim.

**Non-goals:** adding a Help-menu item for the tutorial, embedding the tutorial in-app, changing Markdown Reference sections/shortcuts, fetching or checking URL availability, or adding a third language-specific tutorial URL.

This change does not touch per-document-version Markdown caches, syntax-highlight memoization, text-handle reuse, or any `crates/*` member.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `chrome-platform`: Markdown Reference overlay presents a language-selected Kenhuang tutorial link at the top and opens it in the system browser.
- `ui-i18n`: The tutorial-link label is routed through the i18n catalog for every supported interface language.

## Impact

- Overlay rendering in `src/app/root_view.rs` (`markdown_reference_view`).
- Link activation in `src/app/search.rs` (same `cx.open_url` pattern as About and Help-menu external links).
- Destination constants and language selection next to the existing About/Help URL constants in `src/app/mod.rs`.
- New `Msg` variants and translations in `src/i18n.rs`.
- Focused tests in `src/app/tests.rs` for URL selection, placement, activation, localization, and document-state isolation.
- No persisted-data migration, public API change, new dependency, or Help-menu structure change.
