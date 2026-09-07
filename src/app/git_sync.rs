use super::*;
use markion_git_sync::{
    Authentication, BackgroundFetchResult, BackgroundNotification, CancellationToken,
    ConflictManager, ConflictResolution, ConflictSide, ExclusiveAdmission, GitCommandRunner,
    GitRepository, GitSyncEngine, JournalStore, NewFileClass, NewFileRule, RecoveryAssessment,
    RecoveryManager, RemoteTransport, RemoteUrl, RepositoryPolicy, SyncOptions, SyncOutcome,
    SyncPlan, SyncPolicies, inspect_note_attachments,
};
use std::collections::BTreeSet;
use std::time::Instant;

struct GitBufferSnapshot {
    instance: DocumentInstanceId,
    version: u64,
    path: PathBuf,
    known: Option<DiskIdentity>,
    text: String,
    dirty: bool,
}

pub(super) struct GitBufferResult {
    pub(super) instance: DocumentInstanceId,
    pub(super) version: u64,
    pub(super) previous_path: PathBuf,
    pub(super) path: PathBuf,
    pub(super) source: Option<(String, DiskIdentity)>,
}

struct GitImageSnapshot {
    path: PathBuf,
    fingerprint: Option<(u64, u128)>,
}

pub(super) struct GitImageResult {
    pub(super) previous_path: PathBuf,
    pub(super) path: PathBuf,
    pub(super) changed: bool,
}

pub(super) struct GitSyncCompletion {
    pub(super) _exclusive: ExclusiveAdmission,
    pub(super) outcome: SyncOutcome,
    pub(super) buffers: Vec<GitBufferResult>,
    pub(super) images: Vec<GitImageResult>,
}

#[derive(Clone, Copy)]
enum GitConflictFinalAction {
    FinishAndPush,
    FinishMerge,
    Abort,
}

#[derive(Debug, PartialEq, Eq)]
enum SyncNowRoute {
    Configured(RepositoryPolicy),
    SetupExistingRepository,
}

fn sync_now_route(
    policies: SyncPolicies,
    workspace_root: &Path,
    _active_tab_path: Option<&Path>,
) -> SyncNowRoute {
    policies
        .repositories
        .into_iter()
        .filter(|policy| workspace_root.starts_with(&policy.identity.worktree_root))
        .max_by_key(|policy| policy.identity.worktree_root.components().count())
        .map(SyncNowRoute::Configured)
        .unwrap_or(SyncNowRoute::SetupExistingRepository)
}

impl MarkionApp {
    pub(super) fn arm_git_background_checks(&mut self, cx: &mut Context<Self>) {
        self.poll_git_background_check(cx);
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(Duration::from_secs(30)).await;
                if this
                    .update(cx, |app, cx| app.poll_git_background_check(cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    pub(super) fn poll_git_background_check(&mut self, cx: &mut Context<Self>) {
        let workspace_root = self.workspace_root.clone();
        let globally_enabled = self.git_preferences.background_check;
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        cx.spawn(async move |this, cx| {
            let prepared = cx
                .background_spawn({
                    let executable = executable.clone();
                    async move {
                        let policies = PolicyStore::new(default_git_sync_policy_path())
                            .load()
                            .map_err(|error| error.to_string())?;
                        let policy = policies
                            .repositories
                            .into_iter()
                            .filter(|policy| {
                                workspace_root.starts_with(&policy.identity.worktree_root)
                            })
                            .max_by_key(|policy| {
                                policy.identity.worktree_root.components().count()
                            });
                        let Some(policy) = policy else {
                            return Ok::<_, String>(None);
                        };
                        let transport = RemoteUrl::parse(policy.target.fetch_url.clone())
                            .map_err(|error| error.to_string())?
                            .transport;
                        let helper_is_noninteractive = match transport {
                            RemoteTransport::Local => true,
                            RemoteTransport::Https => {
                                Authentication::new(GitCommandRunner::new(executable))
                                    .credential_helpers(&policy.identity.worktree_root)
                                    .map_err(|error| error.to_string())?
                                    .into_iter()
                                    .any(|helper| helper.supports_noninteractive_use)
                            }
                            RemoteTransport::Ssh => std::env::var_os("SSH_AUTH_SOCK").is_some(),
                        };
                        Ok(Some((policy, helper_is_noninteractive)))
                    }
                })
                .await;
            let Ok(prepared) = prepared else {
                return;
            };
            let Some((policy, helper_is_noninteractive)) = prepared else {
                let _ = this.update(cx, |app, _| app.git_background_scheduler.deactivate());
                return;
            };
            let started = this.update(cx, |app, _| {
                if app.git_background_scheduler.active_identity() != Some(&policy.identity) {
                    app.git_background_scheduler.activate(
                        policy.identity.clone(),
                        globally_enabled && policy.background_fetch,
                        helper_is_noninteractive,
                        Instant::now(),
                        true,
                    );
                }
                app.git_background_scheduler.start(Instant::now())
            });
            let Ok(Some(identity)) = started else {
                return;
            };
            let operation_id = format!(
                "background-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            );
            let result = cx
                .background_spawn({
                    let executable = executable.clone();
                    let policy = policy.clone();
                    let checked_identity = identity.clone();
                    async move {
                        let repository = GitRepository::discover(
                            GitCommandRunner::new(executable),
                            &checked_identity.worktree_root,
                        )
                        .map_err(|error| error.to_string());
                        match repository {
                            Ok(repository) if repository.identity() == &checked_identity => {
                                let data = markion::default_git_sync_data_dir();
                                GitSyncEngine::new(
                                    repository,
                                    JournalStore::new(
                                        data.join("journal.toml"),
                                        data.join("recovery"),
                                    ),
                                    SyncOptions::default(),
                                )
                                .background_check(
                                    &operation_id,
                                    &policy,
                                    &CancellationToken::new(),
                                )
                            }
                            Ok(_) => BackgroundFetchResult::ActionableError(
                                "repository identity changed".into(),
                            ),
                            Err(error) => BackgroundFetchResult::ActionableError(error),
                        }
                    }
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                match app
                    .git_background_scheduler
                    .finish(&identity, result, Instant::now())
                {
                    BackgroundNotification::Quiet => {}
                    BackgroundNotification::Incoming { commits } => {
                        app.status = app.trf(Msg::StatusGitIncoming, &[&commits.to_string()]);
                        cx.notify();
                    }
                    BackgroundNotification::AuthenticationNeeded => {
                        app.status = t(app.language, Msg::StatusGitAuthenticationPaused).into();
                        cx.notify();
                    }
                    BackgroundNotification::ActionableError(error) => {
                        app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }

    pub(super) fn arm_git_recovery(&mut self, cx: &mut Context<Self>) {
        let store = PolicyStore::new(default_git_sync_policy_path());
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        let registry = self.git_operations.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let policies = store.load().map_err(|error| error.to_string())?;
                    let data = markion::default_git_sync_data_dir();
                    for policy in policies.repositories {
                        let repository = match GitRepository::discover(
                            GitCommandRunner::new(executable.clone()),
                            &policy.identity.worktree_root,
                        ) {
                            Ok(repository) if repository.identity() == &policy.identity => {
                                repository
                            }
                            _ => continue,
                        };
                        let assessments = RecoveryManager::new(
                            repository,
                            JournalStore::new(data.join("journal.toml"), data.join("recovery")),
                        )
                        .assess()
                        .map_err(|error| error.to_string())?;
                        if let Some(operation_id) =
                            assessments
                                .into_iter()
                                .find_map(|assessment| match assessment {
                                    RecoveryAssessment::ConflictSession { operation_id } => {
                                        Some(operation_id)
                                    }
                                    _ => None,
                                })
                        {
                            registry.register(policy.identity.clone());
                            let admission = registry
                                .begin_exclusive(&policy.identity, &operation_id)
                                .map_err(|error| error.to_string())?;
                            return Ok::<_, String>(Some(admission));
                        }
                    }
                    Ok(None)
                })
                .await;
            let _ = this.update(cx, |app, cx| match result {
                Ok(Some(admission)) => {
                    app.git_conflict_admission = Some(admission);
                    app.status = t(app.language, Msg::ItemGitResolveConflict).into();
                    cx.notify();
                }
                Ok(None) => {}
                Err(error) => {
                    app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn setup_git_sync(
        &mut self,
        _: &SetupGitSync,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.begin_git_sync_setup(false, window, cx);
    }

    fn begin_git_sync_setup(
        &mut self,
        sync_after_setup: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let start = self.workspace_root.clone();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        let window_handle = window.window_handle();
        let language = self.language;
        let background_fetch = self.git_preferences.background_check;
        self.status = t(self.language, Msg::StatusGitSyncRunning).into();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let candidate = cx
                .background_spawn(async move {
                    let repository =
                        GitRepository::discover(GitCommandRunner::new(executable), &start)
                            .map_err(|error| error.to_string())?;
                    let state = repository.status().map_err(|error| error.to_string())?;
                    if !state.capabilities.supports_write_sync() {
                        return Err("this repository shape is read-only in Markion".to_string());
                    }
                    let target = repository
                        .resolve_sync_target()
                        .map_err(|error| error.to_string())?;
                    let policy = RepositoryPolicy {
                        identity: repository.identity().clone(),
                        target,
                        tracked_roots: vec![PathBuf::new()],
                        new_file_rules: vec![NewFileRule {
                            root: PathBuf::new(),
                            classes: BTreeSet::from([
                                NewFileClass::Markdown,
                                NewFileClass::Text,
                                NewFileClass::Image,
                            ]),
                        }],
                        message_template: String::new(),
                        include_device_name: false,
                        background_fetch,
                        last_confirmed_remote: None,
                        acknowledged_resource_omissions: Vec::new(),
                    };
                    Ok((policy, state.worktree.changes.len()))
                })
                .await;
            match candidate {
                Ok((policy, pending)) => {
                    let root = policy.identity.worktree_root.display().to_string();
                    let branch = policy.target.local_branch.clone();
                    let detail = tf(
                        language,
                        Msg::DialogGitSyncSetupDetail,
                        &[&root, &branch, &pending.to_string()],
                    );
                    let prompt = cx.update_window(window_handle, |_, window, cx| {
                        window.prompt(
                            PromptLevel::Warning,
                            t(language, Msg::ItemGitSyncSetup),
                            Some(&detail),
                            &[
                                PromptButton::ok(t(language, Msg::DialogButtonEnableGitSync)),
                                PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
                            ],
                            cx,
                        )
                    });
                    let accepted = match prompt {
                        Ok(prompt) => matches!(prompt.await, Ok(0)),
                        Err(_) => false,
                    };
                    let _ = this.update(cx, |app, cx| {
                        if !accepted {
                            app.status = t(app.language, Msg::StatusCanceled).into();
                            cx.notify();
                            return;
                        }
                        let store = PolicyStore::new(default_git_sync_policy_path());
                        match store.update(|policies| policies.upsert(policy.clone())) {
                            Ok(_) => {
                                if sync_after_setup {
                                    app.start_git_sync(policy, store, cx);
                                } else {
                                    app.git_operations.register(policy.identity.clone());
                                    app.status = app.trf(Msg::StatusGitSyncConfigured, &[&root]);
                                }
                            }
                            Err(error) => {
                                app.status =
                                    app.trf(Msg::StatusGitSyncFailed, &[&error.to_string()]);
                            }
                        }
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = this.update(cx, |app, cx| {
                        app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                        cx.notify();
                    });
                }
            }
        })
        .detach();
    }

    pub(super) fn sync_now(&mut self, _: &SyncNow, window: &mut Window, cx: &mut Context<Self>) {
        let store = PolicyStore::new(default_git_sync_policy_path());
        let policies = match store.load() {
            Ok(policies) => policies,
            Err(error) => {
                self.status = self.trf(Msg::StatusGitSyncFailed, &[&error.to_string()]);
                cx.notify();
                return;
            }
        };
        let active_tab_path = self.active_tab().path().map(Path::to_path_buf);
        let policy =
            match sync_now_route(policies, &self.workspace_root, active_tab_path.as_deref()) {
                SyncNowRoute::Configured(policy) => policy,
                SyncNowRoute::SetupExistingRepository => {
                    self.begin_git_sync_setup(true, window, cx);
                    return;
                }
            };
        self.start_git_sync(policy, store, cx);
    }

    fn start_git_sync(
        &mut self,
        policy: RepositoryPolicy,
        store: PolicyStore,
        cx: &mut Context<Self>,
    ) {
        if self.tabs.iter().any(|tab| {
            tab.path().is_some_and(|path| {
                path.starts_with(&policy.identity.worktree_root) && tab.external_conflict.is_some()
            })
        }) {
            self.status = self.trf(
                Msg::StatusGitSyncFailed,
                &["resolve disk conflicts before synchronizing"],
            );
            cx.notify();
            return;
        }
        self.git_operations.register(policy.identity.clone());
        let buffers = self
            .tabs
            .iter()
            .filter_map(|tab| {
                let tab = tab.document_tab()?;
                let path = tab.document.path()?.to_path_buf();
                path.starts_with(&policy.identity.worktree_root)
                    .then(|| GitBufferSnapshot {
                        instance: tab.document.instance_id(),
                        version: tab.document.version(),
                        path,
                        known: tab.document.disk_identity().cloned(),
                        text: tab.document.text().to_string(),
                        dirty: tab.document.is_dirty(),
                    })
            })
            .collect::<Vec<_>>();
        let images = self
            .tabs
            .iter()
            .filter(|tab| tab.is_image())
            .filter_map(|tab| {
                let path = tab.path()?.to_path_buf();
                path.starts_with(&policy.identity.worktree_root)
                    .then(|| GitImageSnapshot {
                        fingerprint: file_fingerprint(&path),
                        path,
                    })
            })
            .collect::<Vec<_>>();
        let registry = self.git_operations.clone();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        self.status = t(self.language, Msg::StatusGitSyncRunning).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    run_git_sync(registry, store, executable, policy, buffers, images)
                })
                .await;
            let _ = this.update(cx, |app, cx| match result {
                Ok(completion) => app.apply_git_sync_completion(completion, cx),
                Err(error) => {
                    app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn resolve_git_conflict(
        &mut self,
        _: &ResolveGitConflict,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(admission) = self.git_conflict_admission.take() else {
            self.status = self.trf(
                Msg::StatusGitSyncFailed,
                &["there is no Markion-owned conflict session"],
            );
            cx.notify();
            return;
        };
        let Some(identity) = admission.identity().cloned() else {
            self.status = self.trf(
                Msg::StatusGitSyncFailed,
                &["the conflict repository is no longer registered"],
            );
            cx.notify();
            return;
        };
        let operation_id = admission.operation_id().to_string();
        let store = PolicyStore::new(default_git_sync_policy_path());
        let policy = match store.load().ok().and_then(|policies| {
            policies
                .repositories
                .into_iter()
                .find(|policy| policy.identity == identity)
        }) {
            Some(policy) => policy,
            None => {
                self.git_conflict_admission = Some(admission);
                self.status = self.trf(
                    Msg::StatusGitSyncFailed,
                    &["the repository policy is unavailable"],
                );
                cx.notify();
                return;
            }
        };
        let buffers = self
            .tabs
            .iter()
            .filter_map(|tab| {
                let tab = tab.document_tab()?;
                let path = tab.document.path()?.to_path_buf();
                path.starts_with(&identity.worktree_root)
                    .then(|| GitBufferSnapshot {
                        instance: tab.document.instance_id(),
                        version: tab.document.version(),
                        path,
                        known: tab.document.disk_identity().cloned(),
                        text: tab.document.text().to_string(),
                        dirty: tab.document.is_dirty(),
                    })
            })
            .collect::<Vec<_>>();
        let images = self
            .tabs
            .iter()
            .filter(|tab| tab.is_image())
            .filter_map(|tab| {
                let path = tab.path()?.to_path_buf();
                path.starts_with(&identity.worktree_root)
                    .then(|| GitImageSnapshot {
                        fingerprint: file_fingerprint(&path),
                        path,
                    })
            })
            .collect::<Vec<_>>();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        let language = self.language;
        let window_handle = window.window_handle();
        self.status = t(language, Msg::ItemGitResolveConflict).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let mut admission = Some(admission);
            let prepared = cx
                .background_spawn({
                    let executable = executable.clone();
                    let identity = identity.clone();
                    let operation_id = operation_id.clone();
                    async move {
                        let repository = GitRepository::discover(
                            GitCommandRunner::new(executable),
                            &identity.worktree_root,
                        )
                        .map_err(|error| error.to_string())?;
                        if repository.identity() != &identity {
                            return Err("repository identity changed".to_string());
                        }
                        let data = markion::default_git_sync_data_dir();
                        let manager = ConflictManager::new(
                            repository,
                            JournalStore::new(data.join("journal.toml"), data.join("recovery")),
                        );
                        let session = manager
                            .restore_session(&operation_id, &CancellationToken::new())
                            .map_err(|error| error.to_string())?;
                        Ok::<_, String>((manager, session))
                    }
                })
                .await;
            let (manager, session) = match prepared {
                Ok(prepared) => prepared,
                Err(error) => {
                    let _ = this.update(cx, |app, cx| {
                        app.git_conflict_admission = admission.take();
                        app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                        cx.notify();
                    });
                    return;
                }
            };

            for file in session.files.clone() {
                let path = file.path.display().to_string();
                let detail = tf(language, Msg::DialogGitConflictDetail, &[&path]);
                let local_label = if file.local.is_some() {
                    t(language, Msg::DialogButtonThisComputer)
                } else {
                    t(language, Msg::DialogButtonDelete)
                };
                let remote_label = if file.remote.is_some() {
                    t(language, Msg::DialogButtonRemoteVersion)
                } else {
                    t(language, Msg::DialogButtonDelete)
                };
                let prompt = cx.update_window(window_handle, |_, window, cx| {
                    window.prompt(
                        PromptLevel::Warning,
                        t(language, Msg::ItemGitResolveConflict),
                        Some(&detail),
                        &[
                            PromptButton::new(local_label),
                            PromptButton::new(remote_label),
                            PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
                        ],
                        cx,
                    )
                });
                let resolution = match prompt {
                    Ok(prompt) => match prompt.await {
                        Ok(0) if file.local.is_some() => {
                            Some(ConflictResolution::Choose(ConflictSide::ThisComputer))
                        }
                        Ok(0) => Some(ConflictResolution::Delete),
                        Ok(1) if file.remote.is_some() => {
                            Some(ConflictResolution::Choose(ConflictSide::Remote))
                        }
                        Ok(1) => Some(ConflictResolution::Delete),
                        _ => None,
                    },
                    Err(_) => None,
                };
                let Some(resolution) = resolution else {
                    let _ = this.update(cx, |app, cx| {
                        app.git_conflict_admission = admission.take();
                        app.status = t(app.language, Msg::StatusCanceled).into();
                        cx.notify();
                    });
                    return;
                };
                let resolved = cx
                    .background_spawn({
                        let manager = manager.clone();
                        let session = session.clone();
                        let path = file.path.clone();
                        async move {
                            manager
                                .mark_resolved(
                                    &session,
                                    &path,
                                    resolution,
                                    &CancellationToken::new(),
                                )
                                .map_err(|error| error.to_string())
                        }
                    })
                    .await;
                if let Err(error) = resolved {
                    let _ = this.update(cx, |app, cx| {
                        app.git_conflict_admission = admission.take();
                        app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                        cx.notify();
                    });
                    return;
                }
            }

            let prompt = cx.update_window(window_handle, |_, window, cx| {
                window.prompt(
                    PromptLevel::Warning,
                    t(language, Msg::ItemGitResolveConflict),
                    None,
                    &[
                        PromptButton::ok(t(language, Msg::DialogButtonFinishAndPush)),
                        PromptButton::new(t(language, Msg::DialogButtonFinishMerge)),
                        PromptButton::new(t(language, Msg::DialogButtonAbortMerge)),
                        PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
                    ],
                    cx,
                )
            });
            let action = match prompt {
                Ok(prompt) => match prompt.await {
                    Ok(0) => Some(GitConflictFinalAction::FinishAndPush),
                    Ok(1) => Some(GitConflictFinalAction::FinishMerge),
                    Ok(2) => Some(GitConflictFinalAction::Abort),
                    _ => None,
                },
                Err(_) => None,
            };
            let Some(action) = action else {
                let _ = this.update(cx, |app, cx| {
                    app.git_conflict_admission = admission.take();
                    app.status = t(app.language, Msg::StatusCanceled).into();
                    cx.notify();
                });
                return;
            };

            let finalization = cx
                .background_spawn({
                    let manager = manager.clone();
                    let session = session.clone();
                    let target = policy.target.clone();
                    let identity = identity.clone();
                    let store = store.clone();
                    async move {
                        let result = match action {
                            GitConflictFinalAction::FinishAndPush => manager
                                .finish_and_push(&session, &target, &CancellationToken::new())
                                .map(|commit| {
                                    store
                                        .update(|policies| {
                                            if let Some(policy) = policies
                                                .repositories
                                                .iter_mut()
                                                .find(|policy| policy.identity == identity)
                                            {
                                                policy.last_confirmed_remote = Some(commit.clone());
                                            }
                                        })
                                        .map_err(|error| error.to_string())?;
                                    Ok::<_, String>(())
                                })
                                .map_err(|error| error.to_string())
                                .and_then(|result| result),
                            GitConflictFinalAction::FinishMerge => manager
                                .finish_merge(&session, &CancellationToken::new())
                                .map(|_| ())
                                .map_err(|error| error.to_string()),
                            GitConflictFinalAction::Abort => manager
                                .abort(&session, &CancellationToken::new())
                                .map_err(|error| error.to_string()),
                        };
                        let merge_active = identity.git_dir.join("MERGE_HEAD").is_file();
                        let buffers = buffers
                            .into_iter()
                            .map(|buffer| GitBufferResult {
                                instance: buffer.instance,
                                version: buffer.version,
                                source: read_document_source(&buffer.path).ok(),
                                previous_path: buffer.path.clone(),
                                path: buffer.path,
                            })
                            .collect::<Vec<_>>();
                        let images = images
                            .into_iter()
                            .map(|image| GitImageResult {
                                changed: file_fingerprint(&image.path) != image.fingerprint,
                                previous_path: image.path.clone(),
                                path: image.path,
                            })
                            .collect::<Vec<_>>();
                        (result, merge_active, buffers, images)
                    }
                })
                .await;
            let (result, merge_active, buffers, images) = finalization;
            let _ = this.update(cx, |app, cx| {
                if result.is_err() && merge_active {
                    app.git_conflict_admission = admission.take();
                    app.status = app.trf(Msg::StatusGitSyncFailed, &[result.as_ref().unwrap_err()]);
                    cx.notify();
                    return;
                }
                let outcome = match result {
                    Ok(()) => SyncOutcome::UpToDate,
                    Err(reason) => SyncOutcome::NeedsAttention {
                        phase: markion_git_sync::OperationPhase::Pushing,
                        reason,
                    },
                };
                app.apply_git_sync_completion(
                    GitSyncCompletion {
                        _exclusive: admission.take().expect("conflict admission is owned"),
                        outcome,
                        buffers,
                        images,
                    },
                    cx,
                );
                if matches!(action, GitConflictFinalAction::Abort) {
                    app.status = t(app.language, Msg::StatusCanceled).into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn apply_git_sync_completion(
        &mut self,
        completion: GitSyncCompletion,
        cx: &mut Context<Self>,
    ) {
        let GitSyncCompletion {
            _exclusive: exclusive,
            outcome,
            buffers,
            images,
        } = completion;
        let conflict_active = matches!(
            &outcome,
            SyncOutcome::NeedsAttention {
                phase: markion_git_sync::OperationPhase::Resolving,
                ..
            }
        );
        for result in buffers {
            let Some(index) = self.tabs.iter().position(|tab| {
                tab.document_tab()
                    .is_some_and(|tab| tab.document.instance_id() == result.instance)
            }) else {
                continue;
            };
            let tab = &mut self.tabs[index];
            if tab.document.version() != result.version
                || tab.document.path() != Some(result.previous_path.as_path())
            {
                continue;
            }
            if result.path != result.previous_path {
                match result.source {
                    Some((text, identity)) => {
                        if tab
                            .document
                            .apply_external_move_checked(
                                result.instance,
                                result.version,
                                result.path,
                                text,
                                identity,
                            )
                            .is_ok()
                        {
                            tab.external_conflict = None;
                            tab.undo_stack.clear();
                            tab.redo_stack.clear();
                            tab.reset_preview_list();
                        }
                    }
                    None => tab.external_conflict = Some(DiskState::Missing),
                }
                continue;
            }
            match result.source {
                Some((text, identity)) if text == tab.document.text() => {
                    tab.document.record_disk_identity(identity, true);
                    tab.external_conflict = None;
                }
                Some((text, identity)) => {
                    if tab
                        .document
                        .apply_external_reload_checked(
                            result.instance,
                            result.version,
                            text,
                            identity,
                        )
                        .is_ok()
                    {
                        tab.external_conflict = None;
                        tab.undo_stack.clear();
                        tab.redo_stack.clear();
                        tab.reset_preview_list();
                    }
                }
                None => tab.external_conflict = Some(DiskState::Missing),
            }
        }
        for image in images {
            let Some(index) = self.tabs.iter().position(|tab| {
                tab.is_image() && tab.path() == Some(image.previous_path.as_path())
            }) else {
                continue;
            };
            if image.changed || image.path != image.previous_path {
                self.release_tab_image_claims(index, cx);
            }
            if image.path != image.previous_path
                && let WorkspaceTab::Image(tab) = &mut self.tabs[index]
            {
                tab.path = image.path.clone();
                tab.key = PreviewImageKey::from_local_path(&image.path);
            }
        }
        self.status = match outcome {
            SyncOutcome::Synchronized {
                pending_local_changes,
                ..
            } if pending_local_changes > 0 => self.trf(
                Msg::StatusGitSyncFailed,
                &["synchronized captured changes; newer local edits remain"],
            ),
            SyncOutcome::Synchronized { .. } | SyncOutcome::UpToDate => {
                t(self.language, Msg::StatusGitSyncComplete).into()
            }
            SyncOutcome::NeedsAttention { reason, .. }
            | SyncOutcome::AwaitingUpload { reason, .. }
            | SyncOutcome::UncertainDelivery { reason, .. } => {
                self.trf(Msg::StatusGitSyncFailed, &[&reason])
            }
            SyncOutcome::CommittedLocally { .. } => {
                t(self.language, Msg::StatusGitSyncComplete).into()
            }
        };
        if conflict_active {
            self.git_conflict_admission = Some(exclusive);
        } else {
            drop(exclusive);
        }
        self.refresh_file_tree(cx);
        cx.notify();
    }
}

fn run_git_sync(
    registry: GitOperationRegistry,
    store: PolicyStore,
    executable: String,
    mut policy: RepositoryPolicy,
    buffers: Vec<GitBufferSnapshot>,
    images: Vec<GitImageSnapshot>,
) -> Result<GitSyncCompletion, String> {
    let operation_id = format!(
        "sync-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let exclusive = registry
        .begin_exclusive(&policy.identity, &operation_id)
        .map_err(|error| error.to_string())?;
    if exclusive.coalesced() {
        return Err("this synchronization request is already running".to_string());
    }
    for buffer in &buffers {
        if buffer.dirty {
            save_text_snapshot(&buffer.path, buffer.known.as_ref(), &buffer.text)
                .map_err(|error| error.to_string())?;
        }
    }
    let repository = GitRepository::discover(
        GitCommandRunner::new(executable),
        &policy.identity.worktree_root,
    )
    .map_err(|error| error.to_string())?;
    if repository.identity() != &policy.identity {
        return Err("repository identity changed".to_string());
    }
    for buffer in &buffers {
        let is_markdown = buffer
            .path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "md" | "markdown" | "mdown"
                )
            });
        if is_markdown {
            let report = inspect_note_attachments(&repository, &policy, &buffer.path, &buffer.text)
                .map_err(|error| error.to_string())?;
            if let Some(issue) = report
                .issues
                .iter()
                .find(|issue| !issue.previously_acknowledged)
            {
                return Err(format!(
                    "attachment '{}' requires attention ({:?})",
                    issue.authored_url, issue.kind
                ));
            }
        }
    }
    let state = repository.status().map_err(|error| error.to_string())?;
    let paths = state
        .worktree
        .changes
        .iter()
        .map(|change| change.path.clone())
        .collect::<Vec<_>>();
    let before_head = state.head.clone();
    let worktree_root = policy.identity.worktree_root.clone();
    let plan = SyncPlan {
        operation_id: operation_id.clone(),
        identity: policy.identity.clone(),
        target: policy.target.clone(),
        expected_head: state.head,
        content_fingerprints: paths
            .iter()
            .map(|path| {
                let metadata = fs::metadata(policy.identity.worktree_root.join(path)).ok();
                (
                    path.clone(),
                    metadata
                        .map(|metadata| format!("{}", metadata.len()))
                        .unwrap_or_else(|| "missing".to_string()),
                )
            })
            .collect(),
        message: policy.render_message(paths.len(), None),
        paths,
    };
    let data = markion::default_git_sync_data_dir();
    let journal = JournalStore::new(data.join("journal.toml"), data.join("recovery"));
    let engine = GitSyncEngine::new(repository, journal, SyncOptions::default());
    let cancellation = CancellationToken::new();
    let outcome = engine
        .sync_now(&plan, &mut policy, None, None, &cancellation)
        .map_err(|error| error.to_string())?;
    store
        .update(|policies| policies.upsert(policy))
        .map_err(|error| error.to_string())?;
    let resolving = matches!(
        outcome,
        SyncOutcome::NeedsAttention {
            phase: markion_git_sync::OperationPhase::Resolving,
            ..
        }
    );
    let after_head = engine
        .repository()
        .status()
        .map_err(|error| error.to_string())?
        .head;
    let renames = match (before_head.as_ref(), after_head.as_ref()) {
        (Some(before), Some(after)) if before != after && !resolving => engine
            .repository()
            .renames_between(before, after, &cancellation)
            .map_err(|error| error.to_string())?,
        _ => Vec::new(),
    };
    let mapped_path = |path: &Path| {
        let relative = path.strip_prefix(&worktree_root).ok();
        relative
            .and_then(|relative| {
                renames
                    .iter()
                    .find_map(|(from, to)| (from == relative).then_some(to))
            })
            .map(|relative| worktree_root.join(relative))
            .unwrap_or_else(|| path.to_path_buf())
    };
    let buffers = if resolving {
        Vec::new()
    } else {
        buffers
            .into_iter()
            .map(|buffer| {
                let path = mapped_path(&buffer.path);
                GitBufferResult {
                    instance: buffer.instance,
                    version: buffer.version,
                    source: read_document_source(&path).ok(),
                    previous_path: buffer.path,
                    path,
                }
            })
            .collect()
    };
    let images = images
        .into_iter()
        .map(|image| {
            let path = mapped_path(&image.path);
            GitImageResult {
                changed: file_fingerprint(&path) != image.fingerprint,
                previous_path: image.path,
                path,
            }
        })
        .collect();
    Ok(GitSyncCompletion {
        _exclusive: exclusive,
        outcome,
        buffers,
        images,
    })
}

fn file_fingerprint(path: &Path) -> Option<(u64, u128)> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some((metadata.len(), modified))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_now_without_markion_policy_starts_existing_repository_setup() {
        assert_eq!(
            sync_now_route(SyncPolicies::empty(), Path::new("notes"), None),
            SyncNowRoute::SetupExistingRepository
        );
    }

    #[test]
    fn sync_now_uses_workspace_policy_when_active_tab_is_foreign() {
        let repository_root = PathBuf::from("repository");
        let policy = RepositoryPolicy {
            identity: markion_git_sync::RepositoryIdentity::new(
                repository_root.clone(),
                repository_root.join(".git"),
                repository_root.join(".git"),
            ),
            target: markion_git_sync::SyncTarget {
                local_branch: "main".into(),
                remote: "origin".into(),
                remote_branch: "main".into(),
                destination_ref: "refs/heads/main".into(),
                fetch_url: "https://example.invalid/notes.git".into(),
                push_url: "https://example.invalid/notes.git".into(),
            },
            tracked_roots: vec![PathBuf::new()],
            new_file_rules: Vec::new(),
            message_template: String::new(),
            include_device_name: false,
            background_fetch: false,
            last_confirmed_remote: None,
            acknowledged_resource_omissions: Vec::new(),
        };
        let mut policies = SyncPolicies::empty();
        policies.upsert(policy.clone());

        assert_eq!(
            sync_now_route(
                policies,
                &repository_root.join("notes"),
                Some(Path::new("another-repository/foreign.md")),
            ),
            SyncNowRoute::Configured(policy)
        );
    }
}
