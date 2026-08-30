## Why

The update-available dialog currently renders a long GitHub asset URL as plain Windows task-dialog text, so the address is truncated and is neither clickable nor copyable. Markion should provide an actionable download control on every platform and a user-approved, cryptographically verified one-click installation path for the primary Windows NSIS distribution.

## What Changes

- Replace the raw update URL in the native prompt with localized `Download and Install` / `Download Update` and `Later` actions.
- Add signed, user-initiated one-click updates for Windows x86_64 NSIS installations: download the newer installer off the render path, verify its cargo-packager Minisign signature, launch the current-user installer in passive mode, and exit only after the installer starts.
- Keep macOS, Linux, unsupported architectures, and builds without updater key material on an actionable system-browser download fallback; Linux `.deb` and AppImage self-replacement and macOS bundle replacement remain future work.
- Refuse to begin automatic installation while any document has unsaved changes, and surface download, signature, manifest, and installer-launch failures without terminating Markion.
- Extend the tagged-release workflow to sign the Windows installer, generate an updater-compatible `update.json`, attach the signature and manifest to GitHub Releases, and mirror them with the installer to Aliyun OSS.
- Document the required cargo-packager signing secrets and the distinction between updater Minisign verification and platform code signing.
- **Non-goals:** background or forced updates, automatic downgrade/rollback, Windows Authenticode signing, Apple Developer ID signing/notarization, macOS self-replacement, Linux `.deb` or AppImage self-replacement, or changes to cached Markdown-derived state.

## Capabilities

### New Capabilities

<!-- No new capability. Update behavior remains part of release-packaging. -->

### Modified Capabilities

- `release-packaging`: Replace the notify-only raw-URL result with actionable downloads on every supported platform and a signed, consent-driven Windows one-click installation path backed by release-generated updater metadata.

## Impact

- Rust application code: `src/app/update.rs`, localized messages in `src/i18n.rs`, and focused updater tests.
- Dependencies: add `cargo-packager-updater` only for Windows builds.
- Release infrastructure: `.github/workflows/release.yml` gains a signed update-metadata job and publishes/mirrors `update.json` plus the NSIS `.sig` file.
- Operations: tagged releases require `CARGO_PACKAGER_SIGN_PRIVATE_KEY`, `CARGO_PACKAGER_SIGN_PRIVATE_KEY_PASSWORD`, and `CARGO_PACKAGER_SIGN_PUBLIC_KEY` repository secrets.
- Security: the embedded public key authenticates updater payloads; the private key remains outside the repository. Existing unsigned-installer warnings remain unchanged because Minisign is not Authenticode.
- Architecture invariants: update work remains off the GPUI render path and does not read or invalidate per-document preview, outline, statistics, syntax-highlighting, or cached text-handle state.
