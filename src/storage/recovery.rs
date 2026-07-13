//! Crash-recovery copy management: list, load, delete, and name generation.

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::model::RecoveryDocument;

pub fn list_recovery_files(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

pub fn load_recovery_file(path: impl AsRef<Path>) -> io::Result<RecoveryDocument> {
    let payload = fs::read_to_string(path)?;
    let mut lines = payload.splitn(4, '\n');
    if lines.next() != Some("markion-recovery-v1") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported recovery file format",
        ));
    }
    let original_path = lines
        .next()
        .and_then(|line| line.strip_prefix("path:"))
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);
    if lines.next() != Some("---") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "recovery file is missing body marker",
        ));
    }
    let text = lines.next().unwrap_or_default().to_string();
    Ok(RecoveryDocument {
        original_path,
        text,
    })
}

pub fn delete_recovery_file(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub(crate) fn recovery_file_path(dir: &Path, original_path: Option<&Path>) -> PathBuf {
    let label = original_path
        .and_then(Path::file_stem)
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("untitled");
    let label = sanitize_file_label(label);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    dir.join(format!("{label}-{millis}.md"))
}

fn sanitize_file_label(label: &str) -> String {
    let sanitized = label
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if sanitized.is_empty() {
        "untitled".to_string()
    } else {
        sanitized
    }
}
