//! Persistence and file-system layer.
//!
//! Submodules:
//! - [`file_tree`]: workspace scanning and entry CRUD
//! - [`preferences`]: app preference file (TOML `config.toml`)
//! - [`session`]: session / recent-files file (TOML `session.toml`)
//! - [`logging`]: diagnostic file logging
//! - [`theme_file`]: user `.toml` custom theme files (with `.theme` migration)
//! - [`recovery`]: crash-recovery copies
//! - [`atomic`]: same-directory durable atomic file replacement

pub mod atomic;
pub mod file_tree;
pub mod logging;
pub mod preferences;
pub mod recovery;
pub mod resources;
pub mod session;
pub mod theme_file;

pub use atomic::atomic_write;

pub use file_tree::{
    FileTree, FileTreeEntry, FileTreeEntryKind, FileTreeFileKind, MARKDOWN_EXTENSIONS,
    TEXT_EXTENSIONS, is_markdown_path, is_text_path,
};
pub use logging::init_logging;
pub use preferences::{
    load_app_preferences, parse_app_preferences, parse_legacy_app_preferences,
    render_app_preferences, save_app_preferences,
};
pub use recovery::{
    RecoveryInventoryEntry, RecoverySourceState, delete_recovery_file, inspect_recovery_files,
    list_recovery_files, load_recovery_file,
};
pub use resources::{
    IMAGE_EXTENSIONS, ImportedImage, image_extension_supported, import_image_bytes,
    import_image_file,
};
pub use session::{
    load_session_state, parse_session_state, render_session_state, save_session_state,
};
pub use theme_file::{
    list_theme_definitions, load_theme_definition, parse_theme_definition, render_theme_definition,
    save_theme_definition,
};
