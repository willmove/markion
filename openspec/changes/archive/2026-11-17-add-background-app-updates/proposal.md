## Why

Users currently have to check for releases and initiate the whole update themselves. Markion should discover updates during long editing sessions and prepare authenticated packages without interrupting work.

## What Changes

- Check for newer published releases every five hours, including an overdue check after startup, with a persisted last-check timestamp and no overlapping work.
- Add a default-on automatic-download preference to General preferences, including backward-compatible configuration loading.
- Separate signed download/verification from installation. Offer Restart and Update or Later only after a verified package is ready; keep it available for later confirmation in the current session.
- Recheck unsaved documents immediately before installation, preserve manual checking and browser-download fallbacks, and keep routine background failures silent.
- Non-goals: adding a new macOS/DEB/RPM self-installation channel, changing release signing, or changing Markdown caches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-packaging`: periodic release discovery and staged authenticated updates with explicit installation consent.
- `theme-preferences`: persistent default-on automatic update downloads in General preferences.

## Impact

Touches `src/app/update.rs`, app initialization/state, General preferences, `AppPreferences`, TOML persistence, localization, and update tests. Reuses cargo-packager-updater and existing platform support. Network, signature verification and package I/O run off the render path; document versions and derived Markdown caches are untouched. Existing changes to release documentation and engineering-quality specs remain outside this change.
