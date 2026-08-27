# Proposal: relax-runtime-workspace-verification

## Why

In-place upgrades of the Windows NSIS package (including the in-app updater) overwrite the
packaged MarkNice workspace but never delete files that the new package no longer ships. A real
v0.1.24 → v0.2.2 upgrade left 63 orphaned KaTeX files next to the 21-file MathJax-era manifest, so
every "Publish to WeChat (MarkNice)" click failed with `the local publishing workspace contains an
unlisted runtime file` and the browser never opened. The runtime launch path re-runs the
release-grade full-bundle verifier, turning benign installer leftovers on an upgradable install
into a hard feature failure for every affected user.

## What Changes

- Introduce a **minimal runtime launch gate** for the publishing workspace: the manifest must
  parse, its provenance must be valid, and the entry shell (`index.html`) must match its recorded
  LF-normalized digest. Nothing else is checked at launch time.
- `discover_workspace_assets()` and `WorkspaceService::new()` use the minimal gate, so files on
  disk that are absent from the manifest (upgrade leftovers, OS metadata) no longer block
  publishing.
- Keep `verify_bundle()` (full digests, unlisted-file rejection, remote-dependency and prohibited
  artifact scans) exactly as strict as today for the release pipeline and the `verify-bundle`
  maintainer CLI.
- Add regression tests: a workspace polluted with removed-from-bundle files must still launch;
  a tampered or missing `index.html` must still fail setup.
- No file deletion, quarantine, or installer changes are introduced.

**Non-goals:** auto-deleting or quarantining leftover files, changing NSIS/updater packaging
behavior, adding a manifest-based serve allow-list to the loopback `/static/` route, and any
change to release-time verification strictness.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `wechat-publishing-workspace`: Adds a requirement that launching the workspace performs only the
  minimal runtime gate and tolerates non-manifest files on disk. (This capability is currently
  pending sync from the unarchived `add-local-marknice-publishing-workspace` change; this change's
  delta is stacked on it and must be archived after it.)

## Impact

- **Code:** `crates/wechat-workspace/src/assets.rs` (new gate function, discovery rewiring),
  `crates/wechat-workspace/src/server.rs` (`WorkspaceService::new` calls the gate),
  `crates/wechat-workspace/src/lib.rs` (exports). The GPUI app layer in `src/app/publishing.rs`
  is untouched — the same setup-failure status path remains for genuinely broken installs.
- **Specified behavior:** Runtime launching no longer enforces full-bundle equality; release
  verification (`release-packaging` requirement "Packaged local WeChat publishing workspace") is
  unchanged, because its SHALL already scopes strictness to before publication.
- **Security posture:** Deliberate maintainer decision — the runtime gate is install sanity, not a
  security boundary; an attacker with write access to the install directory can already replace
  the unsigned application binary. Release verification remains the integrity boundary.
- **Invariants:** No document-model, caching, or typing-path invariants are touched (publishing
  still consumes an immutable snapshot). The checked-in workspace must still pass full
  `verify_bundle()` in tests and CI.
