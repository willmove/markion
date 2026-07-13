//! Diagnostic file logging: daily-rotated plain-text logs in the Markion log
//! directory, with a compact console layer for development runs.
//!
//! Design adopted from Typune's filesystem crate logger (daily rotation, keep
//! 7 files, `RUST_LOG` override), with two deliberate deviations: plain-text
//! file output instead of JSON (user-serviceable logs for a desktop app), and
//! no crash sentinel (Markion's recovery subsystem covers crash handling).

use std::path::{Path, PathBuf};

use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Number of daily log files retained.
const MAX_LOG_FILES: usize = 7;

/// Initializes tracing with a daily-rotated file layer in the default Markion
/// log directory plus a compact console layer. Returns the log directory on
/// success. Failures (unwritable directory, subscriber already set) are
/// swallowed — logging must never prevent the editor from starting.
pub fn init_logging() -> Option<PathBuf> {
    let log_dir = crate::paths::default_log_dir();
    init_logging_to(&log_dir).ok()?;
    Some(log_dir)
}

/// Initializes tracing writing to `log_dir`. Split from [`init_logging`] so
/// tests can target a temp directory.
pub fn init_logging_to(log_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(log_dir)?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("markion")
        .filename_suffix("log")
        .max_log_files(MAX_LOG_FILES)
        .build(log_dir)?;

    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(file_appender)
        .with_target(true);

    let console_layer = fmt::layer().compact().with_target(false);

    // RUST_LOG wins when set; otherwise default to info.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_writes_to_target_directory() {
        let dir = tempfile::tempdir().unwrap();

        // The global subscriber can only be installed once per process; the
        // first call must succeed and produce a log file, whichever test
        // thread gets there first.
        if init_logging_to(dir.path()).is_ok() {
            tracing::info!("logging smoke event");
            let has_log_file = std::fs::read_dir(dir.path())
                .unwrap()
                .filter_map(Result::ok)
                .any(|entry| entry.file_name().to_string_lossy().starts_with("markion"));
            assert!(has_log_file);
        }
    }
}
