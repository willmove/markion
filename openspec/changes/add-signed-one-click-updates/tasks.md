## 1. Actionable Update Prompt

- [x] 1.1 Add localized download/install, download, later, save-first, automatic-update progress, failure, and manual-download messages for every supported language.
- [x] 1.2 Replace the raw-URL update prompt with awaited actions that open the browser fallback or gate signed installation on a clean document set.

## 2. Signed Windows Installation

- [x] 2.1 Add `cargo-packager-updater` as a Windows-only dependency and configure the stable update-manifest endpoint plus compile-time public-key availability.
- [x] 2.2 Run Windows manifest fetch, installer download, Minisign verification, and passive NSIS launch on the background executor, while preserving the running app and offering manual download on recoverable failure.
- [x] 2.3 Add focused unit/source-contract tests for actionable prompt text, updater availability, platform fallback, version comparison, and failure-safe behavior without touching Markdown-derived caches.

## 3. Signed Release Metadata

- [x] 3.1 Extend the tag workflow with a key-validating job that signs the exact Windows NSIS artifact and generates updater-compatible `update.json` using the OSS installer URL.
- [x] 3.2 Publish and mirror `update.json` and the NSIS `.sig` file while keeping GitHub Release and OSS publication independent consumers of the prepared metadata.
- [x] 3.3 Document production signing-key generation, required repository secrets, release verification, rotation expectations, and the distinction between Minisign updater authentication and platform code signing.

## 4. Validation

- [x] 4.1 Run formatting plus focused root-package tests covering the updater and i18n changes.
- [x] 4.2 Run `cargo test --workspace`, validate the OpenSpec change, and confirm every task and release artifact contract is complete.
