use std::path::{Component, Path, PathBuf};

use percent_encoding::percent_decode_str;
use pulldown_cmark::{Event, Options, Parser, Tag};
use sha2::{Digest, Sha256};

use crate::{
    CancellationToken, CommandLimits, GitCommand, GitRepository, NewFileClass, RepositoryError,
    RepositoryPolicy,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentIssueKind {
    Ignored,
    Missing,
    OutsideRepository,
    OutsidePolicy,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachmentIssue {
    pub authored_url: String,
    pub resolved_path: Option<PathBuf>,
    pub kind: AttachmentIssueKind,
    pub reference_fingerprint: String,
    pub previously_acknowledged: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AttachmentReport {
    pub selected_resources: Vec<PathBuf>,
    pub issues: Vec<AttachmentIssue>,
}

pub fn inspect_note_attachments(
    repository: &GitRepository,
    policy: &RepositoryPolicy,
    note_path: &Path,
    markdown: &str,
) -> Result<AttachmentReport, RepositoryError> {
    let root = &repository.identity().worktree_root;
    let note_relative = if note_path.is_absolute() {
        note_path
            .strip_prefix(root)
            .map(Path::to_path_buf)
            .map_err(|_| RepositoryError::OutsideWorktree(note_path.to_path_buf()))?
    } else {
        note_path.to_path_buf()
    };
    let base = note_relative.parent().unwrap_or(Path::new(""));
    let mut report = AttachmentReport::default();
    let parser = Parser::new_ext(markdown, Options::all());
    for event in parser {
        let Event::Start(Tag::Image { dest_url, .. }) = event else {
            continue;
        };
        let authored_url = dest_url.into_string();
        if is_nonlocal_url(&authored_url) {
            continue;
        }
        let fingerprint = attachment_reference_fingerprint(&note_relative, &authored_url);
        let acknowledged = policy
            .acknowledged_resource_omissions
            .iter()
            .any(|item| item.path == note_relative && item.content_fingerprint == fingerprint);
        let decoded = match percent_decode_str(authored_url.split(['?', '#']).next().unwrap_or(""))
            .decode_utf8()
        {
            Ok(decoded) => decoded,
            Err(_) => {
                report.issues.push(AttachmentIssue {
                    authored_url,
                    resolved_path: None,
                    kind: AttachmentIssueKind::Unsupported,
                    reference_fingerprint: fingerprint,
                    previously_acknowledged: acknowledged,
                });
                continue;
            }
        };
        let decoded_path = Path::new(decoded.as_ref());
        let Some(relative) = normalize_relative(base, decoded_path) else {
            report.issues.push(AttachmentIssue {
                authored_url,
                resolved_path: None,
                kind: AttachmentIssueKind::OutsideRepository,
                reference_fingerprint: fingerprint,
                previously_acknowledged: acknowledged,
            });
            continue;
        };
        let absolute = root.join(&relative);
        let kind = if !absolute.is_file() {
            Some(AttachmentIssueKind::Missing)
        } else if is_ignored(repository, &relative) {
            Some(AttachmentIssueKind::Ignored)
        } else if !matches!(NewFileClass::classify(&relative), Some(NewFileClass::Image)) {
            Some(AttachmentIssueKind::Unsupported)
        } else if is_tracked(repository, &relative)
            || policy.new_path_allowed(&relative, false, false)
        {
            report.selected_resources.push(relative.clone());
            None
        } else {
            Some(AttachmentIssueKind::OutsidePolicy)
        };
        if let Some(kind) = kind {
            report.issues.push(AttachmentIssue {
                authored_url,
                resolved_path: Some(relative),
                kind,
                reference_fingerprint: fingerprint,
                previously_acknowledged: acknowledged,
            });
        }
    }
    report.selected_resources.sort();
    report.selected_resources.dedup();
    Ok(report)
}

pub fn attachment_reference_fingerprint(note_path: &Path, authored_url: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(note_path.to_string_lossy().as_bytes());
    digest.update([0]);
    digest.update(authored_url.as_bytes());
    format!("{:x}", digest.finalize())
}

fn is_nonlocal_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("mailto:")
        || url.starts_with('#')
}

fn normalize_relative(base: &Path, path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return None;
    }
    let mut components = Vec::new();
    for component in base.components().chain(path.components()) {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => components.push(value.to_owned()),
            Component::ParentDir => {
                components.pop()?;
            }
            Component::Prefix(_) | Component::RootDir => return None,
        }
    }
    let mut normalized = PathBuf::new();
    normalized.extend(components);
    Some(normalized)
}

fn is_ignored(repository: &GitRepository, path: &Path) -> bool {
    repository
        .runner()
        .run(
            &GitCommand::new(&repository.identity().worktree_root)
                .args(["check-ignore", "-q"])
                .literal_paths([path])
                .read_only(true),
            CommandLimits::default(),
            &CancellationToken::new(),
        )
        .is_ok()
}

fn is_tracked(repository: &GitRepository, path: &Path) -> bool {
    repository
        .runner()
        .run(
            &GitCommand::new(&repository.identity().worktree_root)
                .args(["ls-files", "--error-unmatch"])
                .literal_paths([path])
                .read_only(true),
            CommandLimits::default(),
            &CancellationToken::new(),
        )
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GitCommandRunner, NewFileRule, RepositoryIdentity, ScopeException, SyncTarget};
    use std::{collections::BTreeSet, fs};

    #[test]
    fn detects_selected_missing_outside_and_unchanged_acknowledgments() {
        let dir = tempfile::tempdir().unwrap();
        run(dir.path(), ["init", "-q", "-b", "main"]);
        fs::create_dir(dir.path().join("notes")).unwrap();
        fs::create_dir(dir.path().join("assets")).unwrap();
        fs::write(dir.path().join("assets/photo.png"), b"png").unwrap();
        let repository = GitRepository::discover(GitCommandRunner::new("git"), dir.path()).unwrap();
        let missing_fingerprint =
            attachment_reference_fingerprint(Path::new("notes/today.md"), "../assets/missing.png");
        let policy = RepositoryPolicy {
            identity: RepositoryIdentity::new(
                repository.identity().worktree_root.clone(),
                repository.identity().git_dir.clone(),
                repository.identity().common_dir.clone(),
            ),
            target: SyncTarget {
                local_branch: "main".into(),
                remote: "origin".into(),
                remote_branch: "main".into(),
                destination_ref: "refs/heads/main".into(),
                fetch_url: "remote".into(),
                push_url: "remote".into(),
            },
            tracked_roots: vec![PathBuf::new()],
            new_file_rules: vec![NewFileRule {
                root: PathBuf::new(),
                classes: BTreeSet::from([NewFileClass::Image]),
            }],
            message_template: String::new(),
            include_device_name: false,
            background_fetch: false,
            last_confirmed_remote: None,
            acknowledged_resource_omissions: vec![ScopeException {
                path: PathBuf::from("notes/today.md"),
                content_fingerprint: missing_fingerprint,
            }],
        };
        let report = inspect_note_attachments(
            &repository,
            &policy,
            Path::new("notes/today.md"),
            "![ok](../assets/photo.png) ![missing](../assets/missing.png) ![outside](../../x.png) ![remote](https://example.com/x.png)",
        )
        .unwrap();
        assert_eq!(
            report.selected_resources,
            [PathBuf::from("assets/photo.png")]
        );
        assert_eq!(report.issues.len(), 2);
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.previously_acknowledged)
        );
    }

    fn run<const N: usize>(directory: &Path, arguments: [&str; N]) {
        let output = std::process::Command::new("git")
            .current_dir(directory)
            .args(arguments)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
}
