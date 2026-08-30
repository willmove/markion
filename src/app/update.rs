//! In-app "Check for Updates" action.
//!
//! Fetches the repository's latest published GitHub Release through the GitHub
//! REST API, compares its `tag_name` against `env!("CARGO_PKG_VERSION")`, and
//! surfaces the result through an actionable modal dialog:
//!
//! - newer version -> dialog offers signed install or browser download;
//! - same/older version -> "up to date";
//! - fetch or parse failure -> error dialog (no crash).
//!
//! Windows x86_64 tagged builds with an embedded updater public key can, after
//! explicit user confirmation, download a signed NSIS installer, verify its
//! cargo-packager Minisign signature, launch it in passive mode, and exit.
//! Other builds open the matching Release asset in the system browser. All
//! network and installer work runs off the render path and never touches
//! cached-per-version Markdown state.

use super::*;
use anyhow::{Context as _, Result, anyhow};
use gpui::AnyWindowHandle;
use serde::Deserialize;
use serde_json;
use std::env::consts;

/// GitHub's unauthenticated "Get the latest release" endpoint. GitHub Releases
/// remains the source of truth, and the shared HTTP helper supplies the
/// required Markion User-Agent header.
const GITHUB_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/willmove/markion/releases/latest";

/// Human-facing latest Release page used when discovery or an exact platform
/// asset lookup fails. Keeping this separate from the API URL gives every
/// recoverable failure an actionable browser fallback.
const GITHUB_LATEST_RELEASE_URL: &str = "https://github.com/willmove/markion/releases/latest";

/// Signed update manifest endpoints in fallback order. The release workflow
/// publishes one manifest per host — the Aliyun OSS mirror and the GitHub
/// Release — and each manifest names the signed installer on its own host
/// with the identical minisign signature, so whichever endpoint succeeds can
/// serve the whole update from a single channel. OSS goes first: Markion's
/// mirror exists for networks where GitHub's release CDN
/// (`objects.githubusercontent.com`) is unreachable even though
/// `api.github.com` works, and cargo-packager-updater falls through to the
/// next endpoint on a network failure or non-success status. GitHub's API
/// remains the version authority for the initial update check.
#[cfg_attr(not(windows), allow(dead_code))]
const OSS_SIGNED_UPDATE_MANIFEST_URL: &str =
    "https://marknice.oss-cn-heyuan.aliyuncs.com/markion-releases/latest/update.json";

/// GitHub-hosted fallback manifest for clients that cannot reach the OSS
/// mirror at all.
#[cfg_attr(not(windows), allow(dead_code))]
const GITHUB_SIGNED_UPDATE_MANIFEST_URL: &str =
    "https://github.com/willmove/markion/releases/latest/download/update.json";

/// Ordered manifest endpoints consumed by the signed updater and asserted by
/// the unit tests. Non-Windows builds keep these reachable for the tests
/// without tripping the dead-code lint that CI's `-D warnings` rejects.
#[cfg_attr(not(windows), allow(dead_code))]
fn signed_update_manifest_endpoints() -> [&'static str; 2] {
    [
        OSS_SIGNED_UPDATE_MANIFEST_URL,
        GITHUB_SIGNED_UPDATE_MANIFEST_URL,
    ]
}

/// Subset of GitHub's release response used by the update checker. Serde
/// ignores the API's other fields, keeping this model resilient to additions.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

/// Outcome of a single update check. Built off the UI thread, then applied on
/// the app context to set `status` and show a `window.prompt` dialog.
#[derive(Debug)]
enum UpdateCheckOutcome {
    UpToDate,
    Available { version: String, url: String },
    Failed { error: String, manual_url: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdatePrimaryAction {
    SignedInstall,
    BrowserDownload,
}

fn configured_update_public_key() -> Option<&'static str> {
    option_env!("MARKION_UPDATE_PUBLIC_KEY")
        .map(str::trim)
        .filter(|key| !key.is_empty())
}

fn update_primary_action_for(
    os: &str,
    arch: &str,
    public_key: Option<&str>,
) -> UpdatePrimaryAction {
    if os == "windows" && arch == "x86_64" && public_key.is_some_and(|key| !key.trim().is_empty()) {
        UpdatePrimaryAction::SignedInstall
    } else {
        UpdatePrimaryAction::BrowserDownload
    }
}

fn current_update_primary_action() -> UpdatePrimaryAction {
    update_primary_action_for(consts::OS, consts::ARCH, configured_update_public_key())
}

impl MarkionApp {
    pub(super) fn check_for_updates(
        &mut self,
        _: &CheckForUpdates,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Capture the window handle so the dialog can be shown after the async
        // fetch resolves, mirroring the pattern in `editing.rs` (`quit_confirm`).
        let window_handle = window.window_handle();
        self.status = self.tr(Msg::StatusUpdateChecking).into();
        self.active_menu = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let outcome = cx
                .background_executor()
                .spawn(async move {
                    match fetch_latest_release(GITHUB_LATEST_RELEASE_API_URL) {
                        Ok(release) => compare_with_running(&release),
                        Err(err) => UpdateCheckOutcome::Failed {
                            error: err.to_string(),
                            manual_url: GITHUB_LATEST_RELEASE_URL.to_string(),
                        },
                    }
                })
                .await;

            let _ = this.update(cx, |app, cx| {
                match &outcome {
                    UpdateCheckOutcome::UpToDate => {
                        app.status = t(app.language, Msg::StatusUpdateUpToDate).into();
                        let language = app.language;
                        let _ = window_handle.update(cx, |_, window, cx| {
                            std::mem::drop(window.prompt(
                                PromptLevel::Info,
                                t(language, Msg::DialogUpToDateTitle),
                                Some(t(language, Msg::DialogUpToDateDetail)),
                                &[PromptButton::ok(t(language, Msg::DialogButtonOk))],
                                cx,
                            ));
                        });
                    }
                    UpdateCheckOutcome::Available { version, url } => {
                        app.status =
                            tf(app.language, Msg::StatusUpdateAvailable, &[version]).into();
                        app.prompt_update_available(
                            version.clone(),
                            url.clone(),
                            window_handle,
                            cx,
                        );
                    }
                    UpdateCheckOutcome::Failed { error, manual_url } => app
                        .show_update_check_failure(
                            error.clone(),
                            manual_url.clone(),
                            window_handle,
                            cx,
                        ),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show_update_check_failure(
        &mut self,
        error: String,
        manual_url: String,
        window_handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        self.status = tf(self.language, Msg::StatusUpdateCheckFailed, &[&error]).into();
        cx.notify();

        let language = self.language;
        let detail = tf(language, Msg::DialogUpdateCheckFailedDetail, &[&error]);
        let Ok(answer) = window_handle.update(cx, |_, window, cx| {
            window.prompt(
                PromptLevel::Warning,
                t(language, Msg::DialogUpdateCheckFailedTitle),
                Some(&detail),
                &[
                    PromptButton::ok(t(language, Msg::DialogButtonDownloadManually)),
                    PromptButton::cancel(t(language, Msg::DialogButtonLater)),
                ],
                cx,
            )
        }) else {
            return;
        };

        cx.spawn(async move |this, cx| {
            if matches!(answer.await, Ok(0)) {
                let _ = this.update(cx, |_, cx| cx.open_url(&manual_url));
            }
        })
        .detach();
    }

    fn prompt_update_available(
        &mut self,
        version: String,
        url: String,
        window_handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        let action = current_update_primary_action();
        let language = self.language;
        let detail = tf(language, Msg::DialogUpdateAvailableDetail, &[&version]);
        let primary_label = match action {
            UpdatePrimaryAction::SignedInstall => t(language, Msg::DialogButtonDownloadAndInstall),
            UpdatePrimaryAction::BrowserDownload => t(language, Msg::DialogButtonDownloadUpdate),
        };
        let Ok(answer) = window_handle.update(cx, |_, window, cx| {
            window.prompt(
                PromptLevel::Info,
                t(language, Msg::DialogUpdateAvailableTitle),
                Some(&detail),
                &[
                    PromptButton::ok(primary_label),
                    PromptButton::cancel(t(language, Msg::DialogButtonLater)),
                ],
                cx,
            )
        }) else {
            return;
        };

        cx.spawn(async move |this, cx| {
            if !matches!(answer.await, Ok(0)) {
                return;
            }
            let _ = this.update(cx, |app, cx| {
                app.activate_update_action(action, version, url, window_handle, cx);
            });
        })
        .detach();
    }

    fn activate_update_action(
        &mut self,
        action: UpdatePrimaryAction,
        version: String,
        url: String,
        window_handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        if action == UpdatePrimaryAction::BrowserDownload {
            cx.open_url(&url);
            return;
        }

        if self.tabs.iter().any(EditorTab::is_dirty) {
            let language = self.language;
            let _ = window_handle.update(cx, |_, window, cx| {
                std::mem::drop(window.prompt(
                    PromptLevel::Warning,
                    t(language, Msg::DialogUpdateSaveFirstTitle),
                    Some(t(language, Msg::DialogUpdateSaveFirstDetail)),
                    &[PromptButton::ok(t(language, Msg::DialogButtonOk))],
                    cx,
                ));
            });
            return;
        }

        self.status = tf(self.language, Msg::StatusUpdateDownloading, &[&version]).into();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { install_signed_update() })
                .await;
            if let Err(error) = result {
                let _ = this.update(cx, |app, cx| {
                    app.show_update_install_failure(error.to_string(), url, window_handle, cx);
                });
            }
        })
        .detach();
    }

    fn show_update_install_failure(
        &mut self,
        error: String,
        manual_url: String,
        window_handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        self.status = tf(self.language, Msg::StatusUpdateInstallFailed, &[&error]).into();
        cx.notify();

        let language = self.language;
        let detail = tf(language, Msg::DialogUpdateInstallFailedDetail, &[&error]);
        let Ok(answer) = window_handle.update(cx, |_, window, cx| {
            window.prompt(
                PromptLevel::Warning,
                t(language, Msg::DialogUpdateInstallFailedTitle),
                Some(&detail),
                &[
                    PromptButton::ok(t(language, Msg::DialogButtonDownloadManually)),
                    PromptButton::cancel(t(language, Msg::DialogButtonLater)),
                ],
                cx,
            )
        }) else {
            return;
        };

        cx.spawn(async move |this, cx| {
            if matches!(answer.await, Ok(0)) {
                let _ = this.update(cx, |_, cx| cx.open_url(&manual_url));
            }
        })
        .detach();
    }
}

#[cfg(windows)]
fn install_signed_update() -> Result<()> {
    use cargo_packager_updater::{
        Config, WindowsConfig, WindowsUpdateInstallMode, check_update, semver::Version, url::Url,
    };

    let public_key = configured_update_public_key()
        .ok_or_else(|| anyhow!("this build does not contain an updater public key"))?;
    let current_version =
        Version::parse(env!("CARGO_PKG_VERSION")).context("parsing the running Markion version")?;
    let endpoints = signed_update_manifest_endpoints()
        .into_iter()
        .map(|url| Url::parse(url).context("parsing the signed update manifest URL"))
        .collect::<Result<Vec<_>>>()?;
    let config = Config {
        endpoints,
        pubkey: public_key.to_string(),
        windows: Some(WindowsConfig {
            installer_args: None,
            install_mode: Some(WindowsUpdateInstallMode::Passive),
        }),
    };
    let update = check_update(current_version, config)
        .context("checking the signed update manifest")?
        .ok_or_else(|| anyhow!("the signed update manifest contains no newer version"))?;
    // cargo-packager-updater 0.2.3 returns ordinary network/signature errors,
    // but its NSIS launcher uses `expect` if PowerShell cannot be spawned.
    // Catch that library panic so the app can keep running and offer the
    // immutable GitHub asset as a manual fallback.
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        update.download_and_install()
    })) {
        Ok(result) => result.context("downloading, verifying, and launching the signed update"),
        Err(_) => Err(anyhow!(
            "the verified installer process could not be started"
        )),
    }
}

#[cfg(not(windows))]
fn install_signed_update() -> Result<()> {
    Err(anyhow!(
        "signed automatic installation is supported only on Windows"
    ))
}

/// Fetches and parses GitHub's latest published release. Runs on the shared
/// HTTP runtime (`network::fetch_url_bytes`), so no new tokio runtime is
/// created and no GPUI `HttpClient` registration is needed.
fn fetch_latest_release(url: &str) -> Result<GitHubRelease> {
    let bytes = network::fetch_url_bytes(url)?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parsing GitHub latest release response from {url}"))
}

/// Compares the release tag against the running build's version and, if newer,
/// maps the user's platform to the matching GitHub asset URL.
fn compare_with_running(release: &GitHubRelease) -> UpdateCheckOutcome {
    compare_with_running_for(release, consts::OS, consts::ARCH)
}

fn compare_with_running_for(release: &GitHubRelease, os: &str, arch: &str) -> UpdateCheckOutcome {
    let Some(version) = release.tag_name.strip_prefix('v') else {
        return UpdateCheckOutcome::Failed {
            error: format!(
                "GitHub release tag {:?} does not start with 'v'",
                release.tag_name
            ),
            manual_url: release.html_url.clone(),
        };
    };
    let Some(remote) = parse_semver(version) else {
        return UpdateCheckOutcome::Failed {
            error: format!(
                "GitHub release tag {:?} is not a valid release version",
                release.tag_name
            ),
            manual_url: release.html_url.clone(),
        };
    };
    let Some(current) = parse_semver(env!("CARGO_PKG_VERSION")) else {
        return UpdateCheckOutcome::Failed {
            error: format!(
                "running version {:?} is not a valid semver",
                env!("CARGO_PKG_VERSION")
            ),
            manual_url: release.html_url.clone(),
        };
    };
    if remote <= current {
        return UpdateCheckOutcome::UpToDate;
    }
    let url = match browser_download_url(release, os, arch) {
        Ok(url) => url,
        Err(error) => {
            return UpdateCheckOutcome::Failed {
                error,
                manual_url: release.html_url.clone(),
            };
        }
    };
    UpdateCheckOutcome::Available {
        version: version.to_string(),
        url,
    }
}

/// Returns the exact supported package URL, or the Release page on an
/// unsupported OS/architecture where no installable package can be promised.
fn browser_download_url(release: &GitHubRelease, os: &str, arch: &str) -> Result<String, String> {
    let asset_suffix = match (os, arch) {
        ("windows", "x86_64") => "_x64-setup.exe",
        ("macos", "aarch64") => "_aarch64.dmg",
        ("linux", "x86_64") => "_amd64.deb",
        _ => return Ok(release.html_url.clone()),
    };
    release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(asset_suffix))
        .map(|asset| asset.browser_download_url.clone())
        .ok_or_else(|| format!("latest GitHub Release has no asset ending with {asset_suffix:?}"))
}

/// Minimal `MAJOR.MINOR.PATCH` parser. Markion tags are strict `vX.Y.Z`, so
/// pre-release/build metadata is intentionally unsupported; if pre-release
/// tags are introduced later, swap this for the `semver` crate.
fn parse_semver(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_with_version(version: &str) -> GitHubRelease {
        GitHubRelease {
            tag_name: format!("v{version}"),
            html_url: format!("https://github.com/willmove/markion/releases/tag/v{version}"),
            assets: [
                ("markion_9.9.9_x64-setup.exe", "windows"),
                ("Markion_9.9.9_aarch64.dmg", "macos"),
                ("markion_9.9.9_amd64.deb", "linux"),
                ("markion_9.9.9_x86_64.AppImage", "appimage"),
            ]
            .into_iter()
            .map(|(name, platform)| GitHubReleaseAsset {
                name: name.to_string(),
                browser_download_url: format!(
                    "https://github.com/willmove/markion/releases/download/v{version}/{platform}"
                ),
            })
            .collect(),
        }
    }

    #[test]
    fn semver_parser_handles_releases() {
        assert_eq!(parse_semver("0.1.12"), Some((0, 1, 12)));
        assert_eq!(parse_semver("1.0.0"), Some((1, 0, 0)));
        assert_eq!(parse_semver("0.0.0"), Some((0, 0, 0)));
        assert_eq!(parse_semver("0.1"), None);
        assert_eq!(parse_semver("0.1.12.3"), None);
        assert_eq!(parse_semver("0.1.x"), None);
        assert_eq!(parse_semver(""), None);
    }

    #[test]
    fn signed_install_requires_windows_x86_64_and_a_public_key() {
        assert_eq!(
            update_primary_action_for("windows", "x86_64", Some("encoded-public-key")),
            UpdatePrimaryAction::SignedInstall
        );
        assert_eq!(
            update_primary_action_for("windows", "x86_64", None),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("windows", "x86_64", Some("   ")),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("windows", "aarch64", Some("encoded-public-key")),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("macos", "aarch64", Some("encoded-public-key")),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("linux", "x86_64", Some("encoded-public-key")),
            UpdatePrimaryAction::BrowserDownload
        );
    }

    #[test]
    fn update_available_detail_never_contains_the_raw_download_url() {
        for language in [
            Language::En,
            Language::ZhHans,
            Language::ZhHant,
            Language::Ja,
            Language::Fr,
            Language::De,
            Language::Es,
        ] {
            let detail = tf(language, Msg::DialogUpdateAvailableDetail, &["9.9.9"]);
            assert!(detail.contains("9.9.9"), "missing version in {language:?}");
            assert!(!detail.contains("http"), "raw URL leaked in {language:?}");
        }
    }

    #[test]
    fn updater_flow_preserves_manual_and_dirty_document_fallbacks() {
        let source = include_str!("update.rs");
        assert!(source.contains("tab.document.is_dirty()"));
        assert!(source.contains("background_executor()"));
        assert!(source.contains("catch_unwind"));
        assert!(source.contains("Msg::DialogButtonDownloadManually"));
        let endpoints = signed_update_manifest_endpoints();
        assert!(endpoints[0].ends_with("/markion-releases/latest/update.json"));
        assert!(endpoints[1].ends_with("/latest/download/update.json"));
    }

    #[test]
    fn newer_github_release_yields_available_outcome() {
        // Include every supported asset so the test passes on each CI host.
        let release = release_with_version("9.9.9");
        match compare_with_running(&release) {
            UpdateCheckOutcome::Available { version, url } => {
                assert!(version.starts_with("9.9.9"));
                assert!(
                    url.starts_with(
                        "https://github.com/willmove/markion/releases/download/v9.9.9/"
                    ),
                    "url should point at the GitHub Release asset: {url}"
                );
            }
            other => panic!("expected Available, got {other:?}"),
        }
    }

    #[test]
    fn unsupported_platform_uses_the_release_page_as_browser_fallback() {
        let release = release_with_version("9.9.9");
        assert_eq!(
            browser_download_url(&release, "windows", "aarch64").unwrap(),
            release.html_url
        );
        assert_eq!(
            browser_download_url(&release, "freebsd", "x86_64").unwrap(),
            release.html_url
        );
    }

    #[test]
    fn missing_supported_asset_retains_the_release_page_fallback() {
        let mut release = release_with_version("9.9.9");
        release.assets.clear();
        assert!(browser_download_url(&release, "windows", "x86_64").is_err());
        match compare_with_running_for(&release, "windows", "x86_64") {
            UpdateCheckOutcome::Failed { manual_url, .. } => {
                assert_eq!(manual_url, release.html_url);
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn equal_or_older_github_release_is_up_to_date() {
        let release = release_with_version(env!("CARGO_PKG_VERSION"));
        assert!(matches!(
            compare_with_running(&release),
            UpdateCheckOutcome::UpToDate
        ));
    }

    #[test]
    fn unparseable_github_release_tag_is_failure() {
        let mut release = release_with_version("9.9.9");
        release.tag_name = "not-a-version".to_string();
        assert!(matches!(
            compare_with_running(&release),
            UpdateCheckOutcome::Failed { .. }
        ));
    }

    #[test]
    fn github_latest_release_json_is_parsed() {
        let release: GitHubRelease = serde_json::from_str(
            r#"{
                "tag_name": "v9.9.9",
                "html_url": "https://github.com/willmove/markion/releases/tag/v9.9.9",
                "assets": [{
                    "name": "markion_9.9.9_x64-setup.exe",
                    "browser_download_url": "https://github.com/willmove/markion/releases/download/v9.9.9/markion_9.9.9_x64-setup.exe"
                }]
            }"#,
        )
        .unwrap();

        assert_eq!(release.tag_name, "v9.9.9");
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].name, "markion_9.9.9_x64-setup.exe");
    }

    #[test]
    #[ignore = "requires external network access to api.github.com"]
    fn live_github_latest_release_can_be_checked() {
        let release = fetch_latest_release(GITHUB_LATEST_RELEASE_API_URL).unwrap();
        assert!(release.tag_name.starts_with('v'));
        assert!(parse_semver(&release.tag_name[1..]).is_some());
        assert!(!release.assets.is_empty());
        assert!(!matches!(
            compare_with_running(&release),
            UpdateCheckOutcome::Failed { .. }
        ));
    }
}
