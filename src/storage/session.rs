//! Session and recent-files persistence (`session.toml`).
//!
//! Keeps workspace continuity separate from [`crate::storage::preferences`]:
//! every field is optional and defaults to an empty session.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::model::{SessionState, MAX_RECENT_FILES};

/// Serde-facing shape of `session.toml`. Kept separate so `model` stays
/// dependency-free. Missing fields default to empty / none.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
struct SessionFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_root: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    open_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_file: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    recent_files: Vec<String>,
}

impl From<&SessionState> for SessionFile {
    fn from(session: &SessionState) -> Self {
        Self {
            workspace_root: session
                .workspace_root
                .as_ref()
                .map(|path| path.display().to_string()),
            open_files: session
                .open_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            active_file: session
                .active_file
                .as_ref()
                .map(|path| path.display().to_string()),
            recent_files: session
                .recent_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
        }
    }
}

impl From<SessionFile> for SessionState {
    fn from(file: SessionFile) -> Self {
        let mut recent_files = file
            .recent_files
            .into_iter()
            .map(|path| path.trim().to_string())
            .filter(|path| !path.is_empty())
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>();
        // Preserve on-disk order (most recent first) while capping length and
        // dropping later duplicates.
        let mut deduped = Vec::with_capacity(recent_files.len().min(MAX_RECENT_FILES));
        for path in recent_files.drain(..) {
            if deduped.iter().any(|existing| existing == &path) {
                continue;
            }
            deduped.push(path);
            if deduped.len() == MAX_RECENT_FILES {
                break;
            }
        }
        recent_files = deduped;

        let open_files = file
            .open_files
            .into_iter()
            .map(|path| path.trim().to_string())
            .filter(|path| !path.is_empty())
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>();

        let active_file = file
            .active_file
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(std::path::PathBuf::from);

        let workspace_root = file
            .workspace_root
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(std::path::PathBuf::from);

        Self {
            workspace_root,
            open_files,
            active_file,
            recent_files,
        }
    }
}

/// Loads session state from `path`. Missing files yield the default empty session.
pub fn load_session_state(path: impl AsRef<Path>) -> io::Result<SessionState> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(SessionState::default());
    }
    parse_session_state(&fs::read_to_string(path)?)
}

/// Writes session state to `path`, creating parent directories as needed.
pub fn save_session_state(path: impl AsRef<Path>, session: &SessionState) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, render_session_state(session))
}

/// Parses the TOML session format. Missing fields take their defaults.
pub fn parse_session_state(text: &str) -> io::Result<SessionState> {
    let file: SessionFile = toml::from_str(text)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
    Ok(file.into())
}

/// Renders session state as TOML (the on-disk `session.toml` format).
pub fn render_session_state(session: &SessionState) -> String {
    toml::to_string_pretty(&SessionFile::from(session)).expect("session serialize to TOML")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{touch_recent_file, MAX_RECENT_FILES};
    use std::path::PathBuf;

    #[test]
    fn missing_session_file_loads_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.toml");
        assert_eq!(load_session_state(&path).unwrap(), SessionState::default());
    }

    #[test]
    fn session_roundtrip_preserves_paths() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.toml");
        let session = SessionState {
            workspace_root: Some(PathBuf::from("D:/Notes")),
            open_files: vec![
                PathBuf::from("D:/Notes/a.md"),
                PathBuf::from("D:/Notes/b.md"),
            ],
            active_file: Some(PathBuf::from("D:/Notes/b.md")),
            recent_files: vec![
                PathBuf::from("D:/Notes/b.md"),
                PathBuf::from("D:/Other/c.md"),
            ],
        };

        save_session_state(&path, &session).unwrap();
        assert_eq!(load_session_state(&path).unwrap(), session);

        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("workspace_root"));
        assert!(written.contains("open_files"));
        assert!(written.contains("recent_files"));
    }

    #[test]
    fn partial_toml_takes_defaults() {
        let parsed = parse_session_state("workspace_root = \"D:/Notes\"\n").unwrap();
        assert_eq!(parsed.workspace_root, Some(PathBuf::from("D:/Notes")));
        assert!(parsed.open_files.is_empty());
        assert!(parsed.active_file.is_none());
        assert!(parsed.recent_files.is_empty());
    }

    #[test]
    fn recent_list_dedupes_and_caps() {
        let mut recent = Vec::new();
        for i in 0..(MAX_RECENT_FILES + 3) {
            touch_recent_file(
                &mut recent,
                PathBuf::from(format!("D:/f{i}.md")),
                MAX_RECENT_FILES,
            );
        }
        assert_eq!(recent.len(), MAX_RECENT_FILES);
        assert_eq!(recent[0], PathBuf::from(format!("D:/f{}.md", MAX_RECENT_FILES + 2)));

        touch_recent_file(&mut recent, PathBuf::from("D:/f5.md"), MAX_RECENT_FILES);
        assert_eq!(recent[0], PathBuf::from("D:/f5.md"));
        assert_eq!(
            recent.iter().filter(|path| path == &&PathBuf::from("D:/f5.md")).count(),
            1
        );
    }

    #[test]
    fn empty_path_strings_are_ignored() {
        let parsed = parse_session_state(
            r#"
workspace_root = "  "
open_files = ["", " D:/ok.md ", ""]
active_file = ""
recent_files = ["", "D:/recent.md"]
"#,
        )
        .unwrap();
        assert!(parsed.workspace_root.is_none());
        assert_eq!(parsed.open_files, vec![PathBuf::from("D:/ok.md")]);
        assert!(parsed.active_file.is_none());
        assert_eq!(parsed.recent_files, vec![PathBuf::from("D:/recent.md")]);
    }
}
