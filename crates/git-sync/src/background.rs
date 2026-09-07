use std::time::{Duration, Instant};

use crate::RepositoryIdentity;

const NOMINAL_INTERVAL: Duration = Duration::from_secs(5 * 60);
const MAX_BACKOFF: Duration = Duration::from_secs(30 * 60);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundFetchResult {
    Unchanged,
    Incoming { commits: usize },
    AuthenticationNeeded,
    Offline,
    ActionableError(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundNotification {
    Quiet,
    Incoming { commits: usize },
    AuthenticationNeeded,
    ActionableError(String),
}

#[derive(Clone, Debug, Default)]
pub struct BackgroundFetchScheduler {
    active: Option<RepositoryIdentity>,
    enabled: bool,
    helper_is_noninteractive: bool,
    in_flight: bool,
    next_check: Option<Instant>,
    failures: u32,
}

impl BackgroundFetchScheduler {
    pub fn activate(
        &mut self,
        identity: RepositoryIdentity,
        enabled: bool,
        helper_is_noninteractive: bool,
        now: Instant,
        stale: bool,
    ) {
        let changed = self.active.as_ref() != Some(&identity);
        self.active = Some(identity);
        self.enabled = enabled;
        self.helper_is_noninteractive = helper_is_noninteractive;
        if changed {
            self.in_flight = false;
            self.failures = 0;
        }
        self.next_check = (enabled && helper_is_noninteractive)
            .then(|| if stale { now } else { now + NOMINAL_INTERVAL });
    }

    pub fn deactivate(&mut self) {
        self.active = None;
        self.next_check = None;
    }

    pub fn due(&self, now: Instant) -> Option<&RepositoryIdentity> {
        (self.enabled
            && self.helper_is_noninteractive
            && !self.in_flight
            && self.next_check.is_some_and(|deadline| now >= deadline))
        .then_some(self.active.as_ref())?
    }

    pub fn start(&mut self, now: Instant) -> Option<RepositoryIdentity> {
        let identity = self.due(now)?.clone();
        self.in_flight = true;
        Some(identity)
    }

    pub fn finish(
        &mut self,
        identity: &RepositoryIdentity,
        result: BackgroundFetchResult,
        now: Instant,
    ) -> BackgroundNotification {
        if self.active.as_ref() != Some(identity) {
            self.in_flight = false;
            return BackgroundNotification::Quiet;
        }
        self.in_flight = false;
        let notification = match result {
            BackgroundFetchResult::Unchanged => {
                self.failures = 0;
                BackgroundNotification::Quiet
            }
            BackgroundFetchResult::Incoming { commits } => {
                self.failures = 0;
                BackgroundNotification::Incoming { commits }
            }
            BackgroundFetchResult::AuthenticationNeeded => {
                self.enabled = false;
                self.next_check = None;
                return BackgroundNotification::AuthenticationNeeded;
            }
            BackgroundFetchResult::Offline => {
                self.failures = self.failures.saturating_add(1);
                BackgroundNotification::Quiet
            }
            BackgroundFetchResult::ActionableError(error) => {
                self.failures = self.failures.saturating_add(1);
                BackgroundNotification::ActionableError(error)
            }
        };
        let multiplier = 1_u32 << self.failures.min(3);
        let delay = NOMINAL_INTERVAL.saturating_mul(multiplier).min(MAX_BACKOFF);
        self.next_check = Some(now + delay + deterministic_jitter(identity, delay));
        notification
    }

    pub fn in_flight(&self) -> bool {
        self.in_flight
    }

    pub fn active_identity(&self) -> Option<&RepositoryIdentity> {
        self.active.as_ref()
    }
}

fn deterministic_jitter(identity: &RepositoryIdentity, interval: Duration) -> Duration {
    let hash = identity
        .worktree_root
        .to_string_lossy()
        .bytes()
        .fold(0_u64, |hash, byte| {
            hash.wrapping_mul(131).wrapping_add(u64::from(byte))
        });
    let window = (interval.as_secs() / 10).max(1);
    Duration::from_secs(hash % window)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn identity(name: &str) -> RepositoryIdentity {
        let root = PathBuf::from(name);
        RepositoryIdentity::new(root.clone(), root.join(".git"), root.join(".git"))
    }

    #[test]
    fn scheduling_is_default_off_single_flight_and_stops_on_switch_away() {
        let now = Instant::now();
        let mut scheduler = BackgroundFetchScheduler::default();
        scheduler.activate(identity("notes"), false, true, now, true);
        assert!(scheduler.start(now).is_none());
        scheduler.activate(identity("notes"), true, true, now, true);
        let active = scheduler.start(now).unwrap();
        assert!(scheduler.start(now).is_none());
        scheduler.deactivate();
        assert_eq!(
            scheduler.finish(&active, BackgroundFetchResult::Unchanged, now),
            BackgroundNotification::Quiet
        );
        assert!(scheduler.start(now + MAX_BACKOFF).is_none());
    }

    #[test]
    fn authentication_pauses_checks_and_unchanged_is_quiet() {
        let now = Instant::now();
        let id = identity("notes");
        let mut scheduler = BackgroundFetchScheduler::default();
        scheduler.activate(id.clone(), true, true, now, true);
        scheduler.start(now).unwrap();
        assert_eq!(
            scheduler.finish(&id, BackgroundFetchResult::AuthenticationNeeded, now),
            BackgroundNotification::AuthenticationNeeded
        );
        assert!(scheduler.start(now + MAX_BACKOFF).is_none());
    }
}
