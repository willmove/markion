use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    GitObjectId, OperationKind, OperationPhase, RepositoryIdentity, SyncTarget,
    persist::atomic_write,
};

pub const JOURNAL_SCHEMA_VERSION: u32 = 1;
pub const MAX_SUCCESSFUL_SUMMARIES: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreimageEntry {
    pub relative_path: PathBuf,
    pub stored_name: String,
    pub byte_len: u64,
    pub content_fingerprint: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreimageManifest {
    pub entries: Vec<PreimageEntry>,
    pub total_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictDraft {
    pub relative_path: PathBuf,
    pub content: Vec<u8>,
    pub content_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationCheckpoint {
    pub operation_id: String,
    pub kind: OperationKind,
    pub phase: OperationPhase,
    pub identity: RepositoryIdentity,
    pub target: Option<SyncTarget>,
    pub expected_head: Option<GitObjectId>,
    pub resulting_commit: Option<GitObjectId>,
    pub fetched_tip: Option<GitObjectId>,
    pub expected_index_fingerprint: Option<String>,
    pub owned_paths: Vec<(PathBuf, String)>,
    pub recovery_refs: Vec<String>,
    pub preimages: Option<PreimageManifest>,
    pub drafts: Vec<ConflictDraft>,
    pub updated_unix_seconds: u64,
    pub confirmed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationSummary {
    pub operation_id: String,
    pub kind: OperationKind,
    pub resulting_commit: Option<GitObjectId>,
    pub completed_unix_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationJournal {
    pub schema_version: u32,
    #[serde(default)]
    pub active: Vec<OperationCheckpoint>,
    #[serde(default)]
    pub successful: Vec<OperationSummary>,
}

impl Default for OperationJournal {
    fn default() -> Self {
        Self {
            schema_version: JOURNAL_SCHEMA_VERSION,
            active: Vec::new(),
            successful: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct JournalStore {
    journal_path: PathBuf,
    recovery_root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum JournalError {
    #[error("operation journal IO failed: {0}")]
    Io(#[from] io::Error),
    #[error("operation journal is invalid: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("operation journal serialization failed: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("unsupported operation journal schema {0}")]
    UnsupportedVersion(u32),
    #[error("recovery content exceeds the configured byte limit")]
    RecoveryLimit,
    #[error("operation does not own the requested checkpoint")]
    OwnershipMismatch,
}

impl JournalStore {
    pub fn new(journal_path: impl Into<PathBuf>, recovery_root: impl Into<PathBuf>) -> Self {
        Self {
            journal_path: journal_path.into(),
            recovery_root: recovery_root.into(),
        }
    }

    pub fn load(&self) -> Result<OperationJournal, JournalError> {
        match fs::read_to_string(&self.journal_path) {
            Ok(text) => {
                let journal: OperationJournal = toml::from_str(&text)?;
                if journal.schema_version != JOURNAL_SCHEMA_VERSION {
                    return Err(JournalError::UnsupportedVersion(journal.schema_version));
                }
                Ok(journal)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Ok(OperationJournal::default())
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Persisting intent succeeds before callers start a protected mutation.
    pub fn begin(&self, checkpoint: OperationCheckpoint) -> Result<(), JournalError> {
        let mut journal = self.load()?;
        journal
            .active
            .retain(|active| active.operation_id != checkpoint.operation_id);
        journal.active.push(checkpoint);
        self.save(&journal)
    }

    pub fn update(&self, checkpoint: OperationCheckpoint) -> Result<(), JournalError> {
        let mut journal = self.load()?;
        let active = journal
            .active
            .iter_mut()
            .find(|active| active.operation_id == checkpoint.operation_id)
            .ok_or(JournalError::OwnershipMismatch)?;
        *active = checkpoint;
        self.save(&journal)
    }

    pub fn retire_success(
        &self,
        operation_id: &str,
        completed_unix_seconds: u64,
    ) -> Result<(), JournalError> {
        let mut journal = self.load()?;
        let index = journal
            .active
            .iter()
            .position(|active| active.operation_id == operation_id)
            .ok_or(JournalError::OwnershipMismatch)?;
        let checkpoint = journal.active.remove(index);
        if !checkpoint.confirmed || !checkpoint.drafts.is_empty() {
            return Err(JournalError::OwnershipMismatch);
        }
        journal.successful.insert(
            0,
            OperationSummary {
                operation_id: checkpoint.operation_id.clone(),
                kind: checkpoint.kind,
                resulting_commit: checkpoint.resulting_commit.clone(),
                completed_unix_seconds,
            },
        );
        journal.successful.truncate(MAX_SUCCESSFUL_SUMMARIES);
        self.save(&journal)?;
        self.cleanup_owned_recovery(&checkpoint)?;
        Ok(())
    }

    pub fn prepare_preimages(
        &self,
        operation_id: &str,
        worktree_root: &Path,
        paths: &[(PathBuf, String)],
        max_total_bytes: u64,
    ) -> Result<PreimageManifest, JournalError> {
        let operation_root = safe_operation_dir(&self.recovery_root, operation_id)?;
        let operation_dir = operation_root.join("preimages");
        let mut sources = Vec::new();
        let mut required_bytes = 0_u64;
        for (relative, fingerprint) in paths {
            if relative.is_absolute()
                || relative.components().any(|component| {
                    matches!(
                        component,
                        std::path::Component::ParentDir
                            | std::path::Component::RootDir
                            | std::path::Component::Prefix(_)
                    )
                })
            {
                return Err(JournalError::OwnershipMismatch);
            }
            let source = worktree_root.join(relative);
            if !source.is_file() {
                continue;
            }
            let byte_len = fs::metadata(&source)?.len();
            required_bytes = required_bytes.saturating_add(byte_len);
            if required_bytes > max_total_bytes {
                return Err(JournalError::RecoveryLimit);
            }
            sources.push((relative, fingerprint, source, byte_len));
        }
        fs::create_dir_all(&operation_dir)?;
        let mut manifest = PreimageManifest::default();
        for (index, (relative, fingerprint, source, byte_len)) in sources.into_iter().enumerate() {
            let stored_name = format!("{index:08}.bin");
            let target = operation_dir.join(&stored_name);
            let bytes = fs::read(&source)?;
            atomic_write(&target, &bytes)?;
            manifest.entries.push(PreimageEntry {
                relative_path: relative.clone(),
                stored_name,
                byte_len,
                content_fingerprint: fingerprint.clone(),
            });
            manifest.total_bytes = manifest.total_bytes.saturating_add(byte_len);
        }
        let manifest_text = toml::to_string_pretty(&manifest)?;
        atomic_write(
            &operation_root.join("preimages.toml"),
            manifest_text.as_bytes(),
        )?;
        Ok(manifest)
    }

    pub fn save_draft(&self, operation_id: &str, draft: ConflictDraft) -> Result<(), JournalError> {
        let mut journal = self.load()?;
        let active = journal
            .active
            .iter_mut()
            .find(|active| active.operation_id == operation_id)
            .ok_or(JournalError::OwnershipMismatch)?;
        active
            .drafts
            .retain(|current| current.relative_path != draft.relative_path);
        active.drafts.push(draft);
        self.save(&journal)
    }

    pub fn clear_draft(&self, operation_id: &str, path: &Path) -> Result<(), JournalError> {
        let mut journal = self.load()?;
        let active = journal
            .active
            .iter_mut()
            .find(|active| active.operation_id == operation_id)
            .ok_or(JournalError::OwnershipMismatch)?;
        active
            .drafts
            .retain(|current| current.relative_path != path);
        self.save(&journal)
    }

    fn save(&self, journal: &OperationJournal) -> Result<(), JournalError> {
        if journal.schema_version != JOURNAL_SCHEMA_VERSION {
            return Err(JournalError::UnsupportedVersion(journal.schema_version));
        }
        let text = toml::to_string_pretty(journal)?;
        atomic_write(&self.journal_path, text.as_bytes())?;
        Ok(())
    }

    fn cleanup_owned_recovery(&self, checkpoint: &OperationCheckpoint) -> Result<(), JournalError> {
        let operation_dir = safe_operation_dir(&self.recovery_root, &checkpoint.operation_id)?;
        if let Some(manifest) = &checkpoint.preimages {
            let preimages = operation_dir.join("preimages");
            for entry in &manifest.entries {
                if entry.stored_name.contains(['/', '\\']) || entry.stored_name.starts_with('.') {
                    return Err(JournalError::OwnershipMismatch);
                }
                remove_if_file(&preimages.join(&entry.stored_name))?;
            }
            remove_if_file(&operation_dir.join("preimages.toml"))?;
            remove_if_empty_dir(&preimages)?;
        }
        // Deliberately non-recursive: unknown files prove that the directory
        // is not fully operation-owned and are preserved for inspection.
        remove_if_empty_dir(&operation_dir)?;
        Ok(())
    }
}

fn safe_operation_dir(root: &Path, operation_id: &str) -> Result<PathBuf, JournalError> {
    let path = Path::new(operation_id);
    if operation_id.is_empty()
        || path.is_absolute()
        || path.components().count() != 1
        || operation_id == "."
        || operation_id == ".."
    {
        return Err(JournalError::OwnershipMismatch);
    }
    Ok(root.join(path))
}

fn remove_if_file(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => fs::remove_file(path),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_if_empty_dir(path: &Path) -> io::Result<()> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::DirectoryNotEmpty
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(root: &Path, id: &str) -> OperationCheckpoint {
        OperationCheckpoint {
            operation_id: id.into(),
            kind: OperationKind::SyncNow,
            phase: OperationPhase::Preparing,
            identity: RepositoryIdentity::new(
                root.to_path_buf(),
                root.join(".git"),
                root.join(".git"),
            ),
            target: None,
            expected_head: None,
            resulting_commit: None,
            fetched_tip: None,
            expected_index_fingerprint: None,
            owned_paths: Vec::new(),
            recovery_refs: Vec::new(),
            preimages: None,
            drafts: Vec::new(),
            updated_unix_seconds: 1,
            confirmed: false,
        }
    }

    #[test]
    fn intent_is_durable_before_mutation_and_draft_survives_reload() {
        let dir = tempfile::tempdir().unwrap();
        let store = JournalStore::new(dir.path().join("journal.toml"), dir.path().join("recovery"));
        store.begin(checkpoint(dir.path(), "operation-1")).unwrap();
        store
            .save_draft(
                "operation-1",
                ConflictDraft {
                    relative_path: PathBuf::from("notes.md"),
                    content: b"draft".to_vec(),
                    content_fingerprint: "fingerprint".into(),
                },
            )
            .unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.active[0].drafts[0].content, b"draft");
    }

    #[test]
    fn preimages_are_bounded_before_they_are_recorded() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("notes.md"), "important").unwrap();
        let store = JournalStore::new(dir.path().join("journal.toml"), dir.path().join("recovery"));
        assert!(matches!(
            store.prepare_preimages(
                "operation-1",
                dir.path(),
                &[(PathBuf::from("notes.md"), "hash".into())],
                2
            ),
            Err(JournalError::RecoveryLimit)
        ));
    }

    #[test]
    fn successful_summaries_are_bounded_and_drafts_block_retirement() {
        let dir = tempfile::tempdir().unwrap();
        let store = JournalStore::new(dir.path().join("journal.toml"), dir.path().join("recovery"));
        for index in 0..105 {
            let mut value = checkpoint(dir.path(), &format!("operation-{index}"));
            value.confirmed = true;
            store.begin(value).unwrap();
            store
                .retire_success(&format!("operation-{index}"), index)
                .unwrap();
        }
        let journal = store.load().unwrap();
        assert_eq!(journal.successful.len(), MAX_SUCCESSFUL_SUMMARIES);
        assert_eq!(journal.successful[0].operation_id, "operation-104");
    }

    #[test]
    fn persistence_failure_prevents_begin_and_cleanup_preserves_unknown_files() {
        let dir = tempfile::tempdir().unwrap();
        let invalid_journal = dir.path().join("directory-instead-of-journal");
        fs::create_dir(&invalid_journal).unwrap();
        let broken = JournalStore::new(&invalid_journal, dir.path().join("recovery"));
        assert!(broken.begin(checkpoint(dir.path(), "blocked")).is_err());

        let recovery = dir.path().join("recovery");
        let store = JournalStore::new(dir.path().join("journal.toml"), &recovery);
        let mut value = checkpoint(dir.path(), "owned");
        value.confirmed = true;
        value.preimages = Some(PreimageManifest::default());
        store.begin(value).unwrap();
        let operation_dir = recovery.join("owned");
        fs::create_dir_all(&operation_dir).unwrap();
        fs::write(operation_dir.join("user-file.txt"), "preserve").unwrap();
        store.retire_success("owned", 2).unwrap();
        assert!(operation_dir.join("user-file.txt").is_file());
    }
}
