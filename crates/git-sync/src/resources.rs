use std::path::{Component, Path, PathBuf};

use percent_encoding::percent_decode_str;
use pulldown_cmark::{Event, Options, Parser, Tag};
use sha2::{Digest, Sha256};

use crate::{GitRepository, NewFileClass, RepositoryError, RepositoryPolicy};

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
    let mut reports = inspect_notes_attachments(repository, policy, &[(note_path, markdown)])?;
    Ok(reports.pop().unwrap_or_default())
}

/// Inspect several notes at once. Every local image reference is resolved
/// first; the ignored/tracked classification then costs at most two Git
/// processes in total, independent of the number of notes and references.
pub fn inspect_notes_attachments(
    repository: &GitRepository,
    policy: &RepositoryPolicy,
    notes: &[(&Path, &str)],
) -> Result<Vec<AttachmentReport>, RepositoryError> {
    let root = &repository.identity().worktree_root;
    let mut pending = Vec::with_capacity(notes.len());
    let mut existing = Vec::new();
    for (note_path, markdown) in notes {
        let note_relative = if note_path.is_absolute() {
            note_path
                .strip_prefix(root)
                .map(Path::to_path_buf)
                .map_err(|_| RepositoryError::OutsideWorktree(note_path.to_path_buf()))?
        } else {
            note_path.to_path_buf()
        };
        let base = note_relative
            .parent()
            .unwrap_or(Path::new(""))
            .to_path_buf();
        let mut references = Vec::new();
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
            let resolved = percent_decode_str(authored_url.split(['?', '#']).next().unwrap_or(""))
                .decode_utf8()
                .map_err(|_| AttachmentIssueKind::Unsupported)
                .and_then(|decoded| {
                    normalize_relative(&base, Path::new(decoded.as_ref()))
                        .ok_or(AttachmentIssueKind::OutsideRepository)
                });
            if let Ok(relative) = &resolved
                && root.join(relative).is_file()
            {
                existing.push(relative.clone());
            }
            references.push((authored_url, fingerprint, acknowledged, resolved));
        }
        pending.push(references);
    }

    existing.sort();
    existing.dedup();
    let ignored = repository.ignored_among(&existing)?;
    let candidates: Vec<PathBuf> = existing
        .iter()
        .filter(|path| !ignored.contains(*path))
        .filter(|path| matches!(NewFileClass::classify(path), Some(NewFileClass::Image)))
        .cloned()
        .collect();
    let tracked = repository.tracked_among(&candidates)?;

    let mut reports = Vec::with_capacity(pending.len());
    for references in pending {
        let mut report = AttachmentReport::default();
        for (authored_url, fingerprint, acknowledged, resolved) in references {
            let relative = match resolved {
                Ok(relative) => relative,
                Err(kind) => {
                    report.issues.push(AttachmentIssue {
                        authored_url,
                        resolved_path: None,
                        kind,
                        reference_fingerprint: fingerprint,
                        previously_acknowledged: acknowledged,
                    });
                    continue;
                }
            };
            let kind = if !root.join(&relative).is_file() {
                Some(AttachmentIssueKind::Missing)
            } else if ignored.contains(&relative) {
                Some(AttachmentIssueKind::Ignored)
            } else if !matches!(NewFileClass::classify(&relative), Some(NewFileClass::Image)) {
                Some(AttachmentIssueKind::Unsupported)
            } else if tracked.contains(&relative)
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
        reports.push(report);
    }
    Ok(reports)
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
            allowed_extensions: Default::default(),
            included_once: Vec::new(),
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

    #[test]
    fn many_references_across_notes_cost_at_most_two_processes() {
        let dir = tempfile::tempdir().unwrap();
        run(dir.path(), ["init", "-q", "-b", "main"]);
        run(dir.path(), ["config", "user.name", "Markion Test"]);
        run(dir.path(), ["config", "user.email", "test@markion.invalid"]);
        fs::create_dir(dir.path().join("assets")).unwrap();
        fs::write(dir.path().join(".gitignore"), "assets/ignored-*\n").unwrap();
        let mut markdown = String::new();
        for index in 0..20 {
            fs::write(dir.path().join(format!("assets/img-{index}.png")), b"png").unwrap();
            markdown.push_str(&format!("![i](assets/img-{index}.png) "));
        }
        fs::write(dir.path().join("assets/ignored-a.png"), b"png").unwrap();
        fs::write(dir.path().join("assets/doc.pdf"), b"pdf").unwrap();
        markdown.push_str("![x](assets/ignored-a.png) ![d](assets/doc.pdf) ![m](assets/gone.png)");
        run(dir.path(), ["add", "--", "assets/img-0.png"]);
        run(dir.path(), ["commit", "-q", "-m", "one tracked image"]);

        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let repository = GitRepository::discover(
            GitCommandRunner::new("git").with_invocation_counter(counter.clone()),
            dir.path(),
        )
        .unwrap();
        let mut policy = test_policy(&repository);
        policy.new_file_rules[0].root = PathBuf::from("elsewhere");
        counter.store(0, std::sync::atomic::Ordering::Relaxed);
        let reports = inspect_notes_attachments(
            &repository,
            &policy,
            &[
                (Path::new("a.md"), markdown.as_str()),
                (Path::new("b.md"), markdown.as_str()),
            ],
        )
        .unwrap();
        assert!(counter.load(std::sync::atomic::Ordering::Relaxed) <= 2);
        assert_eq!(reports.len(), 2);
        let report = &reports[0];
        assert_eq!(
            report.selected_resources,
            [PathBuf::from("assets/img-0.png")]
        );
        let kind_of = |url: &str| {
            report
                .issues
                .iter()
                .find(|issue| issue.authored_url == url)
                .map(|issue| issue.kind)
        };
        assert_eq!(
            kind_of("assets/img-1.png"),
            Some(AttachmentIssueKind::OutsidePolicy)
        );
        assert_eq!(
            kind_of("assets/ignored-a.png"),
            Some(AttachmentIssueKind::Ignored)
        );
        assert_eq!(
            kind_of("assets/doc.pdf"),
            Some(AttachmentIssueKind::Unsupported)
        );
        assert_eq!(
            kind_of("assets/gone.png"),
            Some(AttachmentIssueKind::Missing)
        );
        // Same references from another note classify the same way (only the
        // note-scoped reference fingerprints differ).
        let classify = |report: &AttachmentReport| {
            report
                .issues
                .iter()
                .map(|issue| (issue.resolved_path.clone(), issue.kind))
                .collect::<Vec<_>>()
        };
        assert_eq!(reports[1].selected_resources, reports[0].selected_resources);
        assert_eq!(classify(&reports[1]), classify(&reports[0]));
    }

    fn test_policy(repository: &GitRepository) -> RepositoryPolicy {
        RepositoryPolicy {
            identity: repository.identity().clone(),
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
            acknowledged_resource_omissions: Vec::new(),
            allowed_extensions: Default::default(),
            included_once: Vec::new(),
        }
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
