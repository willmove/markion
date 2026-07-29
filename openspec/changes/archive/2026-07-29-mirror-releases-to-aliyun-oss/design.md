## Context

Markion's release pipeline today produces three native installers per `v*` tag (Windows NSIS `.exe`, macOS `.dmg`, Linux `.deb` + `.AppImage`) and attaches them to a GitHub Release. There is no domestic mirror and no in-app awareness of newer versions.

This change adds two capabilities on top of the existing pipeline:

1. A `mirror-oss` GitHub Actions job that copies the tagged release's assets to an Aliyun OSS Bucket under a stable `${OSS_PREFIX}/latest/` prefix, alongside a generated `manifest.json`.
2. An in-app "Check for Updates" action that fetches GitHub's latest published Release and tells the user whether a newer version is available. Using the already-live GitHub endpoint decouples initial checker verification from the first future OSS mirror upload.

The maintainer has already configured the OSS credentials as GitHub repository secrets: `OSS_ACCESS_KEY_ID`, `OSS_ACCESS_KEY_SECRET`, `OSS_BUCKET`, `OSS_ENDPOINT`, `OSS_PREFIX`, `OSS_PUBLIC_BASE`.

A hard external constraint shapes the mirror design: **cargo-packager 0.11.x has no `updater` field in its `Config` struct, and every config table is `deny_unknown_fields`** (verified against docs.rs/cargo-packager/0.11.8). So the packager cannot be asked to sign installers or emit updater manifests; the workflow produces the OSS metadata manifest itself. The initial client check instead consumes GitHub's standard latest-release JSON, avoiding any dependency on the not-yet-published OSS manifest.

The existing app already has a complete HTTP layer - `MarkionHttpClient` (`src/app/network.rs`) wraps `zed-reqwest` over a single shared `tokio::runtime::Runtime` and is registered on the GPUI `App` at startup. The update check reuses this; no new network dependency is introduced.

## Goals / Non-Goals

**Goals:**
- Every tagged release is mirrored to OSS at a stable URL so users with poor GitHub connectivity can download the latest installer quickly.
- The running app can discover a newer version via a user-invoked Help menu action, and (when newer) show the correct GitHub Release asset URL for its platform.
- The mirror is a pure distribution channel - it never replaces GitHub Releases as the source of truth, and never mutates the installers.
- No new crate dependencies; the existing `zed-reqwest` + `tokio` runtime is reused.
- Existing `config.toml` preference files remain valid.

**Non-Goals:**
- No `[updater]` block in `packager.toml` (cargo-packager 0.11.x rejects it).
- No automatic download or install of the new version. "Check for Updates" is notify-only.
- No signature verification of the mirrored installers. They are byte-for-byte copies of the GitHub Release assets; the SHA-256 digests in `sha256sums.txt` are for user auditing, not for in-app verification in this change.
- No versioned `${OSS_PREFIX}/<tag>/` OSS path. Only `latest/` is maintained.
- No background startup check unless the user opts in via `check_for_updates_on_startup` (default `false`).

## Decisions

### D1. OSS object layout: `${OSS_PREFIX}/latest/` only

The mirror holds exactly one copy of each asset, under `${OSS_PREFIX}/latest/<filename>`. The `mirror-oss` job overwrites these objects on every tag push. Per-tag history is preserved on GitHub Releases; OSS holds only the newest mirror to keep storage cost minimal and expose a single, stable mirror URL.

- **Alternative considered**: mirror to both `${OSS_PREFIX}/latest/` and `${OSS_PREFIX}/<tag>/`. Rejected for this change: doubles storage and upload time. A future change can add per-tag archiving if a "rollback to a specific version" UX is wanted.

### D2. Manifest format: a single `manifest.json` (not `latest.yml`)

The workflow generates `manifest.json` at `${OSS_PREFIX}/latest/manifest.json`:

```json
{
  "version": "0.1.13",
  "tag": "v0.1.13",
  "pub_date": "2026-07-27T10:30:00Z",
  "assets": {
    "windows-x86_64": { "filename": "markion_0.1.13_x64-setup.exe" },
    "macos-aarch64":  { "filename": "Markion_0.1.13_aarch64.dmg" },
    "linux-amd64":    { "filename": "markion_0.1.13_amd64.deb" },
    "linux-appimage": { "filename": "markion_0.1.13_x86_64.AppImage" }
  }
}
```

The manifest is retained as self-contained mirror metadata and a possible input for a future mirror-backed updater. The initial checker does not consume it; GitHub's latest-release response already supplies `tag_name`, each asset `name`, and each asset's `browser_download_url`.

- **Alternative considered**: emit Tauri-style `latest.yml` / `latest-mac.yml` / `latest-linux.yml` and use `cargo-packager-updater` on the client. Rejected: (a) cargo-packager 0.11.x does not emit these, so the workflow would have to synthesize them anyway; (b) `cargo-packager-updater` expects signed assets and would force a signing scheme onto this change; (c) a single JSON manifest is simpler to produce and parse than three YAML files.

### D3. Initial update discovery uses GitHub's latest-release API

The checker uses a source constant baked into the binary:

```rust
const GITHUB_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/willmove/markion/releases/latest";
```

The response's `tag_name` is the version source, and `assets[].browser_download_url` is the notify-only download link. The existing HTTP helper supplies a Markion User-Agent, which GitHub accepts for unauthenticated public-repository requests. Keeping the endpoint non-configurable prevents preference drift. Moving discovery to the OSS manifest remains a follow-up after its publication path has been verified by a real tag.

### D4. Semver comparison: tiny inline parser, no new dep

A ~30-line `parse_semver(major, minor, patch)` that handles the `MAJOR.MINOR.PATCH` shape Markion actually publishes (no pre-release/build metadata yet). It returns an `(u64, u64, u64)` tuple; comparison is lexicographic on the tuple.

- **Alternative considered**: pull in `cargo-packager-updater::semver::Version` or the `semver` crate. Rejected for this change: both add a dependency for a comparison that, given Markion's strict `vX.Y.Z` tagging, is trivial. If pre-release tags are introduced later, swap the inline parser for the `semver` crate then.

### D5. Update check runs off the render path via `cx.spawn`

The `check_for_updates()` handler spawns an async task that fetches the GitHub Release JSON, parses it, compares versions, and then - back on the app context - sets `self.status` and shows a `window.prompt` dialog. This mirrors the existing async pattern in `src/app/application.rs:298-324` and the `about` dialog pattern in `src/app/search.rs:237-253`. The render path is never blocked, and the cached-per-version Markdown invariants are untouched.

### D6. Preferences: opt-in startup check, default off

Two new fields on `AppPreferences`:

- `check_for_updates_on_startup: bool` - default `false`. When `true`, `MarkionApp::new` schedules the same check the menu action runs, but silently (status-bar only, no dialog) unless a newer version is found.
- `last_update_check: Option<String>` - ISO-8601 timestamp of the most recent check (manual or startup). Surfaced in a future Preferences panel; for now it is just persisted so a startup check can throttle itself.

Both fields are `#[serde(default)]` on `PreferencesFile`, so existing `config.toml` files load with the defaults and remain valid.

### D7. `tvrcgo/oss-action` for the upload step

The `mirror-oss` job uses `tvrcgo/oss-action@master`, whose `assets:` input maps `local:remote` pairs. This matches the style of the existing `softprops/action-gh-release` step and keeps the workflow declarative.

- **Alternative considered**: `aliyun-cli` with a shell loop. Rejected: more code, more failure modes, no benefit at four assets.
- **Pinning**: `@master` is used for now; pinning to a commit SHA is a follow-up hardening task, recorded but not in scope here.

## Risks / Trade-offs

- **[Risk] OSS credentials leak** -> All six OSS values are read from `${{ secrets.* }}` in the workflow; none is committed. The `mirror-oss` job requests only `contents: read`. Recommend (out of band) a RAM sub-account scoped to this one bucket.
- **[Risk] `latest/` overwrite loses the previous version on OSS** -> Accepted: GitHub Releases retains every tagged version; OSS is explicitly a "current" pointer, not an archive.
- **[Risk] Manifest drift from actual assets** -> The `manifest.json` is generated in the same job step that uploads the assets, from the same `dist/` directory, so the filenames in the manifest always match the uploaded files. The SHA-256 digests in `sha256sums.txt` give users an independent audit path.
- **[Risk] GitHub API is rate-limited or unreachable** -> The checker makes one unauthenticated request only when invoked and reports any HTTP failure without crashing. Startup checking remains opt-in.
- **[Risk] Notify-only update check frustrates users who expect auto-install** -> The dialog shows the GitHub asset URL for the supported platform. Full auto-install is an explicit future change.
- **[Trade-off] Inline semver parser** -> Will not handle pre-release/build metadata. Acceptable while Markion tags are strict `vX.Y.Z`; swap to the `semver` crate if that changes.
- **[Trade-off] GitHub remains the discovery dependency during initial verification** -> This does not yet exercise OSS discovery, but it verifies the fetch/parse/compare/dialog path against an already-live authoritative release source before changing channels.

## Migration Plan

- No persisted-data migration. Existing `config.toml` files gain two defaulted fields on next load.
- No code-signing changes; builds remain unsigned.
- First `v*` tag pushed after merge exercises the `mirror-oss` job end-to-end. If it fails, GitHub Releases is unaffected (the two jobs are independent), and the release runbook's §6/§8 additions tell the operator to treat a mirror failure as an incomplete release.

## Open Questions

- Should the startup check (when `check_for_updates_on_startup` is on) throttle to at most once per 24h using `last_update_check`? **Lean: yes, but implement as a simple "skip if checked within 24h" guard in the startup path.** Decided during apply; if it adds complexity, ship the menu-only check first and add the startup path in a follow-up.
- Whether to widen the Help dropdown (`px(236.)` at `src/app/mod.rs:595`) for the new localized label. Decided during apply by measuring the longest label across the seven languages.
