//! Crash-recovery copy management: list, load, delete, and name generation.

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{DiskIdentity, model::RecoveryDocument};

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
    if payload.starts_with("markion-recovery-v2\n") {
        return load_recovery_v2(&payload);
    }
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
        disk_identity: None,
        text,
    })
}

fn load_recovery_v2(payload: &str) -> io::Result<RecoveryDocument> {
    let (header, text) = payload.split_once("\n---\n").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "recovery file is missing body marker",
        )
    })?;
    let mut original_path = None;
    let mut len = None;
    let mut digest = None;
    let mut modified = None;
    for line in header.lines().skip(1) {
        if let Some(value) = line.strip_prefix("path:").filter(|value| !value.is_empty()) {
            original_path = Some(PathBuf::from(value));
        } else if let Some(value) = line.strip_prefix("disk-len:") {
            len = value.parse::<u64>().ok();
        } else if let Some(value) = line.strip_prefix("disk-digest:") {
            digest = value.parse::<u64>().ok();
        } else if let Some(value) = line.strip_prefix("disk-modified-ms:") {
            modified = value
                .parse::<u64>()
                .ok()
                .map(|millis| UNIX_EPOCH + std::time::Duration::from_millis(millis));
        }
    }
    let disk_identity = match (len, digest) {
        (Some(len), Some(digest)) => Some(DiskIdentity {
            modified,
            len,
            digest,
        }),
        _ => None,
    };
    Ok(RecoveryDocument {
        original_path,
        disk_identity,
        text: text.to_string(),
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

pub(crate) fn stable_recovery_file_path(
    dir: &Path,
    original_path: Option<&Path>,
    recovery_id: u64,
) -> PathBuf {
    let label = original_path
        .and_then(Path::file_stem)
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("untitled");
    dir.join(format!(
        "{}-{recovery_id:016x}.md",
        sanitize_file_label(label)
    ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarkdownDocument;

    #[test]
    fn recovery_v2_roundtrip_preserves_identity_and_uses_stable_path() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("important.md");
        let recovery_dir = dir.path().join("recovery");
        let mut document = MarkdownDocument::from_text("saved");
        document.save_as(&source).unwrap();
        document.set_text("dirty one");
        let first = document
            .save_recovery_copy_with_id(&recovery_dir, 42)
            .unwrap();
        document.set_text("dirty two");
        let second = document
            .save_recovery_copy_with_id(&recovery_dir, 42)
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(
            list_recovery_files(&recovery_dir).unwrap(),
            vec![first.clone()]
        );

        let recovered = load_recovery_file(&first).unwrap();
        assert_eq!(recovered.original_path.as_deref(), Some(source.as_path()));
        assert_eq!(recovered.text, "dirty two");
        let recovered_identity = recovered.disk_identity.unwrap();
        let document_identity = document.disk_identity().unwrap();
        assert_eq!(recovered_identity.len, document_identity.len);
        assert_eq!(recovered_identity.digest, document_identity.digest);
        assert_eq!(
            recovered_identity
                .modified
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            document_identity
                .modified
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
    }

    #[test]
    fn legacy_v1_recovery_remains_readable() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("legacy.md");
        fs::write(&path, "markion-recovery-v1\npath:D:/note.md\n---\nold text").unwrap();
        let recovered = load_recovery_file(&path).unwrap();
        assert_eq!(recovered.original_path, Some(PathBuf::from("D:/note.md")));
        assert!(recovered.disk_identity.is_none());
        assert_eq!(recovered.text, "old text");
    }
}
