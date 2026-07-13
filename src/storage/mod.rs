//! Persistence and file-system layer.
//!
//! Submodules:
//! - [`file_tree`]: workspace scanning and entry CRUD
//! - [`preferences`]: app preference file (TOML `config.toml`)
//! - [`logging`]: diagnostic file logging
//! - [`theme_file`]: user `.toml` custom theme files (with `.theme` migration)
//! - [`recovery`]: crash-recovery copies

pub mod file_tree;
pub mod logging;
pub mod preferences;
pub mod recovery;
pub mod theme_file;

pub use file_tree::{
    FileTree, FileTreeEntry, FileTreeEntryKind, MARKDOWN_EXTENSIONS, is_markdown_path,
};
pub use logging::init_logging;
pub use preferences::{
    load_app_preferences, parse_app_preferences, parse_legacy_app_preferences,
    render_app_preferences, save_app_preferences,
};
pub use recovery::{delete_recovery_file, list_recovery_files, load_recovery_file};
pub use theme_file::{
    list_theme_definitions, load_theme_definition, parse_theme_definition, render_theme_definition,
    save_theme_definition,
};
