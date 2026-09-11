//! Workspace Git presentation. All repository reads happen on the background executor.
use super::*;
use markion_git_sync::{
    BackgroundFetchResult, BackupSyncAction, BackupSyncFacts, BackupSyncPresentation,
    BackupSyncState, CancellationToken, ConnectionState, DiffRequest, DocumentSyncState,
    GitCommandRunner, GitObjectId, GitRepository, HistoryEntry, HistoryRelation, JournalStore,
    OperationCheckpoint, OperationKind, OperationPhase, OperationProgress, PathDecision, RemoteUrl,
    RepositoryIdentity, RepositoryPolicy, RepositoryState, SyncOutcome,
};
use std::time::SystemTime;

const PAGE_SIZE: usize = 50;
const PREVIEW_LIMIT: usize = 64 * 1024;

pub(super) type GitRestoreTarget = (DocumentInstanceId, u64, Option<DiskIdentity>, bool);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum GitPage {
    #[default]
    Changes,
    Recent,
    FileHistory,
    Outgoing,
}

#[derive(Clone, Debug)]
pub(super) struct GitCommitDraft {
    pub identity: RepositoryIdentity,
    pub expected_head: Option<GitObjectId>,
    pub selected: HashSet<PathBuf>,
    pub reviewed_fingerprints: HashMap<PathBuf, String>,
    pub message: SearchFieldState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GitOnboardingMode {
    UseCurrentFolder,
    CloneRepository,
    InitializeFolder,
}

pub(super) struct GitOnboarding {
    pub mode: GitOnboardingMode,
    /// Sync address, local destination, branch, author name, author email.
    pub fields: [SearchFieldState; 5],
    pub repository_root: Option<PathBuf>,
    pub advanced: bool,
    pub advanced_required: bool,
    pub alternatives_open: bool,
    pub destination_edited: bool,
    pub sync_after_setup: bool,
    pub busy: bool,
    pub cancellation: Option<CancellationToken>,
    pub error: Option<String>,
}

#[derive(Default)]
pub(super) struct GitUi {
    pub center_open: bool,
    pub advanced_open: bool,
    pub snapshot: Option<Arc<GitDetails>>,
    pub refreshing: bool,
    pub page: GitPage,
    pub offset: usize,
    pub history_path_override: Option<PathBuf>,
    pub inspection: Option<GitInspection>,
    pub inspection_generation: u64,
    pub inspection_loading: bool,
    pub inspection_pending: Option<(
        RepositoryIdentity,
        Option<GitObjectId>,
        Option<PathBuf>,
        bool,
        bool,
    )>,
    pub running: Option<(RepositoryIdentity, OperationKind, CancellationToken)>,
    pub reports: HashMap<PathBuf, String>,
    pub recovery_guards: HashMap<RepositoryIdentity, ExclusiveAdmission>,
    pub phase: Arc<std::sync::Mutex<Option<markion_git_sync::OperationPhase>>>,
    pub pending_exit: Option<(gpui::AnyWindowHandle, UnsavedExitKind)>,
    pub remote_checks: HashMap<PathBuf, SystemTime>,
    pub remote_confirmations: HashMap<PathBuf, SystemTime>,
    pub last_outcomes: HashMap<RepositoryIdentity, SyncOutcome>,
    pub background_results: HashMap<RepositoryIdentity, BackgroundFetchResult>,
    pub settings: Option<GitSettings>,
    pub onboarding: Option<GitOnboarding>,
    pub onboarding_probe_generation: u64,
    pub conflict: Option<git_conflicts::ConflictView>,
    pub conflict_surface_open: bool,
    pub conflict_busy: bool,
    pub executable_status: Option<String>,
    pub preferences_advanced: bool,
    pub commit_draft: Option<GitCommitDraft>,
    pub retry_operation: Option<(RepositoryIdentity, OperationKind)>,
}

pub(super) struct GitSettings {
    pub policy: RepositoryPolicy,
    pub fields: [SearchFieldState; 5],
    pub advanced: bool,
}

pub(super) struct GitDetails {
    pub workspace: PathBuf,
    pub identity: RepositoryIdentity,
    pub state: RepositoryState,
    pub policy: Option<RepositoryPolicy>,
    pub history: Vec<HistoryEntry>,
    pub history_path: Option<PathBuf>,
    pub attention: HashSet<PathBuf>,
    pub fingerprints: HashMap<PathBuf, String>,
    pub decorations: HashMap<PathBuf, String>,
    pub attachments: Vec<(PathBuf, markion_git_sync::AttachmentIssue)>,
    pub recovery: Vec<OperationCheckpoint>,
    pub assessments: Vec<markion_git_sync::RecoveryAssessment>,
    pub page: GitPage,
    pub offset: usize,
}

pub(super) struct GitInspection {
    pub identity: RepositoryIdentity,
    pub title: String,
    pub text: SharedString,
    pub limited: bool,
    pub copy: Option<Vec<u8>>,
    pub path: Option<PathBuf>,
    pub commit: Option<GitObjectId>,
    pub metadata: Option<HistoryEntry>,
    pub binary: bool,
    pub compare_current: bool,
    pub paths: Vec<PathBuf>,
    pub offset: usize,
}

pub(super) fn journal() -> JournalStore {
    let data = markion::default_git_sync_data_dir();
    JournalStore::new(data.join("journal.toml"), data.join("recovery"))
}

impl MarkionApp {
    pub(super) fn resume_pending_git_exit(&mut self, cx: &mut Context<Self>) {
        if let Some((handle, kind)) = self.git_ui.pending_exit.take() {
            cx.spawn(async move |this, cx| {
                let _ = handle.update(cx, |_, window, cx| {
                    let _ = this.update(cx, |app, cx| app.begin_unsaved_exit(window, cx, kind));
                });
            })
            .detach();
        }
    }

    pub(super) fn edit_git_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(policy) = self
            .git_ui
            .snapshot
            .as_ref()
            .and_then(|details| details.policy.clone())
        else {
            return;
        };
        let roots = |roots: Vec<PathBuf>| {
            roots
                .iter()
                .map(|path| {
                    if path.as_os_str().is_empty() {
                        ".".to_string()
                    } else {
                        path.display().to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("; ")
        };
        let fields = [
            SearchFieldState::new(roots(policy.tracked_roots.clone())),
            SearchFieldState::new(roots(
                policy
                    .new_file_rules
                    .iter()
                    .map(|rule| rule.root.clone())
                    .collect(),
            )),
            SearchFieldState::new(policy.message_template.clone()),
            SearchFieldState::default(),
            SearchFieldState::default(),
        ];
        self.git_ui.settings = Some(GitSettings {
            policy,
            fields,
            advanced: false,
        });
        self.search_visible = false;
        self.search_focus = Some(SearchField::Git(0));
        self.search_control_focus = None;
        window.focus(&self.focus_handle);
        cx.notify();
    }

    pub(super) fn save_git_settings(&mut self, cx: &mut Context<Self>) {
        if self.git_ui.running.is_some() {
            return;
        }
        let Some(form) = &self.git_ui.settings else {
            return;
        };
        let mut policy = form.policy.clone();
        let (Ok(tracked), Ok(new)) = (
            parse_scope(&form.fields[0].buffer),
            parse_scope(&form.fields[1].buffer),
        ) else {
            self.status = self.git_label(GitMsg::InvalidScope).into();
            cx.notify();
            return;
        };
        policy.tracked_roots = tracked;
        policy.new_file_rules = new
            .into_iter()
            .map(|root| markion_git_sync::NewFileRule {
                root,
                classes: [
                    markion_git_sync::NewFileClass::Markdown,
                    markion_git_sync::NewFileClass::Text,
                    markion_git_sync::NewFileClass::Image,
                ]
                .into_iter()
                .collect(),
            })
            .collect();
        policy.message_template = form.fields[2].buffer.clone();
        let author = markion_git_sync::RepositoryAuthor {
            name: form.fields[3].buffer.trim().into(),
            email: form.fields[4].buffer.trim().into(),
        };
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let runner = GitCommandRunner::new(executable);
                    let repository =
                        GitRepository::discover(runner.clone(), &policy.identity.worktree_root)
                            .map_err(|error| error.to_string())?;
                    if repository.identity() != &policy.identity
                        || repository
                            .resolve_sync_target()
                            .map_err(|error| error.to_string())?
                            != policy.target
                    {
                        return Err("repository identity or target changed".to_string());
                    }
                    if !author.name.is_empty() || !author.email.is_empty() {
                        markion_git_sync::Authentication::new(runner)
                            .configure_author(&policy.identity.worktree_root, &author)
                            .map_err(|error| error.to_string())?;
                    }
                    PolicyStore::new(default_git_sync_policy_path())
                        .update(|policies| policies.upsert(policy))
                        .map_err(|error| error.to_string())
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                match result {
                    Ok(_) => {
                        app.git_ui.settings = None;
                        app.search_focus = None;
                        app.git_background_scheduler.deactivate();
                        app.refresh_git_details(cx);
                    }
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn choose_git_executable(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let picked = prompt_for_open_file(window, self.language, Msg::PrefPanelGitSection, None);
        cx.spawn(async move |this, cx| {
            if let Some(path) = picked.await {
                let _ = this.update(cx, |app, cx| {
                    app.git_preferences.executable = Some(path.display().to_string());
                    app.persist_preferences();
                    app.detect_git_executable(cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn detect_git_executable(&mut self, cx: &mut Context<Self>) {
        let executable = self.git_preferences.executable.clone();
        cx.spawn(async move |this, cx| {
            let probe = executable.clone();
            let result = cx
                .background_spawn(
                    async move { markion_git_sync::detect_git(probe.map(PathBuf::from)) },
                )
                .await;
            let _ = this.update(cx, |app, cx| {
                if app.git_preferences.executable == executable {
                    app.git_ui.executable_status = Some(match result {
                        markion_git_sync::GitAvailability::Available { version, .. } => {
                            format!("Git {}.{}.{}", version.major, version.minor, version.patch)
                        }
                        _ => app.git_label(GitMsg::GitUnavailable).into(),
                    });
                    app.refresh_git_details(cx);
                    cx.notify();
                }
            });
        })
        .detach();
    }
    pub(super) fn git_label(&self, msg: GitMsg) -> &'static str {
        git_t(self.language, msg)
    }

    pub(super) fn show_git_sync(
        &mut self,
        _: &ShowGitSync,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.git_ui.center_open = true;
        self.active_menu = None;
        self.refresh_git_details(cx);
        cx.notify();
    }

    pub(super) fn arm_git_details_poll(&mut self, cx: &mut Context<Self>) {
        self.refresh_git_details(cx);
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(Duration::from_secs(5)).await;
                if this
                    .update(cx, |app, cx| {
                        app.refresh_git_details(cx);
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }

    pub(super) fn refresh_git_details(&mut self, cx: &mut Context<Self>) {
        if self.git_ui.refreshing {
            return;
        }
        self.git_ui.refreshing = true;
        let workspace = self.workspace_root.clone();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        let page = self.git_ui.page;
        let offset = self.git_ui.offset;
        let active_path = self
            .git_ui
            .history_path_override
            .clone()
            .or_else(|| self.active_tab().path().map(Path::to_path_buf));
        let live_notes: HashMap<_, _> = self
            .tabs
            .iter()
            .filter_map(|tab| {
                let tab = tab.document_tab()?;
                Some((
                    tab.document.path()?.to_path_buf(),
                    tab.document.text().to_string(),
                ))
            })
            .collect();
        let dirty_paths = self
            .tabs
            .iter()
            .filter(|tab| tab.is_dirty())
            .filter_map(|tab| tab.path().map(Path::to_path_buf))
            .collect::<Vec<_>>();
        cx.spawn(async move |this, cx| {
            let root = workspace.clone();
            let queried_active_path = active_path.clone();
            let result = cx
                .background_spawn(async move {
                    let repository =
                        GitRepository::discover(GitCommandRunner::new(executable), &root)
                            .map_err(|error| error.to_string())?;
                    let mut state = repository.status().map_err(|error| error.to_string())?;
                    for absolute in &dirty_paths {
                        let Ok(relative) =
                            absolute.strip_prefix(&repository.identity().worktree_root)
                        else {
                            continue;
                        };
                        if !state
                            .worktree
                            .changes
                            .iter()
                            .any(|change| change.path == relative)
                        {
                            state.worktree.changes.push(markion_git_sync::FileChange {
                                path: relative.to_path_buf(),
                                original_path: None,
                                kind: markion_git_sync::ChangeKind::Modified,
                                index_status: '.',
                                worktree_status: 'M',
                                conflict: None,
                            });
                        }
                    }
                    state
                        .worktree
                        .changes
                        .sort_by(|left, right| left.path.cmp(&right.path));
                    let policy = PolicyStore::new(default_git_sync_policy_path())
                        .load()
                        .map_err(|error| error.to_string())?
                        .repositories
                        .into_iter()
                        .find(|policy| policy.identity == *repository.identity());
                    if let Some(policy) = &policy
                        && let (Some(local), Some(remote)) = (
                            state.head.as_ref(),
                            repository.observed_remote_tip(&policy.target),
                        )
                    {
                        state.history = repository
                            .classify_history(local, &remote, policy.last_confirmed_remote.as_ref())
                            .map_err(|e| e.to_string())?;
                    }
                    let attention = state
                        .worktree
                        .changes
                        .iter()
                        .filter(|change| {
                            policy.as_ref().is_some_and(|policy| {
                                let symlink = fs::symlink_metadata(
                                    repository.identity().worktree_root.join(&change.path),
                                )
                                .is_ok_and(|metadata| metadata.file_type().is_symlink());
                                policy.decide_path(&change.path, change.kind, false, symlink)
                                    == PathDecision::Attention
                                    || change.original_path.as_ref().is_some_and(|from| {
                                        policy.rename_needs_review(from, &change.path)
                                    })
                            })
                        })
                        .map(|change| change.path.clone())
                        .collect();
                    let history_path = queried_active_path
                        .as_ref()
                        .and_then(|path| {
                            path.strip_prefix(&repository.identity().worktree_root).ok()
                        })
                        .map(Path::to_path_buf);
                    let history = if state.head.is_none() {
                        Vec::new()
                    } else {
                        match page {
                            GitPage::Changes => Vec::new(),
                            GitPage::Recent | GitPage::Outgoing => repository
                                .history_page(offset, PAGE_SIZE, page == GitPage::Outgoing)
                                .map_err(|error| error.to_string())?,
                            GitPage::FileHistory => match history_path.as_deref() {
                                Some(path) => repository
                                    .history_page_for_path(path, offset, PAGE_SIZE)
                                    .map_err(|error| error.to_string())?,
                                None => Vec::new(),
                            },
                        }
                    };
                    let decorations = state
                        .worktree
                        .changes
                        .iter()
                        .map(|change| {
                            (
                                change.path.clone(),
                                format!("{}{}", change.index_status, change.worktree_status),
                            )
                        })
                        .collect();
                    let fingerprints = state
                        .worktree
                        .changes
                        .iter()
                        .map(|change| {
                            let absolute = repository.identity().worktree_root.join(&change.path);
                            let fingerprint = if dirty_paths.contains(&absolute) {
                                live_notes
                                    .get(&absolute)
                                    .map(|text| {
                                        markion_git_sync::content_fingerprint_bytes(text.as_bytes())
                                    })
                                    .unwrap_or_else(|| "missing".into())
                            } else {
                                repository
                                    .content_fingerprint(&change.path)
                                    .unwrap_or_else(|_| "unavailable".into())
                            };
                            (change.path.clone(), fingerprint)
                        })
                        .collect();
                    let mut attachments = Vec::new();
                    if let Some(policy) = &policy {
                        for change in state.worktree.changes.iter().skip(offset).take(PAGE_SIZE) {
                            let absolute = repository.identity().worktree_root.join(&change.path);
                            if !matches!(
                                markion_git_sync::NewFileClass::classify(&change.path),
                                Some(markion_git_sync::NewFileClass::Markdown)
                            ) {
                                continue;
                            }
                            let text = live_notes.get(&absolute).cloned().or_else(|| {
                                fs::metadata(&absolute)
                                    .ok()
                                    .filter(|m| m.len() <= 2 * 1024 * 1024)
                                    .and_then(|_| fs::read_to_string(&absolute).ok())
                            });
                            if let Some(text) = text {
                                let report = markion_git_sync::inspect_note_attachments(
                                    &repository,
                                    policy,
                                    &change.path,
                                    &text,
                                )
                                .map_err(|e| e.to_string())?;
                                attachments.extend(
                                    report
                                        .issues
                                        .into_iter()
                                        .filter(|issue| !issue.previously_acknowledged)
                                        .map(|issue| (change.path.clone(), issue)),
                                );
                            }
                        }
                    }
                    let recovery = journal()
                        .load()
                        .map_err(|error| error.to_string())?
                        .active
                        .into_iter()
                        .filter(|checkpoint| checkpoint.identity == *repository.identity())
                        .collect();
                    Ok::<_, String>(GitDetails {
                        workspace: root,
                        identity: repository.identity().clone(),
                        state,
                        policy,
                        history,
                        history_path,
                        attention,
                        fingerprints,
                        decorations,
                        attachments,
                        recovery,
                        page,
                        offset,
                        assessments: markion_git_sync::RecoveryManager::new(repository, journal())
                            .assess()
                            .map_err(|e| e.to_string())?,
                    })
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.refreshing = false;
                if workspace != app.workspace_root
                    || page != app.git_ui.page
                    || offset != app.git_ui.offset
                    || (page == GitPage::FileHistory
                        && active_path != app.active_tab().path().map(Path::to_path_buf))
                {
                    app.refresh_git_details(cx);
                    return;
                }
                match result {
                    Ok(details) => {
                        app.sync_git_commit_draft(&details);
                        app.git_ui.snapshot = Some(Arc::new(details));
                    }
                    Err(error) => {
                        app.git_ui.snapshot = None;
                        app.git_ui.reports.insert(workspace, error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn set_git_page(&mut self, page: GitPage, cx: &mut Context<Self>) {
        self.git_ui.history_path_override = None;
        self.git_ui.page = page;
        self.git_ui.offset = 0;
        self.refresh_git_details(cx);
        cx.notify();
    }

    pub(super) fn show_file_version_history(
        &mut self,
        _: &ShowFileVersionHistory,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = self.active_tab().path().map(Path::to_path_buf) else {
            return;
        };
        self.open_file_version_history(path, cx);
    }

    pub(super) fn open_file_version_history(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if !self.select_file_version_history(path) {
            return;
        }
        self.refresh_git_details(cx);
        cx.notify();
    }

    pub(super) fn select_file_version_history(&mut self, path: PathBuf) -> bool {
        if !path.starts_with(&self.workspace_root) {
            return false;
        }
        self.git_ui.history_path_override = Some(path);
        self.git_ui.page = GitPage::FileHistory;
        self.git_ui.offset = 0;
        self.git_ui.center_open = true;
        self.git_ui.advanced_open = false;
        self.active_menu = None;
        self.file_tree_context_menu = None;
        self.tab_context_menu = None;
        true
    }

    fn sync_git_commit_draft(&mut self, details: &GitDetails) {
        if details.policy.is_none() {
            self.git_ui.commit_draft = None;
            if self.search_focus == Some(SearchField::GitCommit) {
                self.search_focus = None;
            }
            return;
        }
        if self
            .git_ui
            .commit_draft
            .as_ref()
            .is_some_and(|draft| draft.identity == details.identity)
        {
            return;
        }
        self.git_ui.commit_draft = Some(GitCommitDraft {
            identity: details.identity.clone(),
            expected_head: details.state.head.clone(),
            selected: committable_paths(details),
            reviewed_fingerprints: details.fingerprints.clone(),
            message: SearchFieldState::default(),
        });
    }

    pub(super) fn toggle_git_commit_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        let Some(details) = self.git_ui.snapshot.as_ref() else {
            return;
        };
        if !committable_paths(details).contains(path) {
            return;
        }
        let fingerprint = self
            .tabs
            .iter()
            .filter_map(|tab| tab.document_tab())
            .find(|tab| {
                tab.document
                    .path()
                    .is_some_and(|candidate| candidate == details.identity.worktree_root.join(path))
            })
            .filter(|tab| tab.document.is_dirty())
            .map(|tab| markion_git_sync::content_fingerprint_bytes(tab.document.text().as_bytes()))
            .or_else(|| details.fingerprints.get(path).cloned());
        let Some(draft) = &mut self.git_ui.commit_draft else {
            return;
        };
        if !draft.selected.remove(path) {
            draft.selected.insert(path.to_path_buf());
            if let Some(fingerprint) = fingerprint {
                draft
                    .reviewed_fingerprints
                    .insert(path.to_path_buf(), fingerprint);
            }
        }
        cx.notify();
    }

    pub(super) fn select_all_git_commit_paths(&mut self, selected: bool, cx: &mut Context<Self>) {
        let paths = self
            .git_ui
            .snapshot
            .as_ref()
            .map(|details| committable_paths(details))
            .unwrap_or_default();
        let fingerprints = self
            .git_ui
            .snapshot
            .as_ref()
            .map(|details| details.fingerprints.clone())
            .unwrap_or_default();
        if let Some(draft) = &mut self.git_ui.commit_draft {
            if selected {
                draft.selected = paths;
                draft.reviewed_fingerprints.extend(fingerprints);
            } else {
                draft.selected.clear();
            }
        }
        cx.notify();
    }

    pub(super) fn inspect_git(
        &mut self,
        identity: RepositoryIdentity,
        commit: Option<GitObjectId>,
        path: Option<PathBuf>,
        staged: bool,
        compare_current: bool,
        cx: &mut Context<Self>,
    ) {
        self.git_ui.inspection_generation = self.git_ui.inspection_generation.wrapping_add(1);
        if self.git_ui.inspection_loading {
            self.git_ui.inspection_pending =
                Some((identity, commit, path, staged, compare_current));
            return;
        }
        self.git_ui.inspection_loading = true;
        let generation = self.git_ui.inspection_generation;
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let repository = GitRepository::discover(
                        GitCommandRunner::new(executable),
                        &identity.worktree_root,
                    )
                    .map_err(|error| error.to_string())?;
                    if repository.identity() != &identity {
                        return Err("repository identity changed".to_string());
                    }
                    let title = path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| {
                            commit.as_ref().map(ToString::to_string).unwrap_or_default()
                        });
                    let paths = if let Some(commit) = &commit {
                        if path.is_none() {
                            repository
                                .commit_paths(commit)
                                .map_err(|error| error.to_string())?
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                    let metadata = commit
                        .as_ref()
                        .map(|commit| repository.commit_metadata(commit))
                        .transpose()
                        .map_err(|error| error.to_string())?;
                    let mut binary = false;
                    let content = match (&commit, &path) {
                        (Some(commit), Some(path)) if compare_current => {
                            let comparison = repository
                                .working_diff_against(commit, path, PREVIEW_LIMIT)
                                .map_err(|error| error.to_string())?;
                            binary = comparison.binary;
                            Some(comparison.content)
                        }
                        (Some(commit), Some(path)) => Some(
                            repository
                                .commit_diff(commit, path, PREVIEW_LIMIT)
                                .map_err(|error| error.to_string())?,
                        ),
                        (None, Some(path)) => Some(
                            repository
                                .diff(&DiffRequest {
                                    path: Some(path.clone()),
                                    staged,
                                    max_bytes: PREVIEW_LIMIT,
                                })
                                .map_err(|error| error.to_string())?,
                        ),
                        _ => None,
                    };
                    let copy = match (&commit, &path) {
                        (Some(commit), Some(path)) => repository
                            .historical_blob(commit, path, 2 * 1024 * 1024)
                            .ok()
                            .filter(|blob| !blob.truncated)
                            .map(|blob| blob.bytes),
                        _ => None,
                    };
                    let limited = binary
                        || content.as_ref().is_some_and(|content| {
                            content.truncated || std::str::from_utf8(&content.bytes).is_err()
                        });
                    let text = content
                        .map(|content| String::from_utf8_lossy(&content.bytes).into_owned())
                        .unwrap_or_default()
                        .into();
                    Ok::<_, String>(GitInspection {
                        identity,
                        title,
                        text,
                        limited,
                        copy,
                        path,
                        commit,
                        metadata,
                        binary,
                        compare_current,
                        paths,
                        offset: 0,
                    })
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.inspection_loading = false;
                if let Some((identity, commit, path, staged, compare_current)) =
                    app.git_ui.inspection_pending.take()
                {
                    app.inspect_git(identity, commit, path, staged, compare_current, cx);
                    return;
                }
                if app.git_ui.inspection_generation != generation {
                    return;
                }
                match result {
                    Ok(inspection) => app.git_ui.inspection = Some(inspection),
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn save_git_copy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(inspection) = self.git_ui.inspection.as_ref() else {
            return;
        };
        let Some(bytes) = inspection.copy.clone() else {
            return;
        };
        let directory = inspection.identity.worktree_root.clone();
        let name = inspection
            .path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| format!("historical-{}", name.to_string_lossy()))
            .unwrap_or_else(|| "historical.md".into());
        let picked =
            save_dialog::prompt_for_historical_copy(window, &directory, &name, self.language);
        let registry = self.git_operations.clone();
        cx.spawn(async move |this, cx| {
            let Some(path) = picked.await else {
                return;
            };
            let result = cx
                .background_spawn(async move {
                    let _admission = registry
                        .try_write(&path)
                        .map_err(|error| error.to_string())?;
                    // Historical recovery always creates a separate file, even if a picker confirmed replacement.
                    use std::io::Write;
                    let mut file = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(&path)
                        .map_err(|error| error.to_string())?;
                    file.write_all(&bytes)
                        .and_then(|_| file.sync_all())
                        .map_err(|error| error.to_string())?;
                    Ok::<_, String>(path)
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.status = match result {
                    Ok(path) => app.trf(Msg::StatusExported, &[&path.display().to_string()]),
                    Err(error) => app.trf(Msg::StatusGitSyncFailed, &[&error]),
                };
                app.refresh_git_details(cx);
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn restore_git_version(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(inspection) = self.git_ui.inspection.as_ref() else {
            return;
        };
        let (Some(commit), Some(path)) = (inspection.commit.clone(), inspection.path.clone())
        else {
            return;
        };
        if inspection.copy.as_ref().is_none_or(|bytes| {
            bytes.contains(&0)
                || std::str::from_utf8(bytes).is_err()
                || !(is_markdown_path(&path) || is_text_path(&path))
        }) {
            self.status = self.git_label(GitMsg::BinaryContent).into();
            cx.notify();
            return;
        }
        let identity = inspection.identity.clone();
        let absolute = identity.worktree_root.join(&path);
        let target = find_tab_with_document_path(&self.tabs, &absolute).and_then(|index| {
            let tab = self.tabs[index].document_tab()?;
            Some((
                tab.document.instance_id(),
                tab.document.version(),
                tab.document.disk_identity().cloned(),
                tab.document.is_dirty(),
            ))
        });
        let confirmation = target
            .as_ref()
            .is_some_and(|(_, _, _, dirty)| *dirty)
            .then(|| {
                window.prompt(
                    PromptLevel::Warning,
                    self.git_label(GitMsg::RestoreEditor),
                    Some(self.git_label(GitMsg::RestoreConfirm)),
                    &[
                        PromptButton::ok(t(self.language, Msg::DialogButtonRestore)),
                        PromptButton::cancel(t(self.language, Msg::DialogButtonCancel)),
                    ],
                    cx,
                )
            });
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        let generation = self.git_ui.inspection_generation;
        cx.spawn(async move |this, cx| {
            if let Some(prompt) = confirmation
                && !matches!(prompt.await, Ok(0))
            {
                return;
            }
            let loaded = cx
                .background_spawn({
                    let identity = identity.clone();
                    let path = path.clone();
                    let absolute = absolute.clone();
                    async move {
                        let repository = GitRepository::discover(
                            GitCommandRunner::new(executable),
                            &identity.worktree_root,
                        )
                        .map_err(|error| error.to_string())?;
                        if repository.identity() != &identity {
                            return Err("repository identity changed".to_string());
                        }
                        repository
                            .validate_write_path(&path)
                            .map_err(|error| error.to_string())?;
                        let blob = repository
                            .historical_blob(&commit, &path, 2 * 1024 * 1024)
                            .map_err(|error| error.to_string())?;
                        if blob.truncated || blob.bytes.contains(&0) {
                            return Err("historical content is binary or too large".into());
                        }
                        let historical = String::from_utf8(blob.bytes)
                            .map_err(|_| "historical content is not UTF-8".to_string())?;
                        let (current, disk_identity) =
                            read_document_source(&absolute).map_err(|error| error.to_string())?;
                        Ok::<_, String>((historical, current, disk_identity))
                    }
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                if app.git_ui.inspection_generation != generation {
                    return;
                }
                let (historical, current, disk_identity) = match loaded {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                        cx.notify();
                        return;
                    }
                };
                let index = match resolve_git_restore_target(
                    &app.tabs,
                    &absolute,
                    target.as_ref(),
                    &disk_identity,
                ) {
                    Ok(Some(index)) => index,
                    Ok(None) => {
                        app.open_in_new_tab(
                            MarkdownDocument::from_loaded(current, absolute.clone(), disk_identity),
                            cx,
                        );
                        app.active_tab
                    }
                    Err(()) => {
                        app.status = app.git_label(GitMsg::VersionDraftStale).into();
                        cx.notify();
                        return;
                    }
                };
                app.switch_active_tab(index, cx);
                app.git_ui.inspection = None;
                app.git_ui.inspection_pending = None;
                app.git_ui.inspection_generation = app.git_ui.inspection_generation.wrapping_add(1);
                if app.apply_git_restore_source(historical, cx) {
                    app.status = app.git_label(GitMsg::RestoreSuccess).into();
                    app.sync_and_persist_session();
                    app.refresh_git_details(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn apply_git_restore_source(
        &mut self,
        historical: String,
        cx: &mut Context<Self>,
    ) -> bool {
        let snapshot = self.snapshot();
        let mutation = self
            .active_tab()
            .document
            .prepare_whole_mutation(MutationOrigin::Recovery, historical);
        let Some(receipt) = self.apply_document_mutation("git_restore_version", mutation) else {
            return false;
        };
        if receipt.changed {
            self.commit_undo_snapshot(snapshot);
            let end = self.active_tab().document.text().len();
            self.active_tab_mut().selected_range = 0..end;
            self.after_document_changed(cx);
        }
        receipt.changed
    }

    pub(super) fn cancel_git_operation(&mut self, cx: &mut Context<Self>) {
        if let Some((_, _, cancellation)) = &self.git_ui.running {
            cancellation.cancel();
        }
        self.status = self.git_label(GitMsg::Busy).into();
        cx.notify();
    }

    pub(super) fn disconnect_git(&mut self, cx: &mut Context<Self>) {
        if self.git_ui.running.is_some() {
            return;
        }
        let Some(details) = self.git_ui.snapshot.clone() else {
            return;
        };
        let identity = details.identity.clone();
        self.git_background_scheduler.deactivate();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    PolicyStore::new(default_git_sync_policy_path()).update(|policies| {
                        policies
                            .repositories
                            .retain(|policy| policy.identity != identity);
                    })
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.status = app.trf(Msg::StatusGitSyncFailed, &[&error.to_string()]);
                }
                // Retain registry and recovery guards after disconnect.
                app.refresh_git_details(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

pub(super) fn committable_paths(details: &GitDetails) -> HashSet<PathBuf> {
    if details.state.worktree.has_external_staging
        || details.state.worktree.has_conflicts
        || details.state.worktree.operation_in_progress.is_some()
    {
        return HashSet::new();
    }
    details
        .state
        .worktree
        .changes
        .iter()
        .filter(|change| !details.attention.contains(&change.path))
        .map(|change| change.path.clone())
        .collect()
}

pub(super) fn resolve_git_restore_target(
    tabs: &[EditorTab],
    path: &Path,
    expected: Option<&GitRestoreTarget>,
    disk_identity: &DiskIdentity,
) -> Result<Option<usize>, ()> {
    let existing = find_tab_with_document_path(tabs, path);
    match (expected, existing) {
        (Some((instance, version, known, _)), Some(index)) => {
            let tab = tabs[index].document_tab().ok_or(())?;
            if tab.document.instance_id() == *instance
                && tab.document.version() == *version
                && known.as_ref() == Some(disk_identity)
            {
                Ok(Some(index))
            } else {
                Err(())
            }
        }
        (None, None) => Ok(None),
        _ => Err(()),
    }
}

pub(super) fn button(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    enabled: bool,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
    action: impl Fn(&mut MarkionApp, &mut Window, &mut Context<MarkionApp>) + 'static,
) -> Stateful<Div> {
    let action = Rc::new(action);
    let click = action.clone();
    div()
        .id(id)
        .focusable()
        .tab_stop(enabled)
        .px_2()
        .py_1()
        .rounded_md()
        .text_size(px(12.))
        .text_color(if enabled { palette.text } else { palette.muted })
        .border_1()
        .border_color(palette.border)
        .focus(|style| style.border_2().border_color(palette.active_text))
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(move |style| style.bg(palette.active_bg))
        })
        .child(label.into())
        .on_click(cx.listener(move |app, _: &ClickEvent, window, cx| {
            cx.stop_propagation();
            if enabled {
                window.focus(&app.focus_handle);
                click(app, window, cx);
            }
        }))
        .on_key_down(cx.listener(move |app, event: &KeyDownEvent, window, cx| {
            if enabled && event.keystroke.key == "tab" {
                cx.stop_propagation();
                if event.keystroke.modifiers.shift {
                    window.focus_prev();
                } else {
                    window.focus_next();
                }
                return;
            }
            if enabled && matches!(event.keystroke.key.as_str(), "enter" | "space") {
                cx.stop_propagation();
                window.focus(&app.focus_handle);
                action(app, window, cx);
            }
        }))
}

pub(super) fn backup_sync_presentation(app: &MarkionApp) -> BackupSyncPresentation {
    let details = app
        .git_ui
        .snapshot
        .as_ref()
        .filter(|details| details.workspace == app.workspace_root);
    let Some(details) = details else {
        return BackupSyncPresentation::project(BackupSyncFacts {
            connection: ConnectionState::NotConfigured,
            documents: DocumentSyncState::default(),
            pending_paths: 0,
            history: HistoryRelation::Unknown,
            operation: None,
            has_conflicts: false,
            recovery_items: 0,
            attention_items: 0,
            has_external_staging: false,
            has_external_operation: false,
            delivery_uncertain: false,
            last_confirmed_at: None,
        });
    };

    let dirty_named_paths = app
        .tabs
        .iter()
        .filter_map(|tab| {
            (tab.is_dirty())
                .then(|| tab.path())
                .flatten()
                .and_then(|path| path.strip_prefix(&details.identity.worktree_root).ok())
                .map(Path::to_path_buf)
        })
        .collect::<HashSet<_>>();
    let named_unsaved = dirty_named_paths.len();
    let omitted_untitled = app
        .tabs
        .iter()
        .filter(|tab| tab.is_dirty() && tab.path().is_none())
        .count();
    let external_conflicts = app
        .tabs
        .iter()
        .filter(|tab| {
            tab.path()
                .is_some_and(|path| path.starts_with(&details.identity.worktree_root))
                && tab
                    .document_tab()
                    .is_some_and(|tab| tab.external_conflict.is_some())
        })
        .count();
    let background = app.git_ui.background_results.get(&details.identity);
    let last_outcome = app.git_ui.last_outcomes.get(&details.identity);
    let connection = if details.policy.is_none() {
        ConnectionState::NotConfigured
    } else if !details.state.capabilities.supports_write_sync() {
        ConnectionState::Unsupported
    } else if app
        .git_ui
        .retry_operation
        .as_ref()
        .is_some_and(|(identity, _)| identity == &details.identity)
        || matches!(
            background,
            Some(BackgroundFetchResult::AuthenticationNeeded)
        )
    {
        ConnectionState::AuthenticationNeeded
    } else if matches!(background, Some(BackgroundFetchResult::Offline))
        || matches!(last_outcome, Some(SyncOutcome::AwaitingUpload { .. }))
    {
        ConnectionState::Offline
    } else {
        ConnectionState::Available
    };
    let operation = app
        .git_ui
        .running
        .as_ref()
        .filter(|(identity, _, _)| identity == &details.identity)
        .map(|(_, kind, _)| OperationProgress {
            operation_id: "foreground".into(),
            kind: *kind,
            phase: app
                .git_ui
                .phase
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .unwrap_or(OperationPhase::Preparing),
            detail: None,
        })
        .or_else(|| {
            (app.git_background_scheduler.in_flight()
                && app.git_background_scheduler.active_identity() == Some(&details.identity))
            .then(|| OperationProgress {
                operation_id: "background".into(),
                kind: OperationKind::CheckRemote,
                phase: OperationPhase::Fetching,
                detail: None,
            })
        });
    let history = match (background, details.state.history) {
        (
            Some(BackgroundFetchResult::Incoming { commits }),
            HistoryRelation::Ahead { commits: ahead },
        ) => HistoryRelation::Diverged {
            ahead,
            behind: *commits,
        },
        (
            Some(BackgroundFetchResult::Incoming { commits }),
            HistoryRelation::Equal | HistoryRelation::Unknown,
        ) => HistoryRelation::Behind { commits: *commits },
        _ => details.state.history,
    };
    let outcome_attention = usize::from(matches!(
        last_outcome,
        Some(SyncOutcome::NeedsAttention { .. })
    ));
    let background_attention = usize::from(matches!(
        background,
        Some(BackgroundFetchResult::ActionableError(_))
    ));
    let delivery_uncertain = matches!(last_outcome, Some(SyncOutcome::UncertainDelivery { .. }));
    let pending_paths = details
        .state
        .worktree
        .changes
        .iter()
        .map(|change| change.path.clone())
        .chain(dirty_named_paths)
        .collect::<HashSet<_>>()
        .len();

    BackupSyncPresentation::project(BackupSyncFacts {
        connection,
        documents: DocumentSyncState {
            unsaved: named_unsaved,
            external_conflicts,
            omitted_untitled,
        },
        // Include edits made after the last repository refresh while
        // deduplicating paths already present in the Git inventory.
        pending_paths,
        history,
        operation,
        has_conflicts: details.state.worktree.has_conflicts
            || app
                .git_ui
                .conflict
                .as_ref()
                .is_some_and(|view| view.identity == details.identity),
        recovery_items: details.recovery.len(),
        attention_items: details.attention.len()
            + details.attachments.len()
            + outcome_attention
            + background_attention,
        has_external_staging: details.state.worktree.has_external_staging,
        has_external_operation: details.state.worktree.operation_in_progress.is_some(),
        delivery_uncertain,
        last_confirmed_at: app
            .git_ui
            .remote_confirmations
            .get(&details.identity.worktree_root)
            .copied(),
    })
}

fn backup_sync_state_label(
    app: &MarkionApp,
    presentation: &BackupSyncPresentation,
) -> SharedString {
    match presentation.state {
        BackupSyncState::NotConfigured => app.git_label(GitMsg::StateOff).into(),
        BackupSyncState::Running => {
            let phase = app
                .git_ui
                .phase
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .map(|phase| format!(" · {}", phase_label(app.language, phase)))
                .unwrap_or_default();
            git_tf(app.language, GitMsg::StateRunning, &[&phase]).into()
        }
        BackupSyncState::PendingLocal => git_tf(
            app.language,
            GitMsg::StatePending,
            &[&presentation.local_items.to_string()],
        )
        .into(),
        BackupSyncState::Incoming => git_tf(
            app.language,
            GitMsg::StateIncoming,
            &[&presentation.incoming_commits.to_string()],
        )
        .into(),
        BackupSyncState::PendingBoth => git_tf(
            app.language,
            GitMsg::StatePendingBoth,
            &[
                &presentation.local_items.to_string(),
                &presentation.incoming_commits.to_string(),
            ],
        )
        .into(),
        BackupSyncState::Synchronized => app.git_label(GitMsg::StateSynchronized).into(),
        BackupSyncState::Offline => app.git_label(GitMsg::StateOffline).into(),
        BackupSyncState::AuthenticationNeeded => app.git_label(GitMsg::StateAuthentication).into(),
        BackupSyncState::ConflictOrRecovery => app.git_label(GitMsg::StateConflict).into(),
        BackupSyncState::UncertainDelivery => app.git_label(GitMsg::StateUncertain).into(),
        BackupSyncState::NeedsAttention => app.git_label(GitMsg::StateAttention).into(),
        BackupSyncState::RemoteUnknown => app.git_label(GitMsg::StateUnknown).into(),
    }
}

fn backup_sync_action_label(app: &MarkionApp, action: BackupSyncAction) -> SharedString {
    match action {
        BackupSyncAction::TurnOn => app.git_label(GitMsg::TurnOn).into(),
        BackupSyncAction::ViewProgress => app.git_label(GitMsg::ViewProgress).into(),
        BackupSyncAction::SyncNow => app.tr(Msg::ItemGitSyncNow).into(),
        BackupSyncAction::ViewStatus => app.git_label(GitMsg::ViewStatus).into(),
        BackupSyncAction::Retry => app.git_label(GitMsg::Retry).into(),
        BackupSyncAction::Reconnect => app.git_label(GitMsg::Reconnect).into(),
        BackupSyncAction::Resolve => app.tr(Msg::ItemGitResolveConflict).into(),
        BackupSyncAction::CheckStatus => app.git_label(GitMsg::CheckStatus).into(),
        BackupSyncAction::Review => app.git_label(GitMsg::Review).into(),
    }
}

fn run_backup_sync_action(
    app: &mut MarkionApp,
    action: BackupSyncAction,
    window: &mut Window,
    cx: &mut Context<MarkionApp>,
) {
    match action {
        BackupSyncAction::TurnOn => app.setup_git_sync(&SetupGitSync, window, cx),
        BackupSyncAction::ViewProgress
        | BackupSyncAction::ViewStatus
        | BackupSyncAction::Review => app.show_git_sync(&ShowGitSync, window, cx),
        BackupSyncAction::SyncNow | BackupSyncAction::Retry => app.sync_now(&SyncNow, window, cx),
        BackupSyncAction::Reconnect => {
            if app.git_ui.retry_operation.is_some() {
                app.retry_git_operation(cx);
            } else {
                app.check_remote(&CheckRemote, window, cx);
            }
        }
        BackupSyncAction::Resolve => app.resolve_git_conflict(&ResolveGitConflict, window, cx),
        BackupSyncAction::CheckStatus => app.check_remote(&CheckRemote, window, cx),
    }
}

pub(super) fn workspace_entry(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let presentation = backup_sync_presentation(app);
    let action = presentation.action;
    let enabled = action == BackupSyncAction::ViewProgress || app.git_ui.running.is_none();
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(button(
            "workspace-backup-sync-status",
            backup_sync_state_label(app, &presentation),
            true,
            app.palette(),
            cx,
            |app, window, cx| app.show_git_sync(&ShowGitSync, window, cx),
        ))
        .child(button(
            "workspace-backup-sync-action",
            backup_sync_action_label(app, action),
            enabled,
            app.palette(),
            cx,
            move |app, window, cx| run_backup_sync_action(app, action, window, cx),
        ))
}

fn sync_location(details: &GitDetails) -> Option<String> {
    let policy = details.policy.as_ref()?;
    let remote = RemoteUrl::parse(policy.target.fetch_url.clone()).ok()?;
    Some(match remote.host {
        Some(host) => host,
        None => Path::new(remote.as_str())
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| remote.as_str().to_string()),
    })
}

pub(super) fn panel_body(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> impl IntoElement {
    let palette = app.palette();
    let presentation = backup_sync_presentation(app);
    let action = presentation.action;
    let running = presentation.state == BackupSyncState::Running;
    let details = app
        .git_ui
        .snapshot
        .as_ref()
        .filter(|details| details.workspace == app.workspace_root);
    let action_enabled = action == BackupSyncAction::ViewProgress || app.git_ui.running.is_none();
    let mut body = div()
        .id("backup-sync-center-scroll")
        .overflow_y_scroll()
        .flex_1()
        .min_h_0()
        .p_4()
        .flex()
        .flex_col()
        .gap_3()
        .text_size(px(13.))
        .text_color(palette.text)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(17.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(app.git_label(GitMsg::BackupAndSync)),
                )
                .child(button(
                    "backup-sync-center-close",
                    app.git_label(GitMsg::Close),
                    true,
                    palette,
                    cx,
                    |app, _, cx| {
                        app.git_ui.center_open = false;
                        app.git_ui.advanced_open = false;
                        cx.notify();
                    },
                )),
        )
        .child(
            div()
                .p_3()
                .rounded_lg()
                .border_1()
                .border_color(palette.border)
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(backup_sync_state_label(app, &presentation)),
                )
                .child(button(
                    "backup-sync-center-primary",
                    backup_sync_action_label(app, action),
                    action_enabled,
                    palette,
                    cx,
                    move |app, window, cx| run_backup_sync_action(app, action, window, cx),
                ))
                .when(running, |card| {
                    card.child(button(
                        "backup-sync-center-cancel",
                        app.git_label(GitMsg::Cancel),
                        true,
                        palette,
                        cx,
                        |app, _, cx| app.cancel_git_operation(cx),
                    ))
                }),
        );

    if let Some(details) = details {
        if let Some(location) = sync_location(details) {
            body = body.child(div().text_color(palette.muted).child(format!(
                "{}: {}",
                app.git_label(GitMsg::SyncLocation),
                location
            )));
        }
        body = body
            .child(div().flex().gap_3().flex_wrap().children([
                div().child(git_tf(
                    app.language,
                    GitMsg::LocalItems,
                    &[&presentation.local_items.to_string()],
                )),
                div().child(git_tf(
                    app.language,
                    GitMsg::IncomingItems,
                    &[&presentation.incoming_commits.to_string()],
                )),
            ]))
            .when_some(presentation.last_confirmed_at, |body, confirmed| {
                body.child(
                    div().text_color(palette.muted).child(git_tf(
                        app.language,
                        GitMsg::LastConfirmed,
                        &[&confirmed
                            .elapsed()
                            .unwrap_or_default()
                            .as_secs()
                            .to_string()],
                    )),
                )
            });
    }

    if details
        .and_then(|details| details.policy.as_ref())
        .is_some()
    {
        body = body.child(
            div()
                .flex()
                .flex_wrap()
                .gap_1()
                .child(
                    button(
                        "backup-sync-activity",
                        app.git_label(GitMsg::SyncActivity),
                        true,
                        palette,
                        cx,
                        |app, _, cx| app.set_git_page(GitPage::Recent, cx),
                    )
                    .when(app.git_ui.page == GitPage::Recent, |button| {
                        button.bg(palette.active_bg)
                    }),
                )
                .child(
                    button(
                        "backup-sync-version-history",
                        app.git_label(GitMsg::VersionHistory),
                        true,
                        palette,
                        cx,
                        |app, _, cx| app.set_git_page(GitPage::FileHistory, cx),
                    )
                    .when(app.git_ui.page == GitPage::FileHistory, |button| {
                        button.bg(palette.active_bg)
                    }),
                )
                .child(button(
                    "backup-sync-settings",
                    app.git_label(GitMsg::Settings),
                    true,
                    palette,
                    cx,
                    |app, window, cx| app.edit_git_settings(window, cx),
                )),
        );
    }

    if matches!(app.git_ui.page, GitPage::Recent | GitPage::FileHistory)
        && let Some(details) = details
        && details.page == app.git_ui.page
    {
        let mut history = div().flex().flex_col().gap_1();
        for (index, commit) in details.history.iter().take(12).enumerate() {
            let identity = details.identity.clone();
            let oid = commit.oid.clone();
            let path = (details.page == GitPage::FileHistory)
                .then(|| details.history_path.clone())
                .flatten();
            history = history.child(button(
                ("backup-sync-history", index),
                format!(
                    "{} · {} · {}",
                    commit.subject, commit.author_name, commit.authored_iso
                ),
                true,
                palette,
                cx,
                move |app, _, cx| {
                    app.inspect_git(
                        identity.clone(),
                        Some(oid.clone()),
                        path.clone(),
                        false,
                        false,
                        cx,
                    )
                },
            ));
        }
        if details.history.is_empty() {
            history = history.child(
                if details.page == GitPage::FileHistory && details.history_path.is_none() {
                    app.git_label(GitMsg::FileHistoryUnavailable)
                } else {
                    app.git_label(GitMsg::Empty)
                },
            );
        }
        body = body.child(history);
    }

    body = body.child(button(
        "backup-sync-advanced-toggle",
        app.git_label(if app.git_ui.advanced_open {
            GitMsg::HideAdvancedGitDetails
        } else {
            GitMsg::AdvancedGitDetails
        }),
        true,
        palette,
        cx,
        |app, _, cx| {
            app.git_ui.advanced_open = !app.git_ui.advanced_open;
            cx.notify();
        },
    ));
    body.when(app.git_ui.advanced_open, |body| {
        body.child(advanced_panel_body(app, cx))
    })
}

pub(super) fn center_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> impl IntoElement {
    let palette = app.palette();
    div()
        .id("backup-sync-center-overlay")
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .occlude()
        .bg(rgba(0x00000055))
        .px_4()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("backup-sync-center-panel")
                .occlude()
                .w_full()
                .max_w(px(640.))
                .max_h(px(760.))
                .bg(palette.panel_bg)
                .border_1()
                .border_color(palette.border)
                .rounded_lg()
                .shadow_lg()
                .flex()
                .flex_col()
                .child(panel_body(app, cx)),
        )
}

fn advanced_panel_body(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> impl IntoElement {
    let palette = app.palette();
    let details = app
        .git_ui
        .snapshot
        .as_ref()
        .filter(|details| details.workspace == app.workspace_root);
    let enabled = app.git_ui.running.is_none();
    let mut body = div()
        .id("git-details-scroll")
        .overflow_y_scroll()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap_2()
        .text_size(px(12.))
        .text_color(palette.text)
        .child(
            div()
                .text_size(px(13.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(
                    app.workspace_root
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| app.workspace_root.display().to_string()),
                ),
        )
        .child(button(
            "git-refresh",
            app.git_label(GitMsg::Refresh),
            !app.git_ui.refreshing,
            palette,
            cx,
            |app, _, cx| app.refresh_git_details(cx),
        ));
    if let Some(report) = app.git_ui.reports.get(&app.workspace_root).or_else(|| {
        details.and_then(|details| app.git_ui.reports.get(&details.identity.worktree_root))
    }) {
        body = body.child(div().max_h(px(72.)).overflow_hidden().child(report.clone()));
    }
    if let Some((_, kind, _)) = &app.git_ui.running {
        body = body
            .child(div().child(format!(
                    "{}…",
                    app.git_ui
                        .phase
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .map(|phase| phase_label(app.language, phase))
                        .unwrap_or_else(|| git_operation_label(app.language, *kind))
                )))
            .child(button(
                "git-cancel",
                app.git_label(GitMsg::Cancel),
                true,
                palette,
                cx,
                |app, _, cx| app.cancel_git_operation(cx),
            ));
    }
    let Some(details) = details else {
        return body
            .child(div().child(app.tr(Msg::StatusGitSyncNotConfigured)))
            .child(button(
                "git-setup",
                app.tr(Msg::ItemGitSyncSetup),
                enabled,
                palette,
                cx,
                |app, window, cx| app.setup_git_sync(&SetupGitSync, window, cx),
            ));
    };
    let unsaved = app
        .tabs
        .iter()
        .filter(|tab| {
            tab.is_dirty()
                && tab
                    .path()
                    .is_some_and(|path| path.starts_with(&details.identity.worktree_root))
        })
        .count();
    body = body
        .child(div().child(details.identity.worktree_root.display().to_string()))
        .child(div().child(format!(
            "{} → {}",
            details.state.branch.as_deref().unwrap_or("—"),
            details.state.upstream.as_deref().unwrap_or("—")
        )))
        .child(div().child(git_tf(
            app.language,
            GitMsg::Layers,
            &[
                &unsaved.to_string(),
                &details.state.worktree.changes.len().to_string(),
            ],
        )))
        .child(div().child(history_label(app.language, details.state.history)));
    if let Some(checked) = app
        .git_ui
        .remote_checks
        .get(&details.identity.worktree_root)
    {
        body = body.child(div().text_color(palette.muted).child(git_tf(
            app.language,
            GitMsg::Checked,
            &[&checked.elapsed().unwrap_or_default().as_secs().to_string()],
        )));
    } else {
        body = body.child(
            div()
                .text_color(palette.muted)
                .child(app.git_label(GitMsg::Unknown)),
        );
    }
    let connected = enabled && details.policy.is_some();
    if app
        .git_ui
        .retry_operation
        .as_ref()
        .is_some_and(|(identity, _)| identity == &details.identity)
    {
        body = body.child(button(
            "git-retry-authentication",
            app.git_label(GitMsg::Retry),
            connected,
            palette,
            cx,
            |app, _, cx| app.retry_git_operation(cx),
        ));
    }
    body = body.child(
        div()
            .flex()
            .flex_wrap()
            .gap_1()
            .child(button(
                "git-commit",
                app.git_label(GitMsg::Commit),
                connected,
                palette,
                cx,
                |app, window, cx| app.commit_locally(&CommitLocally, window, cx),
            ))
            .child(button(
                "git-fetch",
                app.git_label(GitMsg::Fetch),
                connected,
                palette,
                cx,
                |app, window, cx| app.check_remote(&CheckRemote, window, cx),
            ))
            .child(button(
                "git-pull",
                app.git_label(GitMsg::Pull),
                connected,
                palette,
                cx,
                |app, window, cx| app.pull_updates(&PullUpdates, window, cx),
            ))
            .child(button(
                "git-push",
                app.git_label(GitMsg::Push),
                connected,
                palette,
                cx,
                |app, window, cx| app.push_commits(&PushCommits, window, cx),
            )),
    );
    if !details.recovery.is_empty() || details.state.worktree.has_conflicts {
        body = body
            .child(div().child(app.git_label(GitMsg::Recovery)))
            .child(button(
                "git-resolve",
                app.tr(Msg::ItemGitResolveConflict),
                enabled,
                palette,
                cx,
                |app, window, cx| app.resolve_git_conflict(&ResolveGitConflict, window, cx),
            ));
    }
    for (index, checkpoint) in details.recovery.iter().take(20).enumerate() {
        let checkpoint = checkpoint.clone();
        let external = details.assessments.iter().any(|a| {
            assessment_id(a) == checkpoint.operation_id
                && matches!(
                    a,
                    markion_git_sync::RecoveryAssessment::ExternalState { .. }
                )
        });
        let label = details
            .assessments
            .iter()
            .find(|a| assessment_id(a) == checkpoint.operation_id)
            .map(|a| match a {
                markion_git_sync::RecoveryAssessment::OwnedStaging { .. } => GitMsg::UnstageOwned,
                markion_git_sync::RecoveryAssessment::UnknownPushResult { .. } => GitMsg::Verifying,
                _ => GitMsg::LocalRecovery,
            })
            .unwrap_or(GitMsg::Recovery);
        let root = if external {
            markion::default_git_sync_data_dir()
                .join("recovery")
                .join(&checkpoint.operation_id)
        } else {
            checkpoint.identity.worktree_root.clone()
        };
        body = body.child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(phase_label(app.language, checkpoint.phase))
                .when(external, |row| {
                    row.child(app.git_label(GitMsg::ExternalRecovery))
                })
                .child(button(
                    ("git-recover", index),
                    app.git_label(label),
                    enabled && !app.git_ui.conflict_busy && !external,
                    palette,
                    cx,
                    move |app, _, cx| app.recover_git_checkpoint(checkpoint.clone(), cx),
                ))
                .child(button(
                    ("git-recovery-files", index),
                    app.git_label(GitMsg::External),
                    true,
                    palette,
                    cx,
                    move |_, _, cx| cx.reveal_path(&root),
                )),
        );
    }
    body = body.child(
        div().flex().flex_wrap().gap_1().children(
            [
                (GitPage::Changes, GitMsg::Changes),
                (GitPage::Recent, GitMsg::Recent),
                (GitPage::FileHistory, GitMsg::FileHistory),
                (GitPage::Outgoing, GitMsg::Outgoing),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (page, label))| {
                button(
                    ("git-page", index),
                    app.git_label(label),
                    true,
                    palette,
                    cx,
                    move |app, _, cx| app.set_git_page(page, cx),
                )
                .when(app.git_ui.page == page, |button| {
                    button.bg(palette.active_bg)
                })
            }),
        ),
    );
    if app.git_ui.page == GitPage::Changes
        && let Some(draft) = app
            .git_ui
            .commit_draft
            .as_ref()
            .filter(|draft| draft.identity == details.identity)
    {
        let commit_ready = connected
            && !draft.selected.is_empty()
            && !draft.message.buffer.trim().is_empty()
            && draft.expected_head == details.state.head
            && !details.state.worktree.has_external_staging
            && !details.state.worktree.has_conflicts
            && details.state.worktree.operation_in_progress.is_none();
        body = body.child(
            div()
                .p_2()
                .border_1()
                .border_color(palette.border)
                .rounded_md()
                .flex()
                .flex_col()
                .gap_1()
                .child(app.git_label(GitMsg::CreateVersion))
                .child(search_field_view(
                    SearchField::GitCommit,
                    &draft.message,
                    app.search_focus == Some(SearchField::GitCommit),
                    draft.message.buffer.trim().is_empty(),
                    palette,
                    cx,
                ))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1()
                        .child(button(
                            "git-select-all",
                            app.git_label(GitMsg::SelectAllChanges),
                            connected,
                            palette,
                            cx,
                            |app, _, cx| app.select_all_git_commit_paths(true, cx),
                        ))
                        .child(button(
                            "git-clear-selection",
                            app.git_label(GitMsg::ClearSelection),
                            !draft.selected.is_empty(),
                            palette,
                            cx,
                            |app, _, cx| app.select_all_git_commit_paths(false, cx),
                        )),
                )
                .child(button(
                    "git-create-version",
                    git_tf(
                        app.language,
                        GitMsg::CreateVersionCount,
                        &[&draft.selected.len().to_string()],
                    ),
                    commit_ready,
                    palette,
                    cx,
                    |app, window, cx| app.create_local_version(window, cx),
                )),
        );
    }
    let mut rows = div()
        .id("git-inventory")
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap_1();
    let total = if details.page == GitPage::Changes {
        details.state.worktree.changes.len()
    } else {
        details.offset + details.history.len()
    };
    if details.page == app.git_ui.page && details.offset == app.git_ui.offset {
        if details.page == GitPage::Changes {
            for (index, change) in details
                .state
                .worktree
                .changes
                .iter()
                .skip(details.offset)
                .take(PAGE_SIZE)
                .enumerate()
            {
                let identity = details.identity.clone();
                let path = change.path.clone();
                let staged = change.index_status != '.' && change.index_status != '?';
                let label = format!(
                    "{}{} {}{}",
                    change.index_status,
                    change.worktree_status,
                    change
                        .original_path
                        .as_ref()
                        .map(|from| format!("{} → ", from.display()))
                        .unwrap_or_default(),
                    change.path.display()
                );
                let selected = app
                    .git_ui
                    .commit_draft
                    .as_ref()
                    .is_some_and(|draft| draft.selected.contains(&change.path));
                let selectable = connected && !details.attention.contains(&change.path);
                let toggle_path = path.clone();
                rows = rows.child(
                    div()
                        .flex()
                        .gap_1()
                        .child(button(
                            ("git-change-select", index),
                            if selected { "☑" } else { "☐" },
                            selectable,
                            palette,
                            cx,
                            move |app, _, cx| app.toggle_git_commit_path(&toggle_path, cx),
                        ))
                        .child(
                            button(
                                ("git-change", index),
                                label,
                                true,
                                palette,
                                cx,
                                move |app, _, cx| {
                                    app.inspect_git(
                                        identity.clone(),
                                        None,
                                        Some(path.clone()),
                                        staged,
                                        false,
                                        cx,
                                    )
                                },
                            )
                            .flex_1()
                            .min_w_0(),
                        ),
                );
                if details.attention.contains(&change.path) {
                    rows = rows.child(
                        div()
                            .text_color(palette.invalid)
                            .child(app.git_label(GitMsg::Attention)),
                    );
                }
            }
        } else {
            for (index, commit) in details.history.iter().enumerate() {
                let identity = details.identity.clone();
                let oid = commit.oid.clone();
                let path = (details.page == GitPage::FileHistory)
                    .then(|| details.history_path.clone())
                    .flatten();
                rows = rows.child(button(
                    ("git-history", index),
                    format!(
                        "{} {} · {} · {}",
                        oid.short(),
                        commit.subject,
                        commit.author_name,
                        commit.authored_iso
                    ),
                    true,
                    palette,
                    cx,
                    move |app, _, cx| {
                        app.inspect_git(
                            identity.clone(),
                            Some(oid.clone()),
                            path.clone(),
                            false,
                            false,
                            cx,
                        )
                    },
                ));
            }
        }
    }
    for (index, (note, issue)) in details.attachments.iter().enumerate() {
        let absolute = details.identity.worktree_root.join(note);
        let note = note.clone();
        let issue = issue.clone();
        rows = rows.child(
            div()
                .p_2()
                .border_1()
                .border_color(palette.border)
                .flex()
                .flex_col()
                .gap_1()
                .child(app.git_label(GitMsg::Attachments))
                .child(format!("{} → {}", note.display(), issue.authored_url))
                .child(button(
                    ("git-repair-attachment", index),
                    app.git_label(GitMsg::Repair),
                    enabled,
                    palette,
                    cx,
                    move |app, _, cx| app.open_file_in_new_tab_from_path(absolute.clone(), cx),
                ))
                .child(button(
                    ("git-include-attachment", index),
                    app.git_label(GitMsg::Include),
                    connected,
                    palette,
                    cx,
                    |app, window, cx| app.edit_git_settings(window, cx),
                ))
                .child(button(
                    ("git-ack-attachment", index),
                    app.git_label(GitMsg::Acknowledged),
                    connected,
                    palette,
                    cx,
                    move |app, _, cx| {
                        app.acknowledge_git_attachment(note.clone(), issue.clone(), cx)
                    },
                )),
        );
    }
    if total == 0 {
        rows = rows.child(
            if details.page == GitPage::FileHistory && details.history_path.is_none() {
                app.git_label(GitMsg::FileHistoryUnavailable)
            } else {
                app.git_label(GitMsg::Empty)
            },
        );
    }
    if !details.attachments.is_empty()
        && app.active_tab().path().is_some_and(|path| {
            details
                .attachments
                .iter()
                .any(|(note, _)| details.identity.worktree_root.join(note) == path)
        })
    {
        rows = rows.child(button(
            "git-organize-images",
            app.tr(Msg::ItemOrganizeLocalImages),
            connected,
            palette,
            cx,
            |app, window, cx| app.organize_local_images(&OrganizeLocalImages, window, cx),
        ));
    }
    body = body.child(rows).child(
        div()
            .flex()
            .gap_1()
            .child(button(
                "git-previous",
                app.git_label(GitMsg::Previous),
                app.git_ui.offset > 0,
                palette,
                cx,
                |app, _, cx| {
                    app.git_ui.offset = app.git_ui.offset.saturating_sub(PAGE_SIZE);
                    app.refresh_git_details(cx);
                },
            ))
            .child(button(
                "git-next",
                app.git_label(GitMsg::Next),
                total >= app.git_ui.offset + PAGE_SIZE,
                palette,
                cx,
                |app, _, cx| {
                    app.git_ui.offset += PAGE_SIZE;
                    app.refresh_git_details(cx);
                },
            )),
    );
    if let Some(policy) = &details.policy {
        body = body
            .child(
                div().text_color(palette.muted).child(git_tf(
                    app.language,
                    GitMsg::Scope,
                    &[&policy
                        .tracked_roots
                        .iter()
                        .map(|path| {
                            if path.as_os_str().is_empty() {
                                ".".into()
                            } else {
                                path.display().to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ")],
                )),
            )
            .child(
                div()
                    .text_color(palette.muted)
                    .child(app.git_label(GitMsg::WholeHistory)),
            )
            .child(button(
                "git-disconnect",
                app.git_label(GitMsg::Disconnect),
                enabled,
                palette,
                cx,
                |app, _, cx| app.disconnect_git(cx),
            ));
        body = body.child(button(
            "git-settings",
            app.git_label(GitMsg::Settings),
            enabled,
            palette,
            cx,
            |app, window, cx| app.edit_git_settings(window, cx),
        ));
    }
    body.child(button(
        "git-reconnect",
        app.tr(Msg::ItemGitSyncSetup),
        enabled,
        palette,
        cx,
        |app, window, cx| app.setup_git_sync(&SetupGitSync, window, cx),
    ))
}

pub(super) fn recovery_policy(checkpoint: OperationCheckpoint) -> Option<RepositoryPolicy> {
    Some(RepositoryPolicy {
        identity: checkpoint.identity,
        target: checkpoint.target?,
        tracked_roots: Vec::new(),
        new_file_rules: Vec::new(),
        message_template: String::new(),
        include_device_name: false,
        background_fetch: false,
        last_confirmed_remote: None,
        acknowledged_resource_omissions: Vec::new(),
    })
}

impl MarkionApp {
    fn acknowledge_git_attachment(
        &mut self,
        note: PathBuf,
        issue: markion_git_sync::AttachmentIssue,
        cx: &mut Context<Self>,
    ) {
        let Some(details) = &self.git_ui.snapshot else {
            return;
        };
        let identity = details.identity.clone();
        let absolute = identity.worktree_root.join(&note);
        let live = self.tabs.iter().find_map(|tab| {
            (tab.path() == Some(absolute.as_path())).then(|| tab.document.text().to_string())
        });
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let repository = GitRepository::discover(
                        GitCommandRunner::new(executable),
                        &identity.worktree_root,
                    )
                    .map_err(|e| e.to_string())?;
                    if repository.identity() != &identity {
                        return Err("repository identity changed".to_string());
                    }
                    let store = PolicyStore::new(default_git_sync_policy_path());
                    let mut policy = store
                        .load()
                        .map_err(|e| e.to_string())?
                        .repositories
                        .into_iter()
                        .find(|policy| policy.identity == identity)
                        .ok_or("repository is disconnected")?;
                    let text = live
                        .map(Ok)
                        .unwrap_or_else(|| fs::read_to_string(&absolute))
                        .map_err(|e| e.to_string())?;
                    let current = markion_git_sync::inspect_note_attachments(
                        &repository,
                        &policy,
                        &note,
                        &text,
                    )
                    .map_err(|e| e.to_string())?;
                    if !current
                        .issues
                        .iter()
                        .any(|current| current.reference_fingerprint == issue.reference_fingerprint)
                    {
                        return Err("attachment reference changed; refresh details".into());
                    }
                    policy
                        .acknowledged_resource_omissions
                        .push(markion_git_sync::ScopeException {
                            path: note,
                            content_fingerprint: issue.reference_fingerprint,
                        });
                    store
                        .update(|policies| policies.upsert(policy))
                        .map_err(|e| e.to_string())
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                if let Err(error) = result {
                    app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]);
                }
                app.refresh_git_details(cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn recover_git_checkpoint(&mut self, checkpoint: OperationCheckpoint, cx: &mut Context<Self>) {
        if self.git_ui.running.is_some() || self.git_ui.conflict_busy {
            return;
        }
        self.git_ui.conflict_busy = true;
        let identity = checkpoint.identity.clone();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        cx.spawn(async move |this, cx| {
            let root = identity.clone();
            let result = cx.background_spawn(async move {
                let repository = GitRepository::discover(GitCommandRunner::new(executable), &root.worktree_root).map_err(|e| e.to_string())?;
                if repository.identity() != &root { return Err("repository identity changed".to_string()); }
                let manager = markion_git_sync::RecoveryManager::new(repository.clone(), journal());
                let assessment = manager.assess().map_err(|e| e.to_string())?.into_iter().find(|assessment| assessment_id(assessment) == checkpoint.operation_id).ok_or("recovery no longer exists")?;
                use markion_git_sync::RecoveryAssessment;
                match assessment {
                    RecoveryAssessment::OwnedStaging { operation_id } => manager.recover_staging(&operation_id).map_err(|e| e.to_string()),
                    RecoveryAssessment::CommitCompleted { operation_id, .. } | RecoveryAssessment::IntegrationCompleted { operation_id, .. } => manager.adopt_completed(&operation_id).map(|_| ()).map_err(|e| e.to_string()),
                    RecoveryAssessment::UnknownPushResult { .. } => {
                        let mut policy = recovery_policy(checkpoint.clone()).ok_or("missing recovery target")?;
                        let engine = markion_git_sync::GitSyncEngine::new(repository, journal(), markion_git_sync::SyncOptions::default())
                            .with_foreground_credentials();
                        match engine.verify_delivery(&checkpoint, &mut policy, &CancellationToken::new()).map_err(|e| e.to_string())? {
                            markion_git_sync::SyncOutcome::Synchronized { .. } => Ok(()),
                            _ => Err("recorded commit has not been confirmed on the remote; use Push Commits when ready".into()),
                        }
                    }
                    _ => Err("repository requires external inspection; recovery files are retained".into()),
                }
            }).await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.conflict_busy = false;
                match result {
                    Ok(()) => {
                        app.git_ui.recovery_guards.remove(&identity);
                        if app.git_conflict_admission.as_ref().and_then(|guard| guard.identity()) == Some(&identity) { app.git_conflict_admission = None; app.git_ui.conflict = None; app.git_ui.conflict_surface_open = false; }
                        app.status = app.tr(Msg::StatusGitSyncComplete).into();
                    }
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                app.refresh_git_details(cx); cx.notify();
            });
        }).detach();
    }
}

fn assessment_id(assessment: &markion_git_sync::RecoveryAssessment) -> &str {
    use markion_git_sync::RecoveryAssessment::*;
    match assessment {
        OwnedStaging { operation_id }
        | CommitCompleted { operation_id, .. }
        | ConflictSession { operation_id }
        | IntegrationCompleted { operation_id, .. }
        | UnknownPushResult { operation_id, .. }
        | ExternalState { operation_id } => operation_id,
    }
}

fn parse_scope(value: &str) -> Result<Vec<PathBuf>, ()> {
    value
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            let path = PathBuf::from(value);
            if path.is_absolute()
                || path.components().any(|part| {
                    !matches!(
                        part,
                        std::path::Component::Normal(_) | std::path::Component::CurDir
                    )
                })
            {
                return Err(());
            }
            Ok(path
                .components()
                .filter_map(|part| match part {
                    std::path::Component::Normal(part) => Some(part),
                    _ => None,
                })
                .collect())
        })
        .collect()
}

pub(super) fn settings_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let form = app.git_ui.settings.as_ref().expect("settings visible");
    let palette = app.palette();
    let presentation = backup_sync_presentation(app);
    let labels = [
        GitMsg::TrackedRoots,
        GitMsg::NewRoots,
        GitMsg::Message,
        GitMsg::Name,
        GitMsg::Email,
    ];
    let mut advanced_fields = div().flex().flex_col().gap_2();
    for (index, label) in labels.into_iter().enumerate() {
        advanced_fields = advanced_fields.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(app.git_label(label))
                .child(search_field_view(
                    SearchField::Git(index),
                    &form.fields[index],
                    app.search_focus == Some(SearchField::Git(index)),
                    false,
                    palette,
                    cx,
                )),
        );
    }
    let location = RemoteUrl::parse(form.policy.target.fetch_url.clone())
        .ok()
        .and_then(|remote| remote.host)
        .unwrap_or_else(|| form.policy.target.remote.clone());
    let recovery_items = app
        .git_ui
        .snapshot
        .as_ref()
        .filter(|details| details.identity == form.policy.identity)
        .map_or(0, |details| details.recovery.len());
    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x00000066))
        .flex()
        .items_center()
        .justify_center()
        .p_4()
        .child(
            div()
                .occlude()
                .w(px(600.))
                .max_w_full()
                .p_4()
                .bg(palette.panel_bg)
                .rounded_lg()
                .text_color(palette.text)
                .text_size(px(13.))
                .flex()
                .flex_col()
                .gap_2()
                .child(app.git_label(GitMsg::BackupAndSync))
                .child(backup_sync_state_label(app, &presentation))
                .child(format!(
                    "{}: {}",
                    app.git_label(GitMsg::SyncLocation),
                    location
                ))
                .child(button(
                    "git-policy-background",
                    format!(
                        "{}: {}",
                        app.git_label(GitMsg::WorkspaceBackgroundCheck),
                        app.git_label(if form.policy.background_fetch {
                            GitMsg::Enabled
                        } else {
                            GitMsg::Disabled
                        })
                    ),
                    app.git_ui.running.is_none(),
                    palette,
                    cx,
                    |app, _, cx| {
                        if let Some(form) = &mut app.git_ui.settings {
                            form.policy.background_fetch = !form.policy.background_fetch;
                        }
                        cx.notify();
                    },
                ))
                .child(
                    div()
                        .text_color(palette.muted)
                        .child(app.git_label(GitMsg::BackgroundCheckHelp)),
                )
                .when(recovery_items > 0, |view| {
                    view.child(format!(
                        "{}: {}",
                        app.git_label(GitMsg::Recovery),
                        recovery_items
                    ))
                })
                .child(button(
                    "git-disconnect-workspace",
                    app.git_label(GitMsg::Disconnect),
                    app.git_ui.running.is_none(),
                    palette,
                    cx,
                    |app, _, cx| {
                        app.git_ui.settings = None;
                        app.search_focus = None;
                        app.disconnect_git(cx);
                    },
                ))
                .child(button(
                    "git-settings-advanced",
                    app.git_label(if form.advanced {
                        GitMsg::HideAdvancedGitSettings
                    } else {
                        GitMsg::AdvancedGitSettings
                    }),
                    app.git_ui.running.is_none(),
                    palette,
                    cx,
                    |app, _, cx| {
                        if let Some(form) = &mut app.git_ui.settings {
                            form.advanced = !form.advanced;
                        }
                        cx.notify();
                    },
                ))
                .when(form.advanced, |view| {
                    view.child(app.git_label(GitMsg::WholeHistory))
                        .child(advanced_fields)
                })
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(button(
                            "git-save-settings",
                            app.git_label(GitMsg::SaveSettings),
                            app.git_ui.running.is_none(),
                            palette,
                            cx,
                            |app, _, cx| app.save_git_settings(cx),
                        ))
                        .child(button(
                            "git-close-settings",
                            app.git_label(GitMsg::Close),
                            true,
                            palette,
                            cx,
                            |app, _, cx| {
                                app.git_ui.settings = None;
                                app.search_focus = None;
                                cx.notify();
                            },
                        )),
                ),
        )
}

pub(super) fn onboarding_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let setup = app.git_ui.onboarding.as_ref().expect("onboarding visible");
    let palette = app.palette();
    let selected_label = |mode: GitOnboardingMode, label: GitMsg| {
        if setup.mode == mode {
            format!("✓ {}", app.git_label(label))
        } else {
            app.git_label(label).to_string()
        }
    };
    let primary = if setup.error.is_some() {
        app.git_label(GitMsg::Retry)
    } else if setup.advanced_required {
        app.git_label(GitMsg::AdvancedRepositorySetup)
    } else {
        app.git_label(match setup.mode {
            GitOnboardingMode::UseCurrentFolder => GitMsg::UseThisFolder,
            GitOnboardingMode::CloneRepository => GitMsg::CloneNotesRepository,
            GitOnboardingMode::InitializeFolder => GitMsg::StartSyncingFolder,
        })
    };
    let labels = [
        if setup.mode == GitOnboardingMode::UseCurrentFolder {
            GitMsg::SyncAddressOptional
        } else {
            GitMsg::SyncAddress
        },
        GitMsg::Destination,
        GitMsg::Branch,
        GitMsg::Name,
        GitMsg::Email,
    ];
    let mut fields = div().flex().flex_col().gap_2();
    for index in 0..5 {
        let visible = match index {
            0 => true,
            1 => setup.mode == GitOnboardingMode::CloneRepository,
            2..=4 => setup.advanced,
            _ => false,
        };
        if visible {
            fields = fields.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(app.git_label(labels[index]))
                    .child(search_field_view(
                        SearchField::GitSetup(index),
                        &setup.fields[index],
                        app.search_focus == Some(SearchField::GitSetup(index)),
                        false,
                        palette,
                        cx,
                    )),
            );
        }
    }

    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x00000066))
        .flex()
        .items_center()
        .justify_center()
        .p_4()
        .child(
            div()
                .occlude()
                .w(px(640.))
                .max_w_full()
                .max_h_full()
                .p_4()
                .bg(palette.panel_bg)
                .rounded_lg()
                .text_color(palette.text)
                .text_size(px(13.))
                .flex()
                .flex_col()
                .gap_3()
                .child(app.git_label(GitMsg::TurnOn))
                .child(
                    div()
                        .text_color(palette.muted)
                        .child(app.git_label(GitMsg::QuickSetupHint)),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.muted)
                        .child(app.git_label(GitMsg::SyncMirrorsChangesHelp)),
                )
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(selected_label(
                            setup.mode,
                            match setup.mode {
                                GitOnboardingMode::UseCurrentFolder => GitMsg::UseThisFolder,
                                GitOnboardingMode::CloneRepository => GitMsg::CloneNotesRepository,
                                GitOnboardingMode::InitializeFolder => GitMsg::StartSyncingFolder,
                            },
                        )),
                )
                .child(button(
                    "git-setup-alternatives",
                    app.git_label(if setup.alternatives_open {
                        GitMsg::HideSetupOptions
                    } else {
                        GitMsg::OtherSetupOptions
                    }),
                    !setup.busy,
                    palette,
                    cx,
                    |app, _, cx| app.toggle_git_onboarding_alternatives(cx),
                ))
                .when(setup.alternatives_open, |view| {
                    view.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_1()
                            .child(button(
                                "git-setup-current",
                                selected_label(
                                    GitOnboardingMode::UseCurrentFolder,
                                    GitMsg::UseThisFolder,
                                ),
                                !setup.busy,
                                palette,
                                cx,
                                |app, window, cx| {
                                    app.set_git_onboarding_mode(
                                        GitOnboardingMode::UseCurrentFolder,
                                        window,
                                        cx,
                                    )
                                },
                            ))
                            .child(button(
                                "git-setup-clone",
                                selected_label(
                                    GitOnboardingMode::CloneRepository,
                                    GitMsg::CloneNotesRepository,
                                ),
                                !setup.busy,
                                palette,
                                cx,
                                |app, window, cx| {
                                    app.set_git_onboarding_mode(
                                        GitOnboardingMode::CloneRepository,
                                        window,
                                        cx,
                                    )
                                },
                            ))
                            .child(button(
                                "git-setup-initialize",
                                selected_label(
                                    GitOnboardingMode::InitializeFolder,
                                    GitMsg::StartSyncingFolder,
                                ),
                                !setup.busy,
                                palette,
                                cx,
                                |app, window, cx| {
                                    app.set_git_onboarding_mode(
                                        GitOnboardingMode::InitializeFolder,
                                        window,
                                        cx,
                                    )
                                },
                            )),
                    )
                })
                .child(fields)
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(palette.muted)
                        .child(app.git_label(GitMsg::SyncAddressHelp)),
                )
                .when(setup.advanced_required, |view| {
                    view.child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(app.git_label(GitMsg::AdvancedRepositorySetup)),
                    )
                })
                .when(!setup.advanced_required, |view| {
                    view.child(button(
                        "git-setup-advanced",
                        app.git_label(GitMsg::Advanced),
                        !setup.busy,
                        palette,
                        cx,
                        |app, _, cx| app.toggle_git_onboarding_advanced(cx),
                    ))
                })
                .when(setup.advanced, |view| {
                    let view = view.child(
                        div()
                            .text_color(palette.muted)
                            .child(app.git_label(GitMsg::WholeHistory)),
                    );
                    match setup.repository_root.as_deref() {
                        Some(repository_root)
                            if repository_root != app.workspace_root.as_path() =>
                        {
                            view.child(div().text_color(palette.muted).child(format!(
                                "{}: {}",
                                app.git_label(GitMsg::NotesFolder),
                                app.workspace_root.display()
                            )))
                            .child(
                                div().text_color(palette.muted).child(format!(
                                    "{}: {}",
                                    app.git_label(GitMsg::RepositoryFolder),
                                    repository_root.display()
                                )),
                            )
                        }
                        _ => view,
                    }
                })
                .when_some(setup.error.clone(), |view, error| {
                    view.child(div().text_color(palette.invalid).child(error))
                })
                .when(setup.busy, |view| {
                    view.child(app.git_label(GitMsg::Preparing))
                })
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(button(
                            "git-setup-submit",
                            primary,
                            !setup.busy,
                            palette,
                            cx,
                            |app, _, cx| app.submit_git_onboarding(cx),
                        ))
                        .child(button(
                            "git-setup-cancel",
                            app.git_label(GitMsg::Cancel),
                            true,
                            palette,
                            cx,
                            |app, _, cx| app.cancel_git_onboarding(cx),
                        )),
                ),
        )
}

pub(super) fn preferences_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let advanced = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.muted)
                .child(app.git_label(GitMsg::AdvancedGitSettings)),
        )
        .child(
            div()
                .id("git-executable-summary")
                .text_size(px(12.))
                .child(format!(
                    "{}: {}",
                    app.git_label(GitMsg::Executable),
                    app.git_preferences
                        .executable
                        .clone()
                        .unwrap_or_else(|| app.git_label(GitMsg::Default).into())
                )),
        )
        .when_some(app.git_ui.executable_status.clone(), |view, status| {
            view.child(
                div()
                    .id("git-executable-status")
                    .text_size(px(12.))
                    .text_color(palette.muted)
                    .child(status),
            )
        })
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_1()
                .child(button(
                    "git-choose-executable",
                    app.git_label(GitMsg::Executable),
                    app.git_ui.running.is_none(),
                    palette,
                    cx,
                    |app, window, cx| app.choose_git_executable(window, cx),
                ))
                .child(button(
                    "git-default-executable",
                    app.git_label(GitMsg::Default),
                    app.git_ui.running.is_none(),
                    palette,
                    cx,
                    |app, _, cx| {
                        app.git_preferences.executable = None;
                        app.persist_preferences();
                        app.detect_git_executable(cx);
                    },
                )),
        );
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.muted)
                .child(app.git_label(GitMsg::BackupAndSync)),
        )
        .child(button(
            "git-global-background-setting",
            format!(
                "{}: {}",
                app.git_label(GitMsg::GlobalBackgroundCheck),
                app.git_label(if app.git_preferences.background_check {
                    GitMsg::Enabled
                } else {
                    GitMsg::Disabled
                })
            ),
            true,
            palette,
            cx,
            |app, _, cx| {
                app.toggle_git_background_check(cx);
                app.poll_git_background_check(cx);
            },
        ))
        .child(
            div()
                .text_size(px(12.))
                .text_color(palette.muted)
                .child(app.git_label(GitMsg::BackgroundCheckHelp)),
        )
        .child(button(
            "git-preferences-advanced",
            app.git_label(if app.git_ui.preferences_advanced {
                GitMsg::HideAdvancedGitSettings
            } else {
                GitMsg::AdvancedGitSettings
            }),
            app.git_ui.running.is_none(),
            palette,
            cx,
            |app, _, cx| {
                app.git_ui.preferences_advanced = !app.git_ui.preferences_advanced;
                cx.notify();
            },
        ))
        .when(app.git_ui.preferences_advanced, |view| view.child(advanced))
}

pub(super) fn inspection_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let inspection = app.git_ui.inspection.as_ref().expect("inspection visible");
    let palette = app.palette();
    let mut content = div()
        .id("git-inspection-content")
        .min_h_0()
        .flex_1()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_1();
    if inspection.binary {
        content = content.child(app.git_label(GitMsg::BinaryContent));
    } else if inspection.limited {
        content = content.child(app.git_label(GitMsg::Limited));
    }
    if inspection.paths.is_empty() && !inspection.binary {
        // One bounded string avoids rebuilding an element for every source line.
        content = content.child(
            div()
                .font_family(
                    app.code_font_family
                        .clone()
                        .unwrap_or_else(|| DEFAULT_CODE_FONT_FAMILY.into()),
                )
                .child(inspection.text.clone()),
        );
    } else {
        for (index, path) in inspection
            .paths
            .iter()
            .skip(inspection.offset)
            .take(PAGE_SIZE)
            .enumerate()
        {
            let identity = inspection.identity.clone();
            let path = path.clone();
            let commit = inspection.commit.clone();
            content = content.child(button(
                ("git-commit-path", index),
                path.display().to_string(),
                true,
                palette,
                cx,
                move |app, _, cx| {
                    app.inspect_git(
                        identity.clone(),
                        commit.clone(),
                        Some(path.clone()),
                        false,
                        false,
                        cx,
                    )
                },
            ));
        }
        content = content.child(
            div()
                .flex()
                .gap_1()
                .child(button(
                    "git-path-previous",
                    app.git_label(GitMsg::Previous),
                    inspection.offset > 0,
                    palette,
                    cx,
                    |app, _, cx| {
                        if let Some(view) = &mut app.git_ui.inspection {
                            view.offset = view.offset.saturating_sub(PAGE_SIZE);
                        }
                        cx.notify();
                    },
                ))
                .child(button(
                    "git-path-next",
                    app.git_label(GitMsg::Next),
                    inspection.paths.len() > inspection.offset + PAGE_SIZE,
                    palette,
                    cx,
                    |app, _, cx| {
                        if let Some(view) = &mut app.git_ui.inspection {
                            view.offset += PAGE_SIZE;
                        }
                        cx.notify();
                    },
                )),
        );
    }
    let root = inspection.identity.worktree_root.clone();
    let compare_identity = inspection.identity.clone();
    let compare_commit = inspection.commit.clone();
    let compare_path = inspection.path.clone();
    let restorable = inspection.copy.as_ref().is_some_and(|bytes| {
        !bytes.contains(&0)
            && std::str::from_utf8(bytes).is_ok()
            && inspection
                .path
                .as_deref()
                .is_some_and(|path| is_markdown_path(path) || is_text_path(path))
    });
    div()
        .absolute()
        .inset_0()
        .bg(rgba(0x00000066))
        .p_4()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .occlude()
                .w_full()
                .max_w(px(980.))
                .h(px(580.))
                .max_h_full()
                .p_4()
                .rounded_lg()
                .bg(palette.panel_bg)
                .text_color(palette.text)
                .text_size(px(13.))
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(inspection.title.clone()),
                )
                .when_some(inspection.metadata.as_ref(), |view, metadata| {
                    view.child(div().text_color(palette.muted).child(format!(
                        "{} <{}> · {} · {} · {} parent(s)",
                        metadata.author_name,
                        metadata.author_email,
                        metadata.authored_iso,
                        metadata.oid.short(),
                        metadata.parents.len()
                    )))
                })
                .when(
                    inspection.commit.is_none() && inspection.path.is_some(),
                    |view| {
                        view.child(
                            div().flex().gap_1().children(
                                [false, true]
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, staged)| {
                                        let identity = inspection.identity.clone();
                                        let path = inspection.path.clone();
                                        button(
                                            ("git-diff-layer", index),
                                            app.git_label(if staged {
                                                GitMsg::Staged
                                            } else {
                                                GitMsg::Unstaged
                                            }),
                                            true,
                                            palette,
                                            cx,
                                            move |app, _, cx| {
                                                app.inspect_git(
                                                    identity.clone(),
                                                    None,
                                                    path.clone(),
                                                    staged,
                                                    false,
                                                    cx,
                                                )
                                            },
                                        )
                                    }),
                            ),
                        )
                    },
                )
                .child(content)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .child(button(
                            "git-compare-current",
                            app.git_label(GitMsg::CompareCurrent),
                            compare_commit.is_some()
                                && compare_path.is_some()
                                && !inspection.compare_current,
                            palette,
                            cx,
                            move |app, _, cx| {
                                app.inspect_git(
                                    compare_identity.clone(),
                                    compare_commit.clone(),
                                    compare_path.clone(),
                                    false,
                                    true,
                                    cx,
                                )
                            },
                        ))
                        .child(button(
                            "git-restore-editor",
                            app.git_label(GitMsg::RestoreEditor),
                            restorable,
                            palette,
                            cx,
                            |app, window, cx| app.restore_git_version(window, cx),
                        ))
                        .child(button(
                            "git-save-copy",
                            app.git_label(GitMsg::Copy),
                            inspection.copy.is_some(),
                            palette,
                            cx,
                            |app, window, cx| app.save_git_copy(window, cx),
                        ))
                        .child(button(
                            "git-open-folder",
                            app.git_label(GitMsg::External),
                            true,
                            palette,
                            cx,
                            move |_, _, cx| cx.reveal_path(&root),
                        ))
                        .child(button(
                            "git-close-inspection",
                            app.git_label(GitMsg::Close),
                            true,
                            palette,
                            cx,
                            |app, window, cx| {
                                app.git_ui.inspection = None;
                                app.git_ui.inspection_pending = None;
                                app.git_ui.inspection_generation =
                                    app.git_ui.inspection_generation.wrapping_add(1);
                                window.focus(&app.focus_handle);
                                cx.notify();
                            },
                        )),
                ),
        )
}

pub(super) fn git_operation_label(language: Language, operation: OperationKind) -> &'static str {
    match operation {
        OperationKind::CommitLocally => git_t(language, GitMsg::Commit),
        OperationKind::CheckRemote => git_t(language, GitMsg::Fetch),
        OperationKind::PullUpdates => git_t(language, GitMsg::Pull),
        OperationKind::PushCommits => git_t(language, GitMsg::Push),
        _ => t(language, Msg::ItemGitSyncNow),
    }
}

pub(super) fn phase_label(
    language: Language,
    phase: markion_git_sync::OperationPhase,
) -> &'static str {
    use markion_git_sync::OperationPhase::*;
    match phase {
        Preparing => git_t(language, GitMsg::Preparing),
        Staging => git_t(language, GitMsg::Staging),
        Committing => git_t(language, GitMsg::Commit),
        Fetching => git_t(language, GitMsg::Fetch),
        Integrating => git_t(language, GitMsg::Integrating),
        Resolving => t(language, Msg::ItemGitResolveConflict),
        Pushing => git_t(language, GitMsg::Push),
        Verifying => git_t(language, GitMsg::Verifying),
        Complete => t(language, Msg::StatusGitSyncComplete),
        _ => git_t(language, GitMsg::Recovery),
    }
}

pub(super) fn history_label(language: Language, relation: HistoryRelation) -> String {
    let (ahead, behind) = match relation {
        HistoryRelation::Equal => (0, 0),
        HistoryRelation::Ahead { commits } => (commits, 0),
        HistoryRelation::Behind { commits } => (0, commits),
        HistoryRelation::Diverged { ahead, behind } => (ahead, behind),
        _ => return git_t(language, GitMsg::Unknown).into(),
    };
    git_tf(
        language,
        GitMsg::Relation,
        &[&ahead.to_string(), &behind.to_string()],
    )
}
