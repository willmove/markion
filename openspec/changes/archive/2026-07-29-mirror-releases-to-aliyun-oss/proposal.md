## Why

GitHub Releases is the only distribution channel for Markion installers today, and downloads are slow or unreliable for users in regions with poor GitHub connectivity. Meanwhile, the running application has no way to learn that a newer version exists - users must poll the Releases page manually. This change mirrors each tagged release's installers and packaging config to a domestic Aliyun OSS Bucket, and adds an in-app "Check for Updates" action. For the first functional verification, update discovery reads GitHub's latest published Release API, which is already populated and lets the checker be exercised independently of the new OSS publishing job.

## What Changes

- **Edited `.github/workflows/release.yml`** - add a `mirror-oss` workflow job that runs only on `v*` tags, depends on the three native `build` jobs, downloads the per-platform packaging artifacts, computes SHA-256 digests, generates a `manifest.json` describing the release, and uploads the four installers plus `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${OSS_PREFIX}/latest/` on the configured Aliyun OSS Bucket via `tvrcgo/oss-action`. The job is independent of the existing `Publish GitHub Release` job.
- **Edited `packager.toml`** - add a documentation comment block pointing at the OSS mirror URL and explaining why no `[updater]` table is present. No functional field changes.
- **Edited `docs/release-process.md`** - add the `mirror-oss` job to the §6 must-succeed job list and the §8 final-verification checklist.
- **Edited `src/app/` (Rust)** - add a `CheckForUpdates` action, wire a new "Check for Updates…" item into the Help menu (both the native OS menu in `bootstrap.rs` and the in-window dropdown in `root_view.rs`), and implement the handler in a new `src/app/update.rs` module. The handler fetches `https://api.github.com/repos/willmove/markion/releases/latest` via the existing HTTP layer, compares the Release's `tag_name` against `env!("CARGO_PKG_VERSION")`, and surfaces the result through a `window.prompt` dialog: a newer version links the matching GitHub `browser_download_url` for the user's platform; the same or older version reports "up to date"; a network or parse failure reports an error without crashing.
- **Edited `src/i18n.rs`** - add new `Msg` variants for the menu item, dialog titles/details, and status strings, translated across all seven supported languages.
- **Edited `src/storage/preferences.rs` + `src/model.rs`** - add `check_for_updates_on_startup: bool` (default `false`, opt-in - never an unsolicited network call) and `last_update_check: Option<String>` (ISO-8601 timestamp of the most recent manual or startup check) to the preference domain type and its serde-facing file shape. Existing `config.toml` files remain valid via `#[serde(default)]`.
- **Delta to `release-packaging` spec** - add a requirement that tagged releases SHALL be mirrored to Aliyun OSS, and a requirement that the app SHALL check GitHub's latest published Release for updates during initial functional verification.

## Capabilities

### New Capabilities
<!-- No new capabilities. The in-app update check extends the existing chrome-platform and release-packaging capabilities. -->

### Modified Capabilities
- `release-packaging`: tagged releases SHALL additionally be mirrored to Aliyun OSS with a generated manifest, and the app SHALL be able to check GitHub's latest published Release for a newer version.

## Non-goals

- No `[updater]` block in `packager.toml`. cargo-packager 0.11.x (the version CI installs via `cargo install cargo-packager --locked`) has no `updater` field in its `Config` struct, and every config table is `deny_unknown_fields`, so adding one breaks the package step immediately. The manifest and any future signature are produced by the workflow, not the packager.
- No automatic download or install of the new version. "Check for Updates" is notify-only: it tells the user a newer version exists and shows the GitHub Release asset URL. Auto-download-and-install (with signature verification and per-platform installer invocation) is a separate, future change.
- No code-signing or signature verification of the mirrored installers. Builds remain unsigned; the mirror is a byte-for-byte copy of the GitHub Release assets.
- No replacement of GitHub Releases. It remains the source of truth; the OSS mirror is a download-acceleration channel that always reflects the latest tag.
- No versioned `${OSS_PREFIX}/<tag>/` OSS path in this change. Only `${OSS_PREFIX}/latest/` is maintained; per-tag history and the current update-discovery source remain on GitHub Releases.

## Impact

- **CI / infrastructure**: a new `mirror-oss` job runs on every `v*` tag. It consumes four repository secrets already configured by the maintainer (`OSS_ACCESS_KEY_ID`, `OSS_ACCESS_KEY_SECRET`, `OSS_BUCKET`, `OSS_ENDPOINT`) and two additional ones (`OSS_PREFIX`, `OSS_PUBLIC_BASE`). None of these are committed to the repository.
- **New source file**: `src/app/update.rs` - the update-check handler and GitHub Release response model.
- **Edited source**: `src/app/mod.rs` (action registration), `src/app/bootstrap.rs` and `src/app/root_view.rs` (Help menu wiring), `src/app/application.rs` (module registration), `src/app/tests.rs` (Help-menu string-match assertions), `src/i18n.rs` (six new message variants across seven languages), `src/model.rs` and `src/storage/preferences.rs` (two new preference fields).
- **Dependencies**: none added. The update check reuses the existing `zed-reqwest` HTTP client, the existing shared `tokio` runtime, and the existing `MarkionHttpClient`. Semver comparison uses a small inline parser to avoid pulling in `cargo-packager-updater`.
- **Preference schema**: two new fields, both `#[serde(default)]`, so existing `config.toml` files load unchanged.
- **Outward-facing**: a new public download channel on Aliyun OSS, and a new "Check for Updates…" entry in the Help menu.
- **Invariants touched**: none of the cached-per-version Markdown, memoized highlighting, or cached text-handle invariants are affected. The update check runs off the main render path inside an async `cx.spawn` task and only mutates `self.status` / `self.preferences` on completion.
