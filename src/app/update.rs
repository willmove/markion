//! In-app "Check for Updates" action.
//!
//! Fetches the repository's latest published GitHub Release through the GitHub
//! REST API, compares its `tag_name` against `env!("CARGO_PKG_VERSION")`, and
//! surfaces the result through a modal dialog:
//!
//! - newer version -> dialog links the GitHub asset for the user's platform;
//! - same/older version -> "up to date";
//! - fetch or parse failure -> error dialog (no crash).
//!
//! The check is notify-only: it never downloads or installs the new version.
//! It runs off the main render path inside an async `cx.spawn` task and does
//! not touch any cached-per-version Markdown state. See the OpenSpec change
//! `mirror-releases-to-aliyun-oss` for the full contract.

use super::*;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use serde_json;
use std::env::consts;

/// GitHub's unauthenticated "Get the latest release" endpoint. GitHub Releases
/// remains the source of truth, and the shared HTTP helper supplies the
/// required Markion User-Agent header.
const GITHUB_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/willmove/markion/releases/latest";

/// Subset of GitHub's release response used by the update checker. Serde
/// ignores the API's other fields, keeping this model resilient to additions.
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
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
    Failed(String),
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
        let language = self.language;
        self.status = self.tr(Msg::StatusUpdateChecking).into();
        self.active_menu = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let outcome = match fetch_latest_release(GITHUB_LATEST_RELEASE_API_URL) {
                Ok(release) => compare_with_running(&release),
                Err(err) => UpdateCheckOutcome::Failed(err.to_string()),
            };

            let _ = this.update(cx, |app, cx| {
                app.status = match &outcome {
                    UpdateCheckOutcome::UpToDate => {
                        t(app.language, Msg::StatusUpdateUpToDate).into()
                    }
                    UpdateCheckOutcome::Available { version, .. } => {
                        tf(app.language, Msg::StatusUpdateAvailable, &[version]).into()
                    }
                    UpdateCheckOutcome::Failed(err) => {
                        tf(app.language, Msg::StatusUpdateCheckFailed, &[err]).into()
                    }
                };
                cx.notify();

                // Show the modal dialog via the captured window handle, after
                // the status bar has been updated. `std::mem::drop` matches the
                // `about` handler: the result future is informational only.
                // The closure receives its own `&mut Context<V>` (the third
                // parameter); we must use THAT `cx` for `window.prompt`, not
                // the outer `cx` which `update` has already borrowed.
                let _ = window_handle.update(cx, |_, window, cx| match &outcome {
                    UpdateCheckOutcome::UpToDate => {
                        let detail = t(language, Msg::DialogUpToDateDetail);
                        std::mem::drop(window.prompt(
                            PromptLevel::Info,
                            t(language, Msg::DialogUpToDateTitle),
                            Some(detail),
                            &[PromptButton::ok(t(language, Msg::DialogButtonOk))],
                            cx,
                        ));
                    }
                    UpdateCheckOutcome::Available { version, url } => {
                        let detail =
                            tf(language, Msg::DialogUpdateAvailableDetail, &[version, url]);
                        std::mem::drop(window.prompt(
                            PromptLevel::Info,
                            t(language, Msg::DialogUpdateAvailableTitle),
                            Some(&detail),
                            &[PromptButton::ok(t(language, Msg::DialogButtonOk))],
                            cx,
                        ));
                    }
                    UpdateCheckOutcome::Failed(err) => {
                        let detail = tf(language, Msg::DialogUpdateCheckFailedDetail, &[err]);
                        std::mem::drop(window.prompt(
                            PromptLevel::Warning,
                            t(language, Msg::DialogUpdateCheckFailedTitle),
                            Some(&detail),
                            &[PromptButton::ok(t(language, Msg::DialogButtonOk))],
                            cx,
                        ));
                    }
                });
            });
        })
        .detach();
    }
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
    let Some(version) = release.tag_name.strip_prefix('v') else {
        return UpdateCheckOutcome::Failed(format!(
            "GitHub release tag {:?} does not start with 'v'",
            release.tag_name
        ));
    };
    let Some(remote) = parse_semver(version) else {
        return UpdateCheckOutcome::Failed(format!(
            "GitHub release tag {:?} is not a valid release version",
            release.tag_name
        ));
    };
    let Some(current) = parse_semver(env!("CARGO_PKG_VERSION")) else {
        return UpdateCheckOutcome::Failed(format!(
            "running version {:?} is not a valid semver",
            env!("CARGO_PKG_VERSION")
        ));
    };
    if remote <= current {
        return UpdateCheckOutcome::UpToDate;
    }
    let asset_suffix = match (consts::OS, consts::ARCH) {
        ("windows", "x86_64") => "_x64-setup.exe",
        ("macos", "aarch64") => "_aarch64.dmg",
        ("linux", "x86_64") => "_amd64.deb",
        (os, arch) => {
            return UpdateCheckOutcome::Failed(format!(
                "no GitHub Release asset mapping for platform {os:?}/{arch:?}"
            ));
        }
    };
    let Some(asset) = release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(asset_suffix))
    else {
        return UpdateCheckOutcome::Failed(format!(
            "latest GitHub Release has no asset ending with {asset_suffix:?}"
        ));
    };
    UpdateCheckOutcome::Available {
        version: version.to_string(),
        url: asset.browser_download_url.clone(),
    }
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
            UpdateCheckOutcome::Failed(_)
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
            UpdateCheckOutcome::Failed(_)
        ));
    }
}
