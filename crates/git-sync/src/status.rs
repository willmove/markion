use std::path::PathBuf;

#[cfg(unix)]
use std::ffi::OsString;

use crate::{ChangeKind, ConflictKind, FileChange, GitObjectId, WorktreeState};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BranchStatus {
    pub head: Option<String>,
    pub oid: Option<GitObjectId>,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedStatus {
    pub branch: BranchStatus,
    pub worktree: WorktreeState,
}

#[derive(Debug, thiserror::Error)]
pub enum StatusParseError {
    #[error("malformed porcelain-v2 record: {0}")]
    Malformed(String),
    #[error("Git path is not representable on this platform")]
    UnrepresentablePath,
    #[error("invalid object id in branch header")]
    InvalidObjectId,
}

pub fn parse_porcelain_v2(bytes: &[u8]) -> Result<ParsedStatus, StatusParseError> {
    let records: Vec<&[u8]> = bytes.split(|byte| *byte == 0).collect();
    let mut branch = BranchStatus::default();
    let mut changes = Vec::new();
    let mut index = 0;
    while index < records.len() {
        let record = records[index];
        index += 1;
        if record.is_empty() {
            continue;
        }
        match record[0] {
            b'#' => parse_header(record, &mut branch)?,
            b'1' => changes.push(parse_ordinary(record)?),
            b'2' => {
                let original = records.get(index).ok_or_else(|| {
                    StatusParseError::Malformed("rename record lacks original path".into())
                })?;
                index += 1;
                changes.push(parse_rename(record, original)?);
            }
            b'u' => changes.push(parse_unmerged(record)?),
            b'?' => changes.push(FileChange {
                path: parse_prefixed_path(record, 2)?,
                original_path: None,
                kind: ChangeKind::Untracked,
                index_status: '?',
                worktree_status: '?',
                conflict: None,
            }),
            b'!' => {}
            other => {
                return Err(StatusParseError::Malformed(format!(
                    "unknown record kind {other}"
                )));
            }
        }
    }
    let has_conflicts = changes.iter().any(|change| change.conflict.is_some());
    let has_external_staging = changes
        .iter()
        .any(|change| change.index_status != '.' && change.index_status != '?');
    Ok(ParsedStatus {
        branch,
        worktree: WorktreeState {
            changes,
            has_external_staging,
            has_conflicts,
            operation_in_progress: None,
        },
    })
}

fn parse_header(record: &[u8], branch: &mut BranchStatus) -> Result<(), StatusParseError> {
    let text = std::str::from_utf8(record)
        .map_err(|_| StatusParseError::Malformed("non-UTF-8 branch header".into()))?;
    if let Some(value) = text.strip_prefix("# branch.oid ") {
        if value != "(initial)" {
            branch.oid =
                Some(GitObjectId::parse(value).map_err(|_| StatusParseError::InvalidObjectId)?);
        }
    } else if let Some(value) = text.strip_prefix("# branch.head ") {
        if value != "(detached)" {
            branch.head = Some(value.to_owned());
        }
    } else if let Some(value) = text.strip_prefix("# branch.upstream ") {
        branch.upstream = Some(value.to_owned());
    } else if let Some(value) = text.strip_prefix("# branch.ab ") {
        let mut parts = value.split_whitespace();
        branch.ahead = parse_count(parts.next(), '+')?;
        branch.behind = parse_count(parts.next(), '-')?;
    }
    Ok(())
}

fn parse_count(value: Option<&str>, prefix: char) -> Result<usize, StatusParseError> {
    value
        .and_then(|value| value.strip_prefix(prefix))
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| StatusParseError::Malformed("invalid branch ahead/behind header".into()))
}

fn parse_ordinary(record: &[u8]) -> Result<FileChange, StatusParseError> {
    let fields = split_prefix(record, 9)?;
    build_change(fields[1], fields[8], None, false)
}

fn parse_rename(record: &[u8], original: &[u8]) -> Result<FileChange, StatusParseError> {
    let fields = split_prefix(record, 10)?;
    build_change(fields[1], fields[9], Some(original), true)
}

fn parse_unmerged(record: &[u8]) -> Result<FileChange, StatusParseError> {
    let fields = split_prefix(record, 11)?;
    let mut change = build_change(fields[1], fields[10], None, false)?;
    change.kind = ChangeKind::Unmerged;
    change.conflict = Some(conflict_kind(change.index_status, change.worktree_status));
    Ok(change)
}

fn split_prefix(record: &[u8], count: usize) -> Result<Vec<&[u8]>, StatusParseError> {
    let fields: Vec<_> = record.splitn(count, |byte| *byte == b' ').collect();
    if fields.len() != count {
        return Err(StatusParseError::Malformed(format!(
            "expected {count} fields, got {}",
            fields.len()
        )));
    }
    Ok(fields)
}

fn build_change(
    xy: &[u8],
    path: &[u8],
    original: Option<&[u8]>,
    rename_record: bool,
) -> Result<FileChange, StatusParseError> {
    if xy.len() != 2 {
        return Err(StatusParseError::Malformed("invalid XY status".into()));
    }
    let index_status = xy[0] as char;
    let worktree_status = xy[1] as char;
    Ok(FileChange {
        path: path_from_git_bytes(path)?,
        original_path: original.map(path_from_git_bytes).transpose()?,
        kind: if rename_record {
            if index_status == 'C' || worktree_status == 'C' {
                ChangeKind::Copied
            } else {
                ChangeKind::Renamed
            }
        } else {
            change_kind(index_status, worktree_status)
        },
        index_status,
        worktree_status,
        conflict: None,
    })
}

fn change_kind(index: char, worktree: char) -> ChangeKind {
    let status = if worktree != '.' { worktree } else { index };
    match status {
        'A' => ChangeKind::Added,
        'D' => ChangeKind::Deleted,
        'R' => ChangeKind::Renamed,
        'C' => ChangeKind::Copied,
        'T' => ChangeKind::TypeChanged,
        'U' => ChangeKind::Unmerged,
        _ => ChangeKind::Modified,
    }
}

fn conflict_kind(index: char, worktree: char) -> ConflictKind {
    match (index, worktree) {
        ('A', 'A') => ConflictKind::BothAdded,
        ('D', 'D') => ConflictKind::BothDeleted,
        ('D', _) => ConflictKind::DeletedByLocal,
        (_, 'D') => ConflictKind::DeletedByRemote,
        ('R', _) | (_, 'R') => ConflictKind::Rename,
        ('U', 'U') => ConflictKind::BothModified,
        _ => ConflictKind::Other,
    }
}

fn parse_prefixed_path(record: &[u8], prefix: usize) -> Result<PathBuf, StatusParseError> {
    if record.len() < prefix {
        return Err(StatusParseError::Malformed("missing path".into()));
    }
    path_from_git_bytes(&record[prefix..])
}

#[cfg(unix)]
fn path_from_git_bytes(bytes: &[u8]) -> Result<PathBuf, StatusParseError> {
    use std::os::unix::ffi::OsStringExt;
    Ok(PathBuf::from(OsString::from_vec(bytes.to_vec())))
}

#[cfg(windows)]
fn path_from_git_bytes(bytes: &[u8]) -> Result<PathBuf, StatusParseError> {
    let text =
        String::from_utf8(bytes.to_vec()).map_err(|_| StatusParseError::UnrepresentablePath)?;
    Ok(PathBuf::from(text))
}

#[cfg(not(any(unix, windows)))]
fn path_from_git_bytes(bytes: &[u8]) -> Result<PathBuf, StatusParseError> {
    let text =
        String::from_utf8(bytes.to_vec()).map_err(|_| StatusParseError::UnrepresentablePath)?;
    Ok(PathBuf::from(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_ordinary_untracked_and_rename_records() {
        let input = b"# branch.oid 0123456789abcdef\x00# branch.head main\x00# branch.upstream origin/main\x00# branch.ab +2 -3\x001 .M N... 100644 100644 100644 0123456 0123456 notes/a.md\x002 R. N... 100644 100644 100644 0123456 0123456 R100 notes/new.md\x00notes/old.md\x00? hidden.txt\x00";
        let parsed = parse_porcelain_v2(input).unwrap();
        assert_eq!(parsed.branch.head.as_deref(), Some("main"));
        assert_eq!((parsed.branch.ahead, parsed.branch.behind), (2, 3));
        assert_eq!(parsed.worktree.changes.len(), 3);
        assert_eq!(
            parsed.worktree.changes[1].original_path.as_deref(),
            Some(std::path::Path::new("notes/old.md"))
        );
        assert_eq!(parsed.worktree.changes[2].kind, ChangeKind::Untracked);
    }

    #[test]
    fn index_status_is_reported_separately() {
        let parsed = parse_porcelain_v2(
            b"# branch.oid 0123456789abcdef\x00# branch.head main\x001 M. N... 100644 100644 100644 0123456 0123456 staged.md\x00",
        )
        .unwrap();
        assert!(parsed.worktree.has_external_staging);
    }
}
