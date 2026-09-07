#![allow(dead_code)]

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::TempDir;

pub struct TwoCloneFixture {
    _root: TempDir,
    pub remote: PathBuf,
    pub first: PathBuf,
    pub second: PathBuf,
}

impl TwoCloneFixture {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let remote = root.path().join("remote.git");
        git(
            root.path(),
            ["init", "--bare", "-q", remote.to_str().unwrap()],
        );
        let seed = root.path().join("seed");
        git(
            root.path(),
            ["init", "-q", "-b", "main", seed.to_str().unwrap()],
        );
        configure_identity(&seed);
        std::fs::write(seed.join("notes.md"), "initial\n").unwrap();
        git(&seed, ["add", "--", "notes.md"]);
        git(&seed, ["commit", "-q", "-m", "initial"]);
        git(&seed, ["remote", "add", "origin", remote.to_str().unwrap()]);
        git(&seed, ["push", "-q", "-u", "origin", "main"]);
        git(&remote, ["symbolic-ref", "HEAD", "refs/heads/main"]);
        let first = root.path().join("first clone");
        let second = root.path().join("second-中文");
        git(
            root.path(),
            [
                "clone",
                "-q",
                remote.to_str().unwrap(),
                first.to_str().unwrap(),
            ],
        );
        git(
            root.path(),
            [
                "clone",
                "-q",
                remote.to_str().unwrap(),
                second.to_str().unwrap(),
            ],
        );
        configure_identity(&first);
        configure_identity(&second);
        Self {
            _root: root,
            remote,
            first,
            second,
        }
    }

    pub fn diverge(&self) {
        std::fs::write(self.first.join("first.md"), "first\n").unwrap();
        git(&self.first, ["add", "--", "first.md"]);
        git(&self.first, ["commit", "-q", "-m", "first"]);
        std::fs::write(self.second.join("second.md"), "second\n").unwrap();
        git(&self.second, ["add", "--", "second.md"]);
        git(&self.second, ["commit", "-q", "-m", "second"]);
    }
}

pub fn empty_remote() -> (TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let remote = root.path().join("empty.git");
    git(
        root.path(),
        ["init", "--bare", "-q", remote.to_str().unwrap()],
    );
    (root, remote)
}

pub fn configure_identity(directory: &Path) {
    git(directory, ["config", "user.name", "Markion Test"]);
    git(directory, ["config", "user.email", "test@markion.invalid"]);
    git(directory, ["config", "commit.gpgsign", "false"]);
}

pub fn git<I, S>(directory: &Path, arguments: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new("git")
        .current_dir(directory)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", empty_global_config_path(directory))
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn empty_global_config_path(directory: &Path) -> PathBuf {
    directory.join(".markion-test-global-config-does-not-exist")
}
