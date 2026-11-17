# Design — Linux AppImage in-app updates

## Context

- `src/app/update.rs` already implements the full dialog/flow. `update_primary_action_for(os, arch, public_key)` gates the signed-install path to `("windows", "x86_64")` + embedded key; everything else opens the browser. `install_signed_update()` is `#[cfg(windows)]`-real / otherwise a stub error.
- `cargo-packager-updater` 0.2.3 (crabnebula-dev/cargo-packager, tag `@crabnebula/updater-v0.2.3`, `crates/updater/src/lib.rs`):
  - Linux `install_inner` accepts **only** `UpdateFormat::AppImage`; other formats → `Error::UnsupportedUpdateFormat`. No privilege escalation exists anywhere on the Linux path.
  - `extract_path` on Linux defaults to `$APPIMAGE` (set by the AppImage runtime), falling back to `current_exe()`.
  - Install = rename current AppImage to a same-filesystem temp backup, write the verified bytes to the original path, restore permissions; on write failure it renames the backup back (rollback). It does **not** relaunch the app.
- `update.json` (release.yml `prepare-update` job) currently carries a single `windows-x86_64` entry `{url, signature, format: "nsis"}`; the OSS mirror job rewrites exactly that entry's URL. The manifest format is the cargo-packager/Tauri `platforms` map; the updater looks up `"{target}-{arch}"` = `linux-x86_64`.
- The updater public key is embedded at build time via `MARKION_UPDATE_PUBLIC_KEY`, gated in CI to `runner.os == 'Windows' && tag`.

## Decisions

1. **AppImage-only signed install, detected via the `APPIMAGE` env var.** This is exactly the signal the updater library itself uses to pick its replace target, so the UI can never offer an install the library would refuse: extracted AppDir runs and deb/rpm installs (no `APPIMAGE`) fall back to browser download. The gate helper stays pure (`update_primary_action_for` takes the env value as a parameter) so tests stay hermetic.

2. **One manifest, two platforms.** `update.json` gains `linux-x86_64: {url: <GitHub AppImage asset>, signature: <.AppImage.sig contents>, format: "appimage"}`. The updater selects its own platform entry and ignores the other, so one manifest serves all clients. The OSS variant's rewrite step is generalized from "rewrite `platforms["windows-x86_64"].url`" to "rewrite every `platforms[*].url` to `${OSS_BASE}/latest/<basename>`" — signatures stay identical because both hosts serve the same signed files.

3. **Relaunch-and-quit after in-place replace.** After `download_and_install()` succeeds, the running process is still the old binary (now on a deleted inode); the on-disk AppImage is new. Markion spawns the AppImage at the original `$APPIMAGE` path and then quits through the existing app-quit path. The dirty-document gate (already shared with the Windows flow) guarantees no unsaved state exists at that point. If spawning the new AppImage fails, Markion stays running and shows the manual fallback — the update is already installed, so the message points at the Release page for verification, not re-download.

4. **Panic-safety parity.** The Windows path wraps `download_and_install` in `catch_unwind` because the library `expect`s in places. The Linux path reuses the same wrapper and error-string conventions (`show_update_install_failure` → manual-download fallback).

5. **Public key embedding widened to tagged Linux builds.** CI gate becomes `(Windows || Linux) && v*-tag`. Untagged/local macOS/Linux/Windows builds stay keyless → browser fallback, unchanged.

6. **Ubuntu keeps the DEB default.** `linux_wants_appimage()` (pacman-family → AppImage, else DEB) is a *browser-download asset* mapping and is untouched. Fresh Ubuntu 24.04 lacks `libfuse2`, so making the AppImage the default first-download would trade an install friction bug for an update convenience. Docs instead explain: want self-update on Linux → use the AppImage.

## Risks / notes

- Both `update.json` variants must exist for a release to be considered published; the `prepare-update` job already fails on any signing gap, and the mirror job's reachability check now also covers `.AppImage.sig`.
- `improve-macos-distribution` (active change) extends the same `mirror-oss` upload/verify lists and matrix; edits are additive lists on both sides, and the spec deltas touch disjoint requirements, so archive order does not conflict.
- The AppImage signature is over the whole file, so the AppImage must be signed *after* packaging completes and before upload — the `prepare-update` job already runs post-`build`, and it downloads the packaged artifacts, which satisfies this ordering.
- Local verification of the Linux binary path is limited on macOS dev machines (`cfg(target_os = "linux")` code compiles only for a Linux target); CI's Linux job compiles and runs the unit tests, and `cargo check --target x86_64-unknown-linux-gnu` can catch cfg mistakes if the toolchain is installed.
