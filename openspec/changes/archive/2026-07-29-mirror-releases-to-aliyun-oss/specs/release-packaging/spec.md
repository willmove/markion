## ADDED Requirements

### Requirement: Tagged releases SHALL be mirrored to Aliyun OSS

Upon successful completion of the per-platform `build` jobs for a `v*` tag, the release workflow SHALL run a `mirror-oss` job that downloads the per-platform packaging artifacts, computes a SHA-256 digest for each installer, generates a `manifest.json` describing the release, and uploads the Windows NSIS installer, macOS DMG, Linux DEB, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to a stable `${OSS_PREFIX}/latest/` path on the configured Aliyun OSS Bucket. The OSS endpoint, Bucket name, AccessKey ID, and AccessKey Secret SHALL be supplied from repository secrets and SHALL NOT appear in the repository, the workflow file, or any OpenSpec artifact. The `mirror-oss` job SHALL depend on the `build` jobs and SHALL NOT depend on the `Publish GitHub Release` job; a failure of either job SHALL NOT prevent the other from running. A failure of the `mirror-oss` job SHALL be treated as an incomplete release requiring correction, even if the GitHub Release has already been published. The mirror SHALL overwrite any previous `${OSS_PREFIX}/latest/` objects so that the URL is a stable pointer to the newest release; per-tag history is retained only on GitHub Releases, not on OSS. The mirrored installers SHALL be byte-for-byte copies of the GitHub Release assets and SHALL NOT be code-signed or otherwise modified by the mirror step.

#### Scenario: Tagged release mirrors installers, config, and manifest to OSS
- **WHEN** a `v*` tag is pushed and all three native `build` jobs succeed
- **THEN** the `mirror-oss` job runs, downloads the per-platform packaging artifacts, and uploads the Windows NSIS installer, macOS DMG, Linux DEB, Linux AppImage, `packager.toml`, `manifest.json`, and `sha256sums.txt` to `${OSS_PREFIX}/latest/` on the configured OSS Bucket
- **AND** each file's OSS object key preserves its original filename under the `latest/` prefix

#### Scenario: OSS credentials come from secrets, not the repository
- **WHEN** the `mirror-oss` job runs
- **THEN** the OSS endpoint, Bucket, AccessKey ID, and AccessKey Secret are read from repository secrets (`OSS_ENDPOINT`, `OSS_BUCKET`, `OSS_ACCESS_KEY_ID`, `OSS_ACCESS_KEY_SECRET`)
- **AND** the OSS prefix and public base URL are read from repository secrets (`OSS_PREFIX`, `OSS_PUBLIC_BASE`)
- **AND** no OSS credential appears in the repository, the workflow file, or the OpenSpec change

#### Scenario: Branch pushes and pull requests do not mirror to OSS
- **WHEN** a commit is pushed to `main` or a pull request is opened
- **THEN** the `build` jobs run and upload workflow artifacts, but the `mirror-oss` job does not run

#### Scenario: Mirror upload is independent of GitHub Release publication
- **WHEN** the `build` jobs succeed for a `v*` tag
- **THEN** the `mirror-oss` job and the `Publish GitHub Release` job both run, neither depending on the other
- **AND** a failure of one job does not prevent the other from running to completion

#### Scenario: Mirror failure marks the release incomplete
- **WHEN** the `mirror-oss` job fails for a `v*` tag
- **THEN** the operator does not report the release as complete and corrects or retries the mirror upload, even though the GitHub Release may already be published
- **AND** the public version tag is preserved unless explicit authorization is given for a destructive correction

#### Scenario: Manifest describes the mirrored release
- **WHEN** the `mirror-oss` job generates `manifest.json`
- **THEN** the manifest contains the release version (without the leading `v`), the tag name, an ISO-8601 publication timestamp, and a map from platform identifier (`windows-x86_64`, `macos-aarch64`, `linux-amd64`, `linux-appimage`) to the installer filename
- **AND** the manifest remains available as mirror metadata without being required for the initial in-app update-check verification

### Requirement: The app SHALL check GitHub's latest published Release for updates

The application SHALL expose a "Check for Updates…" action in the Help menu that, when invoked, fetches `https://api.github.com/repos/willmove/markion/releases/latest` via the existing in-app HTTP layer, parses the latest published Release, compares its `tag_name` (after the required leading `v`) against `env!("CARGO_PKG_VERSION")` using a semantic-version comparison, and surfaces the result through a modal dialog. When the Release version is newer than the running version, the dialog SHALL display the newer version number and a link (or copyable URL) from the matching GitHub asset's `browser_download_url` for the user's supported platform and architecture. When the Release version is equal to or older than the running version, the dialog SHALL report that the application is up to date. When the fetch fails, the response cannot be parsed, the tag is invalid, or a supported-platform asset is missing, the dialog SHALL report the failure without crashing the application. The check SHALL NOT download or install the new version automatically; it is notify-only. The application MAY additionally perform this check on startup when the `check_for_updates_on_startup` preference is `true` (default `false`), in which case it SHALL be silent unless a newer version is found, and SHALL record the check timestamp in the `last_update_check` preference. The update check SHALL run off the main render path inside an async task and SHALL NOT recompute any cached-per-version Markdown state.

#### Scenario: User invokes Check for Updates and a newer version is available
- **WHEN** the user chooses "Check for Updates…" from the Help menu and GitHub's latest published Release version is newer than `CARGO_PKG_VERSION`
- **THEN** a modal dialog appears showing the newer version number and the matching GitHub asset URL for the user's platform
- **AND** no file is downloaded or installed

#### Scenario: User invokes Check for Updates and is up to date
- **WHEN** the user chooses "Check for Updates…" from the Help menu and GitHub's latest published Release version is equal to or older than `CARGO_PKG_VERSION`
- **THEN** a modal dialog appears reporting that the application is up to date

#### Scenario: Update check fails without crashing
- **WHEN** the GitHub latest-release fetch fails, the response is not valid JSON, the tag cannot be parsed, or the matching supported-platform asset is missing
- **THEN** a modal dialog appears reporting that the update check failed
- **AND** the application continues running normally

#### Scenario: Startup check is opt-in and silent unless newer
- **WHEN** the `check_for_updates_on_startup` preference is `true` and the application starts
- **THEN** the application performs the same GitHub latest-release fetch and version comparison as the menu action
- **AND** no dialog appears unless the GitHub Release version is newer than the running version
- **AND** the timestamp of the check is recorded in the `last_update_check` preference

#### Scenario: Update check does not alter cached Markdown state
- **WHEN** the update check runs
- **THEN** it executes inside an async task off the main render path and does not recompute any preview block, outline, stat, syntax-highlighting, or text-handle cache
