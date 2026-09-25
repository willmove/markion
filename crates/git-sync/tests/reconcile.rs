mod support;

use std::{fs, path::Path};

use markion_git_sync::{
    ConflictDraft, GitCommandRunner, GitObjectId, GitRepository, JournalStore, OperationCheckpoint,
    OperationKind, OperationPhase, RecoveryAssessment, RecoveryManager,
};
use support::TwoCloneFixture;

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
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn oid(value: String) -> GitObjectId {
    GitObjectId::parse(value).unwrap()
}

fn commit_file(worktree: &Path, name: &str, text: &str) -> GitObjectId {
    fs::write(worktree.join(name), text).unwrap();
    support::git(worktree, ["add", "--", name]);
    support::git(worktree, ["commit", "-q", "-m", name]);
    oid(git_output(worktree, ["rev-parse", "HEAD"]))
}

fn checkpoint(
    repository: &GitRepository,
    id: &str,
    phase: OperationPhase,
    resulting_commit: Option<GitObjectId>,
    fetched_tip: Option<GitObjectId>,
) -> OperationCheckpoint {
    OperationCheckpoint {
        operation_id: id.into(),
        kind: OperationKind::SyncNow,
        phase,
        identity: repository.identity().clone(),
        target: None,
        expected_head: None,
        resulting_commit,
        fetched_tip,
        expected_index_fingerprint: None,
        owned_paths: Vec::new(),
        recovery_refs: Vec::new(),
        preimages: None,
        drafts: Vec::new(),
        updated_unix_seconds: 1,
        confirmed: false,
    }
}

fn draft(fingerprint: String) -> ConflictDraft {
    ConflictDraft {
        relative_path: "notes.md".into(),
        content: b"draft\n".to_vec(),
        content_fingerprint: fingerprint,
    }
}

#[test]
fn assessment_distinguishes_head_superseded_and_unreachable_commits() {
    let fixture = TwoCloneFixture::new();
    let worktree = &fixture.first;
    let behind = commit_file(worktree, "a.md", "a\n");
    let head = commit_file(worktree, "b.md", "b\n");
    let unreachable = oid(git_output(
        worktree,
        ["commit-tree", "HEAD^{tree}", "-m", "dangling"],
    ));
    let repository = GitRepository::discover(GitCommandRunner::new("git"), worktree).unwrap();
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    for value in [
        checkpoint(
            &repository,
            "at-head",
            OperationPhase::Fetching,
            Some(head.clone()),
            None,
        ),
        checkpoint(
            &repository,
            "behind",
            OperationPhase::Fetching,
            Some(behind.clone()),
            None,
        ),
        checkpoint(
            &repository,
            "unreachable",
            OperationPhase::Fetching,
            Some(unreachable),
            None,
        ),
    ] {
        journal.begin(value).unwrap();
    }

    let assessments = RecoveryManager::new(repository, journal).assess().unwrap();

    assert_eq!(
        assessments,
        vec![
            RecoveryAssessment::CommitCompleted {
                operation_id: "at-head".into(),
                commit: head,
            },
            RecoveryAssessment::Superseded {
                operation_id: "behind".into(),
                commit: behind,
            },
            RecoveryAssessment::ExternalState {
                operation_id: "unreachable".into(),
            },
        ]
    );
}

#[test]
fn reconcile_retires_completed_work_and_keeps_differing_drafts_for_a_choice() {
    let fixture = TwoCloneFixture::new();
    let worktree = &fixture.first;
    let merged_tip = commit_file(worktree, "a.md", "a\n");
    let behind = commit_file(worktree, "b.md", "b\n");
    let head = commit_file(worktree, "c.md", "c\n");
    let committed_notes = git_output(worktree, ["rev-parse", "HEAD:notes.md"]);
    let repository = GitRepository::discover(GitCommandRunner::new("git"), worktree).unwrap();
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );

    // Shape of the reported vault: a stale Resolving record whose merge is
    // already in history but whose draft differs, plus two Fetching records.
    let mut resolving = checkpoint(
        &repository,
        "resolving",
        OperationPhase::Resolving,
        Some(merged_tip.clone()),
        Some(merged_tip),
    );
    resolving.drafts.push(draft("0".repeat(40)));
    journal.begin(resolving).unwrap();
    journal
        .begin(checkpoint(
            &repository,
            "fetching-at-head",
            OperationPhase::Fetching,
            Some(head.clone()),
            None,
        ))
        .unwrap();
    let mut fetching_behind = checkpoint(
        &repository,
        "fetching-behind",
        OperationPhase::Fetching,
        Some(behind),
        None,
    );
    fetching_behind.drafts.push(draft(committed_notes));
    journal.begin(fetching_behind).unwrap();
    let before_head = git_output(worktree, ["rev-parse", "HEAD"]);
    let before_status = git_output(worktree, ["status", "--porcelain=v2"]);

    let report = RecoveryManager::new(repository, journal.clone())
        .reconcile()
        .unwrap();

    assert_eq!(report.retired, vec!["fetching-at-head", "fetching-behind"]);
    assert_eq!(report.needs_choice, vec!["resolving"]);
    let loaded = journal.load().unwrap();
    assert_eq!(loaded.active.len(), 1);
    assert_eq!(loaded.active[0].operation_id, "resolving");
    assert_eq!(loaded.active[0].drafts.len(), 1);
    assert_eq!(loaded.successful.len(), 2);
    assert_eq!(git_output(worktree, ["rev-parse", "HEAD"]), before_head);
    assert_eq!(
        git_output(worktree, ["status", "--porcelain=v2"]),
        before_status
    );
    assert_eq!(head.as_str(), before_head);
}

#[test]
fn discard_stale_removes_a_superseded_record_but_refuses_a_live_conflict() {
    let fixture = TwoCloneFixture::new();
    let worktree = &fixture.first;
    let behind = commit_file(worktree, "a.md", "a\n");
    commit_file(worktree, "b.md", "b\n");
    let repository = GitRepository::discover(GitCommandRunner::new("git"), worktree).unwrap();
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    let mut stale = checkpoint(
        &repository,
        "stale",
        OperationPhase::Fetching,
        Some(behind),
        None,
    );
    stale.drafts.push(draft("0".repeat(40)));
    journal.begin(stale).unwrap();
    let manager = RecoveryManager::new(repository, journal.clone());

    assert!(manager.discard_stale("missing").is_err());
    manager.discard_stale("stale").unwrap();

    assert!(journal.load().unwrap().active.is_empty());
    assert!(fs::read_to_string(worktree.join("notes.md")).is_ok());
}
