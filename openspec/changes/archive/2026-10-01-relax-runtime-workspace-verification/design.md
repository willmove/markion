# Design: relax-runtime-workspace-verification

## Context

`verify_bundle()` in `crates/wechat-workspace/src/assets.rs` is a release-grade check: every
manifest-listed file must exist and match its LF-normalized SHA-256 digest, listed HTML/CSS/JS
must reference only local manifest files, and — the failure in the incident — any file on disk that
the manifest does not list rejects the whole directory (`BundleError::UnlistedFile`). Two runtime
call sites reuse it: `discover_workspace_assets()` (candidate acceptance) and
`WorkspaceService::new()` (service construction). The Windows NSIS installer and the in-app
updater install by overwriting and never remove files dropped by earlier versions, so an upgraded
install carries orphans forever and publishing hard-fails on every launch.

## Decision 1: The runtime gate checks manifest + entry shell only

New function in `assets.rs` (working name `verify_launch_gate(root)`) performing exactly:

1. Read and parse `bundle-manifest.json` (`Unavailable` / `ManifestIo` / `InvalidManifest`).
2. `validate_provenance(&manifest)` — catches a wrong or truncated package install; it is
   string-only validation with no disk I/O beyond the manifest itself.
3. Find the `index.html` manifest entry, read the file, LF-normalize (reuse
   `is_text_extension`/`normalize_line_endings`), and compare digests
   (`MissingFile` / `DigestMismatch { path: "index.html" }`).

Rationale for the entry shell: `index.html` is the single file that defines the entire reference
graph. Release verification already proved (at package time) that it references only manifest
files and that all of them match their digests. Pinning its bytes at launch detects the realistic
broken-install modes — truncated update, corrupted extraction, wrong directory — without walking
or hashing the rest of the tree. Return type: keep `Result<BundleVerification, BundleError>` with
`file_count` from the manifest and `total_bytes` of the entry shell, so callers that surface the
bundle revision in status/logs keep working.

Both runtime call sites switch to the gate. `BundleError::UnlistedFile` stays in the enum: it
remains the correct outcome for `verify_bundle()` (CLI, CI, release checks).

## Decision 2: Release strictness is untouched

`verify_bundle()`, the `verify-bundle` binary, the checked-in-workspace test
(`verifies_the_checked_in_workspace`), and every release-workflow invocation stay byte-for-byte
as strict as today. The `release-packaging` requirement already scopes strictness to
"before publication", so no delta is needed there.

## Decision 3: Accepted residual risk (maintainer decision)

With a minimal gate, manifest-listed runtime files other than `index.html` are trusted as
installed and are served by the loopback `/static/` route without a runtime digest check. The
maintainer accepts this: the gate is install sanity, not a security boundary — an attacker with
write access to the install directory can already replace the unsigned `markion.exe`. The
integrity boundary is the release pipeline.

## Alternatives considered and rejected

- **Full verification plus auto-deletion of unlisted files in installer-owned layouts.** Rejected:
  introduces destructive file operations, requires reliable detection of "installer-owned" versus
  developer checkout (cwd / `CARGO_MANIFEST_DIR` candidates must never be pruned), and is
  unnecessary once the gate no longer cares about disk extras.
- **Manifest-based serve allow-list on `/static/`.** Rejected for this change as scope creep; it
  is a compatible future hardening (it would also close the pre-existing verify-then-serve TOCTOU
  window) and needs no spec change made today.
- **Ignore list for OS metadata files (Thumbs.db, desktop.ini, .DS_Store).** Rejected: whack-a-mole
  that would not have fixed the actual incident (63 orphaned KaTeX assets are not OS metadata).

## Testing

- Unit tests on the gate: polluted directory (extra unlisted files, including nested ones) passes;
  tampered `index.html` fails with `DigestMismatch`; missing manifest / invalid provenance fail.
- Incident-shaped regression: build a v0.2.2-style valid bundle, add KaTeX-era leftover files
  (`static/vendor/katex.min.js`, fonts), assert `discover_workspace_assets` + `WorkspaceService`
  construction and session creation succeed.
- Full verifier stays strict: existing `rejects_remote_and_unlisted_runtime_files` test unchanged
  and still passing.
- No app-layer (`src/app/publishing.rs`) changes or new user-facing strings: the existing
  `StatusPublishSetupFailed` path now only fires for genuinely broken installs.
