use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::PublishingResource;

pub trait Clock: Send + Sync + 'static {
    fn now(&self) -> Duration;
}

#[derive(Debug)]
pub struct SystemClock {
    origin: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}

#[derive(Debug, Default)]
pub struct ManualClock {
    millis: std::sync::atomic::AtomicU64,
}

impl ManualClock {
    pub fn advance(&self, duration: Duration) {
        let millis = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
        self.millis
            .fetch_add(millis, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.millis.load(std::sync::atomic::Ordering::SeqCst))
    }
}

pub trait TokenSource: Send + Sync + 'static {
    fn generate(&self) -> io::Result<String>;
}

#[derive(Debug, Default)]
pub struct OsTokenSource;

impl TokenSource for OsTokenSource {
    fn generate(&self) -> io::Result<String> {
        let mut bytes = [0_u8; 32];
        getrandom::fill(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;
        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SessionLimits {
    pub max_sessions: usize,
    pub idle_timeout: Duration,
}

impl Default for SessionLimits {
    fn default() -> Self {
        Self {
            max_sessions: 8,
            idle_timeout: Duration::from_secs(2 * 60 * 60),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublishingSnapshot {
    pub markdown: Arc<str>,
    pub display_name: String,
    pub language: String,
    pub resources: Vec<PublishingResource>,
    pub unresolved_local_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResourceMetadata {
    pub authored_url: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DocumentPayload {
    pub markdown: Arc<str>,
    pub display_name: String,
    pub language: String,
    pub resources: Vec<ResourceMetadata>,
    pub unresolved_local_images: Vec<String>,
}

#[derive(Debug)]
struct Session {
    id: u64,
    claim_hash: Option<[u8; 32]>,
    bearer_hash: Option<[u8; 32]>,
    snapshot: Arc<PublishingSnapshot>,
    last_touched: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SessionId(pub(crate) u64);

pub(crate) struct SessionStore {
    clock: Arc<dyn Clock>,
    tokens: Arc<dyn TokenSource>,
    limits: SessionLimits,
    state: Mutex<SessionState>,
}

#[derive(Debug, Default)]
struct SessionState {
    next_id: u64,
    sessions: HashMap<u64, Session>,
}

impl SessionStore {
    pub(crate) fn new(
        clock: Arc<dyn Clock>,
        tokens: Arc<dyn TokenSource>,
        limits: SessionLimits,
    ) -> Self {
        Self {
            clock,
            tokens,
            limits,
            state: Mutex::new(SessionState::default()),
        }
    }

    pub(crate) fn create(&self, snapshot: PublishingSnapshot) -> io::Result<(SessionId, String)> {
        let claim = self.tokens.generate()?;
        let now = self.clock.now();
        let mut state = self.state.lock().expect("session state poisoned");
        self.expire_locked(&mut state, now);
        while state.sessions.len() >= self.limits.max_sessions.max(1) {
            let Some(evicted) = state
                .sessions
                .values()
                .min_by_key(|session| (session.last_touched, session.id))
                .map(|session| session.id)
            else {
                break;
            };
            state.sessions.remove(&evicted);
            tracing::debug!(
                active_sessions = state.sessions.len(),
                "publishing session evicted"
            );
        }
        let id = state.next_id;
        state.next_id = state.next_id.wrapping_add(1);
        state.sessions.insert(
            id,
            Session {
                id,
                claim_hash: Some(token_hash(&claim)),
                bearer_hash: None,
                snapshot: Arc::new(snapshot),
                last_touched: now,
            },
        );
        tracing::debug!(
            active_sessions = state.sessions.len(),
            "publishing session created"
        );
        Ok((SessionId(id), claim))
    }

    pub(crate) fn claim(&self, claim: &str) -> io::Result<Option<String>> {
        let now = self.clock.now();
        let claim_hash = token_hash(claim);
        let mut state = self.state.lock().expect("session state poisoned");
        self.expire_locked(&mut state, now);
        let Some(session) = state
            .sessions
            .values_mut()
            .find(|session| session.claim_hash == Some(claim_hash))
        else {
            return Ok(None);
        };
        let bearer = self.tokens.generate()?;
        session.claim_hash = None;
        session.bearer_hash = Some(token_hash(&bearer));
        session.last_touched = now;
        Ok(Some(bearer))
    }

    pub(crate) fn authorize(&self, bearer: &str) -> Option<Arc<PublishingSnapshot>> {
        let now = self.clock.now();
        let bearer_hash = token_hash(bearer);
        let mut state = self.state.lock().expect("session state poisoned");
        self.expire_locked(&mut state, now);
        let session = state
            .sessions
            .values_mut()
            .find(|session| session.bearer_hash == Some(bearer_hash))?;
        session.last_touched = now;
        Some(Arc::clone(&session.snapshot))
    }

    pub(crate) fn revoke(&self, id: SessionId) -> bool {
        let mut state = self.state.lock().expect("session state poisoned");
        let revoked = state.sessions.remove(&id.0).is_some();
        if revoked {
            tracing::debug!(
                active_sessions = state.sessions.len(),
                "publishing session revoked"
            );
        }
        revoked
    }

    fn expire_locked(&self, state: &mut SessionState, now: Duration) {
        let before = state.sessions.len();
        state.sessions.retain(|_, session| {
            now.saturating_sub(session.last_touched) <= self.limits.idle_timeout
        });
        let expired = before - state.sessions.len();
        if expired > 0 {
            tracing::debug!(
                expired,
                active_sessions = state.sessions.len(),
                "publishing sessions expired"
            );
        }
    }
}

impl PublishingSnapshot {
    pub(crate) fn payload(&self) -> DocumentPayload {
        DocumentPayload {
            markdown: Arc::clone(&self.markdown),
            display_name: self.display_name.clone(),
            language: self.language.clone(),
            resources: self
                .resources
                .iter()
                .map(|resource| ResourceMetadata {
                    authored_url: resource.authored_url().to_owned(),
                    id: resource.id().to_owned(),
                })
                .collect(),
            unresolved_local_images: self.unresolved_local_images.clone(),
        }
    }
}

fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::VecDeque, sync::Mutex};

    #[derive(Debug)]
    struct ScriptedTokens(Mutex<VecDeque<String>>);

    impl ScriptedTokens {
        fn new(tokens: &[&str]) -> Self {
            Self(Mutex::new(tokens.iter().map(ToString::to_string).collect()))
        }
    }

    impl TokenSource for ScriptedTokens {
        fn generate(&self) -> io::Result<String> {
            self.0
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| io::Error::other("scripted token source exhausted"))
        }
    }

    fn snapshot(text: &str) -> PublishingSnapshot {
        PublishingSnapshot {
            markdown: Arc::from(text),
            display_name: "note.md".into(),
            language: "en-US".into(),
            resources: Vec::new(),
            unresolved_local_images: Vec::new(),
        }
    }

    #[test]
    fn claim_is_one_time_and_bearer_expires_deterministically() {
        let clock = Arc::new(ManualClock::default());
        let store = SessionStore::new(
            clock.clone(),
            Arc::new(ScriptedTokens::new(&["claim", "bearer"])),
            SessionLimits {
                max_sessions: 8,
                idle_timeout: Duration::from_secs(10),
            },
        );
        let (_, claim) = store.create(snapshot("first")).unwrap();
        let bearer = store.claim(&claim).unwrap().unwrap();
        assert!(store.claim(&claim).unwrap().is_none());
        assert_eq!(store.authorize(&bearer).unwrap().markdown.as_ref(), "first");
        clock.advance(Duration::from_secs(11));
        assert!(store.authorize(&bearer).is_none());
    }

    #[test]
    fn lru_bound_evicts_the_least_recently_touched_session() {
        let clock = Arc::new(ManualClock::default());
        let store = SessionStore::new(
            clock.clone(),
            Arc::new(ScriptedTokens::new(&["c1", "b1", "c2", "b2", "c3"])),
            SessionLimits {
                max_sessions: 2,
                idle_timeout: Duration::from_secs(100),
            },
        );
        let (_, c1) = store.create(snapshot("one")).unwrap();
        let b1 = store.claim(&c1).unwrap().unwrap();
        clock.advance(Duration::from_secs(1));
        let (_, c2) = store.create(snapshot("two")).unwrap();
        let b2 = store.claim(&c2).unwrap().unwrap();
        clock.advance(Duration::from_secs(1));
        assert!(store.authorize(&b1).is_some());
        clock.advance(Duration::from_secs(1));
        store.create(snapshot("three")).unwrap();
        assert!(store.authorize(&b1).is_some());
        assert!(store.authorize(&b2).is_none());
    }
}
