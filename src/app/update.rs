//! Periodic discovery, authenticated update staging and explicit restart handoff.
//!
//! Fetches the repository's latest published GitHub Release through the GitHub
//! REST API, compares its `tag_name` against `env!("CARGO_PKG_VERSION")`, and
//! surfaces the result through an actionable modal dialog:
//!
//! - newer version -> dialog offers signed install or browser download;
//! - same/older version -> "up to date";
//! - fetch or parse failure -> error dialog (no crash).
//!
//! Windows x86_64 and Linux AppImage tagged builds use authenticated packages.
//! Downloads are staged independently of installation; only an explicit restart
//! confirmation with clean documents permits installation. Other packages retain
//! browser downloads. No update work invalidates version-cached Markdown state.

use super::*;
use anyhow::{Context as _, Result, anyhow};
use gpui::AnyWindowHandle;
use serde::Deserialize;
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
#[cfg_attr(not(any(windows, target_os = "linux")), allow(dead_code))]
const OSS_SIGNED_UPDATE_MANIFEST_URL: &str =
    "https://marknice.oss-cn-heyuan.aliyuncs.com/markion-releases/latest/update.json";

/// GitHub-hosted fallback manifest for clients that cannot reach the OSS
/// mirror at all.
#[cfg_attr(not(any(windows, target_os = "linux")), allow(dead_code))]
const GITHUB_SIGNED_UPDATE_MANIFEST_URL: &str =
    "https://github.com/willmove/markion/releases/latest/download/update.json";

/// Ordered manifest endpoints consumed by the signed updater and asserted by
/// the unit tests. Non-updating builds keep these reachable for the tests
/// without tripping the dead-code lint that CI's `-D warnings` rejects.
#[cfg_attr(not(any(windows, target_os = "linux")), allow(dead_code))]
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

/// True when this process was launched through an AppImage runtime, which
/// exports `APPIMAGE` pointing at the .AppImage file on disk — exactly the
/// file the signed updater replaces. DEB/RPM installs and extracted AppDir
/// runs have no such variable and cannot self-replace without root.
fn running_as_appimage() -> bool {
    std::env::var("APPIMAGE").is_ok_and(|path| !path.trim().is_empty())
}

/// Decides the primary update action. Signed automatic installation needs an
/// embedded updater public key and a package the updater can replace itself:
/// the current-user NSIS install on Windows x86_64, or an AppImage launch
/// (`appimage` = `APPIMAGE`'s value) on Linux x86_64. Everything else keeps
/// the browser-download fallback.
fn update_primary_action_for(
    os: &str,
    arch: &str,
    public_key: Option<&str>,
    appimage: Option<&str>,
) -> UpdatePrimaryAction {
    let key = public_key.is_some_and(|key| !key.trim().is_empty());
    let self_updatable_package = match (os, arch) {
        ("windows", "x86_64") => true,
        ("linux", "x86_64") => appimage.is_some_and(|path| !path.trim().is_empty()),
        _ => false,
    };
    if key && self_updatable_package {
        UpdatePrimaryAction::SignedInstall
    } else {
        UpdatePrimaryAction::BrowserDownload
    }
}

fn current_update_primary_action() -> UpdatePrimaryAction {
    update_primary_action_for(
        consts::OS,
        consts::ARCH,
        configured_update_public_key(),
        std::env::var("APPIMAGE").ok().as_deref(),
    )
}

/// Session-scoped update state. Only one operation or prompt can own the flow.
#[derive(Default)]
pub(super) struct UpdateState {
    busy: bool,
    prompt_open: bool,
    pub(super) installing: bool,
    ready: Option<Arc<ReadyUpdate>>,
}

impl UpdateState {
    fn can_check(&self) -> bool {
        !self.busy && !self.prompt_open && !self.installing && self.ready.is_none()
    }

    fn should_check(
        &self,
        last: Option<&str>,
        now: time::OffsetDateTime,
        startup_override: bool,
    ) -> bool {
        self.can_check() && (startup_override || update_check_due(last, now))
    }
}

struct ReadyUpdate {
    version: String,
    manual_url: String,
    package: PreparedUpdate,
}

struct PreparedUpdate {
    #[cfg(any(windows, target_os = "linux"))]
    metadata: cargo_packager_updater::Update,
    #[cfg(any(windows, target_os = "linux"))]
    file: tempfile::TempPath,
    #[cfg(any(windows, target_os = "linux"))]
    digest: [u8; 32],
}

const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(5 * 60 * 60);

fn update_check_due(last: Option<&str>, now: time::OffsetDateTime) -> bool {
    let Some(checked) = last.and_then(|s| {
        time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()
    }) else {
        return true;
    };
    let elapsed = now - checked;
    elapsed.is_negative()
        || elapsed >= time::Duration::seconds(UPDATE_CHECK_INTERVAL.as_secs() as i64)
}

impl MarkionApp {
    pub(super) fn toggle_automatic_update_downloads(&mut self, cx: &mut Context<Self>) {
        self.automatically_download_updates = !self.automatically_download_updates;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn supports_signed_updates(&self) -> bool {
        current_update_primary_action() == UpdatePrimaryAction::SignedInstall
    }

    pub(super) fn arm_update_checks(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Unit tests and the isolated native replay must never contact releases.
        if cfg!(test) || env::var_os("MARKION_NATIVE_SMOKE_DIR").is_some() {
            return;
        }
        let handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            Timer::after(Duration::from_secs(10)).await;
            let mut startup = true;
            loop {
                if this
                    .update(cx, |app, cx| {
                        if app.update_state.should_check(
                            app.last_update_check.as_deref(),
                            time::OffsetDateTime::now_utc(),
                            startup && app.check_for_updates_on_startup,
                        ) {
                            app.begin_update_check(false, handle, cx);
                        }
                    })
                    .is_err()
                {
                    break;
                }
                startup = false;
                // A short wake interval also handles sleep/resume and wall-clock changes.
                Timer::after(Duration::from_secs(60)).await;
            }
        })
        .detach();
    }

    pub(super) fn check_for_updates(
        &mut self,
        _: &CheckForUpdates,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_menu = None;
        cx.notify();
        if self.update_state.busy || self.update_state.prompt_open {
            return;
        }
        let handle = window.window_handle();
        if self.update_state.ready.is_some() {
            // The action currently owns `window`; handle.update cannot borrow
            // it again until this callback returns. Reserve the prompt slot.
            self.update_state.prompt_open = true;
            cx.spawn(async move |this, cx| {
                let _ = this.update(cx, |app, cx| {
                    app.update_state.prompt_open = false;
                    app.prompt_update_ready(handle, cx);
                });
            })
            .detach();
        } else {
            self.begin_update_check(true, handle, cx);
        }
    }

    fn begin_update_check(
        &mut self,
        manual: bool,
        handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        if !self.update_state.can_check() {
            return;
        }
        self.update_state.busy = true;
        self.last_update_check = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .ok();
        self.persist_preferences();
        if manual {
            self.status = self.tr(Msg::StatusUpdateChecking).into();
            cx.notify();
        }
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
                app.update_state.busy = false;
                match outcome {
                    UpdateCheckOutcome::UpToDate if manual => {
                        app.status = app.tr(Msg::StatusUpdateUpToDate).into();
                        let language = app.language;
                        let _ = handle.update(cx, |_, window, cx| {
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
                            tf(app.language, Msg::StatusUpdateAvailable, &[&version]).into();
                        if app.automatically_download_updates && app.supports_signed_updates() {
                            app.download_update(version, url, manual, handle, cx);
                        } else {
                            app.prompt_update_available(version, url, handle, cx);
                        }
                    }
                    UpdateCheckOutcome::Failed { error, manual_url } if manual => {
                        app.show_update_check_failure(error, manual_url, handle, cx);
                    }
                    UpdateCheckOutcome::Failed { error, .. } => {
                        tracing::warn!(%error, "background update check failed");
                    }
                    UpdateCheckOutcome::UpToDate => {}
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn show_update_check_failure(
        &mut self,
        error: String,
        url: String,
        handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        self.status = tf(self.language, Msg::StatusUpdateCheckFailed, &[&error]).into();
        let detail = tf(self.language, Msg::DialogUpdateCheckFailedDetail, &[&error]);
        self.prompt_update_failure(Msg::DialogUpdateCheckFailedTitle, detail, url, handle, cx);
    }

    fn prompt_update_failure(
        &mut self,
        title: Msg,
        detail: String,
        url: String,
        handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        let language = self.language;
        let Ok(answer) = handle.update(cx, |_, window, cx| {
            window.prompt(
                PromptLevel::Warning,
                t(language, title),
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
        self.update_state.prompt_open = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let download = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                app.update_state.prompt_open = false;
                if download {
                    cx.open_url(&url);
                }
            });
        })
        .detach();
    }

    fn prompt_update_available(
        &mut self,
        version: String,
        url: String,
        handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        let action = current_update_primary_action();
        let language = self.language;
        let detail = tf(language, Msg::DialogUpdateAvailableDetail, &[&version]);
        let Ok(answer) = handle.update(cx, |_, window, cx| {
            window.prompt(
                PromptLevel::Info,
                t(language, Msg::DialogUpdateAvailableTitle),
                Some(&detail),
                &[
                    PromptButton::ok(t(language, Msg::DialogButtonDownloadUpdate)),
                    PromptButton::cancel(t(language, Msg::DialogButtonLater)),
                ],
                cx,
            )
        }) else {
            return;
        };
        self.update_state.prompt_open = true;
        cx.spawn(async move |this, cx| {
            let download = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                app.update_state.prompt_open = false;
                if download {
                    match action {
                        UpdatePrimaryAction::BrowserDownload => cx.open_url(&url),
                        UpdatePrimaryAction::SignedInstall => {
                            app.download_update(version, url, true, handle, cx)
                        }
                    }
                }
            });
        })
        .detach();
    }

    fn download_update(
        &mut self,
        version: String,
        url: String,
        manual: bool,
        handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        if !self.update_state.can_check() {
            return;
        }
        self.update_state.busy = true;
        self.status = tf(self.language, Msg::StatusUpdateDownloading, &[&version]).into();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    prepare_signed_update(&version)
                        .map(|package| ReadyUpdate {
                            version,
                            manual_url: url.clone(),
                            package,
                        })
                        .map_err(|error| (format!("{error:#}"), url))
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.update_state.busy = false;
                match result {
                    Ok(ready) => {
                        app.update_state.ready = Some(Arc::new(ready));
                        app.prompt_update_ready(handle, cx);
                    }
                    Err((error, url)) if manual => {
                        app.show_update_install_failure(error, url, handle, cx)
                    }
                    Err((error, _)) => {
                        tracing::warn!(%error, "automatic update download failed");
                        app.status =
                            tf(app.language, Msg::StatusUpdateInstallFailed, &[&error]).into();
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn prompt_update_ready(&mut self, handle: AnyWindowHandle, cx: &mut Context<Self>) {
        if self.update_state.busy || self.update_state.prompt_open {
            return;
        }
        let Some(ready) = self.update_state.ready.as_ref() else {
            return;
        };
        let detail = tf(
            self.language,
            Msg::DialogUpdateReadyDetail,
            &[&ready.version],
        );
        self.status = tf(self.language, Msg::StatusUpdateReady, &[&ready.version]).into();
        let language = self.language;
        let Ok(answer) = handle.update(cx, |_, window, cx| {
            window.prompt(
                PromptLevel::Info,
                t(language, Msg::DialogUpdateReadyTitle),
                Some(&detail),
                &[
                    PromptButton::ok(t(language, Msg::DialogButtonRestartAndUpdate)),
                    PromptButton::cancel(t(language, Msg::DialogButtonLater)),
                ],
                cx,
            )
        }) else {
            return;
        };
        self.update_state.prompt_open = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let restart = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                app.update_state.prompt_open = false;
                if restart {
                    app.install_ready_update(handle, cx);
                }
            });
        })
        .detach();
    }

    fn update_documents_are_clean(
        &mut self,
        handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.tabs.iter().any(EditorTab::is_dirty) {
            return true;
        }
        let language = self.language;
        let _ = handle.update(cx, |_, window, cx| {
            std::mem::drop(window.prompt(
                PromptLevel::Warning,
                t(language, Msg::DialogUpdateSaveFirstTitle),
                Some(t(language, Msg::DialogUpdateSaveFirstDetail)),
                &[PromptButton::ok(t(language, Msg::DialogButtonOk))],
                cx,
            ));
        });
        false
    }

    fn install_ready_update(&mut self, handle: AnyWindowHandle, cx: &mut Context<Self>) {
        if self.update_state.busy || !self.update_documents_are_clean(handle, cx) {
            return;
        }
        let Some(ready) = self.update_state.ready.clone() else {
            return;
        };
        self.update_state.busy = true;
        cx.spawn(async move |this, cx| {
            let package = ready.clone();
            let result = cx
                .background_executor()
                .spawn(async move { read_verified_package(&package.package) })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.update_state.busy = false;
                match result {
                    Ok(bytes) => {
                        // Recheck after disk I/O, since the user may have typed while it ran.
                        if !app.update_documents_are_clean(handle, cx) {
                            return;
                        }
                        app.update_state.busy = true;
                        app.update_state.installing = true;
                        app.status = app.tr(Msg::StatusUpdateInstalling).into();
                        if let Ok(bounds) = handle.update(cx, |_, window, _| {
                            window.blur();
                            window.window_bounds()
                        }) {
                            app.apply_window_bounds(bounds);
                        }
                        app.flush_layout();
                        app.sync_and_persist_session();
                        app.persist_preferences();
                        cx.notify();
                        cx.spawn(async move |this, cx| {
                            let package = ready.clone();
                            let result = cx
                                .background_executor()
                                .spawn(
                                    async move { install_prepared_update(&package.package, bytes) },
                                )
                                .await;
                            let _ = this.update(cx, |app, cx| {
                                app.update_state.busy = false;
                                app.update_state.installing = false;
                                let focus = app.focus_handle.clone();
                                let _ = handle.update(cx, |_, window, _| window.focus(&focus));
                                match result {
                                    Ok(()) => app.finish_signed_update(handle, cx),
                                    Err(error) => app.show_update_install_failure(
                                        format!("{error:#}"),
                                        ready.manual_url.clone(),
                                        handle,
                                        cx,
                                    ),
                                }
                                cx.notify();
                            });
                        })
                        .detach();
                    }
                    Err(error) => {
                        app.update_state.ready = None;
                        app.show_update_install_failure(
                            format!("{error:#}"),
                            ready.manual_url.clone(),
                            handle,
                            cx,
                        );
                    }
                }
            });
        })
        .detach();
    }

    fn show_update_install_failure(
        &mut self,
        error: String,
        url: String,
        handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) {
        self.status = tf(self.language, Msg::StatusUpdateInstallFailed, &[&error]).into();
        let detail = tf(
            self.language,
            Msg::DialogUpdateInstallFailedDetail,
            &[&error],
        );
        self.prompt_update_failure(Msg::DialogUpdateInstallFailedTitle, detail, url, handle, cx);
    }

    fn finish_signed_update(&mut self, handle: AnyWindowHandle, cx: &mut Context<Self>) {
        #[cfg(target_os = "linux")]
        {
            if std::env::var("APPIMAGE")
                .ok()
                .filter(|p| !p.trim().is_empty())
                .is_some_and(|p| std::process::Command::new(p).spawn().is_ok())
            {
                cx.quit();
                return;
            }
            self.update_state.ready = None;
            self.status = self.tr(Msg::StatusUpdateInstalledRestart).into();
            let language = self.language;
            let _ = handle.update(cx, |_, window, cx| {
                std::mem::drop(window.prompt(
                    PromptLevel::Info,
                    t(language, Msg::DialogUpdateInstalledTitle),
                    Some(t(language, Msg::DialogUpdateInstalledRestartDetail)),
                    &[PromptButton::ok(t(language, Msg::DialogButtonOk))],
                    cx,
                ));
            });
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = handle;
            cx.quit();
        }
    }
}

#[cfg(any(windows, target_os = "linux"))]
fn prepare_signed_update(expected_version: &str) -> Result<PreparedUpdate> {
    use cargo_packager_updater::{Config, UpdaterBuilder, semver::Version, url::Url};
    use sha2::{Digest, Sha256};
    use std::io::Write;

    if current_update_primary_action() != UpdatePrimaryAction::SignedInstall {
        return Err(anyhow!("this installation cannot apply signed updates"));
    }
    let public_key =
        configured_update_public_key().ok_or_else(|| anyhow!("missing updater public key"))?;
    let current = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let endpoints = signed_update_manifest_endpoints()
        .into_iter()
        .map(|u| Url::parse(u).context("parsing update manifest URL"))
        .collect::<Result<Vec<_>>>()?;
    let config = Config {
        endpoints,
        pubkey: public_key.into(),
        windows: None,
    };
    let mut update = UpdaterBuilder::new(current, config)
        .timeout(Duration::from_secs(60))
        .build()?
        .check()
        .context("checking signed update manifest")?
        .ok_or_else(|| anyhow!("the signed manifest contains no newer version"))?;
    ensure_update_version(expected_version, &update.version)?;
    #[cfg(windows)]
    if !matches!(update.format, cargo_packager_updater::UpdateFormat::Nsis) {
        return Err(anyhow!(
            "the signed Windows update is not an NSIS installer"
        ));
    }
    #[cfg(target_os = "linux")]
    if !matches!(
        update.format,
        cargo_packager_updater::UpdateFormat::AppImage
    ) {
        return Err(anyhow!("the signed Linux update is not an AppImage"));
    }
    update.timeout = Some(Duration::from_secs(30 * 60));
    let bytes = update
        .download()
        .context("downloading and authenticating update")?;
    let digest = Sha256::digest(&bytes).into();
    let mut file = tempfile::Builder::new()
        .prefix("markion-update-")
        .suffix(if cfg!(windows) { ".exe" } else { ".AppImage" })
        .tempfile()?;
    file.write_all(&bytes)
        .context("staging authenticated update")?;
    file.flush()?;
    Ok(PreparedUpdate {
        metadata: update,
        file: file.into_temp_path(),
        digest,
    })
}

#[cfg(any(windows, target_os = "linux", test))]
fn ensure_update_version(expected: &str, actual: &str) -> Result<()> {
    if parse_semver(expected).is_none() || parse_semver(expected) != parse_semver(actual) {
        return Err(anyhow!(
            "signed manifest version {actual} does not match discovered release {expected}"
        ));
    }
    Ok(())
}

#[cfg(any(windows, target_os = "linux"))]
fn read_verified_package(package: &PreparedUpdate) -> Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(&package.file).context("reading staged update")?;
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    if digest != package.digest {
        return Err(anyhow!("staged update changed after authentication"));
    }
    Ok(bytes)
}

#[cfg(windows)]
fn install_prepared_update(package: &PreparedUpdate, bytes: Vec<u8>) -> Result<()> {
    use std::{io::Write, os::windows::process::CommandExt};
    // Use authenticated bytes in a fresh executable, avoiding the library's
    // PowerShell launcher and unconditional process::exit before state flush.
    let _ = &package.metadata;
    let mut file = tempfile::Builder::new()
        .prefix("markion-install-")
        .suffix(".exe")
        .tempfile()?;
    file.write_all(&bytes)?;
    let (file, path) = file.keep()?;
    drop(file);
    if let Err(error) = std::process::Command::new(&path)
        .args(["/P", "/R"])
        .creation_flags(0x08000000)
        .spawn()
    {
        let _ = fs::remove_file(&path);
        return Err(error).context("starting verified NSIS installer");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_prepared_update(package: &PreparedUpdate, bytes: Vec<u8>) -> Result<()> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        package.metadata.install(bytes)
    })) {
        Ok(result) => result.context("installing authenticated AppImage"),
        Err(_) => Err(anyhow!("the verified update could not be installed")),
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn prepare_signed_update(_: &str) -> Result<PreparedUpdate> {
    Err(anyhow!("signed updates unsupported"))
}
#[cfg(not(any(windows, target_os = "linux")))]
fn read_verified_package(_: &PreparedUpdate) -> Result<Vec<u8>> {
    Err(anyhow!("signed updates unsupported"))
}
#[cfg(not(any(windows, target_os = "linux")))]
fn install_prepared_update(_: &PreparedUpdate, _: Vec<u8>) -> Result<()> {
    Err(anyhow!("signed updates unsupported"))
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
    compare_with_running_for(release, consts::OS, consts::ARCH, running_as_appimage())
}

fn compare_with_running_for(
    release: &GitHubRelease,
    os: &str,
    arch: &str,
    appimage: bool,
) -> UpdateCheckOutcome {
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
    let url = match browser_download_url(release, os, arch, appimage) {
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
fn browser_download_url(
    release: &GitHubRelease,
    os: &str,
    arch: &str,
    appimage: bool,
) -> Result<String, String> {
    browser_download_url_with_os_release(release, os, arch, appimage, read_os_release().as_deref())
}

/// Like [`browser_download_url`], with the `/etc/os-release` contents supplied
/// by the caller so tests stay hermetic.
fn browser_download_url_with_os_release(
    release: &GitHubRelease,
    os: &str,
    arch: &str,
    appimage: bool,
    os_release: Option<&str>,
) -> Result<String, String> {
    let asset_suffix = match (os, arch) {
        ("windows", "x86_64") => "_x64-setup.exe",
        ("macos", "aarch64") => "_aarch64.dmg",
        ("macos", "x86_64") => "_x64.dmg",
        // An AppImage install offers the AppImage on any distro — the user
        // picked that package type and the signed updater replaces it.
        ("linux", "x86_64") if appimage => "_x86_64.AppImage",
        ("linux", "x86_64") if linux_wants_appimage(os_release) => "_x86_64.AppImage",
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

/// Classifies a Linux system from its `/etc/os-release` contents: a
/// pacman-based distribution — the `ID` value, or any token of the
/// space-separated `ID_LIKE` list, equals `arch` (Arch, Omarchy, Manjaro,
/// EndeavourOS, …) — takes the portable AppImage, and every other or
/// undecidable system keeps the DEB default.
fn linux_wants_appimage(os_release: Option<&str>) -> bool {
    let Some(text) = os_release else {
        return false;
    };
    let mut id_is_arch = false;
    let mut id_like_arch = false;
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');
        match key.trim() {
            "ID" => id_is_arch = value == "arch",
            "ID_LIKE" => id_like_arch = value.split_whitespace().any(|token| token == "arch"),
            _ => {}
        }
    }
    id_is_arch || id_like_arch
}

/// Reads `/etc/os-release` for the update-check asset mapping. The file only
/// exists on Linux, so other platforms — and any read failure — yield `None`
/// and keep the default asset choice.
fn read_os_release() -> Option<String> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    std::fs::read_to_string("/etc/os-release").ok()
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
    fn automatic_update_interval_handles_boundaries_and_invalid_clocks() {
        use time::{OffsetDateTime, format_description::well_known::Rfc3339};
        let now = OffsetDateTime::parse("2026-11-17T12:00:00Z", &Rfc3339).unwrap();
        for last in [
            None,
            Some("invalid"),
            Some("2026-11-17T12:00:01Z"),
            Some("2026-11-17T07:00:00Z"),
        ] {
            assert!(update_check_due(last, now), "should check for {last:?}");
        }
        for last in [
            "2026-11-17T07:00:01Z",
            "2026-11-17T12:00:00Z",
            "2026-11-17T15:00:01+08:00",
        ] {
            assert!(!update_check_due(Some(last), now), "should wait for {last}");
        }
    }

    #[test]
    fn manifest_must_match_the_discovered_release() {
        assert!(ensure_update_version("9.9.9", "9.9.9").is_ok());
        for actual in ["9.9.8", "10.0.0", "bad", "9.9.9-beta.1"] {
            assert!(ensure_update_version("9.9.9", actual).is_err());
        }
        assert!(ensure_update_version("bad", "bad").is_err());
    }

    #[test]
    fn legacy_startup_override_bypasses_time_but_never_active_operations() {
        let now = time::OffsetDateTime::now_utc();
        let recent = now
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();
        let mut state = UpdateState::default();
        assert!(!state.should_check(Some(&recent), now, false));
        assert!(state.should_check(Some(&recent), now, true));
        state.busy = true;
        assert!(!state.should_check(None, now, true));
        state.busy = false;
        state.prompt_open = true;
        assert!(!state.should_check(None, now, true));
    }

    #[cfg(any(windows, target_os = "linux"))]
    fn staged_fixture() -> ReadyUpdate {
        use cargo_packager_updater::{Config, Update, UpdateFormat, url::Url};
        use sha2::{Digest, Sha256};
        use std::io::Write;
        let bytes = b"offline authenticated-package stand-in; never execute";
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(bytes).unwrap();
        ReadyUpdate {
            version: "9.9.9".into(),
            manual_url: GITHUB_LATEST_RELEASE_URL.into(),
            package: PreparedUpdate {
                file: file.into_temp_path(),
                digest: Sha256::digest(bytes).into(),
                metadata: Update {
                    config: Config {
                        endpoints: vec![],
                        pubkey: String::new(),
                        windows: None,
                    },
                    body: None,
                    current_version: env!("CARGO_PKG_VERSION").into(),
                    version: "9.9.9".into(),
                    date: None,
                    target: "fixture".into(),
                    extract_path: PathBuf::new(),
                    download_url: Url::parse(GITHUB_LATEST_RELEASE_URL).unwrap(),
                    signature: String::new(),
                    timeout: None,
                    headers: Default::default(),
                    format: if cfg!(windows) {
                        UpdateFormat::Nsis
                    } else {
                        UpdateFormat::AppImage
                    },
                },
            },
        }
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[test]
    fn staged_update_rejects_tampering_and_cleans_up() {
        let ready = staged_fixture();
        let path = ready.package.file.to_path_buf();
        assert!(read_verified_package(&ready.package).is_ok());
        fs::write(&path, b"modified after authentication").unwrap();
        assert!(read_verified_package(&ready.package).is_err());
        drop(ready);
        assert!(!path.exists());
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[test]
    fn active_operations_and_ready_payloads_prevent_duplicate_checks() {
        let mut state = UpdateState::default();
        assert!(state.can_check());
        state.busy = true;
        assert!(!state.can_check());
        state.busy = false;
        state.prompt_open = true;
        assert!(!state.can_check());
        state.prompt_open = false;
        state.installing = true;
        assert!(!state.can_check());
        state.installing = false;
        state.ready = Some(Arc::new(staged_fixture()));
        assert!(!state.can_check());
    }

    #[gpui::test]
    fn automatic_update_preference_persists_without_changing_documents(
        cx: &mut gpui::TestAppContext,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
        app.update(cx, |app, cx| {
            app.preferences_path = path.clone();
            app.automatically_download_updates = true;
            let version = app.active_tab().document.version();
            let preview = app.active_tab().document.preview_blocks_shared();
            let dirty = app.active_tab().is_dirty();
            app.toggle_automatic_update_downloads(cx);
            assert!(!app.automatically_download_updates);
            assert!(!app.current_preferences().automatically_download_updates);
            assert_eq!(app.active_tab().document.version(), version);
            assert_eq!(app.active_tab().is_dirty(), dirty);
            assert!(Arc::ptr_eq(
                &preview,
                &app.active_tab().document.preview_blocks_shared()
            ));
        });
        assert!(
            !load_app_preferences(&path)
                .unwrap()
                .automatically_download_updates
        );
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                app.reset_preferences(&ResetPreferences, window, cx);
            })
        });
        let label = app.update(cx, |app, _| app.tr(Msg::DialogButtonReset).to_string());
        cx.simulate_prompt_answer(&label);
        cx.run_until_parked();
        app.update(cx, |app, _| assert!(app.automatically_download_updates));
        assert!(
            load_app_preferences(&path)
                .unwrap()
                .automatically_download_updates
        );
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[gpui::test]
    fn ready_update_survives_later_and_dirty_restart_rejection(cx: &mut gpui::TestAppContext) {
        let ready = Arc::new(staged_fixture());
        let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
        app.update(cx, |app, _| app.update_state.ready = Some(ready.clone()));
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                app.check_for_updates(&CheckForUpdates, window, cx);
            })
        });
        cx.run_until_parked();
        let later = app.update(cx, |app, _| app.tr(Msg::DialogButtonLater).to_string());
        cx.simulate_prompt_answer(&later);
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert!(Arc::ptr_eq(
                app.update_state.ready.as_ref().unwrap(),
                &ready
            ));
            assert!(!app.update_state.busy);
            assert!(!app.update_state.prompt_open);
        });
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                app.check_for_updates(&CheckForUpdates, window, cx);
            })
        });
        cx.run_until_parked();
        let restart = app.update(cx, |app, _| {
            app.active_tab_mut().document.insert(0, "unsaved");
            app.tr(Msg::DialogButtonRestartAndUpdate).to_string()
        });
        cx.simulate_prompt_answer(&restart);
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert!(app.active_tab().is_dirty());
            assert!(Arc::ptr_eq(
                app.update_state.ready.as_ref().unwrap(),
                &ready
            ));
            assert!(!app.update_state.busy);
            assert!(!app.update_state.installing);
            assert!(read_verified_package(&ready.package).is_ok());
        });
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[gpui::test]
    fn edits_during_staged_read_block_installation(cx: &mut gpui::TestAppContext) {
        let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
        app.update(cx, |app, _| {
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("clean"))];
            app.update_state.ready = Some(Arc::new(staged_fixture()));
        });
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                app.install_ready_update(window.window_handle(), cx);
                assert!(app.update_state.busy);
                app.active_tab_mut()
                    .document
                    .insert(0, "typed during validation ");
            });
        });
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert!(!app.update_state.busy);
            assert!(!app.update_state.installing);
            assert!(app.update_state.ready.is_some());
            assert!(
                app.active_tab()
                    .document
                    .text()
                    .starts_with("typed during validation")
            );
        });
    }

    #[test]
    fn automatic_update_messages_are_localized_and_substitute_versions() {
        for language in Language::all() {
            for message in [
                Msg::PrefPanelUpdatesSection,
                Msg::PrefPanelAutomaticUpdateDownloads,
                Msg::PrefPanelUpdateCheckHint,
                Msg::PrefPanelManualUpdateHint,
                Msg::StatusUpdateInstalling,
                Msg::DialogUpdateReadyTitle,
                Msg::DialogButtonRestartAndUpdate,
            ] {
                assert!(!t(*language, message).trim().is_empty());
            }
            for message in [Msg::StatusUpdateReady, Msg::DialogUpdateReadyDetail] {
                let text = tf(*language, message, &["9.9.9"]);
                assert!(text.contains("9.9.9"));
                assert!(!text.contains("{0}"));
            }
        }
    }

    fn release_with_version(version: &str) -> GitHubRelease {
        GitHubRelease {
            tag_name: format!("v{version}"),
            html_url: format!("https://github.com/willmove/markion/releases/tag/v{version}"),
            assets: [
                ("markion_9.9.9_x64-setup.exe", "windows"),
                ("Markion_9.9.9_aarch64.dmg", "macos"),
                ("Markion_9.9.9_x64.dmg", "macos-x64"),
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
    fn signed_install_requires_a_self_updatable_package_and_a_public_key() {
        let key = Some("encoded-public-key");
        let appimage = Some("/opt/Markion.AppImage");
        assert_eq!(
            update_primary_action_for("windows", "x86_64", key, None),
            UpdatePrimaryAction::SignedInstall
        );
        assert_eq!(
            update_primary_action_for("windows", "x86_64", None, None),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("windows", "x86_64", Some("   "), None),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("windows", "aarch64", key, None),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("linux", "x86_64", key, appimage),
            UpdatePrimaryAction::SignedInstall
        );
        assert_eq!(
            update_primary_action_for("linux", "x86_64", key, None),
            UpdatePrimaryAction::BrowserDownload,
            "deb/rpm installs have no APPIMAGE and must keep the browser fallback"
        );
        assert_eq!(
            update_primary_action_for("linux", "x86_64", key, Some("   ")),
            UpdatePrimaryAction::BrowserDownload
        );
        assert_eq!(
            update_primary_action_for("linux", "x86_64", None, appimage),
            UpdatePrimaryAction::BrowserDownload,
            "untagged/local builds embed no key"
        );
        assert_eq!(
            update_primary_action_for("macos", "aarch64", key, None),
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
            Language::Ko,
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
        assert!(
            source.contains("std::env::var(\"APPIMAGE\")"),
            "the AppImage launch signal gates the Linux signed-install path"
        );
        assert!(
            source.contains("finish_signed_update"),
            "a successful install must relaunch-and-quit (Linux) or defer to the installer (Windows)"
        );
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
            browser_download_url(&release, "windows", "aarch64", false).unwrap(),
            release.html_url
        );
        assert_eq!(
            browser_download_url(&release, "freebsd", "x86_64", false).unwrap(),
            release.html_url
        );
    }

    #[test]
    fn macos_asset_follows_the_cpu_architecture() {
        let release = release_with_version("9.9.9");
        let arm64_url = "https://github.com/willmove/markion/releases/download/v9.9.9/macos";
        let x64_url = "https://github.com/willmove/markion/releases/download/v9.9.9/macos-x64";
        assert_eq!(
            browser_download_url(&release, "macos", "aarch64", false).unwrap(),
            arm64_url
        );
        assert_eq!(
            browser_download_url(&release, "macos", "x86_64", false).unwrap(),
            x64_url,
            "Intel Macs must deep-link the x64 DMG, not fall back to the Release page"
        );
    }

    #[test]
    fn linux_asset_follows_the_distribution_family() {
        let release = release_with_version("9.9.9");
        let appimage_url = "https://github.com/willmove/markion/releases/download/v9.9.9/appimage";
        let deb_url = "https://github.com/willmove/markion/releases/download/v9.9.9/linux";
        let offered = |os_release: Option<&str>, appimage: bool| {
            browser_download_url_with_os_release(&release, "linux", "x86_64", appimage, os_release)
                .unwrap()
        };
        let arch_id = "ID=arch\n";
        let omarchy_id = "ID=omarchy\nID_LIKE=arch\n";
        let quoted_manjaro = "NAME=\"Manjaro Linux\"\nID=manjaro\nID_LIKE=\"arch\"\n";
        let endeavour = "ID=endeavouros\nID_LIKE=arch\n";
        let debian = "ID=debian\nID_LIKE=debian\n";
        for os_release in [
            Some(arch_id),
            Some(omarchy_id),
            Some(quoted_manjaro),
            Some(endeavour),
        ] {
            assert_eq!(
                offered(os_release, false),
                appimage_url,
                "pacman-based {os_release:?} should be offered the AppImage"
            );
        }
        for os_release in [Some(debian), None] {
            assert_eq!(
                offered(os_release, false),
                deb_url,
                "deb-based or unknown {os_release:?} should keep the DEB"
            );
        }
        assert_eq!(
            offered(Some(debian), true),
            appimage_url,
            "an AppImage install must be offered the AppImage even on deb systems"
        );
        assert_eq!(
            offered(None, true),
            appimage_url,
            "an AppImage install must be offered the AppImage even when os-release is unreadable"
        );
    }

    #[test]
    fn missing_supported_asset_retains_the_release_page_fallback() {
        let mut release = release_with_version("9.9.9");
        release.assets.clear();
        assert!(browser_download_url(&release, "windows", "x86_64", false).is_err());
        match compare_with_running_for(&release, "windows", "x86_64", false) {
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
