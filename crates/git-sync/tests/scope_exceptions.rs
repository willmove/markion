//! Held-back scope exceptions never block approved notes from syncing.

mod support;

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use markion_git_sync::{
    CancellationToken, GitCommandRunner, GitRepository, GitSyncEngine, HeldBackReason,
    JournalStore, NewFileClass, NewFileRule, OperationPhase, RepositoryPolicy, ScopeException,
    ScopeFacts, SyncEngineError, SyncOptions, SyncOutcome, SyncPlan,
};
use support::TwoCloneFixture;

fn notes_policy(repository: &GitRepository, tracked_roots: Vec<PathBuf>) -> RepositoryPolicy {
    RepositoryPolicy {
        identity: repository.identity().clone(),
        target: repository.resolve_sync_target().unwrap(),
        tracked_roots,
        new_file_rules: vec![NewFileRule {
            root: "".into(),
            classes: BTreeSet::from([
                NewFileClass::Markdown,
                NewFileClass::Text,
                NewFileClass::Image,
            ]),
        }],
        message_template: String::new(),
        include_device_name: false,
        background_fetch: false,
        last_confirmed_remote: None,
        acknowledged_resource_omissions: Vec::new(),
        allowed_extensions: Default::default(),
        included_once: Vec::new(),
    }
}

struct Operation {
    engine: GitSyncEngine,
    policy: RepositoryPolicy,
    plan: SyncPlan,
    _data: tempfile::TempDir,
}

/// Plan exactly what the policy partition approves, as the app does.
fn operation(worktree: &Path, id: &str, policy: Option<RepositoryPolicy>) -> Operation {
    let repository = GitRepository::discover(GitCommandRunner::new("git"), worktree).unwrap();
    let policy = policy.unwrap_or_else(|| notes_policy(&repository, vec!["".into()]));
    let state = repository.status().unwrap();
    let paths = policy
        .partition(&state.worktree.changes, &repository)
        .automatic;
    let plan = SyncPlan {
        operation_id: id.into(),
        identity: repository.identity().clone(),
        target: policy.target.clone(),
        expected_head: state.head,
        content_fingerprints: paths
            .iter()
            .map(|path| (path.clone(), "captured".into()))
            .collect(),
        paths,
        message: "Sync notes".into(),
    };
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    Operation {
        engine: GitSyncEngine::new(repository, journal, SyncOptions::default()),
        policy,
        plan,
        _data: data,
    }
}

impl Operation {
    fn sync_now(&mut self) -> Result<SyncOutcome, SyncEngineError> {
        self.engine.sync_now(
            &self.plan,
            &mut self.policy,
            None,
            None,
            &CancellationToken::new(),
        )
    }
}

fn remote_tree(fixture: &TwoCloneFixture) -> Vec<String> {
    let output = std::process::Command::new("git")
        .current_dir(&fixture.remote)
        .args(["ls-tree", "-r", "--name-only", "main"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

fn push_from_second(fixture: &TwoCloneFixture, name: &str) {
    fs::write(fixture.second.join(name), "from second\n").unwrap();
    support::git(&fixture.second, ["add", "--", name]);
    support::git(&fixture.second, ["commit", "-q", "-m", "second device"]);
    support::git(&fixture.second, ["push", "-q", "origin", "main"]);
}

#[test]
fn approved_notes_sync_while_exceptions_stay_untracked() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("notes.md"), "edited\n").unwrap();
    fs::write(fixture.first.join("second.md"), "new note\n").unwrap();
    fs::write(fixture.first.join("report.pdf"), "pdf").unwrap();
    fs::create_dir(fixture.first.join(".hidden")).unwrap();
    fs::write(fixture.first.join(".hidden/config"), "secret").unwrap();
    fs::write(fixture.first.join(".DS_Store"), "junk").unwrap();

    let mut operation = operation(&fixture.first, "held-back-sync", None);
    let SyncOutcome::Synchronized {
        held_back,
        pending_local_changes,
        ..
    } = operation.sync_now().unwrap()
    else {
        panic!("expected synchronized outcome");
    };
    assert_eq!((held_back, pending_local_changes), (2, 0));
    let tree = remote_tree(&fixture);
    assert!(tree.contains(&"second.md".to_string()));
    assert!(!tree.iter().any(|path| path.ends_with(".pdf")
        || path.contains(".hidden")
        || path.contains("DS_Store")));
    assert!(fixture.first.join("report.pdf").is_file());
}

#[test]
fn commit_locally_skips_held_back_files() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("notes.md"), "edited\n").unwrap();
    fs::write(fixture.first.join("report.pdf"), "pdf").unwrap();
    let operation = operation(&fixture.first, "held-back-local", None);
    assert!(matches!(
        operation
            .engine
            .commit_locally(
                &operation.plan,
                &operation.policy,
                None,
                &CancellationToken::new()
            )
            .unwrap(),
        SyncOutcome::CommittedLocally { .. }
    ));
    let status = operation.engine.repository().status().unwrap();
    assert_eq!(status.worktree.changes.len(), 1);
    assert_eq!(status.worktree.changes[0].path, PathBuf::from("report.pdf"));
}

#[test]
fn remote_updates_integrate_around_a_held_back_file() {
    let fixture = TwoCloneFixture::new();
    push_from_second(&fixture, "remote.md");
    fs::write(fixture.first.join("local.md"), "local\n").unwrap();
    fs::write(fixture.first.join("report.pdf"), "pdf").unwrap();
    let mut operation = operation(&fixture.first, "held-back-integrate", None);
    assert!(matches!(
        operation.sync_now().unwrap(),
        SyncOutcome::Synchronized { held_back: 1, .. }
    ));
    assert!(fixture.first.join("remote.md").is_file());
    assert!(fixture.first.join("report.pdf").is_file());
    let tree = remote_tree(&fixture);
    assert!(tree.contains(&"local.md".to_string()) && tree.contains(&"remote.md".to_string()));
}

#[test]
fn held_back_tracked_change_pauses_integration_and_keeps_the_local_commit() {
    let fixture = TwoCloneFixture::new();
    fs::create_dir(fixture.first.join("notes")).unwrap();
    fs::write(fixture.first.join("notes/a.md"), "a\n").unwrap();
    support::git(&fixture.first, ["add", "--", "notes/a.md"]);
    support::git(&fixture.first, ["commit", "-q", "-m", "notes folder"]);
    support::git(&fixture.first, ["push", "-q", "origin", "main"]);
    support::git(&fixture.second, ["pull", "-q", "origin", "main"]);
    push_from_second(&fixture, "remote.md");

    fs::write(fixture.first.join("notes/a.md"), "edited\n").unwrap();
    // `notes.md` is tracked but outside the approved tracked root.
    fs::write(fixture.first.join("notes.md"), "outside scope\n").unwrap();
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    let policy = notes_policy(&repository, vec!["notes".into()]);
    let mut operation = operation(&fixture.first, "held-back-tracked", Some(policy));
    assert_eq!(operation.plan.paths, [PathBuf::from("notes/a.md")]);
    let before = operation.plan.expected_head.clone();
    match operation.sync_now() {
        Err(SyncEngineError::HeldBackTrackedChanges { paths }) => {
            assert_eq!(paths, [PathBuf::from("notes.md")]);
        }
        other => panic!("expected held-back tracked pause, got {other:?}"),
    }
    let journal = JournalStore::new(
        operation._data.path().join("journal.toml"),
        operation._data.path().join("recovery"),
    )
    .load()
    .unwrap();
    assert_eq!(
        journal.active[0].phase,
        OperationPhase::NeedsAttention,
        "a paused, not interrupted, integration"
    );
    let status = operation.engine.repository().status().unwrap();
    assert_ne!(status.head, before, "approved change committed locally");
    assert!(
        !fixture.first.join("remote.md").exists(),
        "worktree not integrated"
    );
    assert_eq!(
        fs::read_to_string(fixture.first.join("notes.md")).unwrap(),
        "outside scope\n"
    );
}

#[test]
fn incoming_path_colliding_with_a_held_back_file_is_refused() {
    let fixture = TwoCloneFixture::new();
    push_from_second(&fixture, "report.pdf");
    fs::write(fixture.first.join("report.pdf"), "local pdf").unwrap();
    let mut operation = operation(&fixture.first, "held-back-collision", None);
    assert!(operation.plan.paths.is_empty());
    assert!(matches!(
        operation.sync_now(),
        Err(SyncEngineError::UnsafeIncomingPaths)
    ));
    assert_eq!(
        fs::read_to_string(fixture.first.join("report.pdf")).unwrap(),
        "local pdf"
    );
}

#[test]
fn a_note_created_during_fetch_still_pauses_integration() {
    let fixture = TwoCloneFixture::new();
    push_from_second(&fixture, "remote.md");
    fs::write(fixture.first.join("report.pdf"), "pdf").unwrap();
    let mut operation = operation(&fixture.first, "edit-during-fetch", None);
    let created = fixture.first.join("typed-during-fetch.md");
    operation.engine = operation.engine.clone().with_observer(move |phase| {
        if phase == OperationPhase::Fetching {
            let _ = fs::write(&created, "new\n");
        }
        Ok(())
    });
    assert!(matches!(
        operation.sync_now(),
        Err(SyncEngineError::UnsafeRepositoryState)
    ));
    assert!(!fixture.first.join("remote.md").exists());
}

#[test]
fn included_once_file_is_committed_and_its_approval_pruned() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("data.bin"), "bytes").unwrap();
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    let mut policy = notes_policy(&repository, vec!["".into()]);
    policy.included_once.push(ScopeException {
        path: PathBuf::from("data.bin"),
        content_fingerprint: repository.fingerprint(Path::new("data.bin")).unwrap(),
    });
    let mut operation = operation(&fixture.first, "include-once", Some(policy));
    assert_eq!(operation.plan.paths, [PathBuf::from("data.bin")]);
    assert!(matches!(
        operation.sync_now().unwrap(),
        SyncOutcome::Synchronized { held_back: 0, .. }
    ));
    assert!(remote_tree(&fixture).contains(&"data.bin".to_string()));

    let repository = operation.engine.repository();
    let paths: Vec<PathBuf> = operation
        .policy
        .included_once
        .iter()
        .map(|record| record.path.clone())
        .collect();
    let tracked = repository.tracked_among(&paths).unwrap();
    operation
        .policy
        .prune_included_once(|path| !tracked.contains(path));
    assert!(operation.policy.included_once.is_empty());
}

#[test]
fn moved_tracked_attachment_into_a_hidden_folder_is_held_back_whole() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("photo.pdf"), "pdf bytes").unwrap();
    support::git(&fixture.first, ["add", "--", "photo.pdf"]);
    support::git(&fixture.first, ["commit", "-q", "-m", "attachment"]);
    fs::create_dir(fixture.first.join(".archive")).unwrap();
    fs::rename(
        fixture.first.join("photo.pdf"),
        fixture.first.join(".archive/photo.pdf"),
    )
    .unwrap();
    fs::write(fixture.first.join("notes.md"), "edited\n").unwrap();
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    let policy = notes_policy(&repository, vec!["".into()]);
    let partition = policy.partition(&repository.status().unwrap().worktree.changes, &repository);
    assert_eq!(partition.automatic, [PathBuf::from("notes.md")]);
    let deletion = partition
        .held_back
        .iter()
        .find(|held| held.path == Path::new("photo.pdf"))
        .unwrap();
    assert_eq!(
        deletion.reason,
        HeldBackReason::PairedMove {
            with: Some(PathBuf::from(".archive/photo.pdf"))
        }
    );
}
