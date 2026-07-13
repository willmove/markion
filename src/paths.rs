//! Platform-specific default locations for recovery files, config, and themes.

use std::{env, path::PathBuf};

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
