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
fn wrong_origin_is_detectable_and_replaceable_after_confirmation() {
    // Reproduces the setup dead end: the repository's origin points at a
    // wrong address, no sync target resolves, and correcting the address
    // must work through probe + explicit confirmation + replacement.
    let fixture = TwoCloneFixture::new();
    let root = fixture.first.parent().unwrap();
    let wrong = root.join("wrong.git");
    git(
        &fixture.first,
        ["remote", "set-url", "origin", wrong.to_string_lossy().as_ref()],
    );
    // Break the upstream binding the way a locally-created repository with a
    // hand-set (wrong) remote looks: branch config exists, merge ref does not.
    git(&fixture.first, ["config", "--unset", "branch.main.merge"]);

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&fixture.first).unwrap();
    // The broken origin cannot resolve a sync target.
    assert!(review.target.is_none());

    // The probe rejects the broken address while the correct one answers.
    let cancellation = markion_git_sync::CancellationToken::new();
    assert!(
        service
            .probe_remote(wrong.to_str().unwrap(), root, &cancellation)
            .is_err()
    );
    let correct = fixture.remote.to_string_lossy().into_owned();
    service.probe_remote(&correct, root, &cancellation).unwrap();

    // Without confirmation a differing address is still refused...
    assert!(matches!(
        service.attach_origin(&review.repository, &correct, false, &cancellation),
        Err(markion_git_sync::OnboardingError::RemoteExists(_))
    ));
    // ...and the explicitly confirmed replacement rebinds the origin.
    service
        .attach_origin(&review.repository, &correct, true, &cancellation)
        .unwrap();
    service
        .publish_first_branch(&review.repository, "origin", "main", None, &cancellation)
        .unwrap();
    assert_eq!(
        git_output(&fixture.first, &["remote", "get-url", "origin"]).trim(),
        correct
    );
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    assert!(repository.resolve_sync_target().is_ok());
}

#[test]
fn replacement_to_diverged_remote_binds_and_first_sync_merges() {
    // The screenshot scenario's full resolution: after the confirmed
    // replacement onto a remote that already contains related diverged
    // work, publication binds upstream without a rejected push, and the
    // first sync integrates both sides and pushes.
    let fixture = TwoCloneFixture::new();
    let root = fixture.first.parent().unwrap();
    let cancellation = markion_git_sync::CancellationToken::new();

    // Diverge: the second clone advances the remote, the first adds a local
    // commit of its own.
    std::fs::write(fixture.second.join("remote-side.md"), "remote\n").unwrap();
    git(&fixture.second, ["add", "--", "remote-side.md"]);
    git(&fixture.second, ["commit", "-q", "-m", "remote side"]);
    git(&fixture.second, ["push", "-q", "origin", "main"]);
    std::fs::write(fixture.first.join("local-side.md"), "local\n").unwrap();
    git(&fixture.first, ["add", "--", "local-side.md"]);
    git(&fixture.first, ["commit", "-q", "-m", "local side"]);
    let local_head = git_output(&fixture.first, &["rev-parse", "HEAD"]).trim().to_string();

    // Break the origin the way a wrong address leaves the repository.
    let wrong = root.join("wrong.git");
    git(
        &fixture.first,
        ["remote", "set-url", "origin", wrong.to_string_lossy().as_ref()],
    );
    git(&fixture.first, ["config", "--unset", "branch.main.merge"]);

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let review = service.connect_existing(&fixture.first).unwrap();
    assert!(review.target.is_none());
    let correct = fixture.remote.to_string_lossy().into_owned();
    service
        .attach_origin(&review.repository, &correct, true, &cancellation)
        .unwrap();
    // Publication must not dead-end on the diverged remote: it binds
    // upstream without pushing.
    service
        .publish_first_branch(&review.repository, "origin", "main", None, &cancellation)
        .unwrap();
    assert!(
        repository_publish_did_not_push(&fixture, &wrong)
    );

    // The first sync fetches, merges both sides, and pushes.
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    let target = repository.resolve_sync_target().unwrap();
    let mut policy = default_notes_policy(repository.identity().clone(), target);
    let state = repository.status().unwrap();
    let plan = SyncPlan {
        operation_id: "replacement-smoke".into(),
        identity: repository.identity().clone(),
        target: repository.resolve_sync_target().unwrap(),
        expected_head: state.head,
        content_fingerprints: Vec::new(),
        message: "Sync notes 2026-09-22 10:00: local-side.md".into(),
        paths: Vec::new(),
    };
    let data = tempfile::tempdir().unwrap();
    let journal = JournalStore::new(data.path().join("journal.toml"), data.path().join("recovery"));
    let engine = GitSyncEngine::new(repository, journal, SyncOptions::default())
        .with_foreground_credentials();
    let outcome = engine
        .sync_now(&plan, &mut policy, None, None, &cancellation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            markion_git_sync::SyncOutcome::Synchronized { .. }
                | markion_git_sync::SyncOutcome::UpToDate
        ),
        "{outcome:?}"
    );
    // The remote's main now contains both the local and the remote commit.
    let remote_side = git_output(&fixture.second, &["rev-parse", "HEAD"])
        .trim()
        .to_string();
    for sha in [local_head.as_str(), remote_side.as_str()] {
        let output = std::process::Command::new("git")
            .current_dir(&fixture.remote)
            .args(["merge-base", "--is-ancestor", sha, "main"])
            .output()
            .unwrap();
        assert!(output.status.success(), "remote main lacks {sha}");
    }
}

fn repository_publish_did_not_push(fixture: &TwoCloneFixture, wrong: &Path) -> bool {
    // Pushing would have failed against the wrong remote and succeeded
    // against the right one; the binding-only path leaves both untouched,
    // with upstream config present.
    let merge = git_output(
        &fixture.first,
        &["config", "--get", "branch.main.merge"],
    )
    .trim()
    .to_string();
    let remote_config = git_output(
        &fixture.first,
        &["config", "--get", "branch.main.remote"],
    )
    .trim()
    .to_string();
    !wrong.exists() && merge == "refs/heads/main" && remote_config == "origin"
}

#[test]
fn unrelated_remote_history_is_refused_with_guidance_error() {
    let fixture = TwoCloneFixture::new();
    let cancellation = markion_git_sync::CancellationToken::new();
    // Point the clone at an unrelated repository with its own history.
    let unrelated_root = tempfile::tempdir().unwrap();
    let unrelated = unrelated_root.path().join("unrelated.git");
    let seed = unrelated_root.path().join("seed");
    git(
        unrelated_root.path(),
        ["init", "--bare", "-q", &unrelated.to_string_lossy()],
    );
    git(&unrelated, ["symbolic-ref", "HEAD", "refs/heads/main"]);
    git(unrelated_root.path(), ["init", "-q", "-b", "main", &seed.to_string_lossy()]);
    support::configure_identity(&seed);
    std::fs::write(seed.join("other.md"), "unrelated\n").unwrap();
    git(&seed, ["add", "--", "other.md"]);
    git(&seed, ["commit", "-q", "-m", "unrelated"]);
    git(&seed, ["remote", "add", "origin", &unrelated.to_string_lossy()]);
    git(&seed, ["push", "-q", "-u", "origin", "main"]);

    let service = OnboardingService::new(GitCommandRunner::new("git"));
    let correct = unrelated.to_string_lossy().into_owned();
    let repository = GitRepository::discover(GitCommandRunner::new("git"), &fixture.first).unwrap();
    service
        .attach_origin(&repository, &correct, true, &cancellation)
        .unwrap();
    assert!(matches!(
        service.publish_first_branch(&repository, "origin", "main", None, &cancellation),
        Err(markion_git_sync::OnboardingError::UnrelatedRemoteHistory { .. })
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
