## MODIFIED Requirements

### Requirement: The app SHALL check GitHub's latest published Release for updates

The application SHALL expose a localized Help → Check for Updates action and SHALL automatically check `https://api.github.com/repos/willmove/markion/releases/latest` at most once per five-hour interval during normal operation, including an overdue check shortly after startup. It SHALL compare the required `v`-prefixed release tag with the running semantic version, persist each check attempt as an RFC 3339 `last_update_check`, tolerate absent, invalid or future timestamps, and prevent concurrent checks, downloads or installations. The legacy `check_for_updates_on_startup` preference SHALL remain readable; when true it SHALL request a startup check even before the normal interval expires. Manual checks SHALL bypass the interval.

On Windows x86_64 with an embedded updater public key and on keyed Linux x86_64 AppImage launches, a newer release SHALL be downloaded and authenticated automatically when `automatically_download_updates` is true (default true). Otherwise the application SHALL ask before downloading. The signed updater SHALL try the Aliyun OSS manifest first, then GitHub, and use the successful manifest's package URL and signature. The manifest version SHALL match the discovered GitHub release. A downloaded package SHALL be verified with the configured Minisign key before becoming ready. Downloading SHALL NOT install, replace application files, quit or restart Markion. A ready package SHALL remain available for the current session without repeated downloading and SHALL be checked for tampering before installation.

When ready, the application SHALL offer localized Restart and Update and Later actions. Later SHALL preserve the ready package; Help → Check for Updates SHALL offer it again. Installation SHALL require explicit user confirmation and SHALL recheck for unsaved documents immediately before handoff. With clean documents, Windows SHALL launch the verified passive current-user NSIS installer with relaunch requested, persist session/layout state and quit only after successful handoff. Linux AppImage SHALL replace the running AppImage safely, relaunch it and quit only after successful replacement and relaunch. Failed AppImage replacement SHALL leave the old AppImage runnable.

On macOS, non-AppImage Linux installations, unsupported architectures or keyless builds, a newer-release notification SHALL retain the matching platform asset browser-download action (or Release-page fallback), without automatic browser opening or unsigned self-installation. Manual checks SHALL report current-version and error results with an actionable browser fallback. Routine background current-version and discovery failures SHALL remain silent; automatic download failures SHALL retain manual recovery without repeated modal interruption. Network, verification and package I/O SHALL run off the render path and SHALL NOT recompute version-cached Markdown state.


On Linux x86_64 browser-download paths, an AppImage launch SHALL retain the AppImage asset. For other launches, the offered asset SHALL follow the distribution family read from `/etc/os-release`: if `ID` or a space-separated `ID_LIKE` token equals `arch`, offer the pacman `.pkg.tar.zst` package; otherwise offer `.deb`, including absent or unparseable os-release. Authenticated AppImage self-updates remain governed by the existing signed-update lifecycle, and pacman downloads remain browser-only.

#### Scenario: User invokes Check for Updates and a newer version is available
- **WHEN** a newer release is discovered on a supported signed-update installation with automatic downloads enabled
- **THEN** Markion downloads and verifies it without asking first, including while documents are dirty
- **AND** no installer or restart begins before explicit confirmation

#### Scenario: User chooses the download fallback
- **WHEN** a newer release is found on a platform without authenticated self-install support
- **THEN** Markion displays the version and offers Download Update and Later without a raw asset URL
- **AND** only selecting Download Update opens the platform asset or Release page in the browser

#### Scenario: Unsaved documents block automatic installation
- **WHEN** the user confirms Restart and Update with any dirty document
- **THEN** Markion asks the user to save and retry, retains the ready update and leaves all documents unchanged

#### Scenario: User invokes Check for Updates and is up to date
- **WHEN** a manual check finds an equal or older release
- **THEN** Markion displays the localized up-to-date dialog

#### Scenario: Update check fails without crashing
- **WHEN** a manual check cannot fetch or parse the release, its version or matching asset
- **THEN** Markion reports a recoverable error and offers the Release-page download fallback

#### Scenario: Signed installation fails safely
- **WHEN** manifest versions disagree, signature verification fails, the staged package changes, or installation cannot start
- **THEN** no unverified package is executed and Markion remains usable with a manual-download fallback

#### Scenario: Startup check is opt-in and silent unless newer
- **WHEN** Markion restarts less than five hours after a check and the legacy startup override is false
- **THEN** automatic discovery waits for the remaining interval
- **AND** an invalid or future timestamp cannot suppress checks indefinitely

#### Scenario: Update check does not alter cached Markdown state
- **WHEN** discovery, downloading, verification or installation preparation runs
- **THEN** preview blocks, outline, stats, syntax highlighting and cached text handles retain their per-version behavior

#### Scenario: Manifest fetch falls back from OSS to GitHub
- **WHEN** OSS cannot supply a usable manifest
- **THEN** the updater tries the GitHub manifest
- **AND** the package URL and signature come from the manifest that succeeded

#### Scenario: A successful manifest is used end to end on one host
- **WHEN** a manifest endpoint succeeds
- **THEN** its package URL and signature are used together without switching to a failed host

#### Scenario: AppImage install updates itself in place
- **WHEN** the user confirms a ready AppImage update with clean documents
- **THEN** Markion replaces the AppImage at its original path, relaunches it and quits the current instance
- **AND** a replacement failure leaves the old AppImage runnable and a relaunch failure keeps the current process running with manual-restart guidance

#### Scenario: Overdue startup and periodic checks
- **WHEN** no valid recent check exists or the five-hour interval expires while Markion is open
- **THEN** a background check starts and its timestamp is persisted
- **AND** no second check or download overlaps an active update operation

#### Scenario: Automatic downloads disabled
- **WHEN** a newer release is found with automatic downloads disabled
- **THEN** Markion offers a download action and waits for the user to select it
- **AND** changing the preference before discovery completes is honored

#### Scenario: Ready update postponed and resumed
- **WHEN** the user chooses Later after a verified download
- **THEN** editing continues and the ready package remains available for this session
- **AND** the next manual check offers Restart and Update without downloading again

#### Scenario: Windows restart handoff
- **WHEN** the user confirms a ready Windows update and all documents are clean
- **THEN** the authenticated NSIS installer starts in passive mode with relaunch requested
- **AND** Markion persists its session/layout and quits after successful handoff

#### Scenario: Manual current-version and error reporting
- **WHEN** the user invokes Check for Updates and the release is not newer or discovery fails
- **THEN** Markion reports up-to-date status or a recoverable failure with manual-download fallback
- **AND** routine background checks for these same outcomes do not display modal dialogs

#### Scenario: Linux download follows the distribution family
- **WHEN** a newer release is offered through the Linux x86_64 browser-download path
- **THEN** an AppImage launch is offered the AppImage asset regardless of distribution
- **AND** a non-AppImage pacman-based system whose os-release ID or ID_LIKE contains arch is offered the pacman `.pkg.tar.zst` asset
- **AND** every other non-AppImage Linux system, including missing or unparseable os-release, is offered the DEB asset
- **AND** the browser opens only after the user chooses Download Update
