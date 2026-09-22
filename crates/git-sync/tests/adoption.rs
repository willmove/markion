//! End-to-end verification of automatic repository adoption: a healthy
//! dedicated notes repository is adoptable, its sync commit carries the
//! compact date-time-and-files message, and ineligible repositories are
//! left to manual setup.

mod support;

use std::{collections::BTreeSet, path::Path, process::Command, time::SystemTime};

use markion_git_sync::{
    GitCommandRunner, GitObjectId, GitRepository, GitSyncEngine, JournalStore, NewFileClass,
    NewFileRule, OnboardingService, RepositoryPolicy, SyncOptions, SyncPlan, SyncTarget,
    repository_adoptable,
};
use support::{TwoCloneFixture, configure_identity, git};

fn git_output(directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(directory)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn default_notes_policy(
    identity: markion_git_sync::RepositoryIdentity,
    target: SyncTarget,
) -> RepositoryPolicy {
    RepositoryPolicy {
        identity,
        target,
        tracked_roots: vec![Path::new("").into()],
        new_file_rules: vec![NewFileRule {
            root: Path::new("").into(),
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
    }
}

#[test]
fn eligible_repository_is_adopted_and_commits_compact_message() {
    let fixture = TwoCloneFixture::new();
    std::fs::write(fixture.first.join("notes.md"), "edited\n").unwrap();
    std::fs::write(fixture.first.join("todo.md"), "todo\n").unwrap();

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&fixture.first).unwrap();
    let tracked_paths = review.repository.tracked_paths().unwrap();
    let identity = review.repository.identity().clone();
    let target = review
        .target
        .as_ref()
        .expect("clone with upstream resolves a sync target");

    assert!(repository_adoptable(
        &identity.worktree_root,
        &identity,
        &review.state,
        &tracked_paths,
        Some(target),
    ));

    let policy = default_notes_policy(identity.clone(), target.clone());
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    let state = repository.status().unwrap();
    let paths = state
        .worktree
        .changes
        .iter()
        .map(|change| change.path.clone())
        .collect::<Vec<_>>();
    let plan = SyncPlan {
        operation_id: "adoption-smoke".into(),
        identity: identity.clone(),
        target: target.clone(),
        expected_head: state.head,
        content_fingerprints: paths
            .iter()
            .map(|path| (path.clone(), "captured".into()))
            .collect(),
        message: policy.render_message(&paths, None, SystemTime::now()),
        paths,
    };
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(data.path().join("journal.toml"), data.path().join("recovery"));
    let engine = GitSyncEngine::new(repository, journal, SyncOptions::default());
    engine
        .commit_locally(&plan, &policy, None, &markion_git_sync::CancellationToken::new())
        .unwrap();

    let subject = git_output(&fixture.first, &["log", "-1", "--pretty=%s"]);
    let (prefix, stamp, tail) = {
        let rest = subject.strip_prefix("Sync notes ").expect(&subject);
        let (stamp, tail) = rest.split_once(": ").expect(&subject);
        ("Sync notes", stamp, tail)
    };
    assert_eq!(prefix, "Sync notes");
    assert_eq!(stamp.len(), 16, "{subject}");
    assert!(
        tail.contains("notes.md") && tail.contains("todo.md"),
        "{subject}"
    );
}

#[test]
fn mixed_content_repository_is_not_adopted() {
    let fixture = TwoCloneFixture::new();
    std::fs::write(fixture.first.join("Cargo.toml"), "[package]\n").unwrap();
    git(&fixture.first, ["add", "--", "Cargo.toml"]);
    git(&fixture.first, ["commit", "-q", "-m", "add build file"]);

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&fixture.first).unwrap();
    let tracked_paths = review.repository.tracked_paths().unwrap();
    let identity = review.repository.identity().clone();

    assert!(!repository_adoptable(
        &identity.worktree_root,
        &identity,
        &review.state,
        &tracked_paths,
        review.target.as_ref(),
    ));
}

#[test]
fn repository_with_hidden_and_furniture_files_is_adopted() {
    // Real-world notes repositories usually carry `.gitignore`, editor or
    // tool folders, and a LICENSE; adoption must not treat these as mixed
    // content the way a source tree would be.
    let fixture = TwoCloneFixture::new();
    std::fs::write(fixture.first.join(".gitignore"), "*.tmp\n").unwrap();
    std::fs::create_dir_all(fixture.first.join(".obsidian")).unwrap();
    std::fs::write(fixture.first.join(".obsidian/app.json"), "{}\n").unwrap();
    std::fs::write(fixture.first.join("LICENSE"), "MIT\n").unwrap();
    git(&fixture.first, ["add", "--", ".gitignore", ".obsidian", "LICENSE"]);
    git(&fixture.first, ["commit", "-q", "-m", "add repo furniture"]);

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&fixture.first).unwrap();
    let tracked_paths = review.repository.tracked_paths().unwrap();
    let identity = review.repository.identity().clone();

    assert!(repository_adoptable(
        &identity.worktree_root,
        &identity,
        &review.state,
        &tracked_paths,
        review.target.as_ref(),
    ));
}

#[test]
fn workspace_below_repository_root_is_not_adopted() {
    let fixture = TwoCloneFixture::new();
    let nested = fixture.first.join("subdir");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("note.md"), "nested\n").unwrap();
    git(&fixture.first, ["add", "--", "subdir/note.md"]);
    git(&fixture.first, ["commit", "-q", "-m", "nested notes"]);
    configure_identity(&fixture.first);

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&nested).unwrap();
    let tracked_paths = review.repository.tracked_paths().unwrap();
    let identity = review.repository.identity().clone();

    // The repository itself remains adoptable at its root, but a workspace
    // inside it must fall back to manual setup.
    assert!(repository_adoptable(
        &identity.worktree_root,
        &identity,
        &review.state,
        &tracked_paths,
        review.target.as_ref(),
    ));
    assert!(!repository_adoptable(
        &nested,
        &identity,
        &review.state,
        &tracked_paths,
        review.target.as_ref(),
    ));
}

#[test]
fn repository_without_history_is_not_adopted() {
    let root = tempfile::tempdir().unwrap();
    let repository_path = root.path().join("fresh");
    git(root.path(), ["init", "-q", "-b", "main", &repository_path.to_string_lossy()]);
    std::fs::write(repository_path.join("note.md"), "note\n").unwrap();

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&repository_path).unwrap();
    let identity = review.repository.identity().clone();

    assert!(review.state.head.is_none());
    assert!(!repository_adoptable(
        &identity.worktree_root,
        &identity,
        &review.state,
        &[],
        review.target.as_ref(),
    ));
}

#[test]
fn adoption_head_and_target_come_from_the_reviewed_repository() {
    // Guards the smoke expectation that adopted policies point at the real
    // upstream branch of the cloned workspace.
    let fixture = TwoCloneFixture::new();
    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&fixture.first).unwrap();
    let target = review.target.as_ref().unwrap();
    assert_eq!(target.local_branch, "main");
    assert_eq!(target.remote, "origin");
    assert!(review.state.head.is_some_and(|head: GitObjectId| {
        !head.as_str().is_empty()
    }));
}
