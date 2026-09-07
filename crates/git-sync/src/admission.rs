use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex, MutexGuard},
};

use crate::RepositoryIdentity;

#[derive(Clone, Debug, Default)]
pub struct GitOperationRegistry {
    inner: Arc<RegistryInner>,
}

#[derive(Debug, Default)]
struct RegistryInner {
    state: Mutex<RegistryState>,
    changed: Condvar,
}

#[derive(Debug, Default)]
struct RegistryState {
    repositories: HashMap<RepositoryIdentity, GateState>,
}

#[derive(Clone, Debug)]
struct GateState {
    active_writers: usize,
    exclusive_operation: Option<String>,
    epoch: u64,
}

impl Default for GateState {
    fn default() -> Self {
        Self {
            active_writers: 0,
            exclusive_operation: None,
            epoch: 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadEpoch {
    identity: RepositoryIdentity,
    value: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("repository write is temporarily blocked by Git synchronization")]
    Deferred,
    #[error("another Git operation already owns this repository")]
    Busy,
    #[error("repository is not registered")]
    UnknownRepository,
}

#[derive(Debug)]
pub struct WriteAdmission {
    inner: Arc<RegistryInner>,
    identity: Option<RepositoryIdentity>,
}

impl Drop for WriteAdmission {
    fn drop(&mut self) {
        let Some(identity) = self.identity.take() else {
            return;
        };
        let mut state = lock_state(&self.inner);
        if let Some(gate) = state.repositories.get_mut(&identity) {
            gate.active_writers = gate.active_writers.saturating_sub(1);
        }
        self.inner.changed.notify_all();
    }
}

#[derive(Debug)]
pub struct ExclusiveAdmission {
    inner: Arc<RegistryInner>,
    identity: Option<RepositoryIdentity>,
    operation_id: String,
    coalesced: bool,
}

impl ExclusiveAdmission {
    pub fn coalesced(&self) -> bool {
        self.coalesced
    }

    pub fn epoch(&self) -> u64 {
        let state = lock_state(&self.inner);
        self.identity
            .as_ref()
            .and_then(|identity| state.repositories.get(identity))
            .map(|gate| gate.epoch)
            .unwrap_or_default()
    }

    pub fn identity(&self) -> Option<&RepositoryIdentity> {
        self.identity.as_ref()
    }

    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }
}

impl Drop for ExclusiveAdmission {
    fn drop(&mut self) {
        if self.coalesced {
            return;
        }
        let Some(identity) = self.identity.take() else {
            return;
        };
        let mut state = lock_state(&self.inner);
        if let Some(gate) = state.repositories.get_mut(&identity)
            && gate.exclusive_operation.as_deref() == Some(&self.operation_id)
        {
            gate.exclusive_operation = None;
        }
        self.inner.changed.notify_all();
    }
}

impl GitOperationRegistry {
    pub fn register(&self, identity: RepositoryIdentity) {
        lock_state(&self.inner)
            .repositories
            .entry(identity)
            .or_default();
    }

    pub fn unregister(&self, identity: &RepositoryIdentity) -> Result<(), AdmissionError> {
        let mut state = lock_state(&self.inner);
        let gate = state
            .repositories
            .get(identity)
            .ok_or(AdmissionError::UnknownRepository)?;
        if gate.active_writers > 0 || gate.exclusive_operation.is_some() {
            return Err(AdmissionError::Busy);
        }
        state.repositories.remove(identity);
        Ok(())
    }

    pub fn repository_for_path(&self, path: &Path) -> Option<RepositoryIdentity> {
        lock_state(&self.inner)
            .repositories
            .keys()
            .filter(|identity| path.starts_with(&identity.worktree_root))
            .max_by_key(|identity| identity.worktree_root.components().count())
            .cloned()
    }

    pub fn try_write(&self, path: &Path) -> Result<WriteAdmission, AdmissionError> {
        let mut state = lock_state(&self.inner);
        let identity = state
            .repositories
            .keys()
            .filter(|identity| path.starts_with(&identity.worktree_root))
            .max_by_key(|identity| identity.worktree_root.components().count())
            .cloned();
        let Some(identity) = identity else {
            return Ok(WriteAdmission {
                inner: self.inner.clone(),
                identity: None,
            });
        };
        let gate = state
            .repositories
            .get_mut(&identity)
            .expect("key selected above");
        if gate.exclusive_operation.is_some() {
            return Err(AdmissionError::Deferred);
        }
        gate.active_writers += 1;
        Ok(WriteAdmission {
            inner: self.inner.clone(),
            identity: Some(identity),
        })
    }

    /// Blocks only the calling background worker. New writes are refused as
    /// soon as the operation is registered, then already admitted writers are
    /// drained before this returns.
    pub fn begin_exclusive(
        &self,
        identity: &RepositoryIdentity,
        operation_id: &str,
    ) -> Result<ExclusiveAdmission, AdmissionError> {
        let mut state = lock_state(&self.inner);
        let gate = state
            .repositories
            .get_mut(identity)
            .ok_or(AdmissionError::UnknownRepository)?;
        if gate.exclusive_operation.as_deref() == Some(operation_id) {
            return Ok(ExclusiveAdmission {
                inner: self.inner.clone(),
                identity: Some(identity.clone()),
                operation_id: operation_id.to_string(),
                coalesced: true,
            });
        }
        if gate.exclusive_operation.is_some() {
            return Err(AdmissionError::Busy);
        }
        gate.exclusive_operation = Some(operation_id.to_string());
        gate.epoch = gate.epoch.wrapping_add(1).max(1);
        while state
            .repositories
            .get(identity)
            .is_some_and(|gate| gate.active_writers > 0)
        {
            state = self
                .inner
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        Ok(ExclusiveAdmission {
            inner: self.inner.clone(),
            identity: Some(identity.clone()),
            operation_id: operation_id.to_string(),
            coalesced: false,
        })
    }

    pub fn capture_read_epoch(&self, path: &Path) -> Option<ReadEpoch> {
        let state = lock_state(&self.inner);
        let (identity, gate) = state
            .repositories
            .iter()
            .filter(|(identity, _)| path.starts_with(&identity.worktree_root))
            .max_by_key(|(identity, _)| identity.worktree_root.components().count())?;
        Some(ReadEpoch {
            identity: identity.clone(),
            value: gate.epoch,
        })
    }

    pub fn read_epoch_is_current(&self, epoch: &ReadEpoch) -> bool {
        lock_state(&self.inner)
            .repositories
            .get(&epoch.identity)
            .is_some_and(|gate| gate.epoch == epoch.value && gate.exclusive_operation.is_none())
    }

    pub fn paths_in_repository<'a>(
        &self,
        identity: &RepositoryIdentity,
        paths: impl IntoIterator<Item = &'a Path>,
    ) -> Vec<PathBuf> {
        paths
            .into_iter()
            .filter(|path| path.starts_with(&identity.worktree_root))
            .map(Path::to_path_buf)
            .collect()
    }
}

fn lock_state(inner: &RegistryInner) -> MutexGuard<'_, RegistryState> {
    inner
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::mpsc, thread, time::Duration};

    fn identity(root: &Path) -> RepositoryIdentity {
        RepositoryIdentity::new(root.to_path_buf(), root.join(".git"), root.join(".git"))
    }

    #[test]
    fn exclusive_operation_drains_writers_and_invalidates_reads() {
        let root = PathBuf::from("repo");
        let identity = identity(&root);
        let registry = GitOperationRegistry::default();
        registry.register(identity.clone());
        let writer = registry.try_write(&root.join("note.md")).unwrap();
        let epoch = registry.capture_read_epoch(&root.join("note.md")).unwrap();
        let worker = registry.clone();
        let target = identity.clone();
        let (sent, received) = mpsc::channel();
        let thread = thread::spawn(move || {
            let exclusive = worker.begin_exclusive(&target, "sync-1").unwrap();
            sent.send(exclusive).unwrap();
        });
        thread::sleep(Duration::from_millis(30));
        assert_eq!(
            registry.try_write(&root.join("new.md")).unwrap_err(),
            AdmissionError::Deferred
        );
        drop(writer);
        let exclusive = received.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(!registry.read_epoch_is_current(&epoch));
        drop(exclusive);
        thread.join().unwrap();
        assert!(registry.try_write(&root.join("new.md")).is_ok());
    }

    #[test]
    fn duplicate_operation_is_coalesced_and_membership_excludes_foreign_paths() {
        let root = PathBuf::from("repo");
        let identity = identity(&root);
        let registry = GitOperationRegistry::default();
        registry.register(identity.clone());
        let owner = registry.begin_exclusive(&identity, "same").unwrap();
        let duplicate = registry.begin_exclusive(&identity, "same").unwrap();
        assert!(duplicate.coalesced());
        assert_eq!(
            registry.paths_in_repository(
                &identity,
                [root.join("inside.md"), PathBuf::from("foreign.md")]
                    .iter()
                    .map(PathBuf::as_path)
            ),
            vec![root.join("inside.md")]
        );
        drop(duplicate);
        drop(owner);
    }

    #[test]
    fn destination_repository_blocks_a_cross_root_move_even_when_source_is_available() {
        let source_root = PathBuf::from("source-repo");
        let destination_root = PathBuf::from("destination-repo");
        let source = identity(&source_root);
        let destination = identity(&destination_root);
        let registry = GitOperationRegistry::default();
        registry.register(source);
        registry.register(destination.clone());
        let _exclusive = registry
            .begin_exclusive(&destination, "destination-sync")
            .unwrap();

        assert!(registry.try_write(&source_root.join("note.md")).is_ok());
        assert_eq!(
            registry
                .try_write(&destination_root.join("note.md"))
                .unwrap_err(),
            AdmissionError::Deferred
        );
    }
}
