## 1. Build matrix: updater key in tagged Linux builds

- [x] 1.1 In `.github/workflows/release.yml`, widen "Configure updater public key for tagged builds" from `runner.os == 'Windows'` to Windows **or** Linux (still `v*`-tag-only) and update the step comment (macOS stays keyless/browser-only)

## 2. Update metadata: sign the AppImage, extend update.json

- [x] 2.1 In the `prepare-update` job, sign `dist/markion_${VERSION}_x86_64.AppImage` with `cargo packager signer sign` and assert the `.sig` file is non-empty (mirror the Windows NSIS step)
- [x] 2.2 Verify the AppImage signature with `minisign -Vm` against the decoded public key (verify step generalized to loop over both signed packages)
- [x] 2.3 In "Build update.json", add the `linux-x86_64` platform entry — GitHub AppImage asset URL, the `.AppImage.sig` contents, `format: "appimage"` — leaving the `windows-x86_64` entry byte-identical
- [x] 2.4 Add `dist/markion_${VERSION}_x86_64.AppImage.sig` to the `markion-update-metadata` artifact upload list

## 3. OSS mirror

- [x] 3.1 Generalize "Rewrite update.json installer URLs to the OSS host" to rewrite every `platforms[*].url` to `${base}/<basename-of-url>` (identical logic for NSIS and AppImage), keeping version/signature/format untouched
- [x] 3.2 Add `markion_${VERSION}_x86_64.AppImage.sig` to the OSS upload list and to the reachability-verification list; also added to the mirror-repair `gh release download` patterns
- [x] 3.3 The final `update.json` version-grep still passes, and the verification now additionally asserts both platform URLs point at the OSS host

## 4. In-app signed install on Linux (`src/app/update.rs`)

- [x] 4.1 Extend the primary-action decision: `SignedInstall` when (windows x86_64) **or** (linux x86_64 and the `APPIMAGE` env var is set to a non-empty path) and the public key is embedded; the helper stays pure by taking the env value as a parameter; `current_update_primary_action` reads the real environment
- [x] 4.2 Implement `#[cfg(target_os = "linux")] install_signed_update`: same endpoints/pubkey `Config` (no `WindowsConfig`), `check_update` → `download_and_install` inside `catch_unwind` (parity with the Windows path); `cargo-packager-updater` moved from a Windows-only to a Windows+Linux target dependency in `Cargo.toml`
- [x] 4.3 After a successful replace, `finish_signed_update` spawns the AppImage at the `APPIMAGE` path, persists window bounds/preferences, and quits through `cx.quit()`; if the spawn fails, Markion stays running with a localized "Update installed — restart" dialog (3 new Msg strings × 7 languages in `i18n.rs`)
- [x] 4.4 Non-AppImage Linux builds (deb/rpm, extracted AppDir — no `APPIMAGE`) keep `BrowserDownload`; the existing dirty-document gate still blocks the Linux signed-install path (shared `activate_update_action` code)
- [x] 4.5 (added during implementation) Browser-download fallback for an AppImage-launched Linux install now maps to the `.AppImage` asset on any distro (`compare_with_running_for`/`browser_download_url` gained an `appimage` flag), so a failed signed install on an Ubuntu AppImage offers the AppImage, not the DEB

## 5. Tests

- [x] 5.1 `signed_install_requires_a_self_updatable_package_and_a_public_key` (renamed): linux x86_64 + `APPIMAGE` + key → `SignedInstall`; without `APPIMAGE` / blank / keyless → `BrowserDownload`
- [x] 5.2 `updater_flow_preserves_manual_and_dirty_document_fallbacks` additionally asserts the source gates on `APPIMAGE` and routes success through `finish_signed_update`
- [x] 5.3 `cargo test --bin markion update` (17 tests) and `cargo test --workspace` pass (one pre-existing parallel-isolation flake in `preview_image::repeated_source_collection…` is unrelated — global data-URI counters raced by sibling tests; passes in isolation and on re-run)

## 6. Documentation

- [x] 6.1 `README.md` + `README.zh-CN.md`: update paragraph and limitations bullet — AppImage launches get the signed one-click in-place replace with automatic relaunch; `.deb`/`.rpm` installs keep the browser flow
- [x] 6.2 `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, `README.es.md`: same two spots updated factually (no new sections)
- [x] 6.3 `docs/faq.md`: new "Updating Markion" section (Windows NSIS / Linux AppImage one-click, deb/rpm/macOS browser flow, why DEB cannot self-replace, `libfuse2` first-launch note on Ubuntu 24.04+) and updated platform paragraph
- [x] 6.4 `docs/release-process.md`: signing section retitled to cover both installers, dual-platform `update.json` description, mirror job description + job-list line, final checklist gains the AppImage `.sig`, the `linux-x86_64` manifest entry, both OSS URLs, and a Linux AppImage updater smoke check; `packager.toml` comments updated to match

## 7. Validation

- [x] 7.1 `cargo fmt --check` clean (one i18n line re-wrapped), `cargo test --workspace` green (modulo the pre-existing flake above), workflow YAML parses; `cargo check --target x86_64-unknown-linux-gnu` not possible on this machine (target/system libs unavailable) — the Linux `cfg` code mirrors the Windows implementation against the verified cargo-packager-updater 0.2.3 API (`Config { endpoints, pubkey, windows }`, `check_update`, `download_and_install`), and CI's Linux job compiles it with `-D warnings`
- [x] 7.2 `openspec validate support-linux-in-app-updates --strict` could not run (no `openspec` CLI in this environment); deltas were written directly against the current `openspec/specs/release-packaging/spec.md` requirement text (headings quoted verbatim, both modified requirements reproduced in full)
