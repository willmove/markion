## ADDED Requirements

### Requirement: Tagged releases SHALL publish authenticated Windows updater metadata
For every stable `v*` release, the publication pipeline SHALL create a cargo-packager Minisign signature for the Windows x86_64 NSIS installer and an updater manifest containing the release version, the signed installer's public download URL, its signature, and the `nsis` update format. The signature and updater manifest SHALL be attached to the GitHub Release and mirrored beside the installer under the Aliyun OSS `latest/` prefix. The signing private key and password SHALL come from repository secrets, the corresponding public key SHALL be embedded only in tagged application builds, and no private signing material SHALL be written to repository files or build artifacts. Updater signing SHALL be treated as payload authentication and SHALL NOT be represented as Windows Authenticode signing.

#### Scenario: Tagged Windows installer receives updater metadata
- **WHEN** a stable `v*` release packages the Windows x86_64 NSIS installer
- **THEN** the pipeline signs that exact installer with the configured updater private key
- **AND** publishes its `.sig` file and an updater-compatible `update.json` to both the GitHub Release and Aliyun OSS
- **AND** the manifest directs Windows x86_64 clients to the signed installer mirrored on OSS

#### Scenario: Signing secrets are unavailable
- **WHEN** a tagged release cannot read the updater private key, password, or matching public key from repository secrets
- **THEN** signed update metadata publication fails
- **AND** the release is not reported as complete
- **AND** no placeholder key or unsigned automatic-update manifest is published

#### Scenario: Non-tag builds do not publish updater metadata
- **WHEN** a branch push or pull request runs the native build matrix
- **THEN** application compilation remains valid without signing secrets
- **AND** no updater signature or public update manifest is published

## MODIFIED Requirements

### Requirement: The app SHALL check GitHub's latest published Release for updates
The application SHALL expose a "Check for Updates…" action in the Help menu that, when invoked, fetches `https://api.github.com/repos/willmove/markion/releases/latest` via the existing in-app HTTP layer, parses the latest published Release, compares its `tag_name` (after the required leading `v`) against `env!("CARGO_PKG_VERSION")` using a semantic-version comparison, and surfaces the result through a modal dialog. When the Release version is newer than the running version, the dialog SHALL display the newer version number without rendering the raw asset URL and SHALL provide localized actionable buttons. On a Windows x86_64 tagged build containing the configured updater public key, the primary action SHALL download the update described by the published updater manifest, verify the cargo-packager Minisign signature before execution, launch the verified current-user NSIS installer in passive mode, and exit Markion only after the installer starts. Automatic installation SHALL NOT begin while any document has unsaved changes. On macOS, Linux, unsupported architectures, or builds without updater public-key material, the primary action SHALL open the matching GitHub asset URL in the system browser, or the Release page when no platform package mapping exists. When the Release version is equal to or older than the running version, the dialog SHALL report that the application is up to date. When the fetch, manifest, download, signature verification, or installer launch fails, or a supported-platform asset is missing, the application SHALL report the failure without crashing and SHALL retain a manual-download fallback. The application SHALL NOT download or install an update without an explicit user action. The application MAY additionally perform the release check on startup when the `check_for_updates_on_startup` preference is `true` (default `false`), in which case it SHALL be silent unless a newer version is found, and SHALL record the check timestamp in the `last_update_check` preference. Update work SHALL run off the main render path and SHALL NOT recompute any cached-per-version Markdown state.

#### Scenario: User invokes Check for Updates and a newer version is available
- **WHEN** the user chooses "Check for Updates…" on a Windows x86_64 tagged build, the published Release is newer, updater public-key material is present, and all documents are clean
- **THEN** a modal dialog shows the newer version and offers "Download and Install" and "Later" actions without displaying a truncated raw URL
- **AND** choosing "Download and Install" downloads and verifies the signed NSIS installer off the main render path
- **AND** Markion launches the verified installer in passive mode and exits only after the installer starts

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
- **WHEN** the updater manifest is invalid, the download fails, signature verification fails, or the installer cannot be launched
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
