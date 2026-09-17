//! Private, restart-visible recovery manifests for explicit image operations.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::ImageRecoveryRecord;

use super::atomic_write;

#[derive(Debug, Clone)]
pub struct ImageRecoveryEntry {
    pub manifest_path: PathBuf,
    pub record: Result<ImageRecoveryRecord, String>,
}

pub fn default_image_recovery_dir() -> PathBuf {
    crate::paths::default_config_dir().join("image-operation-recovery")
}

pub fn save_image_recovery_record(
    directory: &Path,
    record: &ImageRecoveryRecord,
) -> io::Result<PathBuf> {
    ensure_private_directory(directory)?;
    let name = safe_manifest_name(&record.operation_id);
    let path = directory.join(format!("{name}.json"));
    let bytes = serde_json::to_vec_pretty(record)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    atomic_write(&path, &bytes)?;
    Ok(path)
}

pub fn list_image_recovery_records(directory: &Path) -> io::Result<Vec<ImageRecoveryEntry>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let record = fs::read(&path)
            .map_err(|error| error.to_string())
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|error| error.to_string()));
        entries.push(ImageRecoveryEntry {
            manifest_path: path,
            record,
        });
    }
    entries.sort_by(|left, right| left.manifest_path.cmp(&right.manifest_path));
    Ok(entries)
}

/// Explicitly discards one manifest and only the private inputs it owns.
/// Output URLs and final resource paths are intentionally never deleted.
pub fn discard_image_recovery_record(directory: &Path, manifest_path: &Path) -> io::Result<()> {
    let directory = fs::canonicalize(directory)?;
    let manifest = fs::canonicalize(manifest_path)?;
    if !manifest.starts_with(&directory) || manifest.parent() != Some(directory.as_path()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "recovery manifest is outside the private recovery directory",
        ));
    }
    let record: ImageRecoveryRecord = serde_json::from_slice(&fs::read(&manifest)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    for owned in record.owned_inputs {
        let candidate = if owned.is_absolute() {
            owned
        } else {
            directory.join(owned)
        };
        let Ok(canonical) = fs::canonicalize(&candidate) else {
            continue;
        };
        if canonical.starts_with(&directory) && canonical.is_file() {
            fs::remove_file(canonical)?;
        }
    }
    fs::remove_file(manifest)
}

fn safe_manifest_name(value: &str) -> String {
    let value: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect();
    let value = value.trim_matches('-');
    if value.is_empty() {
        "image-operation".into()
    } else {
        value.into()
    }
}

fn ensure_private_directory(directory: &Path) -> io::Result<()> {
    fs::create_dir_all(directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageItemResult, ImageItemState, ImageOccurrenceId};

    fn record(owned: Vec<PathBuf>, output_path: PathBuf) -> ImageRecoveryRecord {
        ImageRecoveryRecord {
            operation_id: "job/中文-1".into(),
            document_path: Some(PathBuf::from("notes/note.md")),
            created_unix_secs: 42,
            items: vec![ImageItemResult {
                occurrence: Some(ImageOccurrenceId(1)),
                state: ImageItemState::Unapplied,
                source: "clipboard".into(),
                output_url: Some("https://cdn.test/a.png?signature=keep".into()),
                output_path: Some(output_path),
                message: None,
            }],
            owned_inputs: owned,
        }
    }

    #[test]
    fn manifest_round_trip_is_restart_visible_and_never_deletes_final_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("private");
        fs::create_dir_all(&directory).unwrap();
        let owned = directory.join("captured.png");
        let output = temp.path().join("notes/assets/final.png");
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::write(&owned, b"input").unwrap();
        fs::write(&output, b"final").unwrap();
        let manifest =
            save_image_recovery_record(&directory, &record(vec![owned.clone()], output.clone()))
                .unwrap();

        let entries = list_image_recovery_records(&directory).unwrap();
        assert_eq!(entries.len(), 1);
        let loaded = entries[0].record.as_ref().unwrap();
        assert_eq!(loaded.created_unix_secs, 42);
        assert_eq!(
            loaded.items[0].output_url.as_deref(),
            Some("https://cdn.test/a.png?signature=keep")
        );

        discard_image_recovery_record(&directory, &manifest).unwrap();
        assert!(!owned.exists());
        assert!(!manifest.exists());
        assert_eq!(fs::read(output).unwrap(), b"final");
    }

    #[test]
    fn corrupt_and_missing_recovery_entries_are_safe() {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("private");
        fs::create_dir_all(&directory).unwrap();
        let corrupt = directory.join("corrupt.json");
        fs::write(&corrupt, b"not json").unwrap();
        let entries = list_image_recovery_records(&directory).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].record.is_err());
        assert!(discard_image_recovery_record(&directory, &corrupt).is_err());
        assert!(corrupt.exists());
        assert!(
            discard_image_recovery_record(&directory, &directory.join("missing.json")).is_err()
        );
    }
}
