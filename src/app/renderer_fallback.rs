//! Software Vulkan fallback for the Linux AppImage.
//!
//! GPUI's Linux renderer is Vulkan-only. The AppImage carries Mesa's lavapipe
//! driver and a Vulkan loader in `usr/lib/markion-vulkan/` (see
//! `scripts/build-lavapipe.sh`). The host stack is always tried first; only
//! after it failed to start the renderer does Markion re-exec itself once with
//! the loader pointed at the bundled driver. The dynamic linker reads
//! `LD_LIBRARY_PATH` only at process start, hence the re-exec instead of an
//! in-process retry.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]

use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    path::{Path, PathBuf},
};

/// `1`/`force` uses the bundled driver without trying the host one,
/// `0`/`off` disables the fallback; anything else is automatic.
pub(super) const MODE_ENV: &str = "MARKION_SOFTWARE_VULKAN";
/// Set on the re-exec'd process so the fallback runs at most once.
const ACTIVE_ENV: &str = "MARKION_SOFTWARE_VULKAN_ACTIVE";
const STACK_DIR: &str = "markion-vulkan";
const ICD_MANIFEST: &str = "lvp_icd.x86_64.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FallbackMode {
    Auto,
    Force,
    Disabled,
}

impl FallbackMode {
    fn parse(value: Option<&OsStr>) -> Self {
        let value = value
            .and_then(OsStr::to_str)
            .map(|value| value.trim().to_ascii_lowercase());
        match value.as_deref() {
            Some("1" | "force" | "on" | "true" | "yes") => Self::Force,
            Some("0" | "off" | "false" | "no" | "never") => Self::Disabled,
            _ => Self::Auto,
        }
    }

    fn from_env() -> Self {
        Self::parse(env::var_os(MODE_ENV).as_deref())
    }
}

#[derive(Debug)]
pub(super) enum FallbackUnavailable {
    /// This process already runs on the bundled driver.
    AlreadyActive,
    Disabled,
    /// Not an AppImage layout (e.g. the .deb/.rpm install) or not Linux.
    NotBundled,
    ExecFailed(io::Error),
}

pub(super) fn software_stack_active() -> bool {
    env::var_os(ACTIVE_ENV).is_some()
}

/// The bundled stack directory for an executable at `<root>/usr/bin/markion`.
fn bundled_stack_dir(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?.parent()?.join("lib").join(STACK_DIR);
    dir.join(ICD_MANIFEST).is_file().then_some(dir)
}

fn fallback_environment(
    stack: &Path,
    ld_library_path: Option<&OsStr>,
) -> Vec<(&'static str, OsString)> {
    let manifest = stack.join(ICD_MANIFEST).into_os_string();
    let mut library_path = stack.as_os_str().to_os_string();
    if let Some(existing) = ld_library_path.filter(|path| !path.is_empty()) {
        library_path.push(":");
        library_path.push(existing);
    }
    vec![
        ("VK_DRIVER_FILES", manifest.clone()),
        ("VK_ICD_FILENAMES", manifest),
        // Host implicit layers (overlays, Mesa's device_select) were built for
        // the host driver; keep them out of the fallback.
        ("VK_LOADER_LAYERS_DISABLE", "~implicit~".into()),
        ("NODEVICE_SELECT", "1".into()),
        ("LD_LIBRARY_PATH", library_path),
        (ACTIVE_ENV, "1".into()),
    ]
}

/// Replaces the current process with a copy running on the bundled software
/// driver. Returns only when that is not possible.
pub(super) fn restart_on_software_stack() -> FallbackUnavailable {
    if software_stack_active() {
        return FallbackUnavailable::AlreadyActive;
    }
    if FallbackMode::from_env() == FallbackMode::Disabled {
        return FallbackUnavailable::Disabled;
    }
    restart_with_bundled_stack()
}

#[cfg(target_os = "linux")]
fn restart_with_bundled_stack() -> FallbackUnavailable {
    use std::os::unix::process::CommandExt as _;

    let exe = match env::current_exe() {
        Ok(exe) => exe,
        Err(error) => return FallbackUnavailable::ExecFailed(error),
    };
    let Some(stack) = bundled_stack_dir(&exe) else {
        return FallbackUnavailable::NotBundled;
    };
    tracing::warn!(
        stack = %stack.display(),
        "restarting Markion on the bundled software Vulkan driver (Mesa lavapipe)"
    );
    let mut args = env::args_os();
    let mut command = std::process::Command::new(&exe);
    if let Some(argv0) = args.next() {
        command.arg0(argv0);
    }
    command.args(args).envs(fallback_environment(
        &stack,
        env::var_os("LD_LIBRARY_PATH").as_deref(),
    ));
    FallbackUnavailable::ExecFailed(command.exec())
}

#[cfg(not(target_os = "linux"))]
fn restart_with_bundled_stack() -> FallbackUnavailable {
    FallbackUnavailable::NotBundled
}

/// Runs before GPUI starts: honors `MARKION_SOFTWARE_VULKAN=1` and records
/// when the process is already running on the bundled driver.
pub(super) fn prepare_startup() {
    if software_stack_active() {
        tracing::warn!(
            "rendering with the bundled software Vulkan driver (Mesa lavapipe); \
             the host Vulkan stack could not start the renderer"
        );
        return;
    }
    if FallbackMode::from_env() == FallbackMode::Force {
        let reason = restart_with_bundled_stack();
        tracing::warn!(
            ?reason,
            "{MODE_ENV}=1 is set but the bundled software Vulkan driver is unavailable; \
             using the host Vulkan stack"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_parsing_accepts_documented_values_and_defaults_to_auto() {
        for value in ["1", "force", " ON ", "true", "yes"] {
            assert_eq!(
                FallbackMode::parse(Some(OsStr::new(value))),
                FallbackMode::Force
            );
        }
        for value in ["0", "off", "False", "no", "never"] {
            assert_eq!(
                FallbackMode::parse(Some(OsStr::new(value))),
                FallbackMode::Disabled
            );
        }
        for value in ["", "auto", "2"] {
            assert_eq!(
                FallbackMode::parse(Some(OsStr::new(value))),
                FallbackMode::Auto
            );
        }
        assert_eq!(FallbackMode::parse(None), FallbackMode::Auto);
    }

    #[test]
    fn bundled_stack_is_found_next_to_the_appimage_binary_only() {
        let root = tempfile::tempdir().unwrap();
        let exe = root.path().join("usr/bin/markion");
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        assert_eq!(bundled_stack_dir(&exe), None);

        let stack = root.path().join("usr/lib/markion-vulkan");
        std::fs::create_dir_all(&stack).unwrap();
        assert_eq!(bundled_stack_dir(&exe), None, "manifest is required");

        std::fs::write(stack.join(ICD_MANIFEST), "{}").unwrap();
        assert_eq!(bundled_stack_dir(&exe), Some(stack));
        assert_eq!(bundled_stack_dir(Path::new("markion")), None);
    }

    #[cfg(unix)]
    #[test]
    fn fallback_environment_points_the_loader_at_the_bundled_driver() {
        let stack = Path::new("/tmp/.mount_x/usr/lib/markion-vulkan");
        let env = fallback_environment(stack, Some(OsStr::new("/tmp/.mount_x/usr/lib")));
        let get = |name: &str| {
            env.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| value.to_str().unwrap().to_string())
        };
        let manifest = "/tmp/.mount_x/usr/lib/markion-vulkan/lvp_icd.x86_64.json";
        assert_eq!(get("VK_DRIVER_FILES").as_deref(), Some(manifest));
        assert_eq!(get("VK_ICD_FILENAMES").as_deref(), Some(manifest));
        assert_eq!(
            get("LD_LIBRARY_PATH").as_deref(),
            Some("/tmp/.mount_x/usr/lib/markion-vulkan:/tmp/.mount_x/usr/lib")
        );
        assert_eq!(
            get("VK_LOADER_LAYERS_DISABLE").as_deref(),
            Some("~implicit~")
        );
        assert_eq!(get(ACTIVE_ENV).as_deref(), Some("1"));

        let env = fallback_environment(stack, Some(OsStr::new("")));
        let library_path = env.iter().find(|(key, _)| *key == "LD_LIBRARY_PATH").unwrap();
        assert_eq!(library_path.1, OsString::from(stack.as_os_str()));
    }
}
