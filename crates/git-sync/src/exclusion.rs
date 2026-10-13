//! Device-local exclusions: entries Markion writes to `info/exclude` so a
//! held-back file stops being proposed on this computer without creating a
//! repository change or affecting other devices.
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::persist::atomic_write;

/// Comment line written before every entry Markion owns. Only entries that
/// follow it are listed or removed; hand-written lines are never touched.
const MARKER: &str = "# markion: device-local exclusion";

fn exclude_file(common_dir: &Path) -> PathBuf {
    common_dir.join("info").join("exclude")
}

fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text.lines().map(str::to_owned).collect()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn write_lines(path: &Path, lines: &[String]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text = lines.join("\n");
    if !text.is_empty() {
        text.push('\n');
    }
    atomic_write(path, text.as_bytes())
}

/// Repository-relative paths Markion excluded on this device.
pub fn device_exclusions(common_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let lines = read_lines(&exclude_file(common_dir))?;
    Ok(lines
        .windows(2)
        .filter(|pair| pair[0] == MARKER)
        .filter_map(|pair| unescape_pattern(&pair[1]))
        .collect())
}

/// Exclude one repository-relative path on this device. Adding an existing
/// exclusion is a no-op.
pub fn add_device_exclusion(common_dir: &Path, path: &Path) -> io::Result<()> {
    let file = exclude_file(common_dir);
    let mut lines = read_lines(&file)?;
    let pattern = escape_pattern(path);
    if lines
        .windows(2)
        .any(|pair| pair[0] == MARKER && pair[1] == pattern)
    {
        return Ok(());
    }
    lines.push(MARKER.to_owned());
    lines.push(pattern);
    write_lines(&file, &lines)
}

/// Remove a Markion-owned exclusion (its marker and entry).
pub fn remove_device_exclusion(common_dir: &Path, path: &Path) -> io::Result<()> {
    let file = exclude_file(common_dir);
    let lines = read_lines(&file)?;
    let pattern = escape_pattern(path);
    let mut kept = Vec::with_capacity(lines.len());
    let mut index = 0;
    while index < lines.len() {
        if lines[index] == MARKER && lines.get(index + 1) == Some(&pattern) {
            index += 2;
            continue;
        }
        kept.push(lines[index].clone());
        index += 1;
    }
    write_lines(&file, &kept)
}

/// Anchored, literal ignore pattern for one path. The leading `/` anchors it
/// to the repository root and keeps a leading `#` or `!` from being read as a
/// comment or negation.
fn escape_pattern(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let trailing_spaces = text.len() - text.trim_end_matches(' ').len();
    let body = &text[..text.len() - trailing_spaces];
    let mut pattern = String::from("/");
    for character in body.chars() {
        if matches!(character, '*' | '?' | '[' | '\\') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    for _ in 0..trailing_spaces {
        pattern.push_str("\\ ");
    }
    pattern
}

fn unescape_pattern(pattern: &str) -> Option<PathBuf> {
    let body = pattern.strip_prefix('/')?;
    let mut text = String::with_capacity(body.len());
    let mut characters = body.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            text.push(characters.next()?);
        } else {
            text.push(character);
        }
    }
    (!text.is_empty()).then(|| PathBuf::from(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CancellationToken, CommandLimits, GitCommand, GitCommandRunner};

    fn untracked(root: &Path) -> Vec<String> {
        let output = GitCommandRunner::new("git")
            .run(
                &GitCommand::new(root).args([
                    "status",
                    "--porcelain",
                    "-z",
                    "--untracked-files=all",
                ]),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .unwrap();
        output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|record| !record.is_empty())
            .map(|record| String::from_utf8_lossy(&record[3..]).into_owned())
            .collect()
    }

    #[test]
    fn exclusions_hide_literal_paths_and_removal_restores_them() {
        let dir = tempfile::tempdir().unwrap();
        GitCommandRunner::new("git")
            .run(
                &GitCommand::new(dir.path()).args(["init", "-q"]),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .unwrap();
        let common = dir.path().join(".git");
        fs::write(
            common.join("info").join("exclude"),
            "# hand written\n*.tmp\n",
        )
        .unwrap();
        let names = [
            "star[1].pdf",
            "#hash.pdf",
            "!bang.pdf",
            "中文 资料.pdf",
            "trail .pdf",
        ];
        fs::create_dir(dir.path().join("docs")).unwrap();
        for name in names {
            fs::write(dir.path().join("docs").join(name), "x").unwrap();
        }
        fs::write(dir.path().join("docs").join("star1.pdf"), "x").unwrap();
        for name in names {
            add_device_exclusion(&common, &Path::new("docs").join(name)).unwrap();
        }
        add_device_exclusion(&common, Path::new("docs/#hash.pdf")).unwrap();

        assert_eq!(untracked(dir.path()), ["docs/star1.pdf"]);
        let listed = device_exclusions(&common).unwrap();
        assert_eq!(listed.len(), names.len());
        assert!(listed.contains(&PathBuf::from("docs/star[1].pdf")));

        remove_device_exclusion(&common, Path::new("docs/star[1].pdf")).unwrap();
        let mut visible = untracked(dir.path());
        visible.sort();
        assert_eq!(visible, ["docs/star1.pdf", "docs/star[1].pdf"]);
        let text = fs::read_to_string(common.join("info").join("exclude")).unwrap();
        assert!(text.starts_with("# hand written\n*.tmp\n"));
    }

    #[test]
    fn patterns_round_trip() {
        for name in ["a/b.md", "x[1]*?.pdf", "back\\slash", "trail  "] {
            let path = Path::new(name);
            let unescaped = unescape_pattern(&escape_pattern(path)).unwrap();
            assert_eq!(unescaped, PathBuf::from(name.replace('\\', "/")));
        }
    }
}
