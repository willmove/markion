//! Platform-specific default locations for recovery files, config, and themes.

use std::{
    env,
    path::{Component, Path, PathBuf},
};

pub fn default_recovery_dir() -> PathBuf {
    if cfg!(windows) {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("Markion")
            .join("Recovery")
    } else {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
            .unwrap_or_else(env::temp_dir)
            .join("markion")
            .join("recovery")
    }
}

pub fn default_config_dir() -> PathBuf {
    if cfg!(windows) {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("Markion")
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_else(env::temp_dir)
            .join("markion")
    }
}

pub fn default_preferences_path() -> PathBuf {
    default_config_dir().join("config.toml")
}

pub fn default_session_path() -> PathBuf {
    default_config_dir().join("session.toml")
}

pub fn default_log_dir() -> PathBuf {
    if cfg!(windows) {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
            .join("Markion")
            .join("Logs")
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library").join("Logs"))
            .unwrap_or_else(env::temp_dir)
            .join("Markion")
    } else {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
            .unwrap_or_else(env::temp_dir)
            .join("markion")
            .join("logs")
    }
}

pub fn default_themes_dir() -> PathBuf {
    default_config_dir().join("themes")
}

/// Locate a file that ships with Markion.
///
/// Packaged builds copy `assets/` next to the executable (`packager.toml`
/// `resources`). Some packager layouts nest that tree under `resources/` or
/// macOS `Contents/Resources/`, matching `discover_workspace_assets`.
/// Development builds fall back to the compile-time crate directory.
///
/// `relative` is a path under the resource root such as `assets/markion.png`.
/// Absolute paths and parent-directory components are rejected so a Markdown
/// image URL cannot escape the bundle.
pub fn bundled_resource_path(relative: impl AsRef<Path>) -> Option<PathBuf> {
    let relative = relative.as_ref();
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return None;
    }

    let mut candidates = Vec::new();
    if let Ok(executable) = env::current_exe()
        && let Some(binary_dir) = executable.parent()
    {
        candidates.push(binary_dir.join(relative));
        candidates.push(binary_dir.join("resources").join(relative));
        if let Some(contents_dir) = binary_dir.parent() {
            candidates.push(contents_dir.join("Resources").join(relative));
        }
    }
    candidates.push(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative));
    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::bundled_resource_path;
    use std::path::Path;

    #[test]
    fn bundled_welcome_logo_resolves_in_dev_builds() {
        let path = bundled_resource_path("assets/markion.png").expect("bundled logo");
        assert!(path.is_file());
        assert_eq!(path.file_name().unwrap(), "markion.png");
    }

    #[test]
    fn missing_or_escaping_relative_paths_do_not_resolve() {
        assert!(bundled_resource_path("assets/no-such-markion-welcome.png").is_none());
        assert!(bundled_resource_path("../Cargo.toml").is_none());
        assert!(bundled_resource_path(Path::new("")).is_none());
    }
}
