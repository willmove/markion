use super::*;
use markion_git_sync::{
    Authentication, BackgroundFetchResult, BackgroundNotification, CancellationToken,
    ConflictManager, ExclusiveAdmission, GitCommandRunner, GitRepository, GitSyncEngine,
    JournalStore, NewFileClass, NewFileRule, OnboardingService, RecoveryAssessment,
    RecoveryManager, RemoteTransport, RemoteUrl, RepositoryPolicy, RepositoryState, SyncOptions,
    SyncOutcome, SyncPlan, SyncPolicies, SyncTarget, inspect_note_attachments,
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

#[derive(Clone)]
struct GitVersionCommitRequest {
    expected_head: Option<markion_git_sync::GitObjectId>,
    selected: HashSet<PathBuf>,
    reviewed_fingerprints: HashMap<PathBuf, String>,
    message: String,
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

struct GitOnboardingSuccess {
    worktree: PathBuf,
    policy: Option<RepositoryPolicy>,
    pending_files: usize,
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
        if !self.git_preferences.background_check || self.git_ui.running.is_some() {
            self.git_background_scheduler.deactivate();
            return;
        }
        let workspace_root = self.workspace_root.clone();
        let globally_enabled = self.git_preferences.background_check;
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        let requested_workspace = workspace_root.clone();
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
                                    .iter()
                                    .all(|helper| helper.supports_noninteractive_use)
                            }
                            RemoteTransport::Ssh => false,
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
                if app.workspace_root != requested_workspace
                    || !app.git_preferences.background_check
                    || app.git_ui.running.is_some()
                {
                    return None;
                }
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
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        let registry = self.git_operations.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let journal = git_panel::journal();
                    let mut guards = Vec::new();
                    let mut seen = HashSet::new();
                    for checkpoint in journal.load().map_err(|e| e.to_string())?.active {
                        if !seen.insert(checkpoint.identity.clone()) {
                            continue;
                        }
                        let repository = match GitRepository::discover(
                            GitCommandRunner::new(executable.clone()),
                            &checkpoint.identity.worktree_root,
                        ) {
                            Ok(repository) if repository.identity() == &checkpoint.identity => {
                                repository
                            }
                            _ => continue,
                        };
                        let assessments = RecoveryManager::new(repository, journal.clone())
                            .assess()
                            .map_err(|e| e.to_string())?;
                        let pending =
                            assessments
                                .into_iter()
                                .find_map(|assessment| match assessment {
                                    RecoveryAssessment::ConflictSession { operation_id } => {
                                        Some((operation_id, true))
                                    }
                                    RecoveryAssessment::OwnedStaging { operation_id } => {
                                        Some((operation_id, false))
                                    }
                                    RecoveryAssessment::ExternalState { operation_id }
                                        if matches!(
                                            checkpoint.phase,
                                            markion_git_sync::OperationPhase::Integrating
                                                | markion_git_sync::OperationPhase::Resolving
                                                | markion_git_sync::OperationPhase::Committing
                                                | markion_git_sync::OperationPhase::Staging
                                        ) =>
                                    {
                                        Some((operation_id, false))
                                    }
                                    _ => None,
                                });
                        registry.register(checkpoint.identity.clone());
                        if let Some((id, conflict)) = pending
                            && let Ok(guard) = registry.begin_exclusive(&checkpoint.identity, &id)
                            && !guard.coalesced()
                        {
                            guards.push((guard, conflict));
                        }
                    }
                    Ok::<_, String>(guards)
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                match result {
                    Ok(guards) => {
                        for (guard, conflict) in guards {
                            let Some(identity) = guard.identity().cloned() else {
                                continue;
                            };
                            if conflict
                                && app.workspace_root.starts_with(&identity.worktree_root)
                                && app.git_conflict_admission.is_none()
                            {
                                app.git_conflict_admission = Some(guard);
                            } else {
                                app.git_ui.recovery_guards.insert(identity, guard);
                            }
                        }
                    }
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                app.refresh_git_details(cx);
                cx.notify();
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
        self.open_git_onboarding(false, window, cx);
    }

    fn open_git_onboarding(
        &mut self,
        sync_after_setup: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.git_ui.running.is_some() || self.git_ui.settings.is_some() {
            self.status = self.git_label(GitMsg::Busy).into();
            cx.notify();
            return;
        }
        let current_repository_hint = self
            .workspace_root
            .ancestors()
            .any(|directory| directory.join(".git").exists());
        let current_folder_has_notes = self
            .file_tree
            .as_ref()
            .is_some_and(|tree| tree.entries.iter().any(|entry| entry.file_kind.is_some()))
            || self.tabs.iter().any(|tab| {
                tab.path().is_some_and(|path| {
                    path.starts_with(&self.workspace_root) && NewFileClass::classify(path).is_some()
                })
            });
        let mode = if current_repository_hint {
            git_panel::GitOnboardingMode::UseCurrentFolder
        } else if current_folder_has_notes {
            git_panel::GitOnboardingMode::InitializeFolder
        } else {
            git_panel::GitOnboardingMode::CloneRepository
        };
        self.git_ui.onboarding = Some(git_panel::GitOnboarding {
            mode,
            fields: [
                SearchFieldState::default(),
                SearchFieldState::default(),
                SearchFieldState::new(if mode == git_panel::GitOnboardingMode::InitializeFolder {
                    "main"
                } else {
                    ""
                }),
                SearchFieldState::default(),
                SearchFieldState::default(),
            ],
            advanced: false,
            destination_edited: false,
            sync_after_setup,
            busy: false,
            cancellation: None,
            error: None,
        });
        self.search_visible = false;
        self.search_control_focus = None;
        self.search_focus = Some(SearchField::GitSetup(0));
        self.active_menu = None;
        window.focus(&self.focus_handle);
        cx.notify();
    }

    pub(super) fn set_git_onboarding_mode(
        &mut self,
        mode: git_panel::GitOnboardingMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(setup) = &mut self.git_ui.onboarding else {
            return;
        };
        if setup.busy {
            return;
        }
        let previous = setup.mode;
        setup.mode = mode;
        setup.error = None;
        if mode == git_panel::GitOnboardingMode::InitializeFolder
            && setup.fields[2].buffer.trim().is_empty()
        {
            setup.fields[2].set_text("main");
        } else if mode == git_panel::GitOnboardingMode::CloneRepository
            && previous == git_panel::GitOnboardingMode::InitializeFolder
            && setup.fields[2].buffer == "main"
        {
            setup.fields[2].set_text("");
        }
        self.search_focus = Some(SearchField::GitSetup(0));
        window.focus(&self.focus_handle);
        self.update_git_onboarding_defaults(0);
        cx.notify();
    }

    pub(super) fn update_git_onboarding_defaults(&mut self, changed_field: usize) {
        if changed_field == 1 {
            if let Some(setup) = &mut self.git_ui.onboarding {
                setup.destination_edited = true;
            }
            return;
        }
        let Some(setup) = &self.git_ui.onboarding else {
            return;
        };
        if changed_field != 0
            || setup.mode != git_panel::GitOnboardingMode::CloneRepository
            || setup.destination_edited
        {
            return;
        }
        let remote = setup.fields[0].buffer.clone();
        let suggested = suggested_clone_destination(&self.workspace_root, &remote)
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        if let Some(setup) = &mut self.git_ui.onboarding {
            setup.fields[1].set_text(suggested);
        }
    }

    pub(super) fn toggle_git_onboarding_advanced(&mut self, cx: &mut Context<Self>) {
        if let Some(setup) = &mut self.git_ui.onboarding
            && !setup.busy
        {
            setup.advanced = !setup.advanced;
            cx.notify();
        }
    }

    pub(super) fn cancel_git_onboarding(&mut self, cx: &mut Context<Self>) {
        let Some(setup) = &mut self.git_ui.onboarding else {
            return;
        };
        if let Some(cancellation) = &setup.cancellation {
            cancellation.cancel();
            setup.error = None;
            self.status = t(self.language, Msg::StatusCanceled).into();
        } else {
            self.git_ui.onboarding = None;
            self.search_focus = None;
            self.status = t(self.language, Msg::StatusCanceled).into();
        }
        cx.notify();
    }

    pub(super) fn submit_git_onboarding(&mut self, cx: &mut Context<Self>) {
        let Some(setup) = &self.git_ui.onboarding else {
            return;
        };
        if setup.busy || self.git_ui.running.is_some() {
            return;
        }
        if setup.mode == git_panel::GitOnboardingMode::InitializeFolder
            && self.tabs.iter().any(|tab| {
                tab.is_dirty()
                    && tab
                        .path()
                        .is_some_and(|path| path.starts_with(&self.workspace_root))
            })
        {
            let error = self.git_label(GitMsg::SaveFirst).to_string();
            if let Some(setup) = &mut self.git_ui.onboarding {
                setup.error = Some(error);
            }
            cx.notify();
            return;
        }
        let mode = setup.mode;
        let remote = setup.fields[0].buffer.trim().to_string();
        let destination = setup.fields[1].buffer.trim().to_string();
        let branch = setup.fields[2].buffer.trim().to_string();
        let author_name = setup.fields[3].buffer.trim().to_string();
        let author_email = setup.fields[4].buffer.trim().to_string();
        if author_name.is_empty() != author_email.is_empty() {
            let error = self.git_label(GitMsg::SetupAuthorPair).to_string();
            if let Some(setup) = &mut self.git_ui.onboarding {
                setup.advanced = true;
                setup.error = Some(error);
            }
            cx.notify();
            return;
        }
        if matches!(
            mode,
            git_panel::GitOnboardingMode::CloneRepository
                | git_panel::GitOnboardingMode::InitializeFolder
        ) && validate_user_remote(&remote, self.language).is_err()
        {
            let error = self.git_label(GitMsg::SetupRemoteHelp).to_string();
            if let Some(setup) = &mut self.git_ui.onboarding {
                setup.error = Some(error);
            }
            cx.notify();
            return;
        }
        let workspace_root = self.workspace_root.clone();
        let clone_destination = if mode == git_panel::GitOnboardingMode::CloneRepository {
            resolve_clone_destination(&workspace_root, &remote, &destination, self.language)
        } else {
            Ok(workspace_root.clone())
        };
        let clone_destination = match clone_destination {
            Ok(destination) => destination,
            Err(error) => {
                if let Some(setup) = &mut self.git_ui.onboarding {
                    setup.error = Some(error);
                }
                cx.notify();
                return;
            }
        };
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        let background_fetch = self.git_preferences.background_check;
        let language = self.language;
        let sync_after_setup = setup.sync_after_setup;
        let cancellation = CancellationToken::new();
        if let Some(setup) = &mut self.git_ui.onboarding {
            setup.busy = true;
            setup.error = None;
            setup.cancellation = Some(cancellation.clone());
        }
        self.status = t(self.language, Msg::StatusGitSyncRunning).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    run_git_onboarding(
                        executable,
                        language,
                        mode,
                        workspace_root,
                        clone_destination,
                        remote,
                        branch,
                        author_name,
                        author_email,
                        background_fetch,
                        cancellation,
                    )
                })
                .await;
            let _ = this.update(cx, |app, cx| match result {
                Ok(success) => {
                    app.git_ui.onboarding = None;
                    app.search_focus = None;
                    if app.workspace_root != success.worktree {
                        app.switch_to_workspace(success.worktree.clone(), cx);
                    }
                    if let Some(policy) = success.policy {
                        app.git_operations.register(policy.identity.clone());
                        let root = policy.identity.worktree_root.display().to_string();
                        if sync_after_setup {
                            app.start_git_sync(
                                policy,
                                PolicyStore::new(default_git_sync_policy_path()),
                                cx,
                            );
                        } else {
                            app.status = app.trf(Msg::StatusGitSyncConfigured, &[&root]);
                            app.refresh_git_details(cx);
                        }
                    } else {
                        app.status = app.git_label(GitMsg::FirstPublicationPending).into();
                        app.refresh_git_details(cx);
                    }
                    let _ = success.pending_files;
                    cx.notify();
                }
                Err(error) => {
                    if let Some(setup) = &mut app.git_ui.onboarding {
                        setup.busy = false;
                        setup.cancellation = None;
                        setup.error = Some(error.clone());
                    }
                    app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                    app.refresh_git_details(cx);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn secondary_git_operation(
        &mut self,
        operation: markion_git_sync::OperationKind,
        cx: &mut Context<Self>,
    ) {
        let store = PolicyStore::new(default_git_sync_policy_path());
        match store.load() {
            Ok(policies) => match sync_now_route(policies, &self.workspace_root, None) {
                SyncNowRoute::Configured(policy) => {
                    self.start_git_operation(policy, store, operation, cx)
                }
                _ => {
                    self.status = t(self.language, Msg::StatusGitSyncNotConfigured).into();
                    cx.notify();
                }
            },
            Err(error) => {
                self.status = self.trf(Msg::StatusGitSyncFailed, &[&error.to_string()]);
                cx.notify();
            }
        }
    }

    pub(super) fn commit_locally(
        &mut self,
        _: &CommitLocally,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.secondary_git_operation(markion_git_sync::OperationKind::CommitLocally, cx);
    }

    pub(super) fn check_remote(&mut self, _: &CheckRemote, _: &mut Window, cx: &mut Context<Self>) {
        self.secondary_git_operation(markion_git_sync::OperationKind::CheckRemote, cx);
    }

    pub(super) fn pull_updates(&mut self, _: &PullUpdates, _: &mut Window, cx: &mut Context<Self>) {
        self.secondary_git_operation(markion_git_sync::OperationKind::PullUpdates, cx);
    }

    pub(super) fn push_commits(&mut self, _: &PushCommits, _: &mut Window, cx: &mut Context<Self>) {
        self.secondary_git_operation(markion_git_sync::OperationKind::PushCommits, cx);
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
                    self.open_git_onboarding(true, window, cx);
                    return;
                }
            };
        self.start_git_sync(policy, store, cx);
    }

    pub(super) fn start_git_sync(
        &mut self,
        policy: RepositoryPolicy,
        store: PolicyStore,
        cx: &mut Context<Self>,
    ) {
        self.start_git_operation(policy, store, markion_git_sync::OperationKind::SyncNow, cx);
    }

    pub(super) fn create_local_version(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.git_ui.running.is_some() {
            self.status = self.git_label(GitMsg::Busy).into();
            cx.notify();
            return;
        }
        let Some(details) = self.git_ui.snapshot.as_ref() else {
            return;
        };
        let Some(policy) = details.policy.clone() else {
            self.status = t(self.language, Msg::StatusGitSyncNotConfigured).into();
            cx.notify();
            return;
        };
        let Some(draft) = self.git_ui.commit_draft.as_ref() else {
            return;
        };
        if draft.message.buffer.trim().is_empty() || draft.selected.is_empty() {
            self.status = self.git_label(GitMsg::VersionInputRequired).into();
            cx.notify();
            return;
        }
        if draft.identity != details.identity || draft.expected_head != details.state.head {
            self.status = self.git_label(GitMsg::VersionDraftStale).into();
            cx.notify();
            return;
        }
        let available = git_panel::committable_paths(details);
        if !draft.selected.iter().all(|path| available.contains(path)) {
            self.status = self.git_label(GitMsg::VersionDraftStale).into();
            cx.notify();
            return;
        }
        let request = GitVersionCommitRequest {
            expected_head: draft.expected_head.clone(),
            selected: draft.selected.clone(),
            reviewed_fingerprints: draft.reviewed_fingerprints.clone(),
            message: draft.message.buffer.trim().to_string(),
        };
        self.search_focus = None;
        window.focus(&self.focus_handle);
        self.start_git_operation_inner(
            policy,
            PolicyStore::new(default_git_sync_policy_path()),
            markion_git_sync::OperationKind::CommitLocally,
            Some(request),
            cx,
        );
    }

    fn start_git_operation(
        &mut self,
        policy: RepositoryPolicy,
        store: PolicyStore,
        operation: markion_git_sync::OperationKind,
        cx: &mut Context<Self>,
    ) {
        self.start_git_operation_inner(policy, store, operation, None, cx);
    }

    fn start_git_operation_inner(
        &mut self,
        policy: RepositoryPolicy,
        store: PolicyStore,
        operation: markion_git_sync::OperationKind,
        version_request: Option<GitVersionCommitRequest>,
        cx: &mut Context<Self>,
    ) {
        use markion_git_sync::OperationKind;
        if self.git_ui.running.is_some() || self.git_ui.settings.is_some() {
            self.status = self.git_label(GitMsg::Busy).into();
            cx.notify();
            return;
        }
        let captures_content = matches!(
            operation,
            OperationKind::SyncNow | OperationKind::CommitLocally
        );
        if operation == OperationKind::PullUpdates
            && self.tabs.iter().any(|tab| {
                tab.is_dirty()
                    && tab
                        .path()
                        .is_some_and(|path| path.starts_with(&policy.identity.worktree_root))
            })
        {
            self.status = self.git_label(GitMsg::SaveFirst).into();
            cx.notify();
            return;
        }
        if captures_content
            && self.tabs.iter().any(|tab| {
                tab.path().is_some_and(|path| {
                    path.starts_with(&policy.identity.worktree_root)
                        && version_request.as_ref().is_none_or(|request| {
                            path.strip_prefix(&policy.identity.worktree_root)
                                .is_ok_and(|relative| request.selected.contains(relative))
                        })
                        && tab.external_conflict.is_some()
                })
            })
        {
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
        let generation = self.git_operations.content_generation(&policy.identity);
        let registry = self.git_operations.clone();
        let phase = self.git_ui.phase.clone();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".to_string());
        let identity = policy.identity.clone();
        let cancellation = CancellationToken::new();
        let authored_version = version_request.is_some();
        self.git_ui.retry_operation = None;
        self.git_ui.running = Some((identity.clone(), operation, cancellation.clone()));
        self.status = git_panel::git_operation_label(self.language, operation).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    run_git_operation(
                        registry,
                        store,
                        executable,
                        policy,
                        buffers,
                        images,
                        operation,
                        cancellation,
                        generation,
                        phase,
                        version_request,
                    )
                })
                .await;
            let pending_exit = this
                .update(cx, |app, cx| {
                    app.git_ui.running = None;
                    *app.git_ui.phase.lock().unwrap_or_else(|e| e.into_inner()) = None;
                    let visible = app.workspace_root.starts_with(&identity.worktree_root);
                    let previous_status = app.status.clone();
                    match result {
                        Ok(completion) => {
                            app.git_ui.retry_operation =
                                sync_outcome_needs_authentication(&completion.outcome)
                                    .then(|| (identity.clone(), operation));
                            if authored_version
                                && let SyncOutcome::CommittedLocally { commit } =
                                    &completion.outcome
                                && let Some(draft) = &mut app.git_ui.commit_draft
                                && draft.identity == identity
                            {
                                draft.expected_head = Some(commit.clone());
                                draft.selected.clear();
                                draft.message.set_text("");
                            }
                            app.apply_git_sync_completion(completion, cx);
                        }
                        Err(error) => {
                            app.git_ui.retry_operation = authentication_message(&error)
                                .then(|| (identity.clone(), operation));
                            app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                        }
                    }
                    app.git_ui
                        .reports
                        .insert(identity.worktree_root.clone(), app.status.to_string());
                    if !visible {
                        app.status = previous_status;
                    }
                    app.refresh_git_details(cx);
                    cx.notify();
                    app.git_ui.pending_exit.take()
                })
                .ok()
                .flatten();
            if let Some((handle, kind)) = pending_exit {
                let _ = handle.update(cx, |_, window, cx| {
                    let _ = this.update(cx, |app, cx| app.begin_unsaved_exit(window, cx, kind));
                });
            }
        })
        .detach();
    }

    pub(super) fn retry_git_operation(&mut self, cx: &mut Context<Self>) {
        let Some((identity, operation)) = self.git_ui.retry_operation.take() else {
            return;
        };
        let store = PolicyStore::new(default_git_sync_policy_path());
        match store.load() {
            Ok(policies) => {
                let Some(policy) = policies
                    .repositories
                    .into_iter()
                    .find(|policy| policy.identity == identity)
                else {
                    self.status = t(self.language, Msg::StatusGitSyncNotConfigured).into();
                    cx.notify();
                    return;
                };
                self.start_git_operation(policy, store, operation, cx);
            }
            Err(error) => {
                self.status = self.trf(Msg::StatusGitSyncFailed, &[&error.to_string()]);
                cx.notify();
            }
        }
    }

    pub(super) fn finish_git_conflict(
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
        let policy = store
            .load()
            .ok()
            .and_then(|policies| {
                policies
                    .repositories
                    .into_iter()
                    .find(|policy| policy.identity == identity)
            })
            .or_else(|| {
                git_panel::journal()
                    .load()
                    .ok()?
                    .active
                    .into_iter()
                    .find(|entry| entry.operation_id == operation_id)
                    .and_then(git_panel::recovery_policy)
            });
        let Some(policy) = policy else {
            self.git_conflict_admission = Some(admission);
            self.status = self.git_label(GitMsg::Recovery).into();
            cx.notify();
            return;
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

            let cancellation = CancellationToken::new();
            let _ = this.update(cx, |app, cx| {
                app.git_ui.running = Some((
                    identity.clone(),
                    markion_git_sync::OperationKind::ResolveConflict,
                    cancellation.clone(),
                ));
                cx.notify();
            });
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
                                .finish_and_push(&session, &target, &cancellation)
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
                                .finish_merge(&session, &cancellation)
                                .map(|_| ())
                                .map_err(|error| error.to_string()),
                            GitConflictFinalAction::Abort => manager
                                .abort(&session, &cancellation)
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
                app.git_ui.running = None;
                app.resume_pending_git_exit(cx);
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
        let local_recovery = matches!(
            &outcome,
            SyncOutcome::NeedsAttention {
                phase: markion_git_sync::OperationPhase::Integrating
                    | markion_git_sync::OperationPhase::Committing
                    | markion_git_sync::OperationPhase::Staging,
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
        if matches!(
            &outcome,
            SyncOutcome::RemoteChecked { .. } | SyncOutcome::Synchronized { .. }
        ) {
            if let Some(identity) = exclusive.identity() {
                self.git_ui
                    .remote_checks
                    .insert(identity.worktree_root.clone(), std::time::SystemTime::now());
            }
        }
        self.status = match outcome {
            SyncOutcome::RemoteChecked { relation } => {
                git_panel::history_label(self.language, relation).into()
            }
            SyncOutcome::Synchronized {
                pending_local_changes,
                ..
            } if pending_local_changes > 0
                || exclusive.identity().is_some_and(|identity| {
                    self.tabs.iter().any(|tab| {
                        tab.is_dirty()
                            && tab
                                .path()
                                .is_some_and(|path| path.starts_with(&identity.worktree_root))
                    })
                }) =>
            {
                self.git_label(GitMsg::NewEdits).into()
            }
            SyncOutcome::Synchronized { .. } | SyncOutcome::UpToDate => {
                t(self.language, Msg::StatusGitSyncComplete).into()
            }
            SyncOutcome::AwaitingUpload { reason, .. } => {
                git_tf(self.language, GitMsg::Awaiting, &[&reason]).into()
            }
            SyncOutcome::UncertainDelivery { reason, .. } => {
                git_tf(self.language, GitMsg::Uncertain, &[&reason]).into()
            }
            SyncOutcome::NeedsAttention { reason, .. } => {
                self.trf(Msg::StatusGitSyncFailed, &[&reason])
            }
            SyncOutcome::CommittedLocally { .. } => self.git_label(GitMsg::Committed).into(),
        };
        if conflict_active {
            self.git_conflict_admission = Some(exclusive);
        } else if local_recovery && let Some(identity) = exclusive.identity().cloned() {
            self.git_ui.recovery_guards.insert(identity, exclusive);
        } else {
            drop(exclusive);
        }
        self.refresh_file_tree(cx);
        cx.notify();
    }
}

#[allow(clippy::too_many_arguments)]
fn run_git_onboarding(
    executable: String,
    language: Language,
    mode: git_panel::GitOnboardingMode,
    workspace_root: PathBuf,
    clone_destination: PathBuf,
    remote: String,
    branch: String,
    author_name: String,
    author_email: String,
    background_fetch: bool,
    cancellation: CancellationToken,
) -> Result<GitOnboardingSuccess, String> {
    let runner = GitCommandRunner::new(executable);
    let service = OnboardingService::new(runner.clone()).with_foreground_credentials();
    let author = (!author_name.is_empty()).then_some(markion_git_sync::RepositoryAuthor {
        name: author_name,
        email: author_email,
    });

    let (worktree, review, allow_empty_initial) = match mode {
        git_panel::GitOnboardingMode::CloneRepository => {
            validate_user_remote(&remote, language)?;
            let cloned = service
                .clone_repository(
                    &remote,
                    &clone_destination,
                    (!branch.is_empty()).then_some(branch.as_str()),
                    None,
                    &cancellation,
                )
                .map_err(|error| error.to_string())?;
            let review = service
                .connect_existing(&cloned.worktree)
                .map_err(|error| error.to_string())?;
            (cloned.worktree, review, cloned.empty_remote)
        }
        git_panel::GitOnboardingMode::InitializeFolder => {
            validate_user_remote(&remote, language)?;
            let branch = if branch.is_empty() { "main" } else { &branch };
            let review = service
                .initialize_notes(&workspace_root, branch, Some(&remote))
                .map_err(|error| error.to_string())?;
            (workspace_root, review, true)
        }
        git_panel::GitOnboardingMode::UseCurrentFolder => {
            let review = service
                .connect_existing(&workspace_root)
                .map_err(|error| error.to_string())?;
            (workspace_root, review, false)
        }
    };

    if !review.state.capabilities.supports_write_sync() {
        return Err(git_t(language, GitMsg::SetupReadOnly).into());
    }
    if let Some(author) = &author {
        Authentication::new(runner)
            .configure_author(&worktree, author)
            .map_err(|error| error.to_string())?;
    }

    let pending_files = review.state.worktree.changes.len();
    let target = if let Some(target) = review.target {
        target
    } else {
        let origin = service
            .origin_url(&review.repository, &cancellation)
            .map_err(|error| error.to_string())?;
        if remote.is_empty() && origin.is_none() {
            return Err(git_t(language, GitMsg::SetupRemoteHelp).into());
        }
        if mode == git_panel::GitOnboardingMode::CloneRepository
            && allow_empty_initial
            && review.state.head.is_none()
            && initial_note_paths(&review.state).is_empty()
        {
            return Ok(GitOnboardingSuccess {
                worktree,
                policy: None,
                pending_files,
            });
        }
        if !remote.is_empty() {
            service
                .attach_origin(&review.repository, &remote, &cancellation)
                .map_err(|error| error.to_string())?;
        }
        let publication_branch = if branch.is_empty() {
            service
                .current_branch_name(&review.repository, &cancellation)
                .map_err(|error| error.to_string())?
        } else {
            branch.clone()
        };
        let current = review
            .repository
            .status()
            .map_err(|error| error.to_string())?;
        if current.head.is_none() {
            let paths = initial_note_paths(&current);
            if paths.is_empty() {
                return Ok(GitOnboardingSuccess {
                    worktree,
                    policy: None,
                    pending_files,
                });
            }
            service
                .create_initial_commit(&review.repository, &paths, "Start notes", &cancellation)
                .map_err(|error| error.to_string())?;
        }
        service
            .publish_first_branch(
                &review.repository,
                "origin",
                &publication_branch,
                None,
                &cancellation,
            )
            .map_err(|error| format!("{error}. {}", git_t(language, GitMsg::PublicationRetryHint)))?
            .target
    };
    let policy = default_notes_policy(
        review.repository.identity().clone(),
        target,
        background_fetch,
    );
    PolicyStore::new(default_git_sync_policy_path())
        .update(|policies| policies.upsert(policy.clone()))
        .map_err(|error| error.to_string())?;
    Ok(GitOnboardingSuccess {
        worktree,
        policy: Some(policy),
        pending_files,
    })
}

fn default_notes_policy(
    identity: markion_git_sync::RepositoryIdentity,
    target: SyncTarget,
    background_fetch: bool,
) -> RepositoryPolicy {
    RepositoryPolicy {
        identity,
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
    }
}

fn initial_note_paths(state: &RepositoryState) -> Vec<PathBuf> {
    let mut paths = state
        .worktree
        .changes
        .iter()
        .filter(|change| {
            !change.path.components().any(|component| {
                matches!(component, std::path::Component::Normal(name) if name.to_string_lossy().starts_with('.'))
            }) && matches!(
                NewFileClass::classify(&change.path),
                Some(NewFileClass::Markdown | NewFileClass::Text | NewFileClass::Image)
            )
        })
        .map(|change| change.path.clone())
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    paths
}

fn validate_user_remote(remote: &str, language: Language) -> Result<RemoteUrl, String> {
    let remote = RemoteUrl::parse(remote).map_err(|error| error.to_string())?;
    if matches!(
        remote.transport,
        RemoteTransport::Https | RemoteTransport::Ssh
    ) {
        Ok(remote)
    } else {
        Err(git_t(language, GitMsg::SetupRemoteHelp).into())
    }
}

fn suggested_clone_destination(workspace_root: &Path, remote: &str) -> Option<PathBuf> {
    let without_query = remote.split(['?', '#']).next().unwrap_or(remote);
    let name = without_query
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', ':', '\\'])
        .next()?
        .strip_suffix(".git")
        .unwrap_or_else(|| {
            without_query
                .trim_end_matches(['/', '\\'])
                .rsplit(['/', ':', '\\'])
                .next()
                .unwrap_or("")
        })
        .trim();
    if name.is_empty()
        || matches!(name, "." | "..")
        || name.chars().any(|character| {
            character.is_control() || matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
        })
    {
        return None;
    }
    Some(workspace_root.parent().unwrap_or(workspace_root).join(name))
}

fn resolve_clone_destination(
    workspace_root: &Path,
    remote: &str,
    entered: &str,
    language: Language,
) -> Result<PathBuf, String> {
    let path = if entered.is_empty() {
        suggested_clone_destination(workspace_root, remote)
            .ok_or_else(|| git_t(language, GitMsg::CloneDestinationRequired).to_string())?
    } else {
        let entered = PathBuf::from(entered);
        if entered.is_absolute() {
            entered
        } else {
            workspace_root
                .parent()
                .unwrap_or(workspace_root)
                .join(entered)
        }
    };
    let path = std::path::absolute(path).map_err(|error| error.to_string())?;
    let current = std::path::absolute(workspace_root).map_err(|error| error.to_string())?;
    if path.starts_with(&current) {
        return Err(git_t(language, GitMsg::CloneDestinationOutside).into());
    }
    if path.exists() {
        return Err(git_t(language, GitMsg::CloneDestinationExists).into());
    }
    Ok(path)
}

fn sync_outcome_needs_authentication(outcome: &SyncOutcome) -> bool {
    match outcome {
        SyncOutcome::AwaitingUpload { reason, .. }
        | SyncOutcome::NeedsAttention { reason, .. }
        | SyncOutcome::UncertainDelivery { reason, .. } => authentication_message(reason),
        _ => false,
    }
}

fn authentication_message(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    [
        "authentication",
        "credential",
        "could not read username",
        "could not read password",
        "permission denied",
        "enter passphrase for key",
        "authenticity of host",
        "host key is not cached",
        "host key verification failed",
        "remote host identification has changed",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

fn run_git_operation(
    registry: GitOperationRegistry,
    store: PolicyStore,
    executable: String,
    mut policy: RepositoryPolicy,
    buffers: Vec<GitBufferSnapshot>,
    images: Vec<GitImageSnapshot>,
    operation: markion_git_sync::OperationKind,
    cancellation: CancellationToken,
    generation: u64,
    phase: Arc<std::sync::Mutex<Option<markion_git_sync::OperationPhase>>>,
    version_request: Option<GitVersionCommitRequest>,
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
    let captures_content = matches!(
        operation,
        markion_git_sync::OperationKind::SyncNow | markion_git_sync::OperationKind::CommitLocally
    );
    let repository = GitRepository::discover(
        GitCommandRunner::new(executable),
        &policy.identity.worktree_root,
    )
    .map_err(|error| error.to_string())?;
    if repository.identity() != &policy.identity {
        return Err("repository identity changed".to_string());
    }
    if registry.content_generation(&policy.identity) != generation {
        return Err("documents changed while preparing the snapshot; retry synchronization".into());
    }
    let before = repository.status().map_err(|error| error.to_string())?;
    if captures_content {
        if let Some(request) = &version_request {
            if request.message.trim().is_empty()
                || request.selected.is_empty()
                || request.expected_head != before.head
                || request.selected.iter().any(|path| {
                    let current = buffers
                        .iter()
                        .find(|buffer| {
                            buffer.dirty
                                && buffer
                                    .path
                                    .strip_prefix(&policy.identity.worktree_root)
                                    .is_ok_and(|relative| relative == path)
                        })
                        .map(|buffer| {
                            markion_git_sync::content_fingerprint_bytes(buffer.text.as_bytes())
                        })
                        .unwrap_or_else(|| {
                            repository
                                .content_fingerprint(path)
                                .unwrap_or_else(|_| "unavailable".into())
                        });
                    request.reviewed_fingerprints.get(path) != Some(&current)
                })
                || request.selected.iter().any(|path| {
                    !before
                        .worktree
                        .changes
                        .iter()
                        .any(|change| change.path == *path)
                        && !buffers.iter().any(|buffer| {
                            buffer.dirty
                                && buffer
                                    .path
                                    .strip_prefix(&policy.identity.worktree_root)
                                    .is_ok_and(|relative| relative == path)
                        })
                })
            {
                return Err("the local-version draft changed; refresh and review it again".into());
            }
        }
        if before.worktree.has_external_staging
            || before.worktree.has_conflicts
            || before.worktree.operation_in_progress.is_some()
            || !before.capabilities.supports_write_sync()
            || repository
                .resolve_sync_target()
                .map_err(|e| e.to_string())?
                != policy.target
        {
            return Err(
                "resolve repository staging or active operations before saving a sync snapshot"
                    .into(),
            );
        }
        // Check the entire inventory before saving any buffer. Policy exceptions stay visible.
        for change in before.worktree.changes.iter().filter(|change| {
            version_request
                .as_ref()
                .is_none_or(|request| request.selected.contains(&change.path))
        }) {
            repository
                .validate_write_path(&change.path)
                .map_err(|e| e.to_string())?;
            if policy.decide_path(&change.path, change.kind, false, false)
                != markion_git_sync::PathDecision::Automatic
                || change
                    .original_path
                    .as_ref()
                    .is_some_and(|from| policy.rename_needs_review(from, &change.path))
            {
                return Err(format!(
                    "{} is outside the approved synchronization scope",
                    change.path.display()
                ));
            }
        }
        for buffer in buffers.iter().filter(|buffer| {
            buffer.dirty
                && version_request.as_ref().is_none_or(|request| {
                    buffer
                        .path
                        .strip_prefix(&policy.identity.worktree_root)
                        .is_ok_and(|relative| request.selected.contains(relative))
                })
        }) {
            let relative = buffer
                .path
                .strip_prefix(&policy.identity.worktree_root)
                .map_err(|e| e.to_string())?;
            repository
                .validate_write_path(relative)
                .map_err(|e| e.to_string())?;
            let tracked = repository.path_is_tracked(relative);
            if !(if tracked {
                policy.tracked_path_allowed(relative)
            } else {
                policy.new_path_allowed(relative, repository.path_is_ignored(relative), false)
            }) {
                return Err(format!(
                    "{} is outside the approved synchronization scope",
                    relative.display()
                ));
            }
        }
    }
    if captures_content {
        let mut notes = std::collections::BTreeMap::new();
        for change in before.worktree.changes.iter().filter(|change| {
            version_request
                .as_ref()
                .is_none_or(|request| request.selected.contains(&change.path))
        }) {
            if matches!(
                markion_git_sync::NewFileClass::classify(&change.path),
                Some(markion_git_sync::NewFileClass::Markdown)
            ) {
                let absolute = policy.identity.worktree_root.join(&change.path);
                if let Ok(text) = fs::read_to_string(&absolute) {
                    notes.insert(absolute, text);
                }
            }
        }
        for buffer in buffers.iter().filter(|buffer| {
            version_request.as_ref().is_none_or(|request| {
                buffer
                    .path
                    .strip_prefix(&policy.identity.worktree_root)
                    .is_ok_and(|relative| request.selected.contains(relative))
            })
        }) {
            if buffer.dirty || notes.contains_key(&buffer.path) {
                notes.insert(buffer.path.clone(), buffer.text.clone());
            }
        }
        for (path, text) in notes {
            let report = inspect_note_attachments(&repository, &policy, &path, &text)
                .map_err(|error| error.to_string())?;
            if let Some(issue) = report
                .issues
                .iter()
                .find(|issue| !issue.previously_acknowledged)
            {
                return Err(format!(
                    "{}: attachment '{}' requires attention ({:?})",
                    path.display(),
                    issue.authored_url,
                    issue.kind
                ));
            }
        }
    }
    for buffer in buffers.iter().filter(|buffer| {
        captures_content
            && buffer.dirty
            && version_request.as_ref().is_none_or(|request| {
                buffer
                    .path
                    .strip_prefix(&policy.identity.worktree_root)
                    .is_ok_and(|relative| request.selected.contains(relative))
            })
    }) {
        if !read_document_source(&buffer.path).is_ok_and(|(disk, _)| disk == buffer.text) {
            save_text_snapshot(&buffer.path, buffer.known.as_ref(), &buffer.text)
                .map_err(|error| error.to_string())?;
        }
    }
    let state = repository.status().map_err(|error| error.to_string())?;
    let paths = if let Some(request) = &version_request {
        if request.expected_head != state.head
            || request.selected.iter().any(|path| {
                !state
                    .worktree
                    .changes
                    .iter()
                    .any(|change| change.path == *path)
            })
        {
            return Err("the local-version draft changed; refresh and review it again".into());
        }
        let mut paths = request.selected.iter().cloned().collect::<Vec<_>>();
        paths.sort();
        paths
    } else {
        state
            .worktree
            .changes
            .iter()
            .filter(|_| captures_content)
            .map(|change| change.path.clone())
            .collect::<Vec<_>>()
    };
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
                repository
                    .content_fingerprint(path)
                    .map(|fingerprint| (path.clone(), fingerprint))
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?,
        message: version_request
            .as_ref()
            .map(|request| request.message.clone())
            .unwrap_or_else(|| policy.render_message(paths.len(), None)),
        paths,
    };
    let data = markion::default_git_sync_data_dir();
    let journal = JournalStore::new(data.join("journal.toml"), data.join("recovery"));
    let gate = Arc::new(std::sync::Mutex::new(Some(exclusive)));
    let observer_gate = gate.clone();
    let observer_registry = registry.clone();
    let observer_identity = policy.identity.clone();
    let observer_id = operation_id.clone();
    let engine = GitSyncEngine::new(repository, journal.clone(), SyncOptions::default())
        .with_foreground_credentials()
        .with_observer(move |current| {
            use markion_git_sync::{OperationPhase, SyncEngineError};
            *phase.lock().unwrap_or_else(|e| e.into_inner()) = Some(current);
            let mut gate = observer_gate.lock().unwrap_or_else(|e| e.into_inner());
            if matches!(
                current,
                OperationPhase::Fetching | OperationPhase::Pushing | OperationPhase::Verifying
            ) {
                gate.take();
            } else if matches!(
                current,
                OperationPhase::Staging
                    | OperationPhase::Committing
                    | OperationPhase::Integrating
                    | OperationPhase::Resolving
            ) && gate.is_none()
            {
                *gate = Some(
                    observer_registry
                        .begin_exclusive(&observer_identity, &observer_id)
                        .map_err(|_| SyncEngineError::UnsafeRepositoryState)?,
                );
                if current == OperationPhase::Integrating
                    && observer_registry.content_generation(&observer_identity) != generation
                {
                    return Err(SyncEngineError::StalePlan);
                }
            }
            Ok(())
        });
    if operation == markion_git_sync::OperationKind::CheckRemote {
        gate.lock().unwrap_or_else(|e| e.into_inner()).take();
    }
    let outcome = match operation {
        markion_git_sync::OperationKind::CommitLocally if version_request.is_some() => {
            engine.commit_selected(&plan, &policy, &cancellation)
        }
        markion_git_sync::OperationKind::CommitLocally => {
            engine.commit_locally(&plan, &policy, None, &cancellation)
        }
        markion_git_sync::OperationKind::CheckRemote => {
            engine.check_remote(&plan, &policy, None, &cancellation)
        }
        markion_git_sync::OperationKind::PullUpdates => {
            engine.pull_updates(&plan, &mut policy, None, &cancellation)
        }
        markion_git_sync::OperationKind::PushCommits => {
            engine.push_commits(&plan, &mut policy, None, &cancellation)
        }
        _ => engine.sync_now(&plan, &mut policy, None, None, &cancellation),
    }
    .unwrap_or_else(|error| SyncOutcome::NeedsAttention {
        phase: journal
            .load()
            .ok()
            .and_then(|entries| {
                entries
                    .active
                    .into_iter()
                    .find(|entry| entry.operation_id == operation_id)
            })
            .map(|entry| entry.phase)
            .unwrap_or(markion_git_sync::OperationPhase::Preparing),
        reason: error.to_string(),
    });
    let exclusive = gate
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
        .map(Ok)
        .unwrap_or_else(|| registry.begin_exclusive(&policy.identity, &operation_id))
        .map_err(|e| e.to_string())?;
    let _ = store.update(|policies| {
        // Disconnect while a network request is running must not reconnect the policy.
        if policies
            .repositories
            .iter()
            .any(|saved| saved.identity == policy.identity)
        {
            policies.upsert(policy);
        }
    });
    let cancellation = CancellationToken::new();
    let resolving = matches!(
        outcome,
        SyncOutcome::NeedsAttention {
            phase: markion_git_sync::OperationPhase::Resolving
                | markion_git_sync::OperationPhase::Integrating
                | markion_git_sync::OperationPhase::Committing
                | markion_git_sync::OperationPhase::Staging,
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
    let buffers = if resolving
        || matches!(
            operation,
            markion_git_sync::OperationKind::CheckRemote
                | markion_git_sync::OperationKind::PushCommits
        ) {
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

    #[test]
    fn clone_destination_is_derived_from_https_and_ssh_repository_names() {
        let workspace = PathBuf::from("C:/notes/current");
        assert_eq!(
            suggested_clone_destination(&workspace, "https://example.invalid/user/My Notes.git"),
            Some(PathBuf::from("C:/notes/My Notes"))
        );
        assert_eq!(
            suggested_clone_destination(&workspace, "git@example.invalid:user/wiki.git"),
            Some(PathBuf::from("C:/notes/wiki"))
        );
    }

    #[test]
    fn user_onboarding_rejects_local_transport_and_nested_clone_destination() {
        assert!(validate_user_remote("../remote.git", Language::En).is_err());
        assert!(
            validate_user_remote("https://example.invalid/user/notes.git", Language::En).is_ok()
        );
        let workspace = tempfile::tempdir().unwrap();
        let nested = workspace.path().join("child").display().to_string();
        assert!(
            resolve_clone_destination(
                workspace.path(),
                "https://example.invalid/notes.git",
                &nested,
                Language::En
            )
            .is_err()
        );
    }

    #[test]
    fn initial_notes_filter_excludes_hidden_and_arbitrary_files() {
        let change = |path: &str| markion_git_sync::FileChange {
            path: PathBuf::from(path),
            original_path: None,
            kind: markion_git_sync::ChangeKind::Untracked,
            index_status: '?',
            worktree_status: '?',
            conflict: None,
        };
        let state = RepositoryState {
            head: None,
            branch: Some("main".into()),
            upstream: None,
            worktree: markion_git_sync::WorktreeState {
                changes: vec![
                    change("note.md"),
                    change("images/photo.png"),
                    change(".env"),
                    change("private.key"),
                    change(".hidden/note.md"),
                ],
                ..Default::default()
            },
            capabilities: markion_git_sync::RepositoryCapabilities {
                ordinary_worktree: true,
                ..Default::default()
            },
            history: markion_git_sync::HistoryRelation::Unknown,
            checked_at: std::time::SystemTime::now(),
        };
        assert_eq!(
            initial_note_paths(&state),
            vec![PathBuf::from("images/photo.png"), PathBuf::from("note.md")]
        );
    }

    #[test]
    fn authentication_failures_offer_an_explicit_retry_only_for_auth_messages() {
        assert!(authentication_message(
            "fatal: Authentication failed for 'https://example.invalid/notes.git'"
        ));
        assert!(authentication_message("Permission denied (publickey)."));
        assert!(authentication_message(
            "The authenticity of host cannot be established"
        ));
        assert!(!authentication_message("non-fast-forward update rejected"));
    }
}
