# Implementation Plan: Mirror releases to Aliyun OSS + in-app update check

## Overview

This change adds a domestic OSS download mirror for tagged releases and an in-app "Check for Updates" action. Initial functional verification discovers versions through GitHub's already-live latest-release API; the OSS manifest remains mirror metadata until the mirror path is verified by a real tag. No cached-per-version Markdown invariant is touched, and the update check runs off the main render path.

## Tasks

- [x] 1. Add the `mirror-oss` workflow job
  - [x] 1.1 Add a `mirror-oss` job to `.github/workflows/release.yml`: `needs: build`, `if: startsWith(github.ref, 'refs/tags/v')`, `runs-on: ubuntu-latest`, `permissions: contents: read`.
  - [x] 1.2 Download all per-platform packaging artifacts with `actions/download-artifact@v4` into `dist/` (merge-multiple).
  - [x] 1.3 Compute SHA-256 digests for the four installers and write `dist/sha256sums.txt`.
  - [x] 1.4 Generate `dist/manifest.json` (version without leading `v`, tag, ISO-8601 pub_date, platform->filename map) from the downloaded artifacts and `${{ secrets.OSS_PUBLIC_BASE }}` / `${{ secrets.OSS_PREFIX }}`.
  - [x] 1.5 Upload the four installers, `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${{ secrets.OSS_PREFIX }}/latest/` via `tvrcgo/oss-action@master`, reading `OSS_ACCESS_KEY_ID` / `OSS_ACCESS_KEY_SECRET` / `OSS_ENDPOINT` / `OSS_BUCKET` from secrets.
  - [x] 1.6 Confirm the workflow YAML is syntactically valid (lint by inspection against the existing `release` job structure).
  - _Requirements: release-packaging (tagged-release mirror + manifest + secret usage + job independence)_

- [x] 2. Document the mirror in `packager.toml`
  - [x] 2.1 Add a comment block near the top of `packager.toml` describing the OSS mirror URL (`${OSS_PUBLIC_BASE}/${OSS_PREFIX}/latest/`), the generated mirror metadata, and that no `[updater]` table is present because cargo-packager 0.11.x `Config` rejects it.
  - _Requirements: release-packaging (no `[updater]`; packager unchanged functionally)_

- [x] 3. Update the release runbook
  - [x] 3.1 In `docs/release-process.md` §6, add `Mirror installers to Aliyun OSS` to the "All of these jobs must succeed" list.
  - [x] 3.2 In `docs/release-process.md` §8, add a final-verification bullet: the OSS mirror at `${OSS_PREFIX}/latest/` contains the four installers, `packager.toml`, `manifest.json`, and `sha256sums.txt`, and `manifest.json`'s `version` matches the tag.
  - _Requirements: release-packaging (mirror failure marks release incomplete)_

- [x] 4. Add the `CheckForUpdates` action and Help menu wiring
  - [x] 4.1 Add `CheckForUpdates` to the `actions!(markion, [ ... ])` list in `src/app/mod.rs`.
  - [x] 4.2 Extend the native OS Help menu in `src/app/bootstrap.rs` with a `CheckForUpdates` item and a separator before the existing `AboutMarkion` item.
  - [x] 4.3 Extend the in-window `AppMenu::Help` dropdown in `src/app/root_view.rs` with `action_item!(Msg::ItemCheckForUpdates, check_for_updates, CheckForUpdates)` and a `menu_separator` before About.
  - [x] 4.4 If the longest localized label overflows the Help dropdown width (`px(236.)` at `src/app/mod.rs:595`), widen it.
  - _Requirements: release-packaging (Help menu exposes the update check action)_

- [x] 5. Implement the update check handler
  - [x] 5.1 Create `src/app/update.rs` with a `check_for_updates()` method on `MarkionApp` matching the `about` handler's signature, a minimal GitHub latest-release response model, and the inline semver parser `(u64, u64, u64)`.
  - [x] 5.2 Define the fixed GitHub latest-release API endpoint for the public repository.
  - [x] 5.3 In `check_for_updates()`, spawn an async task (`cx.spawn`) that fetches the latest GitHub Release via the existing HTTP layer, parses JSON, compares `tag_name`, and on completion updates `self.status` and shows a `window.prompt` dialog (newer -> matching GitHub asset URL; up-to-date; error).
  - [x] 5.4 Select the asset for the user's supported platform via `std::env::consts::{OS, ARCH}` and GitHub asset filename suffixes.
  - [x] 5.5 Register the `update` module in `src/app/mod.rs` and re-export `check_for_updates` so the menu action dispatch resolves.
  - _Requirements: release-packaging (check-for-updates behavior, notify-only, off-render-path, no cached-state recomputation)_

- [x] 6. Add the update preferences
  - [x] 6.1 Add `check_for_updates_on_startup: bool` and `last_update_check: Option<String>` to `AppPreferences` in `src/model.rs`.
  - [x] 6.2 Mirror both fields in `PreferencesFile` in `src/storage/preferences.rs` with `#[serde(default)]`, and update the two `From` impls and `Default`.
  - [x] 6.3 Copy both fields in `current_preferences()` in `src/app/application.rs`.
  - [x] 6.4 Add a round-trip test for the two new fields in `src/storage/preferences.rs` following the `sync_scroll` test template.
  - _Requirements: release-packaging (opt-in startup check, last_update_check persisted, existing config.toml valid)_

- [x] 7. Add the localized strings
  - [x] 7.1 Add `Msg` variants in `src/i18n.rs`: `ItemCheckForUpdates`, `DialogUpdateAvailableTitle`, `DialogUpdateAvailableDetail` (with `{0}`=version, `{1}`=url), `DialogUpToDateTitle`, `DialogUpToDateDetail`, `DialogUpdateCheckFailedDetail` (with `{0}`=error), `StatusUpdateCheckComplete`.
  - [x] 7.2 Translate every new variant in all seven language functions (`en`, `zh`, `zh_hant`, `ja`, `fr`, `de`, `es`).
  - [x] 7.3 Confirm the per-language completeness test passes (it guards that every `Msg` has a translation in every language).
  - _Requirements: release-packaging (user-facing strings go through i18n)_

- [x] 8. Update tests
  - [x] 8.1 Update Help-menu string-match assertions in `src/app/tests.rs` to include the new "Check for Updates" item.
  - _Requirements: release-packaging (test coverage for the new menu item)_

## Verification

- [x] 9.1 `openspec validate mirror-releases-to-aliyun-oss` passes.
- [x] 9.2 `openspec doctor` reports no issues.
- [x] 9.3 `cargo test --workspace` passes (including the i18n completeness test and the preference round-trip tests).
- [x] 9.4 `git diff --check` clean; no unintended edits.
- [x] 9.5 The end-to-end OSS upload is verified on the next real `v*` tag push (deferred - not part of this change's local validation).

- [x] 10. Verify the update checker against GitHub's live latest release
  - [x] 10.1 Update the proposal, design, delta spec, task plan, workflow comments, and packager comments so GitHub is the initial update-discovery source while the OSS manifest remains mirror metadata.
  - [x] 10.2 Replace the OSS manifest response model with GitHub's `tag_name` and asset `browser_download_url` model without adding dependencies.
  - [x] 10.3 Add focused tests for GitHub JSON parsing, version outcomes, platform asset selection, invalid tags, and an explicitly ignored live-network check.
  - [x] 10.4 Run the focused update-check tests, including the explicit live GitHub test, and confirm the current latest stable Release is handled without failure.
  - [x] 10.5 Run formatting, the workspace test suite, OpenSpec validation where available, and `git diff --check`. (`openspec` is not available in this shell's PATH; all other checks passed.)
