//! Platform-keychain coordination. Only non-secret credential references are persisted here.
use markion_ai::{Profile, Secret};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    future::Future,
    io,
    path::{Path, PathBuf},
    pin::Pin,
};

pub type StoreFuture<T> = Pin<Box<dyn Future<Output = Result<T, CredentialError>>>>;
pub fn write_local_data(path: &Path, bytes: &[u8]) -> io::Result<()> {
    crate::storage::atomic_write(path, bytes)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialError {
    Unavailable,
    InvalidProfile,
    IndexFailure,
}
pub trait CredentialStore {
    fn read(&self, reference: &str) -> StoreFuture<Option<Secret>>;
    fn write(&self, reference: &str, key: Secret) -> StoreFuture<()>;
    fn forget(&self, reference: &str) -> StoreFuture<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialReference {
    pub reference: String,
    pub label: String,
}
#[derive(Debug)]
pub struct Credentials {
    index_path: PathBuf,
    references: Vec<CredentialReference>,
    session: BTreeMap<String, Secret>,
    index_unavailable: bool,
}
impl Credentials {
    pub fn load(index_path: PathBuf) -> io::Result<Self> {
        if std::fs::metadata(&index_path).is_ok_and(|m| m.len() > 256 * 1024) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "AI key index exceeds limit",
            ));
        }
        let references = match std::fs::read(&index_path) {
            Ok(bytes) => {
                if bytes.len() > 256 * 1024 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "AI key index exceeds limit",
                    ));
                }
                serde_json::from_slice(&bytes).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid AI key index")
                })?
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        Ok(Self {
            index_path,
            references,
            session: BTreeMap::new(),
            index_unavailable: false,
        })
    }
    pub fn empty(index_path: PathBuf) -> Self {
        Self {
            index_path: index_path.clone(),
            references: Vec::new(),
            session: BTreeMap::new(),
            // An unreadable existing index must never be overwritten by a fresh empty one.
            index_unavailable: index_path.exists(),
        }
    }
    pub fn references(&self) -> &[CredentialReference] {
        &self.references
    }
    pub fn session_key(&self, profile: &Profile) -> Option<Secret> {
        self.session
            .get(&profile.credential_reference().ok()?)
            .cloned()
    }
    pub fn use_for_session(
        &mut self,
        profile: &Profile,
        key: Secret,
    ) -> Result<(), CredentialError> {
        let reference = profile
            .credential_reference()
            .map_err(|_| CredentialError::InvalidProfile)?;
        self.session.insert(reference, key);
        Ok(())
    }
    pub fn remove_session(&mut self, reference: &str) {
        self.session.remove(reference);
    }
    pub fn record(&mut self, profile: &Profile) -> Result<String, CredentialError> {
        let reference = profile
            .credential_reference()
            .map_err(|_| CredentialError::InvalidProfile)?;
        if !self.references.iter().any(|r| r.reference == reference) {
            let mut next = self.references.clone();
            next.push(CredentialReference {
                reference: reference.clone(),
                label: profile.name.clone(),
            });
            self.persist(&next)
                .map_err(|_| CredentialError::IndexFailure)?;
            self.references = next;
        }
        Ok(reference)
    }
    pub fn remove_record(&mut self, reference: &str) -> Result<(), CredentialError> {
        let next = self
            .references
            .iter()
            .filter(|r| r.reference != reference)
            .cloned()
            .collect::<Vec<_>>();
        self.persist(&next)
            .map_err(|_| CredentialError::IndexFailure)?;
        self.references = next;
        self.session.remove(reference);
        Ok(())
    }
    fn persist(&self, entries: &[CredentialReference]) -> io::Result<()> {
        if self.index_unavailable {
            return Err(io::Error::other(
                "AI key index unavailable; use a session key",
            ));
        }
        if let Some(parent) = self.index_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec(entries).map_err(io::Error::other)?;
        if bytes.len() > 256 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "AI key index exceeds limit",
            ));
        }
        crate::storage::atomic_write(&self.index_path, &bytes)
    }
    pub async fn save<S: CredentialStore>(
        &mut self,
        store: &S,
        profile: &Profile,
        key: Secret,
    ) -> Result<(), CredentialError> {
        // Remember ownership before platform write so even an interrupted save remains manageable.
        let reference = self.record(profile)?;
        store.write(&reference, key).await?;
        self.session.remove(&reference);
        Ok(())
    }
    pub async fn read<S: CredentialStore>(
        &self,
        store: &S,
        profile: &Profile,
    ) -> Result<Option<Secret>, CredentialError> {
        if let Some(key) = self.session_key(profile) {
            return Ok(Some(key));
        }
        let reference = profile
            .credential_reference()
            .map_err(|_| CredentialError::InvalidProfile)?;
        store.read(&reference).await
    }
    pub async fn forget<S: CredentialStore>(
        &mut self,
        store: &S,
        reference: &str,
    ) -> Result<(), CredentialError> {
        self.session.remove(reference);
        store.forget(reference).await?;
        self.remove_record(reference)
    }
    pub fn index_path(&self) -> &Path {
        &self.index_path
    }
}

/// GPUI returns owned tasks; no live `App` is carried through the await.
pub struct PlatformStore<'a>(pub &'a gpui::App);
impl CredentialStore for PlatformStore<'_> {
    fn read(&self, reference: &str) -> StoreFuture<Option<Secret>> {
        let task = self.0.read_credentials(reference);
        Box::pin(async move {
            let key = task.await.map_err(|_| CredentialError::Unavailable)?;
            key.map(|(_, bytes)| {
                String::from_utf8(bytes)
                    .map(Secret::new)
                    .map_err(|_| CredentialError::Unavailable)
            })
            .transpose()
        })
    }
    fn write(&self, reference: &str, key: Secret) -> StoreFuture<()> {
        let task = self
            .0
            .write_credentials(reference, "markion-ai", key.expose().as_bytes());
        Box::pin(async move { task.await.map_err(|_| CredentialError::Unavailable) })
    }
    fn forget(&self, reference: &str) -> StoreFuture<()> {
        let read = self.0.read_credentials(reference);
        let cx = self.0.to_async();
        let reference = reference.to_owned();
        Box::pin(async move {
            if read
                .await
                .map_err(|_| CredentialError::Unavailable)?
                .is_none()
            {
                return Ok(());
            }
            cx.update(|cx| cx.delete_credentials(&reference))
                .map_err(|_| CredentialError::Unavailable)?
                .await
                .map_err(|_| CredentialError::Unavailable)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    #[derive(Default)]
    struct Fake {
        keys: Rc<RefCell<BTreeMap<String, Secret>>>,
        unavailable: bool,
    }
    impl CredentialStore for Fake {
        fn read(&self, r: &str) -> StoreFuture<Option<Secret>> {
            let result = if self.unavailable {
                Err(CredentialError::Unavailable)
            } else {
                Ok(self.keys.borrow().get(r).cloned())
            };
            Box::pin(async move { result })
        }
        fn write(&self, r: &str, k: Secret) -> StoreFuture<()> {
            let keys = self.keys.clone();
            let reference = r.to_string();
            let unavailable = self.unavailable;
            Box::pin(async move {
                if unavailable {
                    return Err(CredentialError::Unavailable);
                }
                keys.borrow_mut().insert(reference, k);
                Ok(())
            })
        }
        fn forget(&self, r: &str) -> StoreFuture<()> {
            let keys = self.keys.clone();
            let reference = r.to_string();
            Box::pin(async move {
                keys.borrow_mut().remove(&reference);
                Ok(())
            })
        }
    }
    #[tokio::test]
    async fn keychain_roundtrip_forget_and_index_survive_reset_without_secret() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ai-keys.json");
        let mut manager = Credentials::load(path.clone()).unwrap();
        let fake = Fake::default();
        let mut profile = Profile::preset("local", "test");
        manager
            .save(&fake, &profile, Secret::new("fixture-private-key".into()))
            .await
            .unwrap();
        assert_eq!(
            manager
                .read(&fake, &profile)
                .await
                .unwrap()
                .unwrap()
                .expose(),
            "fixture-private-key"
        );
        assert!(
            !String::from_utf8(std::fs::read(&path).unwrap())
                .unwrap()
                .contains("fixture-private-key")
        );
        let reference = manager.references()[0].reference.clone();
        profile.endpoint = "http://localhost:9999/v1".into();
        assert!(manager.read(&fake, &profile).await.unwrap().is_none());
        let mut restored = Credentials::load(path).unwrap();
        assert_eq!(restored.references().len(), 1);
        restored.forget(&fake, &reference).await.unwrap();
        restored.forget(&fake, &reference).await.unwrap();
        assert!(restored.references().is_empty());
    }
    #[tokio::test]
    async fn corrupt_reference_index_is_retained_and_session_keys_still_work() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ai-keys.json");
        std::fs::write(&path, b"interrupted-index").unwrap();
        assert!(Credentials::load(path.clone()).is_err());
        let mut manager = Credentials::empty(path.clone());
        let fake = Fake::default();
        let profile = Profile::preset("local", "test");
        assert_eq!(
            manager
                .save(&fake, &profile, Secret::new("private".into()))
                .await,
            Err(CredentialError::IndexFailure)
        );
        assert!(fake.keys.borrow().is_empty());
        manager
            .use_for_session(&profile, Secret::new("session".into()))
            .unwrap();
        assert_eq!(
            manager
                .read(&fake, &profile)
                .await
                .unwrap()
                .unwrap()
                .expose(),
            "session"
        );
        assert_eq!(std::fs::read(path).unwrap(), b"interrupted-index");
    }
    #[tokio::test]
    async fn unavailable_store_requires_explicit_session_key_and_never_writes_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ai-keys.json");
        let mut manager = Credentials::load(path.clone()).unwrap();
        let fake = Fake {
            unavailable: true,
            ..Default::default()
        };
        let profile = Profile::preset("local", "test");
        assert_eq!(
            manager
                .save(&fake, &profile, Secret::new("session-only-key".into()))
                .await,
            Err(CredentialError::Unavailable)
        );
        manager
            .use_for_session(&profile, Secret::new("session-only-key".into()))
            .unwrap();
        assert_eq!(
            manager
                .read(&fake, &profile)
                .await
                .unwrap()
                .unwrap()
                .expose(),
            "session-only-key"
        );
        assert!(
            !String::from_utf8(std::fs::read(&path).unwrap())
                .unwrap()
                .contains("session-only-key")
        );
        assert!(
            Credentials::load(path)
                .unwrap()
                .session_key(&profile)
                .is_none()
        );
    }
}
