mod support;

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use markion_git_sync::{
    CancellationToken, ConflictKind, ConflictManager, ConflictResolution, ConflictSide,
    GitCommandRunner, GitRepository, GitSyncEngine, JournalStore, NewFileClass, NewFileRule,
    OperationPhase, RepositoryPolicy, SyncOptions, SyncOutcome, SyncPlan,
};
use support::TwoCloneFixture;

#[test]
fn alternating_clones_commit_merge_and_push_nonconflicting_notes() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("first.md"), "first device\n").unwrap();
    let (first_engine, mut first_policy, first_plan, _first_data) =
        operation(&fixture.first, "first-sync");
    assert!(matches!(
        first_engine
            .sync_now(
                &first_plan,
                &mut first_policy,
                None,
                None,
                &CancellationToken::new()
            )
            .unwrap(),
        SyncOutcome::Synchronized { .. }
    ));

    fs::write(fixture.second.join("second.md"), "second device\n").unwrap();
    let (second_engine, mut second_policy, second_plan, _second_data) =
        operation(&fixture.second, "second-sync");
    assert!(matches!(
        second_engine
            .sync_now(
                &second_plan,
                &mut second_policy,
                None,
                None,
                &CancellationToken::new()
            )
            .unwrap(),
        SyncOutcome::Synchronized { .. }
    ));
    assert!(fixture.second.join("first.md").is_file());

    support::git(&fixture.first, ["fetch", "-q", "origin", "main"]);
    let tree = git_output(
        &fixture.first,
        ["ls-tree", "-r", "--name-only", "origin/main"],
    );
    assert!(tree.lines().any(|path| path == "first.md"));
    assert!(tree.lines().any(|path| path == "second.md"));
}

#[test]
fn conflicting_two_device_edits_enter_durable_resolution_state() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("notes.md"), "from first\n").unwrap();
    let (first_engine, mut first_policy, first_plan, _first_data) =
        operation(&fixture.first, "first-conflict-sync");
    first_engine
        .sync_now(
            &first_plan,
            &mut first_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();

    fs::write(fixture.second.join("notes.md"), "from second\n").unwrap();
    let (second_engine, mut second_policy, second_plan, second_data) =
        operation(&fixture.second, "second-conflict-sync");
    let outcome = second_engine
        .sync_now(
            &second_plan,
            &mut second_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();
    assert!(matches!(
        outcome,
        SyncOutcome::NeedsAttention {
            phase: OperationPhase::Resolving,
            ..
        }
    ));
    let journal = JournalStore::new(
        second_data.path().join("journal.toml"),
        second_data.path().join("recovery"),
    )
    .load()
    .unwrap();
    assert_eq!(journal.active[0].phase, OperationPhase::Resolving);
    assert!(fixture.second.join(".git/MERGE_HEAD").is_file());

    let manager = ConflictManager::new(
        second_engine.repository().clone(),
        JournalStore::new(
            second_data.path().join("journal.toml"),
            second_data.path().join("recovery"),
        ),
    );
    let cancellation = CancellationToken::new();
    let session = manager
        .restore_session("second-conflict-sync", &cancellation)
        .unwrap();
    assert_eq!(session.files.len(), 1);
    assert!(manager.finish_merge(&session, &cancellation).is_err());
    let authored_conflict = fs::read(fixture.second.join("notes.md")).unwrap();
    fs::write(fixture.second.join("notes.md"), "external edit\n").unwrap();
    assert!(manager.abort(&session, &cancellation).is_err());
    assert!(fixture.second.join(".git/MERGE_HEAD").is_file());
    fs::write(fixture.second.join("notes.md"), authored_conflict).unwrap();
    assert_eq!(
        manager
            .source(
                &session.files[0],
                ConflictSide::ThisComputer,
                1024,
                &cancellation
            )
            .unwrap()
            .bytes,
        b"from second\n"
    );
    let result = b"literal <<<<<<< text is valid Markdown\n".to_vec();
    manager
        .save_draft(&session, "notes.md".into(), result.clone(), &cancellation)
        .unwrap();
    manager
        .mark_resolved(
            &session,
            Path::new("notes.md"),
            ConflictResolution::Content(result.clone()),
            &cancellation,
        )
        .unwrap();
    manager
        .finish_and_push(&session, &second_plan.target, &cancellation)
        .unwrap();
    assert_eq!(
        git_output(&fixture.remote, ["show", "refs/heads/main:notes.md"]),
        String::from_utf8(result).unwrap()
    );
}

#[test]
fn failed_required_signing_never_becomes_an_unsigned_success() {
    let fixture = TwoCloneFixture::new();
    let before = git_output(&fixture.first, ["rev-parse", "HEAD"]);
    fs::write(fixture.first.join("signed.md"), "must be signed\n").unwrap();
    support::git(
        &fixture.first,
        ["config", "gpg.program", "markion-definitely-missing-gpg"],
    );
    support::git(&fixture.first, ["config", "commit.gpgsign", "true"]);
    let (engine, mut policy, plan, data) = operation(&fixture.first, "signed-sync");
    assert!(
        engine
            .sync_now(&plan, &mut policy, None, None, &CancellationToken::new())
            .is_err()
    );
    assert_eq!(git_output(&fixture.first, ["rev-parse", "HEAD"]), before);
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    )
    .load()
    .unwrap();
    assert_eq!(journal.active[0].phase, OperationPhase::Committing);
}

#[test]
fn non_fast_forward_push_is_retried_once_after_remote_verification() {
    let fixture = TwoCloneFixture::new();
    install_reject_once_hook(&fixture.remote);
    fs::write(fixture.first.join("retried.md"), "retry\n").unwrap();
    let (engine, mut policy, plan, _data) = operation(&fixture.first, "retry-sync");
    assert!(matches!(
        engine
            .sync_now(&plan, &mut policy, None, None, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::Synchronized { .. }
    ));
    assert_eq!(
        git_output(&fixture.remote, ["show", "refs/heads/main:retried.md"]),
        "retry\n"
    );
}

#[test]
fn two_clone_remote_rewrite_and_deletion_are_reported_without_moving_local_head() {
    let fixture = TwoCloneFixture::new();
    let (engine, mut policy, plan, _data) = operation(&fixture.first, "target-rewritten");
    let local_head = plan.expected_head.clone().unwrap();
    policy.last_confirmed_remote = Some(local_head.clone());
    let tree = git_output(&fixture.second, ["rev-parse", "HEAD^{tree}"]);
    let rewritten = git_output(
        &fixture.second,
        ["commit-tree", tree.trim(), "-m", "rewritten root"],
    );
    support::git(
        &fixture.second,
        [
            "push",
            "-q",
            "--force",
            "origin",
            &format!("{}:refs/heads/main", rewritten.trim()),
        ],
    );
    assert!(matches!(
        engine
            .check_remote(&plan, &policy, None, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::RemoteChecked {
            relation: markion_git_sync::HistoryRelation::Rewritten
        }
    ));
    assert_eq!(
        git_output(&fixture.first, ["rev-parse", "HEAD"]).trim(),
        local_head.as_str()
    );

    let fixture = TwoCloneFixture::new();
    let (engine, mut policy, plan, _data) = operation(&fixture.first, "target-deleted");
    let local_head = plan.expected_head.clone().unwrap();
    policy.last_confirmed_remote = Some(local_head.clone());
    support::git(
        &fixture.remote,
        ["config", "receive.denyDeleteCurrent", "ignore"],
    );
    support::git(
        &fixture.second,
        ["push", "-q", "origin", ":refs/heads/main"],
    );
    assert!(matches!(
        engine
            .check_remote(&plan, &policy, None, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::RemoteChecked {
            relation: markion_git_sync::HistoryRelation::DeletedTarget
        }
    ));
    assert_eq!(
        git_output(&fixture.first, ["rev-parse", "HEAD"]).trim(),
        local_head.as_str()
    );
}

#[test]
fn delete_modify_and_add_add_require_explicit_conflict_choices() {
    let fixture = TwoCloneFixture::new();
    fs::remove_file(fixture.first.join("notes.md")).unwrap();
    let (first_engine, mut first_policy, first_plan, _data) =
        operation(&fixture.first, "delete-first");
    first_engine
        .sync_now(
            &first_plan,
            &mut first_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();
    fs::write(fixture.second.join("notes.md"), "kept locally\n").unwrap();
    let (second_engine, mut second_policy, second_plan, second_data) =
        operation(&fixture.second, "delete-modify");
    assert!(matches!(
        second_engine
            .sync_now(
                &second_plan,
                &mut second_policy,
                None,
                None,
                &CancellationToken::new(),
            )
            .unwrap(),
        SyncOutcome::NeedsAttention {
            phase: OperationPhase::Resolving,
            ..
        }
    ));
    let manager = conflict_manager(&second_engine, &second_data);
    let cancellation = CancellationToken::new();
    let session = manager
        .restore_session("delete-modify", &cancellation)
        .unwrap();
    assert_eq!(session.files[0].kind, ConflictKind::DeletedByRemote);
    manager
        .mark_resolved(
            &session,
            Path::new("notes.md"),
            ConflictResolution::Choose(ConflictSide::ThisComputer),
            &cancellation,
        )
        .unwrap();
    manager.abort(&session, &cancellation).unwrap();

    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("same.md"), "first add\n").unwrap();
    let (first_engine, mut first_policy, first_plan, _data) =
        operation(&fixture.first, "add-first");
    first_engine
        .sync_now(
            &first_plan,
            &mut first_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();
    fs::write(fixture.second.join("same.md"), "second add\n").unwrap();
    let (second_engine, mut second_policy, second_plan, second_data) =
        operation(&fixture.second, "add-add");
    second_engine
        .sync_now(
            &second_plan,
            &mut second_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();
    let manager = conflict_manager(&second_engine, &second_data);
    let session = manager.restore_session("add-add", &cancellation).unwrap();
    assert_eq!(session.files[0].kind, ConflictKind::BothAdded);
}

#[test]
fn binary_conflict_can_keep_both_with_a_validated_new_path() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("diagram.png"), b"\x89PNG-first\0").unwrap();
    let (first_engine, mut first_policy, first_plan, _data) =
        operation(&fixture.first, "binary-first");
    first_engine
        .sync_now(
            &first_plan,
            &mut first_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();
    fs::write(fixture.second.join("diagram.png"), b"\x89PNG-second\0").unwrap();
    let (second_engine, mut second_policy, second_plan, second_data) =
        operation(&fixture.second, "binary-second");
    second_engine
        .sync_now(
            &second_plan,
            &mut second_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();
    let manager = conflict_manager(&second_engine, &second_data);
    let cancellation = CancellationToken::new();
    let session = manager
        .restore_session("binary-second", &cancellation)
        .unwrap();
    assert!(
        manager
            .source(
                &session.files[0],
                ConflictSide::ThisComputer,
                1024,
                &cancellation,
            )
            .unwrap()
            .binary
    );
    manager
        .mark_resolved(
            &session,
            Path::new("diagram.png"),
            ConflictResolution::KeepBoth {
                primary: ConflictSide::ThisComputer,
                alternate_path: "diagram-remote.png".into(),
                alternate: ConflictSide::Remote,
            },
            &cancellation,
        )
        .unwrap();
    manager
        .finish_and_push(&session, &second_plan.target, &cancellation)
        .unwrap();
    assert_eq!(
        fs::read(fixture.second.join("diagram.png")).unwrap(),
        b"\x89PNG-second\0"
    );
    assert_eq!(
        fs::read(fixture.second.join("diagram-remote.png")).unwrap(),
        b"\x89PNG-first\0"
    );
}

#[test]
fn simple_remote_rename_carries_the_other_devices_note_edit() {
    let fixture = TwoCloneFixture::new();
    fs::rename(
        fixture.first.join("notes.md"),
        fixture.first.join("renamed.md"),
    )
    .unwrap();
    let (first_engine, mut first_policy, first_plan, _data) =
        operation(&fixture.first, "rename-first");
    first_engine
        .sync_now(
            &first_plan,
            &mut first_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();

    fs::write(fixture.second.join("notes.md"), "edited on second\n").unwrap();
    let (second_engine, mut second_policy, second_plan, _data) =
        operation(&fixture.second, "rename-second");
    assert!(matches!(
        second_engine
            .sync_now(
                &second_plan,
                &mut second_policy,
                None,
                None,
                &CancellationToken::new(),
            )
            .unwrap(),
        SyncOutcome::Synchronized { .. }
    ));
    assert!(!fixture.second.join("notes.md").exists());
    assert_eq!(
        fs::read_to_string(fixture.second.join("renamed.md"))
            .unwrap()
            .replace("\r\n", "\n"),
        "edited on second\n"
    );
}

#[test]
fn rename_rename_conflict_is_preserved_for_external_resolution() {
    let fixture = TwoCloneFixture::new();
    fs::rename(
        fixture.first.join("notes.md"),
        fixture.first.join("first-name.md"),
    )
    .unwrap();
    let (first_engine, mut first_policy, first_plan, _data) =
        operation(&fixture.first, "rename-remote");
    first_engine
        .sync_now(
            &first_plan,
            &mut first_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();

    fs::rename(
        fixture.second.join("notes.md"),
        fixture.second.join("second-name.md"),
    )
    .unwrap();
    let (second_engine, mut second_policy, second_plan, second_data) =
        operation(&fixture.second, "rename-local");
    assert!(matches!(
        second_engine
            .sync_now(
                &second_plan,
                &mut second_policy,
                None,
                None,
                &CancellationToken::new(),
            )
            .unwrap(),
        SyncOutcome::NeedsAttention {
            phase: OperationPhase::Resolving,
            ..
        }
    ));
    let manager = conflict_manager(&second_engine, &second_data);
    let cancellation = CancellationToken::new();
    let session = manager
        .restore_session("rename-local", &cancellation)
        .unwrap();
    let unsupported = session
        .files
        .iter()
        .find(|file| !file.supports_in_app_resolution())
        .expect("rename/rename must retain an unsupported index shape");
    let before = git_output(&fixture.second, ["ls-files", "-u"]);
    assert!(
        manager
            .mark_resolved(
                &session,
                &unsupported.path,
                ConflictResolution::Delete,
                &cancellation,
            )
            .is_err()
    );
    assert_eq!(git_output(&fixture.second, ["ls-files", "-u"]), before);
    assert!(fixture.second.join(".git/MERGE_HEAD").is_file());
}

fn conflict_manager(engine: &GitSyncEngine, data: &tempfile::TempDir) -> ConflictManager {
    ConflictManager::new(
        engine.repository().clone(),
        JournalStore::new(
            data.path().join("journal.toml"),
            data.path().join("recovery"),
        ),
    )
}

fn install_reject_once_hook(remote: &Path) {
    let hook = remote.join("hooks/pre-receive");
    fs::write(
        &hook,
        "#!/bin/sh\nmarker=\"$GIT_DIR/markion-rejected-once\"\nif test ! -f \"$marker\"; then\n  : > \"$marker\"\n  echo '[rejected] non-fast-forward race' >&2\n  exit 1\nfi\nexit 0\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn operation(
    worktree: &Path,
    operation_id: &str,
) -> (GitSyncEngine, RepositoryPolicy, SyncPlan, tempfile::TempDir) {
    let repository = GitRepository::discover(GitCommandRunner::new("git"), worktree).unwrap();
    let state = repository.status().unwrap();
    let target = repository.resolve_sync_target().unwrap();
    let paths = state
        .worktree
        .changes
        .iter()
        .map(|change| change.path.clone())
        .collect::<Vec<_>>();
    let policy = RepositoryPolicy {
        identity: repository.identity().clone(),
        target: target.clone(),
        tracked_roots: vec!["".into()],
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
    };
    let plan = SyncPlan {
        operation_id: operation_id.into(),
        identity: repository.identity().clone(),
        target,
        expected_head: state.head,
        content_fingerprints: paths
            .iter()
            .map(|path| (path.clone(), "captured".into()))
            .collect(),
        paths,
        message: "Sync notes: 1 files".into(),
    };
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    (
        GitSyncEngine::new(repository, journal, SyncOptions::default()),
        policy,
        plan,
        data,
    )
}

#[test]
fn selected_local_commit_leaves_unselected_changes_and_uses_authored_message() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("selected.md"), "selected\n").unwrap();
    fs::write(fixture.first.join("later.md"), "later\n").unwrap();
    let (engine, policy, mut plan, _data) = operation(&fixture.first, "selected-version");
    plan.paths = vec![PathBuf::from("selected.md")];
    plan.content_fingerprints = vec![(
        PathBuf::from("selected.md"),
        engine
            .repository()
            .content_fingerprint(Path::new("selected.md"))
            .unwrap(),
    )];
    plan.message = "Write selected version".into();

    let outcome = engine
        .commit_selected(&plan, &policy, &CancellationToken::new())
        .unwrap();
    assert!(matches!(outcome, SyncOutcome::CommittedLocally { .. }));
    assert_eq!(
        git_output(&fixture.first, ["log", "-1", "--format=%s"]).trim(),
        "Write selected version"
    );
    assert_eq!(
        git_output(&fixture.first, ["show", "--format=", "--name-only", "HEAD"])
            .lines()
            .collect::<Vec<_>>(),
        ["selected.md"]
    );
    let state = engine.repository().status().unwrap();
    assert!(
        state
            .worktree
            .changes
            .iter()
            .any(|change| change.path == Path::new("later.md"))
    );
}

#[test]
fn selected_local_commit_rejects_empty_stale_and_externally_staged_drafts() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("selected.md"), "selected\n").unwrap();
    let (engine, policy, mut plan, _data) = operation(&fixture.first, "invalid-version");
    plan.paths = vec![PathBuf::from("selected.md")];
    plan.content_fingerprints = vec![(
        PathBuf::from("selected.md"),
        engine
            .repository()
            .content_fingerprint(Path::new("selected.md"))
            .unwrap(),
    )];
    plan.message = "   ".into();
    assert!(matches!(
        engine.commit_selected(&plan, &policy, &CancellationToken::new()),
        Err(markion_git_sync::SyncEngineError::InvalidCommitMessage)
    ));

    plan.message = "Selected".into();
    fs::write(fixture.first.join("selected.md"), "changed after review\n").unwrap();
    assert!(matches!(
        engine.commit_selected(&plan, &policy, &CancellationToken::new()),
        Err(markion_git_sync::SyncEngineError::StalePlan)
    ));

    plan.content_fingerprints[0].1 = engine
        .repository()
        .content_fingerprint(Path::new("selected.md"))
        .unwrap();
    fs::write(fixture.first.join("external.md"), "external\n").unwrap();
    support::git(&fixture.first, ["add", "external.md"]);
    assert!(matches!(
        engine.commit_selected(&plan, &policy, &CancellationToken::new()),
        Err(markion_git_sync::SyncEngineError::UnsafeRepositoryState)
    ));
    assert_eq!(
        git_output(&fixture.first, ["diff", "--cached", "--name-only"]).trim(),
        "external.md"
    );
}

fn git_output<const N: usize>(directory: &Path, arguments: [&str; N]) -> String {
    let output = std::process::Command::new("git")
        .current_dir(directory)
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn fetch_and_push_preserve_staged_and_unstaged_edits() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("snapshot.md"), "committed\n").unwrap();
    support::git(&fixture.first, ["add", "snapshot.md"]);
    support::git(&fixture.first, ["commit", "-qm", "snapshot"]);
    fs::write(fixture.first.join("notes.md"), "staged\n").unwrap();
    support::git(&fixture.first, ["add", "notes.md"]);
    fs::write(fixture.first.join("notes.md"), "new working edit\n").unwrap();
    let (engine, mut policy, mut plan, _data) = operation(&fixture.first, "secondary-actions");
    plan.paths.clear();
    plan.content_fingerprints.clear();
    let index = git_output(&fixture.first, ["write-tree"]);
    let head = git_output(&fixture.first, ["rev-parse", "HEAD"]);
    assert!(matches!(
        engine
            .check_remote(&plan, &policy, None, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::RemoteChecked { .. }
    ));
    assert!(matches!(
        engine
            .push_commits(&plan, &mut policy, None, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::Synchronized {
            pending_local_changes: 1,
            ..
        }
    ));
    assert_eq!(git_output(&fixture.first, ["write-tree"]), index);
    assert_eq!(
        fs::read_to_string(fixture.first.join("notes.md")).unwrap(),
        "new working edit\n"
    );
    assert_eq!(git_output(&fixture.remote, ["rev-parse", "main"]), head);
    assert_eq!(
        engine.repository().observed_remote_tip(&policy.target),
        policy.last_confirmed_remote
    );

    assert_eq!(
        git_output(&fixture.remote, ["show", "main:notes.md"]),
        "initial\n"
    );
}

#[test]
fn pull_conflicts_resume_after_each_resolution_and_after_last_path() {
    let fixture = TwoCloneFixture::new();
    for (root, content) in [(&fixture.first, "remote\n"), (&fixture.second, "local\n")] {
        fs::write(root.join("notes.md"), content).unwrap();
        fs::write(root.join("other.md"), content).unwrap();
        support::git(root, ["add", "."]);
        support::git(root, ["commit", "-qm", "two conflicts"]);
    }
    support::git(&fixture.first, ["push", "-q"]);
    let (engine, mut policy, plan, data) = operation(&fixture.second, "pull-conflicts");
    assert!(matches!(
        engine
            .pull_updates(&plan, &mut policy, None, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::NeedsAttention {
            phase: OperationPhase::Resolving,
            ..
        }
    ));
    for remaining in [2, 1] {
        let manager = conflict_manager(&engine, &data);
        let session = manager
            .restore_session("pull-conflicts", &CancellationToken::new())
            .unwrap();
        assert_eq!(session.files.len(), remaining);
        let path = &session.files[0].path;
        manager
            .mark_resolved(
                &session,
                path,
                ConflictResolution::Content(
                    b"<<<<<<< literal example\n=======\n>>>>>>> text\n".to_vec(),
                ),
                &CancellationToken::new(),
            )
            .unwrap();
    }
    let manager = conflict_manager(&engine, &data);
    let session = manager
        .restore_session("pull-conflicts", &CancellationToken::new())
        .unwrap();
    assert!(session.files.is_empty());
    manager
        .finish_merge(&session, &CancellationToken::new())
        .unwrap();
    assert!(!fixture.second.join(".git/MERGE_HEAD").exists());
    assert!(
        fs::read_to_string(fixture.second.join("notes.md"))
            .unwrap()
            .starts_with("<<<<<<< literal")
    );
}

#[test]
fn recovery_recognizes_commit_before_checkpoint_and_rejects_foreign_index() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("new.md"), "recover me\n").unwrap();
    let (engine, policy, plan, data) = operation(&fixture.first, "interrupt-commit");
    let engine = engine.with_observer(|phase| {
        if phase == OperationPhase::Complete {
            Err(markion_git_sync::SyncEngineError::StalePlan)
        } else {
            Ok(())
        }
    });
    assert!(
        engine
            .commit_locally(&plan, &policy, None, &CancellationToken::new())
            .is_err()
    );
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    let manager =
        markion_git_sync::RecoveryManager::new(engine.repository().clone(), journal.clone());
    assert!(matches!(
        manager.assess().unwrap()[0],
        markion_git_sync::RecoveryAssessment::CommitCompleted { .. }
    ));
    let head = git_output(&fixture.first, ["rev-parse", "HEAD"]);
    manager.adopt_completed("interrupt-commit").unwrap();
    assert_eq!(git_output(&fixture.first, ["rev-parse", "HEAD"]), head);
    assert!(journal.load().unwrap().active.is_empty());

    fs::write(fixture.first.join("new.md"), "requires signing\n").unwrap();
    support::git(
        &fixture.first,
        ["config", "gpg.program", "markion-missing-gpg"],
    );
    support::git(&fixture.first, ["config", "commit.gpgsign", "true"]);
    let (engine, policy, plan, data) = operation(&fixture.first, "failed-staging");
    assert!(
        engine
            .commit_locally(&plan, &policy, None, &CancellationToken::new())
            .is_err()
    );
    let manager = markion_git_sync::RecoveryManager::new(
        engine.repository().clone(),
        JournalStore::new(
            data.path().join("journal.toml"),
            data.path().join("recovery"),
        ),
    );
    assert!(matches!(
        manager.assess().unwrap()[0],
        markion_git_sync::RecoveryAssessment::OwnedStaging { .. }
    ));
    fs::write(fixture.first.join("external.md"), "external index\n").unwrap();
    support::git(&fixture.first, ["add", "external.md"]);
    let index = git_output(&fixture.first, ["write-tree"]);
    assert!(manager.recover_staging("failed-staging").is_err());
    assert_eq!(git_output(&fixture.first, ["write-tree"]), index);
}

#[test]
fn interrupted_checkout_recovery_keeps_preimages_and_refuses_external_path_or_index_changes() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("notes.md"), "incoming\n").unwrap();
    let (first_engine, mut first_policy, first_plan, _data) =
        operation(&fixture.first, "incoming-before-checkout");
    first_engine
        .sync_now(
            &first_plan,
            &mut first_policy,
            None,
            None,
            &CancellationToken::new(),
        )
        .unwrap();

    let (engine, mut policy, plan, data) = operation(&fixture.second, "checkout-interrupted");
    let integrating_boundaries = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = integrating_boundaries.clone();
    let interrupted = engine.clone().with_observer(move |phase| {
        if phase == OperationPhase::Integrating
            && observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) == 1
        {
            Err(markion_git_sync::SyncEngineError::StalePlan)
        } else {
            Ok(())
        }
    });
    assert!(
        interrupted
            .sync_now(&plan, &mut policy, None, None, &CancellationToken::new())
            .is_err()
    );

    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    let checkpoint = journal.load().unwrap().active.remove(0);
    assert_eq!(checkpoint.phase, OperationPhase::Integrating);
    let preimages = checkpoint.preimages.as_ref().expect("durable preimages");
    let note_preimage = preimages
        .entries
        .iter()
        .find(|entry| entry.relative_path == Path::new("notes.md"))
        .expect("notes preimage");
    let safe_copy = data
        .path()
        .join("recovery")
        .join("checkout-interrupted")
        .join("preimages")
        .join(&note_preimage.stored_name);
    assert_eq!(fs::read(&safe_copy).unwrap(), b"initial\n");
    let fetched = checkpoint.fetched_tip.as_ref().unwrap();
    support::git(
        &fixture.second,
        ["checkout", fetched.as_str(), "--", "notes.md"],
    );
    fs::write(
        fixture.second.join("notes.md"),
        "independent after interruption\n",
    )
    .unwrap();
    fs::write(fixture.second.join("untracked-after-crash.md"), "keep me\n").unwrap();
    let before_index = git_output(&fixture.second, ["write-tree"]);
    let before_path = fs::read(fixture.second.join("notes.md")).unwrap();
    let recovery_ref = checkpoint.recovery_refs.first().unwrap();
    assert!(!git_output(&fixture.second, ["rev-parse", recovery_ref]).is_empty());

    let manager = markion_git_sync::RecoveryManager::new(engine.repository().clone(), journal);
    assert!(matches!(
        manager.assess().unwrap()[0],
        markion_git_sync::RecoveryAssessment::ExternalState { .. }
    ));
    assert_eq!(git_output(&fixture.second, ["write-tree"]), before_index);
    assert_eq!(
        fs::read(fixture.second.join("notes.md")).unwrap(),
        before_path
    );
    assert_eq!(
        fs::read_to_string(fixture.second.join("untracked-after-crash.md")).unwrap(),
        "keep me\n"
    );
    assert_eq!(fs::read(&safe_copy).unwrap(), b"initial\n");
}

#[test]
fn nested_history_and_bounded_inventory_keep_exact_paths() {
    let fixture = TwoCloneFixture::new();
    fs::create_dir_all(fixture.first.join("folder")).unwrap();
    fs::write(fixture.first.join("folder/nested.md"), "historical\n").unwrap();
    support::git(&fixture.first, ["add", "."]);
    support::git(&fixture.first, ["commit", "-qm", "nested"]);
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    let history = repository.history_page(0, 50, true).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(
        repository.commit_paths(&history[0].oid).unwrap(),
        [std::path::PathBuf::from("folder/nested.md")]
    );
    assert_eq!(
        repository
            .historical_blob(&history[0].oid, Path::new("folder/nested.md"), 1024)
            .unwrap()
            .bytes,
        b"historical\n"
    );
    for index in 0..10_000 {
        fs::write(fixture.first.join(format!("folder/note-{index}.md")), "x").unwrap();
    }
    let state = repository.status().unwrap();
    assert_eq!(state.worktree.changes.len(), 10_000);
    assert_eq!(repository.history_page(0, 1000, false).unwrap().len(), 2);
}

#[test]
fn accepted_push_without_local_ack_is_verified_after_remote_advances() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("delivered.md"), "delivered\n").unwrap();
    let (engine, mut policy, plan, data) = operation(&fixture.first, "lost-ack");
    let interrupted = engine.clone().with_observer(|phase| {
        if phase == OperationPhase::Complete {
            Err(markion_git_sync::SyncEngineError::StalePlan)
        } else {
            Ok(())
        }
    });
    assert!(
        interrupted
            .sync_now(&plan, &mut policy, None, None, &CancellationToken::new())
            .is_err()
    );
    let head = git_output(&fixture.first, ["rev-parse", "HEAD"]);
    assert_eq!(git_output(&fixture.remote, ["rev-parse", "main"]), head);
    support::git(&fixture.second, ["pull", "-q", "--ff-only"]);
    fs::write(fixture.second.join("later.md"), "later\n").unwrap();
    support::git(&fixture.second, ["add", "."]);
    support::git(&fixture.second, ["commit", "-qm", "later"]);
    support::git(&fixture.second, ["push", "-q"]);
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    let checkpoint = journal.load().unwrap().active[0].clone();
    assert_eq!(checkpoint.phase, OperationPhase::Pushing);
    assert!(matches!(
        engine
            .verify_delivery(&checkpoint, &mut policy, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::Synchronized { .. }
    ));
    assert_eq!(git_output(&fixture.first, ["rev-parse", "HEAD"]), head);
    assert!(journal.load().unwrap().active.is_empty());
}

#[test]
fn offline_push_preserves_uncertain_checkpoint_and_verification_does_not_upload() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join("offline.md"), "local only\n").unwrap();
    support::git(&fixture.first, ["add", "."]);
    support::git(&fixture.first, ["commit", "-qm", "offline"]);
    let (engine, mut policy, plan, data) = operation(&fixture.first, "offline-push");
    let offline = fixture.remote.with_extension("offline");
    fs::rename(&fixture.remote, &offline).unwrap();
    assert!(matches!(
        engine
            .push_commits(&plan, &mut policy, None, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::UncertainDelivery { .. }
    ));
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    let checkpoint = journal.load().unwrap().active[0].clone();
    assert_eq!(checkpoint.phase, OperationPhase::Verifying);
    fs::rename(&offline, &fixture.remote).unwrap();
    let remote = git_output(&fixture.remote, ["rev-parse", "main"]);
    assert!(matches!(
        engine
            .verify_delivery(&checkpoint, &mut policy, &CancellationToken::new())
            .unwrap(),
        SyncOutcome::AwaitingUpload { .. }
    ));
    assert_eq!(git_output(&fixture.remote, ["rev-parse", "main"]), remote);
    assert!(fixture.first.join("offline.md").is_file());
}
