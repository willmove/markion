//! Git process budgets and incoming-path validation that the refresh/sync
//! overhead work relies on. Counts come from the runner's invocation counter,
//! never from timing.

mod support;

use std::{
    collections::BTreeSet,
    fs,
    path::Path,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use markion_git_sync::{
    CancellationToken, GitCommandRunner, GitRepository, GitSyncEngine, JournalStore, NewFileClass,
    NewFileRule, RepositoryPolicy, SyncEngineError, SyncOptions, SyncOutcome, SyncPlan,
};
use support::TwoCloneFixture;

struct Counted {
    engine: GitSyncEngine,
    policy: RepositoryPolicy,
    plan: SyncPlan,
    counter: Arc<AtomicUsize>,
    _data: tempfile::TempDir,
}

fn counted_operation(worktree: &Path, operation_id: &str) -> Counted {
    let counter = Arc::new(AtomicUsize::new(0));
    let runner = GitCommandRunner::new("git").with_invocation_counter(counter.clone());
    let repository = GitRepository::discover(runner, worktree).unwrap();
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
        allowed_extensions: Default::default(),
        included_once: Vec::new(),
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
        message: "Sync notes".into(),
    };
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(
        data.path().join("journal.toml"),
        data.path().join("recovery"),
    );
    counter.store(0, Ordering::Relaxed);
    Counted {
        engine: GitSyncEngine::new(repository, journal, SyncOptions::default()),
        policy,
        plan,
        counter,
        _data: data,
    }
}

impl Counted {
    fn sync_now(&mut self) -> Result<SyncOutcome, SyncEngineError> {
        self.engine.sync_now(
            &self.plan,
            &mut self.policy,
            None,
            None,
            &CancellationToken::new(),
        )
    }

    fn spawned(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}

fn push_from_second(fixture: &TwoCloneFixture, files: &[&str]) {
    for file in files {
        fs::write(fixture.second.join(file), format!("{file}\n")).unwrap();
    }
    support::git(&fixture.second, ["add", "--all"]);
    support::git(&fixture.second, ["commit", "-q", "-m", "remote notes"]);
    support::git(&fixture.second, ["push", "-q", "origin", "main"]);
}

fn incoming_sync_process_count(incoming: usize) -> usize {
    let fixture = TwoCloneFixture::new();
    let names: Vec<String> = (0..incoming)
        .map(|index| format!("remote-{index}.md"))
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    push_from_second(&fixture, &names);
    let mut operation = counted_operation(&fixture.first, "budget-incoming");
    assert!(matches!(
        operation.sync_now().unwrap(),
        SyncOutcome::Synchronized { .. }
    ));
    assert!(fixture.first.join(names[incoming - 1]).is_file());
    operation.spawned()
}

#[test]
fn integration_process_count_does_not_grow_with_incoming_files() {
    let one = incoming_sync_process_count(1);
    let fifty = incoming_sync_process_count(50);
    assert_eq!(one, fifty, "one incoming file: {one}, fifty: {fifty}");
}

#[test]
fn refresh_reads_of_many_changed_notes_with_images_stay_within_budget() {
    let fixture = TwoCloneFixture::new();
    fs::create_dir(fixture.first.join("assets")).unwrap();
    let mut notes = Vec::new();
    for index in 0..200 {
        fs::write(fixture.first.join(format!("assets/{index}.png")), b"png").unwrap();
        let path = format!("note-{index}.md");
        let text = format!(
            "![a](assets/{index}.png) ![b](assets/{}.png)\n",
            (index + 1) % 200
        );
        fs::write(fixture.first.join(&path), &text).unwrap();
        notes.push((std::path::PathBuf::from(path), text));
    }
    let operation = counted_operation(&fixture.first, "budget-refresh");
    let repository = operation.engine.repository();
    operation.counter.store(0, Ordering::Relaxed);
    let state = repository.status().unwrap();
    assert!(state.worktree.changes.len() >= 400);
    let target = repository.resolve_sync_target().unwrap();
    if let (Some(local), Some(remote)) =
        (state.head.as_ref(), repository.observed_remote_tip(&target))
    {
        repository.classify_history(local, &remote, None).unwrap();
    }
    let inputs = notes
        .iter()
        .map(|(path, text)| (path.as_path(), text.as_str()))
        .collect::<Vec<_>>();
    let reports =
        markion_git_sync::inspect_notes_attachments(repository, &operation.policy, &inputs)
            .unwrap();
    assert_eq!(reports.len(), 200);
    // status + observed tip + two batched attachment queries; target
    // resolution here stands in for the Outgoing page and is counted too.
    let spawned = operation.spawned();
    eprintln!("refresh-equivalent reads spawned {spawned} Git processes");
    assert!(spawned <= 6 + 4, "spawned {spawned}");
}

#[test]
fn no_op_sync_now_stays_within_budget() {
    let fixture = TwoCloneFixture::new();
    let mut operation = counted_operation(&fixture.first, "budget-noop");
    assert!(matches!(
        operation.sync_now().unwrap(),
        SyncOutcome::Synchronized { .. }
    ));
    eprintln!(
        "no-op Sync Now spawned {} Git processes",
        operation.spawned()
    );
    assert!(
        operation.spawned() <= 35,
        "no-op Sync Now spawned {} Git processes",
        operation.spawned()
    );
}

fn local_commit_process_count(new_notes: usize) -> usize {
    let fixture = TwoCloneFixture::new();
    for index in 0..new_notes {
        fs::write(fixture.first.join(format!("local-{index}.md")), "local\n").unwrap();
    }
    let operation = counted_operation(&fixture.first, "budget-local");
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
    operation.spawned()
}

#[test]
fn preflight_process_count_does_not_grow_with_untracked_notes() {
    assert_eq!(
        local_commit_process_count(1),
        local_commit_process_count(20)
    );
}

fn blob_for(directory: &Path, content: &str) -> String {
    let path = directory.join("blob-source.tmp");
    fs::write(&path, content).unwrap();
    let output = Command::new("git")
        .current_dir(directory)
        .args(["hash-object", "-w", "--", "blob-source.tmp"])
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn push_index_entry_from_second(fixture: &TwoCloneFixture, mode: &str, object: &str, path: &str) {
    support::git(
        &fixture.second,
        [
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("{mode},{object},{path}"),
        ],
    );
    support::git(&fixture.second, ["commit", "-q", "-m", "special entry"]);
    support::git(&fixture.second, ["push", "-q", "origin", "main"]);
}

#[test]
fn incoming_symlink_is_refused_before_the_worktree_changes() {
    let fixture = TwoCloneFixture::new();
    let target = blob_for(&fixture.second, "../outside");
    push_index_entry_from_second(&fixture, "120000", &target, "link.md");
    let mut operation = counted_operation(&fixture.first, "incoming-symlink");
    assert!(matches!(
        operation.sync_now(),
        Err(SyncEngineError::UnsafeIncomingPaths)
    ));
    assert!(!fixture.first.join("link.md").exists());
}

#[test]
fn incoming_gitlink_is_refused_before_the_worktree_changes() {
    let fixture = TwoCloneFixture::new();
    let head = Command::new("git")
        .current_dir(&fixture.second)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let head = String::from_utf8(head.stdout).unwrap();
    push_index_entry_from_second(&fixture, "160000", head.trim(), "vendored");
    let mut operation = counted_operation(&fixture.first, "incoming-gitlink");
    assert!(matches!(
        operation.sync_now(),
        Err(SyncEngineError::UnsafeIncomingPaths)
    ));
    assert!(!fixture.first.join("vendored").exists());
}

#[test]
fn incoming_path_colliding_with_an_ignored_local_file_is_refused() {
    let fixture = TwoCloneFixture::new();
    fs::write(fixture.first.join(".git/info/exclude"), "clash.txt\n").unwrap();
    fs::write(fixture.first.join("clash.txt"), "local only\n").unwrap();
    push_from_second(&fixture, &["clash.txt"]);
    let mut operation = counted_operation(&fixture.first, "incoming-collision");
    assert!(matches!(
        operation.sync_now(),
        Err(SyncEngineError::UnsafeIncomingPaths)
    ));
    assert_eq!(
        fs::read_to_string(fixture.first.join("clash.txt")).unwrap(),
        "local only\n"
    );
}
