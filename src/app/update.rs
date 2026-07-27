//! In-app "Check for Updates" action.
//!
//! Fetches `${OSS_PUBLIC_BASE}/${OSS_PREFIX}/latest/manifest.json` from the
//! Aliyun OSS mirror that the release workflow's `mirror-oss` job publishes,
//! compares the manifest's `version` against `env!("CARGO_PKG_VERSION")`, and
//! surfaces the result through a modal dialog:
//!
//! - newer version -> dialog links the OSS download URL for the user's platform;
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

/// OSS public base URL (Bucket-level domain). Injected at build time via the
/// `MARKION_OSS_PUBLIC_BASE` env var (CI reads it from the `OSS_PUBLIC_BASE`
/// GitHub secret); local dev builds fall back to the real mirror so the menu
/// item works out of the box.
const OSS_PUBLIC_BASE: &str = match option_env!("MARKION_OSS_PUBLIC_BASE") {
    Some(value) => value,
    None => "https://marknice.oss-cn-heyuan.aliyuncs.com",
};

/// OSS object-key prefix. Same injection rule; CI sets `MARKION_OSS_PREFIX`
/// from the `OSS_PREFIX` GitHub secret.
const OSS_PREFIX: &str = match option_env!("MARKION_OSS_PREFIX") {
    Some(value) => value,
    None => "releases",
};

/// Manifest shape produced by the `mirror-oss` workflow job. Only the fields
/// the client actually reads are declared; `pub_date` is parsed-but-unused so
/// a future field addition stays backward-compatible.
#[derive(Debug, Deserialize)]
struct UpdateManifest {
    version: String,
    #[allow(dead_code)]
    tag: String,
    #[allow(dead_code)]
    pub_date: String,
    assets: std::collections::BTreeMap<String, UpdateAsset>,
}

#[derive(Debug, Deserialize)]
struct UpdateAsset {
    filename: String,
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
        let manifest_url = format!("{}/{}/latest/manifest.json", OSS_PUBLIC_BASE, OSS_PREFIX);

        self.status = self.tr(Msg::StatusUpdateChecking).into();
        self.active_menu = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let outcome = match fetch_manifest(&manifest_url).await {
                Ok(manifest) => compare_with_running(&manifest),
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

/// Fetches and parses the update manifest from the OSS mirror. Runs on the
/// shared HTTP runtime (`network::fetch_url_bytes`), so no new tokio runtime
/// is created and no GPUI `HttpClient` registration is needed.
async fn fetch_manifest(url: &str) -> Result<UpdateManifest> {
    let url = url.to_string();
    let bytes = network::fetch_url_bytes(&url)?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing update manifest from {url}"))
}

/// Compares the manifest's version against the running build's version and,
/// if newer, maps the user's platform to the matching asset URL.
fn compare_with_running(manifest: &UpdateManifest) -> UpdateCheckOutcome {
    let Some(remote) = parse_semver(&manifest.version) else {
        return UpdateCheckOutcome::Failed(format!(
            "manifest version {:?} is not a valid semver",
            manifest.version
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
    let platform_key = match (consts::OS, consts::ARCH) {
        ("windows", "x86_64") => "windows-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        ("linux", "x86_64") => "linux-amd64",
        _ => "linux-appimage",
    };
    let Some(asset) = manifest.assets.get(platform_key) else {
        return UpdateCheckOutcome::Failed(format!(
            "no download asset declared for platform {platform_key:?}"
        ));
    };
    let url = format!(
        "{}/{}/latest/{}",
        OSS_PUBLIC_BASE, OSS_PREFIX, asset.filename
    );
    UpdateCheckOutcome::Available {
        version: manifest.version.clone(),
        url,
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
    fn newer_manifest_version_yields_available_outcome() {
        // Include all four platform assets so the test passes regardless of
        // which platform it runs on (`compare_with_running` picks the host's key).
        let mut assets = std::collections::BTreeMap::new();
        for (key, filename) in [
            ("windows-x86_64", "markion_9.9.9_x64-setup.exe"),
            ("macos-aarch64", "Markion_9.9.9_aarch64.dmg"),
            ("linux-amd64", "markion_9.9.9_amd64.deb"),
            ("linux-appimage", "markion_9.9.9_x86_64.AppImage"),
        ] {
            assets.insert(
                key.to_string(),
                UpdateAsset {
                    filename: filename.to_string(),
                },
            );
        }
        let manifest = UpdateManifest {
            version: "9.9.9".to_string(),
            tag: "v9.9.9".to_string(),
            pub_date: "2026-01-01T00:00:00Z".to_string(),
            assets,
        };
        match compare_with_running(&manifest) {
            UpdateCheckOutcome::Available { version, url } => {
                assert!(version.starts_with("9.9.9"));
                assert!(
                    url.starts_with(
                        "https://markion.oss-cn-hangzhou.aliyuncs.com/releases/latest/"
                    ) || url.contains("/latest/"),
                    "url should point at the OSS latest path: {url}"
                );
            }
            other => panic!("expected Available, got {other:?}"),
        }
    }

    #[test]
    fn equal_or_older_manifest_version_is_up_to_date() {
        let manifest = UpdateManifest {
            version: env!("CARGO_PKG_VERSION").to_string(),
            tag: String::new(),
            pub_date: String::new(),
            assets: std::collections::BTreeMap::new(),
        };
        assert!(matches!(
            compare_with_running(&manifest),
            UpdateCheckOutcome::UpToDate
        ));
    }

    #[test]
    fn unparseable_manifest_version_is_failure() {
        let manifest = UpdateManifest {
            version: "not-a-version".to_string(),
            tag: String::new(),
            pub_date: String::new(),
            assets: std::collections::BTreeMap::new(),
        };
        assert!(matches!(
            compare_with_running(&manifest),
            UpdateCheckOutcome::Failed(_)
        ));
    }
}
