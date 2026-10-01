## Why

The About Markion dialog currently presents the GitHub repository URL as passive text, so users cannot follow it directly, and it does not expose the project's canonical website. Making both destinations actionable turns the dialog into a useful, low-friction path to official project information.

## What Changes

- Add the canonical project website, `https://markion.app`, to the About Markion dialog immediately above the GitHub repository entry.
- Render both the project website and GitHub repository entries as clearly identifiable links that open their exact HTTPS destinations in the system default browser when activated.
- Preserve the existing localized title, version, product description, close/confirmation behavior, Help-menu placement, and status feedback.
- Keep all new user-visible labels in the existing localization catalog for every supported interface language and style the dialog from the active theme palette.

**Non-goals:** no embedded browser, network preflight, website content fetch, changes to the separate Help-menu documentation/issue links, or changes to Markdown parsing and derived-state caches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Define the About Markion dialog's ordered project links and system-browser navigation behavior.

## Impact

- Affected application chrome: the About action and dialog rendering in `src/app/search.rs` / `src/app/root_view.rs`, transient dialog state in `src/app/mod.rs`, and project URL constants.
- Affected localization: About-dialog messages in `src/i18n.rs` for every supported language.
- Affected verification: focused Rust tests for link order, exact destinations, activation, dismissal, localization coverage, and active-theme rendering.
- No persisted-data migration, public API change, new dependency, or change to the per-document derived-state, highlighting, or cached-text invariants.
