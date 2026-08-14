use super::*;

const GIT_BRANCH_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StatusBarContext {
    pub(super) characters: usize,
    pub(super) words: usize,
    pub(super) caret: Option<(usize, usize)>,
    pub(super) branch: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LocalizedStatusBarContext {
    pub(super) characters: String,
    pub(super) words: String,
    pub(super) caret: Option<String>,
    pub(super) branch: Option<String>,
}

impl StatusBarContext {
    pub(super) fn localized(&self, language: Language) -> LocalizedStatusBarContext {
        let characters = self.characters.to_string();
        let words = self.words.to_string();
        let caret = self.caret.map(|(line, column)| {
            let line = line.to_string();
            let column = column.to_string();
            tf(language, Msg::StatusContextLineColumn, &[&line, &column])
        });
        let branch = self
            .branch
            .as_deref()
            .map(|branch| tf(language, Msg::StatusContextBranch, &[branch]));

        LocalizedStatusBarContext {
            characters: tf(language, Msg::StatusContextCharacters, &[&characters]),
            words: tf(language, Msg::StatusContextWords, &[&words]),
            caret,
            branch,
        }
    }
}

pub(super) fn status_bar_context(
    tab: &EditorTab,
    view_mode: ViewMode,
    branch: Option<&str>,
) -> StatusBarContext {
    if tab.is_image() {
        return StatusBarContext {
            characters: 0,
            words: 0,
            caret: None,
            branch: branch.map(str::to_owned),
        };
    }
    let stats = tab.document.stats();
    let caret = (!matches!(view_mode, ViewMode::Read))
        .then(|| tab.document.line_column_at(tab.cursor_offset()));

    StatusBarContext {
        characters: stats.chars,
        words: stats.words,
        caret,
        branch: branch.map(str::to_owned),
    }
}

pub(super) fn status_bar_feedback(
    title: &str,
    is_dirty: bool,
    save_state: &str,
    status: &str,
) -> String {
    let dirty_marker = if is_dirty { " *" } else { "" };
    format!("Markion - {title}{dirty_marker} | {save_state} | {status}")
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct GitBranchState {
    pub(super) context: Option<PathBuf>,
    pub(super) head_path: Option<PathBuf>,
    pub(super) branch: Option<String>,
    pub(super) generation: u64,
    pub(super) lookup_in_flight: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct GitBranchResolution {
    pub(super) head_path: Option<PathBuf>,
    pub(super) branch: Option<String>,
}

impl GitBranchState {
    pub(super) fn replace_context(&mut self, context: Option<PathBuf>) -> bool {
        if self.context == context {
            return false;
        }
        self.generation = self.generation.wrapping_add(1);
        self.context = context;
        self.head_path = None;
        self.branch = None;
        self.lookup_in_flight = false;
        true
    }

    pub(super) fn begin_lookup(&mut self) -> Option<(u64, PathBuf)> {
        if self.lookup_in_flight {
            return None;
        }
        let context = self.context.clone()?;
        self.generation = self.generation.wrapping_add(1);
        self.lookup_in_flight = true;
        Some((self.generation, context))
    }

    pub(super) fn accept(
        &mut self,
        generation: u64,
        context: &Path,
        resolution: GitBranchResolution,
    ) -> bool {
        if self.generation != generation || self.context.as_deref() != Some(context) {
            return false;
        }

        self.lookup_in_flight = false;
        let branch_changed = self.branch != resolution.branch;
        self.head_path = resolution.head_path;
        self.branch = resolution.branch;
        branch_changed
    }
}

pub(super) fn git_context_path(
    document_path: Option<&Path>,
    workspace_root: &Path,
    workspace_established: bool,
) -> Option<PathBuf> {
    document_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .or_else(|| workspace_established.then(|| workspace_root.to_path_buf()))
}

pub(super) fn resolve_git_branch(start: &Path) -> GitBranchResolution {
    for ancestor in start.ancestors() {
        let marker = ancestor.join(".git");
        let metadata = match fs::metadata(&marker) {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(_) => return GitBranchResolution::default(),
        };

        let git_dir = if metadata.is_dir() {
            marker
        } else if metadata.is_file() {
            let Ok(contents) = fs::read_to_string(&marker) else {
                return GitBranchResolution::default();
            };
            let Some(raw_path) = contents
                .lines()
                .next()
                .and_then(|line| line.trim().strip_prefix("gitdir:"))
                .map(str::trim)
                .filter(|path| !path.is_empty())
            else {
                return GitBranchResolution::default();
            };
            let path = PathBuf::from(raw_path);
            if path.is_absolute() {
                path
            } else {
                ancestor.join(path)
            }
        } else {
            return GitBranchResolution::default();
        };

        let git_dir = fs::canonicalize(&git_dir).unwrap_or(git_dir);
        let head_path = git_dir.join("HEAD");
        let branch = fs::read_to_string(&head_path)
            .ok()
            .and_then(|head| symbolic_branch_from_head(&head));
        return GitBranchResolution {
            head_path: Some(head_path),
            branch,
        };
    }

    GitBranchResolution::default()
}

fn symbolic_branch_from_head(head: &str) -> Option<String> {
    head.trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::trim)
        .filter(|branch| !branch.is_empty())
        .map(str::to_owned)
}

impl MarkionApp {
    pub(super) fn current_status_bar_context(&self) -> StatusBarContext {
        status_bar_context(
            self.active_tab(),
            self.view_mode,
            self.git_branch_state.branch.as_deref(),
        )
    }

    fn desired_git_context(&self) -> Option<PathBuf> {
        git_context_path(
            self.active_tab().path(),
            &self.workspace_root,
            self.file_tree.is_some(),
        )
    }

    pub(super) fn sync_git_branch_context(&mut self, cx: &mut Context<Self>) {
        let changed = self
            .git_branch_state
            .replace_context(self.desired_git_context());
        if changed {
            cx.notify();
            self.start_git_branch_lookup(cx);
        }
    }

    pub(super) fn refresh_git_branch_context(&mut self, cx: &mut Context<Self>) {
        let desired = self.desired_git_context();
        if self.git_branch_state.context != desired {
            self.sync_git_branch_context(cx);
        } else {
            self.start_git_branch_lookup(cx);
        }
    }

    fn start_git_branch_lookup(&mut self, cx: &mut Context<Self>) {
        let Some((generation, requested_context)) = self.git_branch_state.begin_lookup() else {
            return;
        };
        let lookup_context = requested_context.clone();

        cx.spawn(async move |this, cx| {
            let resolution = cx
                .background_executor()
                .spawn(async move { resolve_git_branch(&lookup_context) })
                .await;
            let _ = this.update(cx, |app, cx| {
                if app
                    .git_branch_state
                    .accept(generation, &requested_context, resolution)
                {
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(super) fn arm_git_branch_poll(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(GIT_BRANCH_REFRESH_INTERVAL).await;
                if this
                    .update(cx, |app, cx| app.refresh_git_branch_context(cx))
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
    }
}
