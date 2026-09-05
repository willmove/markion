//! Session and recent-files persistence (`session.toml`).
//!
//! Keeps workspace continuity separate from [`crate::storage::preferences`]:
//! every field is optional and defaults to an empty session.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::model::{
    MAX_RECENT_FILES, MAX_RECENT_WORKSPACES, SessionLayout, SessionState, WorkspaceSnapshot,
};
use crate::storage::atomic_write;

/// Serde-facing shape of `session.toml`. Kept separate so `model` stays
/// dependency-free. Missing fields default to empty / none.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
struct SessionFile {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    workspaces: Vec<WorkspaceFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workspace_root: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    open_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_file: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    recent_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    layout: Option<LayoutFile>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
struct WorkspaceFile {
    root: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    open_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_file: Option<String>,
}

/// `[layout]` table: every field optional; unknown keys are ignored.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
struct LayoutFile {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    x: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    y: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    width: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    height: Option<f32>,
    #[serde(default, deserialize_with = "deserialize_bool_or_false")]
    maximized: bool,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    sidebar_width: Option<f32>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_f32",
        skip_serializing_if = "Option::is_none"
    )]
    editor_split_ratio: Option<f32>,
}

impl From<&SessionLayout> for LayoutFile {
    fn from(layout: &SessionLayout) -> Self {
        Self {
            x: layout.x,
            y: layout.y,
            width: layout.width,
            height: layout.height,
            maximized: layout.maximized,
            sidebar_width: layout.sidebar_width,
            editor_split_ratio: layout.editor_split_ratio,
        }
    }
}

impl From<LayoutFile> for SessionLayout {
    fn from(file: LayoutFile) -> Self {
        Self {
            x: file.x,
            y: file.y,
            width: file.width,
            height: file.height,
            maximized: file.maximized,
            sidebar_width: file.sidebar_width,
            editor_split_ratio: file.editor_split_ratio,
        }
    }
}

impl From<&WorkspaceSnapshot> for WorkspaceFile {
    fn from(snapshot: &WorkspaceSnapshot) -> Self {
        Self {
            root: snapshot.root.display().to_string(),
            open_files: snapshot
                .open_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            active_file: snapshot
                .active_file
                .as_ref()
                .map(|path| path.display().to_string()),
        }
    }
}

fn parse_path_string(path: impl AsRef<str>) -> Option<PathBuf> {
    let trimmed = path.as_ref().trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(sanitize_persisted_path(PathBuf::from(trimmed)))
    }
}

fn parse_path_list(paths: impl IntoIterator<Item = String>) -> Vec<PathBuf> {
    paths.into_iter().filter_map(parse_path_string).collect()
}

impl From<WorkspaceFile> for Option<WorkspaceSnapshot> {
    fn from(file: WorkspaceFile) -> Self {
        let root = parse_path_string(file.root)?;
        let open_files = parse_path_list(file.open_files);
        let active_file = file
            .active_file
            .and_then(parse_path_string)
            .filter(|path| open_files.iter().any(|open| open == path));
        Some(WorkspaceSnapshot {
            root,
            open_files,
            active_file,
        })
    }
}

impl From<&SessionState> for SessionFile {
    fn from(session: &SessionState) -> Self {
        let current = session.current_workspace();
        Self {
            workspaces: session
                .workspaces
                .iter()
                .map(WorkspaceFile::from)
                .collect(),
            workspace_root: current
                .map(|snapshot| snapshot.root.display().to_string())
                .or_else(|| {
                    session
                        .workspace_root
                        .as_ref()
                        .map(|path| path.display().to_string())
                }),
            open_files: current
                .map(|snapshot| {
                    snapshot
                        .open_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect()
                })
                .unwrap_or_else(|| {
                    session
                        .open_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect()
                }),
            active_file: current
                .and_then(|snapshot| snapshot.active_file.as_ref())
                .or(session.active_file.as_ref())
                .map(|path| path.display().to_string()),
            recent_files: session
                .recent_files
                .iter()
                .map(|path| path.display().to_string())
                .collect(),
            layout: (!session.layout.is_empty()).then(|| LayoutFile::from(&session.layout)),
        }
    }
}

/// Heals paths persisted by earlier versions, which on Windows could carry
/// the verbatim `\\?\` prefix from `std::fs::canonicalize`. `dunce::simplified`
/// drops the prefix only when the shortened path still resolves to the same
/// file; it is the identity on other platforms and for paths that genuinely
/// require extended-length syntax.
fn sanitize_persisted_path(path: PathBuf) -> PathBuf {
    dunce::simplified(&path).to_path_buf()
}

fn dedupe_recent_files(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::with_capacity(paths.len().min(MAX_RECENT_FILES));
    for path in paths {
        if deduped.iter().any(|existing| existing == &path) {
            continue;
        }
        deduped.push(path);
        if deduped.len() == MAX_RECENT_FILES {
            break;
        }
    }
    deduped
}

fn dedupe_workspaces(mut workspaces: Vec<WorkspaceSnapshot>) -> Vec<WorkspaceSnapshot> {
    let mut deduped: Vec<WorkspaceSnapshot> =
        Vec::with_capacity(workspaces.len().min(MAX_RECENT_WORKSPACES));
    for snapshot in workspaces.drain(..) {
        if deduped.iter().any(|existing| existing.root == snapshot.root) {
            continue;
        }
        deduped.push(snapshot);
        if deduped.len() == MAX_RECENT_WORKSPACES {
            break;
        }
    }
    deduped
}

impl From<SessionFile> for SessionState {
    fn from(file: SessionFile) -> Self {
        let recent_files = dedupe_recent_files(parse_path_list(file.recent_files));

        let mut workspaces: Vec<WorkspaceSnapshot> = file
            .workspaces
            .into_iter()
            .filter_map(Option::<WorkspaceSnapshot>::from)
            .collect();
        workspaces = dedupe_workspaces(workspaces);

        // Legacy single-snapshot files: synthesize one workspace from top-level fields.
        if workspaces.is_empty() {
            let open_files = parse_path_list(file.open_files);
            let active_file = file
                .active_file
                .and_then(parse_path_string)
                .filter(|path| open_files.iter().any(|open| open == path));
            let workspace_root = file.workspace_root.and_then(parse_path_string);
            if let Some(root) = workspace_root {
                workspaces.push(WorkspaceSnapshot {
                    root,
                    open_files,
                    active_file,
                });
            } else if !open_files.is_empty() {
                // No root but open files — keep aliases only via a synthetic rootless skip;
                // without a root we cannot form a workspace slot.
            }
        }

        let mut session = SessionState {
            workspaces,
            workspace_root: None,
            open_files: Vec::new(),
            active_file: None,
            recent_files,
            layout: file.layout.map(SessionLayout::from).unwrap_or_default(),
        };
        session.sync_aliases_from_current();
        session
    }
}

fn deserialize_optional_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(match value {
        toml::Value::Float(number) if number.is_finite() => Some(number as f32),
        toml::Value::Integer(number) => Some(number as f32),
        _ => None,
    })
}

fn deserialize_bool_or_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(value.as_bool().unwrap_or(false))
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
    atomic_write(path, render_session_state(session).as_bytes())
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
    use crate::model::{MAX_RECENT_FILES, MAX_RECENT_WORKSPACES, touch_recent_file, touch_workspace_snapshot};
    use std::path::PathBuf;

    #[test]
    fn missing_session_file_loads_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.toml");
        assert_eq!(load_session_state(&path).unwrap(), SessionState::default());
    }

    #[test]
    fn session_roundtrip_preserves_workspaces() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.toml");
        let mut session = SessionState {
            workspaces: vec![
                WorkspaceSnapshot::new(
                    PathBuf::from("D:/Notes"),
                    vec![
                        PathBuf::from("D:/Notes/a.md"),
                        PathBuf::from("D:/Notes/cover.png"),
                    ],
                    Some(PathBuf::from("D:/Notes/a.md")),
                ),
                WorkspaceSnapshot::new(
                    PathBuf::from("D:/Blog"),
                    vec![PathBuf::from("D:/Blog/post.md")],
                    Some(PathBuf::from("D:/Blog/post.md")),
                ),
            ],
            workspace_root: None,
            open_files: Vec::new(),
            active_file: None,
            recent_files: vec![
                PathBuf::from("D:/Notes/a.md"),
                PathBuf::from("D:/Other/c.md"),
            ],
            layout: SessionLayout {
                x: Some(120.0),
                y: Some(80.0),
                width: Some(1420.0),
                height: Some(900.0),
                maximized: false,
                sidebar_width: Some(280.0),
                editor_split_ratio: Some(0.42),
            },
        };
        session.sync_aliases_from_current();

        save_session_state(&path, &session).unwrap();
        assert_eq!(load_session_state(&path).unwrap(), session);

        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("[[workspaces]]"));
        assert!(written.contains("workspace_root"));
        assert!(written.contains("open_files"));
        assert!(written.contains("recent_files"));
        assert!(written.contains("[layout]"));
        assert!(written.contains("sidebar_width"));
    }

    #[test]
    fn legacy_top_level_fields_synthesize_one_workspace() {
        let parsed = parse_session_state(
            r#"
workspace_root = "D:/Notes"
open_files = ["D:/Notes/a.md", "D:/Notes/b.md"]
active_file = "D:/Notes/b.md"
recent_files = ["D:/Notes/b.md"]
"#,
        )
        .unwrap();
        assert_eq!(parsed.workspaces.len(), 1);
        assert_eq!(parsed.workspaces[0].root, PathBuf::from("D:/Notes"));
        assert_eq!(
            parsed.workspaces[0].open_files,
            vec![
                PathBuf::from("D:/Notes/a.md"),
                PathBuf::from("D:/Notes/b.md")
            ]
        );
        assert_eq!(
            parsed.workspaces[0].active_file,
            Some(PathBuf::from("D:/Notes/b.md"))
        );
        assert_eq!(parsed.workspace_root, Some(PathBuf::from("D:/Notes")));
        assert_eq!(parsed.active_file, Some(PathBuf::from("D:/Notes/b.md")));
    }

    #[test]
    fn partial_toml_takes_defaults() {
        let parsed = parse_session_state("workspace_root = \"D:/Notes\"\n").unwrap();
        assert_eq!(parsed.workspace_root, Some(PathBuf::from("D:/Notes")));
        assert!(parsed.open_files.is_empty());
        assert!(parsed.active_file.is_none());
        assert!(parsed.recent_files.is_empty());
        assert!(parsed.layout.is_empty());
        assert_eq!(parsed.workspaces.len(), 1);
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
        assert_eq!(
            recent[0],
            PathBuf::from(format!("D:/f{}.md", MAX_RECENT_FILES + 2))
        );

        touch_recent_file(&mut recent, PathBuf::from("D:/f5.md"), MAX_RECENT_FILES);
        assert_eq!(recent[0], PathBuf::from("D:/f5.md"));
        assert_eq!(
            recent
                .iter()
                .filter(|path| path == &&PathBuf::from("D:/f5.md"))
                .count(),
            1
        );
    }

    #[test]
    fn workspace_list_dedupes_and_caps() {
        let mut workspaces = Vec::new();
        for i in 0..(MAX_RECENT_WORKSPACES + 2) {
            touch_workspace_snapshot(
                &mut workspaces,
                WorkspaceSnapshot::new(PathBuf::from(format!("D:/w{i}")), Vec::new(), None),
                MAX_RECENT_WORKSPACES,
            );
        }
        assert_eq!(workspaces.len(), MAX_RECENT_WORKSPACES);
        assert_eq!(
            workspaces[0].root,
            PathBuf::from(format!("D:/w{}", MAX_RECENT_WORKSPACES + 1))
        );

        touch_workspace_snapshot(
            &mut workspaces,
            WorkspaceSnapshot::new(
                PathBuf::from("D:/w3"),
                vec![PathBuf::from("D:/w3/a.md")],
                Some(PathBuf::from("D:/w3/a.md")),
            ),
            MAX_RECENT_WORKSPACES,
        );
        assert_eq!(workspaces[0].root, PathBuf::from("D:/w3"));
        assert_eq!(workspaces[0].open_files, vec![PathBuf::from("D:/w3/a.md")]);
        assert_eq!(
            workspaces
                .iter()
                .filter(|entry| entry.root == PathBuf::from("D:/w3"))
                .count(),
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
[[workspaces]]
root = "D:/Notes"
open_files = ["", "D:/Notes/a.md"]
active_file = ""
"#,
        )
        .unwrap();
        assert_eq!(parsed.workspaces.len(), 1);
        assert_eq!(parsed.workspaces[0].root, PathBuf::from("D:/Notes"));
        assert_eq!(
            parsed.workspaces[0].open_files,
            vec![PathBuf::from("D:/Notes/a.md")]
        );
        assert!(parsed.workspaces[0].active_file.is_none());
        assert_eq!(parsed.recent_files, vec![PathBuf::from("D:/recent.md")]);
    }

    #[test]
    fn parse_session_state_heals_legacy_verbatim_paths() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("vault");
        let file = root.join("a.md");
        fs::create_dir_all(&root).unwrap();
        fs::write(&file, "# a").unwrap();

        let verbatim_root = fs::canonicalize(&root).unwrap().display().to_string();
        let verbatim_file = fs::canonicalize(&file).unwrap().display().to_string();

        let parsed = parse_session_state(&format!(
            "recent_files = [{file}]\n[[workspaces]]\nroot = {root}\nopen_files = [{file}]\nactive_file = {file}\n",
            root = toml_quote(&verbatim_root),
            file = toml_quote(&verbatim_file),
        ))
        .unwrap();

        assert_eq!(parsed.workspaces.len(), 1, "workspaces={:?}", parsed.workspaces);
        assert_eq!(
            parsed.workspaces[0].open_files.len(),
            1,
            "workspace open_files={:?}",
            parsed.workspaces[0].open_files
        );

        for path in parsed
            .workspace_root
            .iter()
            .chain(parsed.open_files.iter())
            .chain(parsed.active_file.iter())
            .chain(parsed.recent_files.iter())
            .chain(parsed.workspaces.iter().map(|w| &w.root))
            .chain(parsed.workspaces.iter().flat_map(|w| w.open_files.iter()))
        {
            let text = path.display().to_string();
            assert!(!text.starts_with(r"\\?\"));
            assert!(!text.starts_with(r"\\?\UNC\"));
        }

        assert_eq!(
            fs::canonicalize(parsed.workspace_root.as_ref().unwrap()).unwrap(),
            fs::canonicalize(&root).unwrap()
        );
        assert_eq!(parsed.open_files.len(), 1);
        assert_eq!(parsed.recent_files.len(), 1);
        assert_eq!(parsed.workspaces.len(), 1);
    }

    #[test]
    fn missing_layout_table_keeps_defaults() {
        let parsed = parse_session_state(
            r#"
workspace_root = "D:/Notes"
open_files = ["D:/Notes/a.md"]
"#,
        )
        .unwrap();
        assert_eq!(parsed.workspace_root, Some(PathBuf::from("D:/Notes")));
        assert_eq!(parsed.open_files, vec![PathBuf::from("D:/Notes/a.md")]);
        assert!(parsed.layout.is_empty());
        assert_eq!(
            parsed.layout.normalized_window_size(),
            (
                crate::model::DEFAULT_WINDOW_WIDTH,
                crate::model::DEFAULT_WINDOW_HEIGHT
            )
        );
        assert_eq!(
            parsed.layout.normalized_sidebar_width(),
            crate::model::DEFAULT_SIDEBAR_WIDTH
        );
        assert_eq!(
            parsed.layout.normalized_split_ratio(),
            crate::model::DEFAULT_EDITOR_SPLIT_RATIO
        );
    }

    #[test]
    fn partial_and_invalid_layout_fields_degrade() {
        let parsed = parse_session_state(
            r#"
[layout]
width = 1420
sidebar_width = "wide"
editor_split_ratio = true
unknown_future_key = 1
"#,
        )
        .unwrap();
        assert_eq!(parsed.layout.width, Some(1420.0));
        assert!(parsed.layout.height.is_none());
        assert!(parsed.layout.sidebar_width.is_none());
        assert!(parsed.layout.editor_split_ratio.is_none());
        assert!(!parsed.layout.maximized);
        assert_eq!(
            parsed.layout.normalized_sidebar_width(),
            crate::model::DEFAULT_SIDEBAR_WIDTH
        );
        assert_eq!(
            crate::model::normalize_window_size(Some(100.0), Some(100.0)),
            (
                crate::model::DEFAULT_WINDOW_WIDTH,
                crate::model::DEFAULT_WINDOW_HEIGHT
            )
        );
        assert_eq!(
            crate::model::normalize_sidebar_width(Some(12.0)),
            crate::model::SIDEBAR_MIN_WIDTH
        );
        assert_eq!(
            crate::model::normalize_sidebar_width(Some(900.0)),
            crate::model::SIDEBAR_MAX_WIDTH
        );
        assert_eq!(
            crate::model::normalize_editor_split_ratio(Some(0.01)),
            crate::model::EDITOR_SPLIT_RATIO_MIN
        );
        assert_eq!(
            crate::model::normalize_editor_split_ratio(Some(0.99)),
            crate::model::EDITOR_SPLIT_RATIO_MAX
        );
        assert!(!crate::model::layout_rect_is_visible(
            (5000.0, 5000.0, 800.0, 600.0),
            &[(0.0, 0.0, 1920.0, 1080.0)]
        ));
        assert!(crate::model::layout_rect_is_visible(
            (100.0, 80.0, 800.0, 600.0),
            &[(0.0, 0.0, 1920.0, 1080.0)]
        ));
    }

    /// Minimal TOML basic-string quoting for single-line Windows paths
    /// (backslashes are the only escape-relevant characters here).
    fn toml_quote(path: &str) -> String {
        format!("\"{}\"", path.replace('\\', "\\\\"))
    }
}
