mod support;

use support::{TwoCloneFixture, empty_remote};

#[test]
fn creates_isolated_empty_remote() {
    let (_root, remote) = empty_remote();
    assert!(remote.join("HEAD").is_file());
    assert!(remote.join("objects").is_dir());
}

#[test]
fn creates_two_clones_with_deterministic_identity_and_divergence() {
    let fixture = TwoCloneFixture::new();
    fixture.diverge();
    assert!(fixture.first.join("first.md").is_file());
    assert!(fixture.second.join("second.md").is_file());
    assert!(fixture.remote.join("refs").is_dir());
}

#[test]
fn fixture_can_represent_shallow_and_linked_worktree_markers() {
    let fixture = TwoCloneFixture::new();
    // A linked worktree is represented by a `.git` file rather than a directory.
    let linked = fixture.first.parent().unwrap().join("linked");
    support::git(
        &fixture.first,
        [
            "worktree",
            "add",
            "-q",
            "--detach",
            linked.to_str().unwrap(),
        ],
    );
    assert!(linked.join(".git").is_file());
    let head = std::fs::read_to_string(fixture.first.join(".git").join("refs/heads/main"))
        .unwrap_or_else(|_| {
            let output = std::process::Command::new("git")
                .current_dir(&fixture.first)
                .args(["rev-parse", "HEAD"])
                .output()
                .unwrap();
            String::from_utf8(output.stdout).unwrap()
        });
    std::fs::write(fixture.first.join(".git").join("shallow"), head).unwrap();
    assert!(fixture.first.join(".git").join("shallow").is_file());
}
