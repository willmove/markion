//! Conflict drafts are ordinary editable documents outside the repository.
//! Only the explicit resolution action writes to the conflict index.
use super::*;
use git_panel::button;
use markion_git_sync::{
    CancellationToken, ConflictManager, ConflictResolution, ConflictSession, ConflictSide,
    ConflictSource, GitCommandRunner, GitRepository, RepositoryIdentity,
};

pub(super) struct ConflictView {
    pub identity: RepositoryIdentity,
    pub manager: ConflictManager,
    pub session: ConflictSession,
    pub sources: [Option<ConflictSource>; 3],
    pub draft_path: Option<PathBuf>,
    pub source_paths: [Option<PathBuf>; 3],
    pub hunks: Vec<TextHunk>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TextHunk {
    range: Range<usize>,
    local: String,
    remote: String,
}

/// Presentation only. Resolution authority remains the actual Git index.
fn text_hunks(text: &str) -> Vec<TextHunk> {
    let mut result = Vec::new();
    let mut start = None;
    let mut local_start = 0;
    let mut local_end = None;
    let mut remote_start = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.starts_with("<<<<<<< ") {
            if start.is_some() {
                return Vec::new();
            }
            start = Some(offset);
            local_start = offset + line.len();
            local_end = None;
            remote_start = None;
        } else if start.is_some() && trimmed.starts_with("||||||| ") {
            local_end = Some(offset);
        } else if start.is_some() && trimmed == "=======" {
            if remote_start.is_some() {
                return Vec::new();
            }
            local_end.get_or_insert(offset);
            remote_start = Some(offset + line.len());
        } else if trimmed.starts_with(">>>>>>> ") {
            if let (Some(begin), Some(end), Some(remote)) = (start.take(), local_end, remote_start)
            {
                result.push(TextHunk {
                    range: begin..offset + line.len(),
                    local: text[local_start..end].into(),
                    remote: text[remote..offset].into(),
                });
            } else {
                return Vec::new();
            }
        }
        offset += line.len();
    }
    if start.is_some() { Vec::new() } else { result }
}

impl MarkionApp {
    pub(super) fn choose_git_hunk(
        &mut self,
        index: usize,
        side: Option<ConflictSide>,
        cx: &mut Context<Self>,
    ) {
        if self.git_ui.conflict_busy {
            return;
        }
        let Some(view) = &self.git_ui.conflict else {
            return;
        };
        let Some(path) = view.draft_path.clone() else {
            return;
        };
        if self
            .tabs
            .iter()
            .any(|tab| tab.path() == Some(path.as_path()) && tab.is_dirty())
        {
            self.status = self.git_label(GitMsg::UnsavedResult).into();
            cx.notify();
            return;
        }
        let Some(expected) = view.hunks.get(index).cloned() else {
            return;
        };
        let manager = view.manager.clone();
        let session = view.session.clone();
        let relative = session.files[0].path.clone();
        self.git_ui.conflict_busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let (mut text, identity) =
                        read_document_source(&path).map_err(|error| error.to_string())?;
                    let hunks = text_hunks(&text);
                    if hunks.get(index) != Some(&expected) {
                        return Err("merge draft changed; refresh the comparison".into());
                    }
                    let replacement = match side {
                        Some(ConflictSide::Remote) => expected.remote,
                        Some(_) => expected.local,
                        None => expected.local + &expected.remote,
                    };
                    text.replace_range(expected.range, &replacement);
                    manager
                        .save_draft(
                            &session,
                            relative,
                            text.as_bytes().to_vec(),
                            &CancellationToken::new(),
                        )
                        .map_err(|error| error.to_string())?;
                    save_text_snapshot(&path, Some(&identity), &text)
                        .map_err(|error| error.to_string())?;
                    let (_, identity) =
                        read_document_source(&path).map_err(|error| error.to_string())?;
                    Ok::<_, String>((path, text, identity))
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.conflict_busy = false;
                match result {
                    Ok((path, text, identity)) => {
                        for tab in &mut app.tabs {
                            if tab.path() == Some(path.as_path()) && !tab.is_dirty() {
                                let instance = tab.document.instance_id();
                                let version = tab.document.version();
                                let _ = tab.document.apply_external_reload_checked(
                                    instance,
                                    version,
                                    text.clone(),
                                    identity.clone(),
                                );
                            }
                        }
                        app.reload_git_conflict(cx);
                    }
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub(super) fn resolve_git_conflict(
        &mut self,
        _: &ResolveGitConflict,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.git_ui.conflict_busy || self.git_ui.running.is_some() {
            return;
        }
        let Some(identity) = self
            .git_ui
            .snapshot
            .as_ref()
            .filter(|details| details.workspace == self.workspace_root)
            .map(|details| details.identity.clone())
        else {
            self.refresh_git_details(cx);
            return;
        };
        let matching = self
            .git_conflict_admission
            .as_ref()
            .and_then(|guard| guard.identity())
            == Some(&identity);
        if !matching {
            if let Some(previous) = self.git_conflict_admission.take()
                && let Some(previous_identity) = previous.identity().cloned()
            {
                self.git_ui
                    .recovery_guards
                    .insert(previous_identity, previous);
            }
            self.git_conflict_admission = self.git_ui.recovery_guards.remove(&identity);
        }
        let Some(admission) = self.git_conflict_admission.as_ref() else {
            self.arm_git_recovery(cx);
            self.status = self.git_label(GitMsg::Recovery).into();
            self.sidebar_visible = true;
            self.set_sidebar_tab(SidebarTab::Sync, cx);
            return;
        };
        let Some(identity) = admission.identity().cloned() else {
            return;
        };
        let operation_id = admission.operation_id().to_string();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        self.git_ui.conflict_busy = true;
        self.sidebar_visible = true;
        self.set_sidebar_tab(SidebarTab::Sync, cx);
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    prepare_conflict_view(identity, operation_id, executable)
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.conflict_busy = false;
                match result {
                    Ok(view) => app.git_ui.conflict = Some(view),
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn open_git_draft(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self
            .git_ui
            .conflict
            .as_ref()
            .and_then(|view| view.draft_path.clone())
        else {
            return;
        };
        if let Some(index) = self
            .tabs
            .iter()
            .position(|tab| tab.path() == Some(path.as_path()))
        {
            self.switch_active_tab(index, cx);
            return;
        }
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { MarkdownDocument::open(&path) })
                .await;
            let _ = this.update(cx, |app, cx| {
                match result {
                    Ok(document) => {
                        app.open_in_new_tab(document, cx);
                        app.view_mode = ViewMode::Edit;
                        // A resolution draft does not change the connected workspace.
                        app.sidebar_visible = true;
                        app.sidebar_tab = SidebarTab::Sync;
                    }
                    Err(error) => {
                        app.status = app.trf(Msg::StatusGitSyncFailed, &[&error.to_string()])
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn choose_git_draft(&mut self, side: Option<ConflictSide>, cx: &mut Context<Self>) {
        if self.git_ui.conflict_busy {
            return;
        }
        let Some(view) = &self.git_ui.conflict else {
            return;
        };
        let Some(path) = view.draft_path.clone() else {
            return;
        };
        // Do not replace an edited result through a source-choice button.
        if self
            .tabs
            .iter()
            .any(|tab| tab.path() == Some(path.as_path()) && tab.is_dirty())
        {
            self.status = self.git_label(GitMsg::UnsavedResult).into();
            cx.notify();
            return;
        }
        let source = |index: usize| {
            view.sources[index]
                .as_ref()
                .map(|source| source.bytes.clone())
                .unwrap_or_default()
        };
        let bytes = match side {
            Some(ConflictSide::Base) => source(0),
            Some(ConflictSide::ThisComputer) => source(1),
            Some(ConflictSide::Remote) => source(2),
            None => {
                let mut bytes = source(1);
                bytes.extend(source(2));
                bytes
            }
        };
        let Ok(text) = String::from_utf8(bytes.clone()) else {
            return;
        };
        let manager = view.manager.clone();
        let session = view.session.clone();
        let relative = session.files[0].path.clone();
        self.git_ui.conflict_busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    manager
                        .save_draft(&session, relative, bytes, &CancellationToken::new())
                        .map_err(|error| error.to_string())?;
                    let known = read_document_source(&path)
                        .ok()
                        .map(|(_, identity)| identity);
                    save_text_snapshot(&path, known.as_ref(), &text)
                        .map_err(|error| error.to_string())?;
                    Ok::<_, String>(path)
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.conflict_busy = false;
                match result {
                    Ok(path) => {
                        if let Ok((text, identity)) = read_document_source(&path) {
                            for tab in &mut app.tabs {
                                if tab.path() == Some(path.as_path()) && !tab.is_dirty() {
                                    let instance = tab.document.instance_id();
                                    let version = tab.document.version();
                                    let _ = tab.document.apply_external_reload_checked(
                                        instance,
                                        version,
                                        text.clone(),
                                        identity.clone(),
                                    );
                                }
                            }
                        }
                        app.status = app.git_label(GitMsg::DraftSaved).into();
                        app.open_git_draft(cx);
                    }
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn save_git_resolution(&mut self, resolve: bool, cx: &mut Context<Self>) {
        if self.git_ui.conflict_busy {
            return;
        }
        let Some(view) = &self.git_ui.conflict else {
            return;
        };
        let Some(draft_path) = view.draft_path.clone() else {
            return;
        };
        let manager = view.manager.clone();
        let session = view.session.clone();
        let relative = session.files[0].path.clone();
        let buffer = self
            .tabs
            .iter()
            .find(|tab| tab.path() == Some(draft_path.as_path()))
            .map(|tab| {
                (
                    tab.document.text().to_string(),
                    tab.document.disk_identity().cloned(),
                    tab.document.version(),
                )
            });
        self.git_ui.conflict_busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    let (text, known, version) = match buffer {
                        Some(buffer) => buffer,
                        None => {
                            let (text, known) = read_document_source(&draft_path)
                                .map_err(|error| error.to_string())?;
                            (text, Some(known), 0)
                        }
                    };
                    save_text_snapshot(&draft_path, known.as_ref(), &text)
                        .map_err(|error| error.to_string())?;
                    manager
                        .save_draft(
                            &session,
                            relative.clone(),
                            text.as_bytes().to_vec(),
                            &CancellationToken::new(),
                        )
                        .map_err(|error| error.to_string())?;
                    if resolve {
                        manager
                            .mark_resolved(
                                &session,
                                &relative,
                                ConflictResolution::Content(text.into_bytes()),
                                &CancellationToken::new(),
                            )
                            .map_err(|error| error.to_string())?;
                    }
                    let (_, identity) =
                        read_document_source(&draft_path).map_err(|error| error.to_string())?;
                    Ok::<_, String>((draft_path, identity, version))
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.conflict_busy = false;
                match result {
                    Ok((path, identity, version)) => {
                        for tab in &mut app.tabs {
                            if tab.path() == Some(path.as_path())
                                && tab.document.version() == version
                            {
                                tab.document.record_disk_identity(identity.clone(), true);
                            }
                        }
                        app.status = app.git_label(GitMsg::DraftSaved).into();
                        if resolve {
                            app.reload_git_conflict(cx);
                        }
                    }
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn choose_git_resolution(
        &mut self,
        resolution: ConflictResolution,
        cx: &mut Context<Self>,
    ) {
        if self.git_ui.conflict_busy {
            return;
        }
        let Some(view) = &self.git_ui.conflict else {
            return;
        };
        let Some(file) = view.session.files.first() else {
            return;
        };
        let path = file.path.clone();
        let manager = view.manager.clone();
        let session = view.session.clone();
        self.git_ui.conflict_busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    manager.mark_resolved(&session, &path, resolution, &CancellationToken::new())
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.conflict_busy = false;
                match result {
                    Ok(()) => app.reload_git_conflict(cx),
                    Err(error) => {
                        app.status = app.trf(Msg::StatusGitSyncFailed, &[&error.to_string()])
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn reload_git_conflict(&mut self, cx: &mut Context<Self>) {
        let Some(view) = &self.git_ui.conflict else {
            return;
        };
        let identity = view.identity.clone();
        let operation_id = view.session.operation_id.clone();
        let executable = self
            .git_preferences
            .executable
            .clone()
            .unwrap_or_else(|| "git".into());
        self.git_ui.conflict_busy = true;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    prepare_conflict_view(identity, operation_id, executable)
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.git_ui.conflict_busy = false;
                match result {
                    Ok(view) => app.git_ui.conflict = Some(view),
                    Err(error) => app.status = app.trf(Msg::StatusGitSyncFailed, &[&error]),
                }
                app.refresh_git_details(cx);
                cx.notify();
            });
        })
        .detach();
    }
}

fn prepare_conflict_view(
    identity: RepositoryIdentity,
    operation_id: String,
    executable: String,
) -> Result<ConflictView, String> {
    let repository =
        GitRepository::discover(GitCommandRunner::new(executable), &identity.worktree_root)
            .map_err(|error| error.to_string())?;
    if repository.identity() != &identity {
        return Err("repository identity changed".into());
    }
    let manager = ConflictManager::new(repository, git_panel::journal());
    let session = manager
        .restore_session(&operation_id, &CancellationToken::new())
        .map_err(|error| error.to_string())?;
    let mut view = ConflictView {
        identity,
        manager,
        session,
        sources: [None, None, None],
        draft_path: None,
        source_paths: [None, None, None],
        hunks: Vec::new(),
    };
    let Some(file) = view.session.files.first() else {
        return Ok(view);
    };
    let mut safe_id = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    file.path.hash(&mut safe_id);
    let folder = markion::default_git_sync_data_dir()
        .join("draft-editors")
        .join(&operation_id);
    fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    for (index, side) in [
        ConflictSide::Base,
        ConflictSide::ThisComputer,
        ConflictSide::Remote,
    ]
    .into_iter()
    .enumerate()
    {
        let exists = match side {
            ConflictSide::Base => file.base.is_some(),
            ConflictSide::ThisComputer => file.local.is_some(),
            ConflictSide::Remote => file.remote.is_some(),
        };
        if !exists {
            continue;
        }
        let source = view
            .manager
            .source(file, side, 2 * 1024 * 1024, &CancellationToken::new())
            .map_err(|error| error.to_string())?;
        if !source.truncated && image_extension_supported(&file.path) {
            let path = folder.join(format!(
                "{}-{}.{}",
                safe_id.finish(),
                index,
                file.path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .unwrap_or("bin")
            ));
            write_new_or_identical(&path, &source.bytes)?;
            view.source_paths[index] = Some(path);
        }
        view.sources[index] = Some(source);
    }
    let editable = view.sources.iter().flatten().all(|source| {
        !source.binary && !source.truncated && std::str::from_utf8(&source.bytes).is_ok()
    });
    if editable {
        let draft_path = folder.join(format!("{}-result.md", safe_id.finish()));
        if !draft_path.exists() {
            let checkpoint = git_panel::journal()
                .load()
                .map_err(|error| error.to_string())?
                .active
                .into_iter()
                .find(|checkpoint| checkpoint.operation_id == operation_id)
                .ok_or("missing conflict checkpoint")?;
            let worktree_path = view.identity.worktree_root.join(&file.path);
            let current_merge = fs::metadata(&worktree_path)
                .ok()
                .filter(|metadata| metadata.len() <= 2 * 1024 * 1024)
                .and_then(|_| fs::read(&worktree_path).ok())
                .filter(|bytes| std::str::from_utf8(bytes).is_ok());
            let content = checkpoint
                .drafts
                .iter()
                .find(|draft| draft.relative_path == file.path)
                .map(|draft| draft.content.clone())
                .or(current_merge)
                .unwrap_or_else(|| {
                    view.sources[1]
                        .as_ref()
                        .or(view.sources[2].as_ref())
                        .map(|source| source.bytes.clone())
                        .unwrap_or_default()
                });
            write_new_or_identical(&draft_path, &content)?;
        }
        let authored_markers = view.sources.iter().flatten().any(|source| {
            String::from_utf8_lossy(&source.bytes).lines().any(|line| {
                line.starts_with("<<<<<<<")
                    || line.starts_with(">>>>>>>")
                    || line.starts_with("|||||||")
                    || line == "======="
            })
        });
        if !authored_markers {
            let text = fs::read_to_string(&draft_path).map_err(|error| error.to_string())?;
            view.hunks = text_hunks(&text);
        }
        view.draft_path = Some(draft_path);
    }
    Ok(view)
}

fn write_new_or_identical(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => file
            .write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| error.to_string()),
        Err(error)
            if error.kind() == io::ErrorKind::AlreadyExists
                && fs::read(path).is_ok_and(|existing| existing == bytes) =>
        {
            Ok(())
        }
        Err(error) => Err(error.to_string()),
    }
}

pub(super) fn conflict_panel(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> impl IntoElement {
    let view = app.git_ui.conflict.as_ref().expect("conflict visible");
    let palette = app.palette();
    let enabled = !app.git_ui.conflict_busy;
    let mut body = div()
        .id("git-conflict-scroll")
        .overflow_y_scroll()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap_2()
        .child(app.tr(Msg::ItemGitResolveConflict))
        .child(view.identity.worktree_root.display().to_string());
    if let Some(file) = view.session.files.first() {
        body = body.child(file.path.display().to_string());
        if !file.supports_in_app_resolution() {
            return body
                .child(app.git_label(GitMsg::ExternalRecovery))
                .child(button(
                    "git-conflict-details",
                    app.git_label(GitMsg::Details),
                    enabled,
                    palette,
                    cx,
                    |app, _, cx| {
                        app.git_ui.conflict = None;
                        app.refresh_git_details(cx);
                        cx.notify();
                    },
                ));
        }
        let mut sources = div()
            .id("git-conflict-sources")
            .overflow_y_scroll()
            .min_h_0()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2();
        for (index, label) in [
            app.git_label(GitMsg::Base),
            app.tr(Msg::DialogButtonThisComputer),
            app.tr(Msg::DialogButtonRemoteVersion),
        ]
        .into_iter()
        .enumerate()
        {
            let mut source_view = div()
                .p_2()
                .border_1()
                .border_color(palette.border)
                .child(label);
            if let Some(source) = &view.sources[index] {
                source_view =
                    source_view.child(format!("{} · {} B", source.oid.short(), source.bytes.len()));
                if let Some(path) = &view.source_paths[index] {
                    source_view = source_view.child(img(path.clone()).max_w_full().max_h(px(180.)));
                } else if !source.binary && !source.truncated {
                    let text = String::from_utf8_lossy(&source.bytes);
                    let mut end = text.len().min(4096);
                    while !text.is_char_boundary(end) {
                        end -= 1;
                    }
                    source_view =
                        source_view.child(div().text_size(px(11.)).child(text[..end].to_string()));
                } else {
                    source_view = source_view.child(app.git_label(GitMsg::Limited));
                }
            } else {
                source_view = source_view.child("—");
            }
            sources = sources.child(source_view);
        }
        body = body.child(sources);
        for (index, hunk) in view.hunks.iter().take(20).enumerate() {
            body = body.child(
                div()
                    .p_1()
                    .border_1()
                    .border_color(palette.border)
                    .child(format!("@@ {} @@", hunk.range.start))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_1()
                            .child(button(
                                ("git-hunk-local", index),
                                app.tr(Msg::DialogButtonThisComputer),
                                enabled,
                                palette,
                                cx,
                                move |app, _, cx| {
                                    app.choose_git_hunk(index, Some(ConflictSide::ThisComputer), cx)
                                },
                            ))
                            .child(button(
                                ("git-hunk-remote", index),
                                app.tr(Msg::DialogButtonRemoteVersion),
                                enabled,
                                palette,
                                cx,
                                move |app, _, cx| {
                                    app.choose_git_hunk(index, Some(ConflictSide::Remote), cx)
                                },
                            ))
                            .child(button(
                                ("git-hunk-both", index),
                                app.git_label(GitMsg::KeepBoth),
                                enabled,
                                palette,
                                cx,
                                move |app, _, cx| app.choose_git_hunk(index, None, cx),
                            )),
                    ),
            );
        }
        if view.draft_path.is_some() {
            body = body.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .child(button(
                        "git-draft-local",
                        app.tr(Msg::DialogButtonThisComputer),
                        enabled,
                        palette,
                        cx,
                        |app, _, cx| app.choose_git_draft(Some(ConflictSide::ThisComputer), cx),
                    ))
                    .child(button(
                        "git-draft-remote",
                        app.tr(Msg::DialogButtonRemoteVersion),
                        enabled,
                        palette,
                        cx,
                        |app, _, cx| app.choose_git_draft(Some(ConflictSide::Remote), cx),
                    ))
                    .child(button(
                        "git-draft-both",
                        app.git_label(GitMsg::KeepBoth),
                        enabled,
                        palette,
                        cx,
                        |app, _, cx| app.choose_git_draft(None, cx),
                    ))
                    .child(button(
                        "git-edit-result",
                        app.git_label(GitMsg::EditResult),
                        enabled,
                        palette,
                        cx,
                        |app, _, cx| app.open_git_draft(cx),
                    ))
                    .child(button(
                        "git-save-draft",
                        app.git_label(GitMsg::Draft),
                        enabled,
                        palette,
                        cx,
                        |app, _, cx| app.save_git_resolution(false, cx),
                    ))
                    .child(button(
                        "git-mark-resolved",
                        app.git_label(GitMsg::Resolved),
                        enabled,
                        palette,
                        cx,
                        |app, _, cx| app.save_git_resolution(true, cx),
                    )),
            );
        } else {
            let alternate = file.path.with_file_name(format!(
                "remote-{}",
                file.path.file_name().unwrap_or_default().to_string_lossy()
            ));
            body = body.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .child(button(
                        "git-binary-local",
                        app.tr(Msg::DialogButtonThisComputer),
                        enabled && file.local.is_some(),
                        palette,
                        cx,
                        |app, _, cx| {
                            app.choose_git_resolution(
                                ConflictResolution::Choose(ConflictSide::ThisComputer),
                                cx,
                            )
                        },
                    ))
                    .child(button(
                        "git-binary-remote",
                        app.tr(Msg::DialogButtonRemoteVersion),
                        enabled && file.remote.is_some(),
                        palette,
                        cx,
                        |app, _, cx| {
                            app.choose_git_resolution(
                                ConflictResolution::Choose(ConflictSide::Remote),
                                cx,
                            )
                        },
                    ))
                    .child(button(
                        "git-binary-both",
                        app.git_label(GitMsg::KeepBoth),
                        enabled && file.local.is_some() && file.remote.is_some(),
                        palette,
                        cx,
                        move |app, _, cx| {
                            app.choose_git_resolution(
                                ConflictResolution::KeepBoth {
                                    primary: ConflictSide::ThisComputer,
                                    alternate: ConflictSide::Remote,
                                    alternate_path: alternate.clone(),
                                },
                                cx,
                            )
                        },
                    )),
            );
        }
        if file.local.is_none() || file.remote.is_none() {
            body = body.child(button(
                "git-delete-resolution",
                app.tr(Msg::DialogButtonDelete),
                enabled,
                palette,
                cx,
                |app, _, cx| app.choose_git_resolution(ConflictResolution::Delete, cx),
            ));
        }
    }
    body = body.child(button(
        "git-conflict-details",
        app.git_label(GitMsg::Details),
        enabled,
        palette,
        cx,
        |app, _, cx| {
            app.git_ui.conflict = None;
            app.refresh_git_details(cx);
            cx.notify();
        },
    ));
    body.child(button(
        "git-finish-or-abort",
        app.tr(Msg::DialogButtonFinishMerge),
        enabled,
        palette,
        cx,
        |app, window, cx| {
            app.git_ui.conflict = None;
            app.finish_git_conflict(&ResolveGitConflict, window, cx);
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hunk_choices_preserve_surrounding_text_and_crlf() {
        let text = "intro\r\n<<<<<<< HEAD\r\nlocal\r\n||||||| base\r\nold\r\n=======\r\nremote\r\n>>>>>>> incoming\r\nend\r\n";
        let chunks = text_hunks(text);
        assert_eq!(chunks.len(), 1);
        let mut result = text.to_string();
        result.replace_range(chunks[0].range.clone(), &chunks[0].remote);
        assert_eq!(result, "intro\r\nremote\r\nend\r\n");
    }
    #[test]
    fn incomplete_or_nested_markers_are_not_actionable_hunks() {
        assert!(text_hunks("<<<<<<< HEAD\na\n").is_empty());
        assert!(
            text_hunks("<<<<<<< HEAD\n<<<<<<< literal\n=======\nb\n>>>>>>> incoming\n").is_empty()
        );
    }
}
