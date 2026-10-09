## Why

Linux users can already run the in-app "Check for Updates…" action, but on Linux it only ever opens the browser at the DEB/AppImage asset page — the user must download the package and reinstall by hand. Windows installs already update themselves in place (minisign-verified NSIS launched passively through `cargo-packager-updater`). Linux deserves the same flow wherever it is technically safe.

The technical boundary is the package format, not the OS: `cargo-packager-updater` 0.2.3's Linux installer supports **AppImage only** — it verifies the signature, then replaces the running AppImage file at the path from the `APPIMAGE` environment variable (no privilege escalation, with a backup/rollback on failed write). For `deb`/`rpm` installs it returns `UnsupportedUpdateFormat`, and a DEB-installed binary lives in root-owned `/usr/bin`, so silent self-replacement would require root. Therefore: AppImage installs get full signed in-app self-update; DEB/RPM installs keep today's browser-download fallback. Markion already ships a signed-AppImage-compatible pipeline piece (one minisign keypair, `update.json`, OSS mirror); only the Linux entry is missing.

Ubuntu-family users still default to the DEB download in the asset mapping; this change documents that Linux users who want in-app self-update should use the AppImage (flipping the Ubuntu default to AppImage is a non-goal — see below).

## What Changes

- Tagged Linux builds embed the updater public key (`MARKION_UPDATE_PUBLIC_KEY`), so the signed-install path can activate; untagged and local builds keep the browser-download path.
- The `prepare-update` CI job signs the Linux `markion_<version>_x86_64.AppImage` with the existing `cargo packager signer sign` keypair, verifies the signature with `minisign`, and extends `update.json` with a `linux-x86_64` platform entry (`url`, `signature`, `format: "appimage"`) alongside the existing `windows-x86_64` entry.
- The Aliyun OSS mirror uploads the `.AppImage.sig` beside the installers, and the OSS variant of `update.json` rewrites **every** platform entry's URL to the OSS `latest/` object (generalized from the current Windows-only rewrite), keeping each channel's manifest/installer/signature triple self-consistent.
- In `src/app/update.rs`, the primary update action becomes "Download and Install" on Linux x86_64 tagged builds that are running as an AppImage (`APPIMAGE` env var set by the AppImage runtime). It downloads the manifest-described AppImage, verifies the minisign signature, replaces the running AppImage in place, relaunches the updated AppImage, and quits the current instance. Failure at any step keeps the current installation usable (the updater restores the backup on a failed write) and offers the manual-download fallback. Non-AppImage Linux installs (deb/rpm), macOS, unsupported architectures, and keyless builds keep the browser-download behavior unchanged.
- Documentation (README ×7, `docs/faq.md`, `docs/release-process.md`) explains the Linux update story: AppImage = in-app self-update; DEB/RPM = manual download/reinstall; the deb default for Ubuntu stays.

Non-goals: silent self-update for DEB/RPM installs (root-owned `/usr/bin`; the updater library rejects non-AppImage formats; a `pkexec`/`apt` or "hand the verified .deb to the system installer" flow would be a separate change), flipping the Ubuntu default download asset to the AppImage (fresh Ubuntu 24.04 lacks `libfuse2`, so the AppImage is a worse *first-install* experience than the DEB), macOS in-app installation (we publish `.dmg`, not the `.app.tar.gz` format the updater requires), and any background/silent update without explicit user action.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `release-packaging`: the in-app update check gains a signed-install path for AppImage Linux installs with relaunch-and-quit semantics and a safe rollback; updater metadata publication covers both the Windows NSIS installer and the Linux AppImage (manifest carries `windows-x86_64` + `linux-x86_64` entries; the public key is embedded in tagged Windows and Linux builds; the OSS mirror variant rewrites all platform URLs).

## Impact

- Code: `.github/workflows/release.yml` (public-key gate, `prepare-update` sign/verify/manifest steps, `mirror-oss` upload/verify lists + generalized URL rewrite), `src/app/update.rs` (primary-action gate, Linux `install_signed_update`, relaunch, tests).
- Docs: `README.md`, `README.zh-CN.md`, `README.zh-TW.md`, `README.ja.md`, `README.fr.md`, `README.de.md`, `README.es.md`, `docs/faq.md`, `docs/release-process.md`.
- Secrets: none new — the existing `CARGO_PACKAGER_SIGN_*` keypair now also signs the AppImage.
- Invariants: no Markdown/editor code paths change; update work stays on the existing off-render-path flow; the dirty-document gate still blocks signed installation; Windows behavior is byte-for-byte unchanged.
- Coordination: `improve-macos-distribution` (active) edits the same `mirror-oss` step lists and the same `release-packaging` spec (different requirements); whichever archives second re-syncs the spec — task-level merge in `release.yml` is additive on both sides.
