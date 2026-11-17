# Proposal: add-updater-manifest-fallback

## Why

The signed Windows updater fetches its `update.json` manifest from a single hardcoded host. When that host is GitHub's release CDN, clients on networks that cannot reach `objects.githubusercontent.com` (common in mainland China even when `api.github.com` works) fail inside `check_update` with "checking the signed update manifest" before any download starts — observed on a real machine updating to v0.2.3. Committing to the OSS mirror alone (current `main`) fixes those clients but makes the updater fully dependent on one CDN and gives overseas clients no path back to GitHub.

## What Changes

- The signed Windows updater SHALL try the update manifest on a list of endpoints in order — Aliyun OSS first, GitHub Release asset second — instead of a single hardcoded URL. `cargo-packager-updater` already falls through to the next endpoint on network failure or non-success status, so this is a client-side endpoint-list change.
- The release pipeline SHALL publish two variants of the updater manifest for the same release: the OSS-mirrored `update.json` whose installer URL points at the OSS object (as today), and the GitHub Release's `update.json` asset whose installer URL points at the GitHub asset. Both variants carry the identical minisign signature string because the signed installer bytes are identical.
- The `mirror-oss` verification and the release procedure's final checks SHALL assert each manifest variant names its own host, so the variants can never be swapped.
- No geo-IP, locale, or region detection is introduced: per-user routing is achieved by endpoint order plus reachability fallback, which self-corrects for VPN, travel, and corporate-network cases.

**Non-goals:** region/IP-based endpoint selection; changes to the initial version check (`api.github.com` latest-release fetch) or the manual browser-download fallback; Linux/macOS automatic updates; updater key rotation; new installer formats.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `release-packaging`: the updater metadata requirement gains the two-variant manifest rule (per-host installer URLs, identical signature, cross-host swap prohibited); the update-check requirement changes the manifest source from a single URL to the ordered OSS-then-GitHub endpoint list with automatic fallback, while keeping the existing signature-verification, dirty-document, and manual-fallback behaviors unchanged.

## Impact

- **Code**: `src/app/update.rs` (manifest URL constants become an ordered endpoint list; test assertions updated). No new dependencies, no i18n changes, no changes to the typing path or cached Markdown state; update work stays on the background executor.
- **Workflow**: `.github/workflows/release.yml` `prepare-update` job (generate the GitHub-asset-URL manifest variant for the Release) and `mirror-oss` job (verify the OSS variant's installer URL host).
- **Docs**: `docs/release-process.md` final-verification checklist.
- **Sequencing**: `add-signed-one-click-updates` (complete, not yet archived) introduces the updater-metadata requirement this change modifies; it should be archived before this change so the delta applies to the synced spec.
