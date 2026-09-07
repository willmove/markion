mod support;

use std::{collections::BTreeSet, fs, path::Path};

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
    assert_eq!(journal.active[0].phase, OperationPhase::NeedsAttention);
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
