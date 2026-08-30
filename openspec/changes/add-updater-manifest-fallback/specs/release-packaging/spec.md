## ADDED Requirements

### Requirement: Updater manifest variants SHALL name their own distribution host
For every stable `v*` release, the publication pipeline SHALL produce two updater manifests for the same signed Windows installer: the manifest attached to the GitHub Release SHALL name the GitHub asset URL as the installer download URL, and the manifest mirrored to Aliyun OSS SHALL name the OSS object URL. Both variants SHALL carry the same release `version`, the same `format`, and byte-identical installer `signature` content, because the signed installer is byte-for-byte identical on both hosts. A variant SHALL NOT be published to the other host: the pipeline and the release procedure SHALL verify that each published manifest's installer URL names its own distribution host and that both variants report the tag version.

#### Scenario: GitHub Release manifest names the GitHub asset
- **WHEN** the `prepare-update` job publishes updater metadata for a stable `v*` release
- **THEN** the `update.json` attached to the GitHub Release directs Windows x86_64 clients to the NSIS installer asset on the GitHub Release
- **AND** its signature content matches the `.sig` published beside that installer

#### Scenario: OSS manifest names the OSS object
- **WHEN** the `mirror-oss` job uploads updater metadata for a stable `v*` release
- **THEN** the mirrored `update.json` directs Windows x86_64 clients to the NSIS installer under the OSS `latest/` prefix
- **AND** its signature content is identical to the GitHub manifest's signature content

#### Scenario: Variant swap is detected
- **WHEN** either publication target verifies its updater manifest
- **THEN** it confirms the manifest version equals the tag version and the installer URL names that target's own host
- **AND** a mismatch fails the release verification instead of being published

## MODIFIED Requirements

### Requirement: The app SHALL check GitHub's latest published Release for updates
The application SHALL expose a "Check for Updates…" action in the Help menu that, when invoked, fetches `https://api.github.com/repos/willmove/markion/releases/latest` via the existing in-app HTTP layer, parses the latest published Release, compares its `tag_name` (after the required leading `v`) against `env!("CARGO_PKG_VERSION")` using a semantic-version comparison, and surfaces the result through a modal dialog. When the Release version is newer than the running version, the dialog SHALL display the newer version number without rendering the raw asset URL and SHALL provide localized actionable buttons. On a Windows x86_64 tagged build containing the configured updater public key, the primary action SHALL download the update described by the published updater manifest, reading that manifest from an ordered endpoint list — the Aliyun OSS mirror first and the GitHub Release asset second — and falling through to the next endpoint when an endpoint is unreachable or responds without success, until an endpoint returns a usable manifest or every endpoint has failed. The updater SHALL take the installer URL and signature from the manifest that succeeded, so a fallback path downloads its installer from the same host as its manifest, and SHALL verify the cargo-packager Minisign signature before execution, launch the verified current-user NSIS installer in passive mode, and exit Markion only after the installer starts. Automatic installation SHALL NOT begin while any document has unsaved changes. On macOS, Linux, unsupported architectures, or builds without updater public-key material, the primary action SHALL open the matching GitHub asset URL in the system browser, or the Release page when no platform package mapping exists. When the Release version is equal to or older than the running version, the dialog SHALL report that the application is up to date. When the fetch, manifest, download, signature verification, or installer launch fails, or a supported-platform asset is missing, the application SHALL report the failure without crashing and SHALL retain a manual-download fallback. The application SHALL NOT download or install an update without an explicit user action. The application MAY additionally perform the release check on startup when the `check_for_updates_on_startup` preference is `true` (default `false`), in which case it SHALL be silent unless a newer version is found, and SHALL record the check timestamp in the `last_update_check` preference. Update work SHALL run off the main render path and SHALL NOT recompute any cached-per-version Markdown state.

#### Scenario: User invokes Check for Updates and a newer version is available
- **WHEN** the user chooses "Check for Updates…" on a Windows x86_64 tagged build, the published Release is newer, updater public-key material is present, and all documents are clean
- **THEN** a modal dialog shows the newer version and offers "Download and Install" and "Later" actions without displaying a truncated raw URL
- **AND** choosing "Download and Install" downloads and verifies the signed NSIS installer off the main render path
- **AND** Markion launches the verified installer in passive mode and exits only after the installer starts

#### Scenario: Manifest fetch falls back from OSS to GitHub
- **WHEN** the signed updater cannot reach the OSS manifest endpoint or it responds without a success status
- **THEN** the updater fetches the manifest from the GitHub Release asset endpoint
- **AND** the installer download and signature verification use the installer URL and signature published in the GitHub manifest

#### Scenario: A successful manifest is used end to end on one host
- **WHEN** an updater manifest endpoint returns a usable manifest
- **THEN** the installer is downloaded from the URL named by that manifest, which is hosted on the same distribution channel as the manifest
- **AND** no step of the signed update crosses back to a host that already failed

#### Scenario: User chooses the download fallback
- **WHEN** a newer Release is available on macOS, Linux, an unsupported architecture, or a build without updater public-key material
- **THEN** the modal dialog shows the newer version and offers "Download Update" and "Later" actions without displaying a truncated raw URL
- **AND** choosing "Download Update" opens the matching platform asset, or the Release page when no package mapping exists, in the system browser
- **AND** no file is installed automatically

#### Scenario: Unsaved documents block automatic installation
- **WHEN** the user chooses "Download and Install" while any open document has unsaved changes
- **THEN** Markion does not download, execute, or install the update
- **AND** it asks the user to save their work and retry
- **AND** all open document state remains unchanged

#### Scenario: User invokes Check for Updates and is up to date
- **WHEN** the user chooses "Check for Updates…" and the latest published Release version is equal to or older than `CARGO_PKG_VERSION`
- **THEN** a modal dialog appears reporting that the application is up to date

#### Scenario: Update check fails without crashing
- **WHEN** the GitHub latest-release fetch fails, the response is not valid JSON, the tag cannot be parsed, or the matching supported-platform asset is missing
- **THEN** a modal dialog appears reporting that the update check failed
- **AND** the user can open the latest Release page through the manual-download fallback
- **AND** the application continues running normally

#### Scenario: Signed installation fails safely
- **WHEN** every updater manifest endpoint fails, the manifest is invalid, the download fails, signature verification fails, or the installer cannot be launched
- **THEN** Markion reports that automatic update failed and does not execute an unverified payload
- **AND** the current installation and open documents remain usable
- **AND** the user can open the matching Release asset through the manual-download fallback

#### Scenario: Startup check is opt-in and silent unless newer
- **WHEN** the `check_for_updates_on_startup` preference is `true` and the application starts
- **THEN** the application performs the same GitHub latest-release fetch and version comparison as the menu action
- **AND** no dialog appears unless the GitHub Release version is newer than the running version
- **AND** the timestamp of the check is recorded in the `last_update_check` preference

#### Scenario: Update check does not alter cached Markdown state
- **WHEN** update discovery, download, verification, or installation preparation runs
- **THEN** it executes off the main render path and does not recompute any preview block, outline, stat, syntax-highlighting, or text-handle cache
