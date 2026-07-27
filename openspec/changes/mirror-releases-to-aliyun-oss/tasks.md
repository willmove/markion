# Implementation Plan: Mirror releases to Aliyun OSS + in-app update check

## Overview

This change adds a domestic OSS download mirror for tagged releases and an in-app "Check for Updates" action that reads a manifest from that mirror. It is config- and additive-source only: the build/packaging pipeline is unchanged, no cached-per-version Markdown invariant is touched, and the update check runs off the main render path. The OpenSpec change scaffolding (proposal, design, spec delta) is already complete; the tasks below implement it.

## Tasks

- [ ] 1. Add the `mirror-oss` workflow job
  - [ ] 1.1 Add a `mirror-oss` job to `.github/workflows/release.yml`: `needs: build`, `if: startsWith(github.ref, 'refs/tags/v')`, `runs-on: ubuntu-latest`, `permissions: contents: read`.
  - [ ] 1.2 Download all per-platform packaging artifacts with `actions/download-artifact@v4` into `dist/` (merge-multiple).
  - [ ] 1.3 Compute SHA-256 digests for the four installers and write `dist/sha256sums.txt`.
  - [ ] 1.4 Generate `dist/manifest.json` (version without leading `v`, tag, ISO-8601 pub_date, platform->filename map) from the downloaded artifacts and `${{ secrets.OSS_PUBLIC_BASE }}` / `${{ secrets.OSS_PREFIX }}`.
  - [ ] 1.5 Upload the four installers, `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${{ secrets.OSS_PREFIX }}/latest/` via `tvrcgo/oss-action@master`, reading `OSS_ACCESS_KEY_ID` / `OSS_ACCESS_KEY_SECRET` / `OSS_ENDPOINT` / `OSS_BUCKET` from secrets.
  - [ ] 1.6 Confirm the workflow YAML is syntactically valid (lint by inspection against the existing `release` job structure).
  - _Requirements: release-packaging (tagged-release mirror + manifest + secret usage + job independence)_

- [ ] 2. Document the mirror in `packager.toml`
  - [ ] 2.1 Add a comment block near the top of `packager.toml` describing the OSS mirror URL (`${OSS_PUBLIC_BASE}/${OSS_PREFIX}/latest/`), that the app's update check fetches `manifest.json` from there, and that no `[updater]` table is present because cargo-packager 0.11.x `Config` rejects it.
  - _Requirements: release-packaging (no `[updater]`; packager unchanged functionally)_

- [ ] 3. Update the release runbook
  - [ ] 3.1 In `docs/release-process.md` §6, add `Mirror installers to Aliyun OSS` to the "All of these jobs must succeed" list.
  - [ ] 3.2 In `docs/release-process.md` §8, add a final-verification bullet: the OSS mirror at `${OSS_PREFIX}/latest/` contains the four installers, `packager.toml`, `manifest.json`, and `sha256sums.txt`, and `manifest.json`'s `version` matches the tag.
  - _Requirements: release-packaging (mirror failure marks release incomplete)_

- [ ] 4. Add the `CheckForUpdates` action and Help menu wiring
  - [ ] 4.1 Add `CheckForUpdates` to the `actions!(markion, [ ... ])` list in `src/app/mod.rs`.
  - [ ] 4.2 Extend the native OS Help menu in `src/app/bootstrap.rs` with a `CheckForUpdates` item and a separator before the existing `AboutMarkion` item.
  - [ ] 4.3 Extend the in-window `AppMenu::Help` dropdown in `src/app/root_view.rs` with `action_item!(Msg::ItemCheckForUpdates, check_for_updates, CheckForUpdates)` and a `menu_separator` before About.
  - [ ] 4.4 If the longest localized label overflows the Help dropdown width (`px(236.)` at `src/app/mod.rs:595`), widen it.
  - _Requirements: release-packaging (Help menu exposes the update check action)_

- [ ] 5. Implement the update check handler
  - [ ] 5.1 Create `src/app/update.rs` with a `check_for_updates()` method on `MarkionApp` matching the `about` handler's signature, and a `UpdateManifest` struct (version, tag, pub_date, assets map) plus the inline semver parser `(u64, u64, u64)`.
  - [ ] 5.2 Define compile-time constants `OSS_PUBLIC_BASE` and `OSS_PREFIX` via `option_env!` with documented defaults.
  - [ ] 5.3 In `check_for_updates()`, spawn an async task (`cx.spawn`) that fetches `${OSS_PUBLIC_BASE}/${OSS_PREFIX}/latest/manifest.json` via the existing `MarkionHttpClient` / `fetch_url_bytes`, parses JSON, compares versions, and on completion updates `self.status` and shows a `window.prompt` dialog (newer -> link OSS asset URL for the detected platform; up-to-date; error).
  - [ ] 5.4 Select the asset for the user's platform via `std::env::consts::{OS, ARCH}` mapped to the manifest's platform keys (`windows-x86_64`, `macos-aarch64`, `linux-amd64`, `linux-appimage`).
  - [ ] 5.5 Register the `update` module in `src/app/mod.rs` and re-export `check_for_updates` so the menu action dispatch resolves.
  - _Requirements: release-packaging (check-for-updates behavior, notify-only, off-render-path, no cached-state recomputation)_

- [ ] 6. Add the update preferences
  - [ ] 6.1 Add `check_for_updates_on_startup: bool` and `last_update_check: Option<String>` to `AppPreferences` in `src/model.rs`.
  - [ ] 6.2 Mirror both fields in `PreferencesFile` in `src/storage/preferences.rs` with `#[serde(default)]`, and update the two `From` impls and `Default`.
  - [ ] 6.3 Copy both fields in `current_preferences()` in `src/app/application.rs`.
  - [ ] 6.4 Add a round-trip test for the two new fields in `src/storage/preferences.rs` following the `sync_scroll` test template.
  - _Requirements: release-packaging (opt-in startup check, last_update_check persisted, existing config.toml valid)_

- [ ] 7. Add the localized strings
  - [ ] 7.1 Add `Msg` variants in `src/i18n.rs`: `ItemCheckForUpdates`, `DialogUpdateAvailableTitle`, `DialogUpdateAvailableDetail` (with `{0}`=version, `{1}`=url), `DialogUpToDateTitle`, `DialogUpToDateDetail`, `DialogUpdateCheckFailedDetail` (with `{0}`=error), `StatusUpdateCheckComplete`.
  - [ ] 7.2 Translate every new variant in all seven language functions (`en`, `zh`, `zh_hant`, `ja`, `fr`, `de`, `es`).
  - [ ] 7.3 Confirm the per-language completeness test passes (it guards that every `Msg` has a translation in every language).
  - _Requirements: release-packaging (user-facing strings go through i18n)_

- [ ] 8. Update tests
  - [ ] 8.1 Update Help-menu string-match assertions in `src/app/tests.rs` to include the new "Check for Updates" item.
  - _Requirements: release-packaging (test coverage for the new menu item)_

## Verification

- [ ] 9.1 `openspec validate mirror-releases-to-aliyun-oss` passes.
- [ ] 9.2 `openspec doctor` reports no issues.
- [ ] 9.3 `cargo test --workspace` passes (including the i18n completeness test and the preference round-trip tests).
- [ ] 9.4 `git diff --check` clean; no unintended edits.
- [ ] 9.5 The end-to-end OSS upload is verified on the next real `v*` tag push (deferred - not part of this change's local validation).
