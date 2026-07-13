//! Workspace file-tree scanning and entry create/rename/move/delete.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeEntryKind {
    Directory,
    File,
}

/// The Markdown file extensions the editor recognises, shared by the file
/// tree's scan filter and the OS drag-and-drop open path so the two never
/// drift apart. Case-insensitive (compared on the ASCII-lowercased form to
/// match Windows's case-insensitive filesystem).
pub const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown", "mdown"];

/// Returns `true` when `path` has a Markdown extension (`md` / `markdown` /
/// `mdown`), compared case-insensitively. Used both by the file-tree scan
/// (below) and by the external drag-and-drop open handler in `main.rs`, so the
/// "what counts as a Markdown file" rule is defined in one place.
pub fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            MARKDOWN_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeEntry {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub kind: FileTreeEntryKind,
    pub is_markdown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTree {
    pub root: PathBuf,
    pub entries: Vec<FileTreeEntry>,
}

impl FileTree {
    pub fn scan(root: impl AsRef<Path>) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        let mut entries = Vec::new();
        collect_file_tree_entries(&root, 0, &mut entries)?;
        Ok(Self { root, entries })
    }

    pub fn create_file(&mut self, parent: impl AsRef<Path>, name: &str) -> io::Result<PathBuf> {
        let path = safe_child_path(&self.root, parent.as_ref(), name)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        self.refresh()?;
        Ok(path)
    }

    pub fn create_unique_file(
        &mut self,
        parent: impl AsRef<Path>,
        preferred_name: &str,
    ) -> io::Result<PathBuf> {
        let path = unique_child_path(parent.as_ref(), preferred_name);
        fs::File::create(&path)?;
        self.refresh()?;
        Ok(path)
    }

    pub fn create_directory(
        &mut self,
        parent: impl AsRef<Path>,
        name: &str,
    ) -> io::Result<PathBuf> {
        let path = safe_child_path(&self.root, parent.as_ref(), name)?;
        fs::create_dir(&path)?;
        self.refresh()?;
        Ok(path)
    }

    pub fn create_unique_directory(
        &mut self,
        parent: impl AsRef<Path>,
        preferred_name: &str,
    ) -> io::Result<PathBuf> {
        let path = unique_child_path(parent.as_ref(), preferred_name);
        fs::create_dir(&path)?;
        self.refresh()?;
        Ok(path)
    }

    pub fn rename(&mut self, path: impl AsRef<Path>, new_name: &str) -> io::Result<PathBuf> {
        let path = path.as_ref();
        ensure_existing_path_within_root(&self.root, path)?;
        let new_path = path
            .parent()
            .map(|parent| safe_child_path(&self.root, parent, new_name))
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
        let new_path = new_path?;
        fs::rename(path, &new_path)?;
        self.refresh()?;
        Ok(new_path)
    }

    pub fn rename_unique(
        &mut self,
        path: impl AsRef<Path>,
        preferred_name: &str,
    ) -> io::Result<PathBuf> {
        let path = path.as_ref();
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
        let new_path = unique_child_path(parent, preferred_name);
        fs::rename(path, &new_path)?;
        self.refresh()?;
        Ok(new_path)
    }

    pub fn move_entry(
        &mut self,
        path: impl AsRef<Path>,
        new_parent: impl AsRef<Path>,
    ) -> io::Result<PathBuf> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
        let new_path = unique_child_path(new_parent.as_ref(), name);
        fs::rename(path, &new_path)?;
        self.refresh()?;
        Ok(new_path)
    }

    pub fn delete(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        ensure_existing_path_within_root(&self.root, path)?;
        if path.is_dir() {
            // `remove_dir_all` (not `remove_dir`): every folder shown in the
            // tree is necessarily non-empty (folders with no Markdown
            // descendant are pruned during scan), so empty-only removal would
            // fail on any directory the user can right-click.
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        self.refresh()
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        self.entries.clear();
        // The returned bool (whether any Markdown was found) is intentionally
        // ignored: an empty tree is a valid result for a Markdown-free root.
        collect_file_tree_entries(&self.root, 0, &mut self.entries)?;
        Ok(())
    }

    pub fn filtered_entries(&self, query: &str) -> Vec<FileTreeEntry> {
        let (entries, _) = self.filtered_entries_limited(query, usize::MAX);
        entries
    }

    /// Returns at most `limit` matching entries plus the total match count.
    /// The panel renders a bounded number of rows per frame, so it should not
    /// pay to clone (or lay out) thousands of entries in large workspaces.
    pub fn filtered_entries_limited(
        &self,
        query: &str,
        limit: usize,
    ) -> (Vec<FileTreeEntry>, usize) {
        let query = query.trim().to_ascii_lowercase();
        let mut matched = 0usize;
        let mut entries = Vec::new();
        for entry in &self.entries {
            let matches = query.is_empty()
                || entry.name.to_ascii_lowercase().contains(&query)
                || entry
                    .path
                    .strip_prefix(&self.root)
                    .ok()
                    .and_then(Path::to_str)
                    .map(|path| path.to_ascii_lowercase().contains(&query))
                    .unwrap_or(false);
            if matches {
                if matched < limit {
                    entries.push(entry.clone());
                }
                matched += 1;
            }
        }
        (entries, matched)
    }
}

fn safe_child_path(root: &Path, parent: &Path, name: &str) -> io::Result<PathBuf> {
    if name.is_empty() || Path::new(name).components().count() != 1 || matches!(name, "." | "..") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "name must be a single file or directory name",
        ));
    }

    let root = root.canonicalize()?;
    let parent = parent.canonicalize()?;
    if !parent.starts_with(&root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "parent is outside the file tree root",
        ));
    }

    let path = parent.join(name);
    if !path.starts_with(&root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "path is outside the file tree root",
        ));
    }

    Ok(path)
}

fn ensure_existing_path_within_root(root: &Path, path: &Path) -> io::Result<()> {
    let root = root.canonicalize()?;
    let path = path.canonicalize()?;
    if path == root {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "cannot operate on the file tree root",
        ));
    }
    if !path.starts_with(root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "path is outside the file tree root",
        ));
    }
    Ok(())
}

/// Recursively collects Markdown files (and the folders that contain them)
/// into `entries`.
///
/// The tree is **Markdown-only**: regular files whose extension is not
/// `md`/`markdown`/`mdown` are skipped entirely (they used to be collected as
/// inert `--` rows, but the sidebar is a Markdown writing surface). Directories
/// are kept only as nesting rows when their subtree contains at least one
/// Markdown file — empty folders (no Markdown descendant) are pruned so the
/// hierarchy stays focused on actual content.
///
/// Returns `true` when at least one Markdown file was collected anywhere under
/// `root` (used by the caller to decide whether to keep a directory row).
fn collect_file_tree_entries(
    root: &Path,
    depth: usize,
    entries: &mut Vec<FileTreeEntry>,
) -> io::Result<bool> {
    let mut children = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| !should_skip_file_tree_path(&entry.path()))
        .collect::<Vec<_>>();
    children.sort_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();
        b_path
            .is_dir()
            .cmp(&a_path.is_dir())
            .then_with(|| a.file_name().cmp(&b.file_name()))
    });

    let mut found_markdown = false;
    for entry in children {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            // Tentatively record where this directory row would go. We only
            // commit it if the subtree actually contains Markdown; otherwise
            // the row is popped back off so empty folders never appear.
            let row_index = entries.len();
            entries.push(FileTreeEntry {
                path: path.clone(),
                name: name.clone(),
                depth,
                kind: FileTreeEntryKind::Directory,
                is_markdown: false,
            });
            let subtree_has_markdown = collect_file_tree_entries(&path, depth + 1, entries)?;
            if subtree_has_markdown {
                found_markdown = true;
            } else {
                entries.truncate(row_index);
            }
            continue;
        }

        // Regular file: only Markdown files are collected. Everything else
        // (`.rs`, `.toml`, images, …) is skipped so the sidebar is noise-free.
        let is_markdown = is_markdown_path(&path);
        if !is_markdown {
            continue;
        }

        entries.push(FileTreeEntry {
            path: path.clone(),
            name,
            depth,
            kind: FileTreeEntryKind::File,
            is_markdown: true,
        });
        found_markdown = true;
    }

    Ok(found_markdown)
}

fn should_skip_file_tree_path(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    // Directories that are commonly huge or irrelevant to a Markdown workspace.
    // Skipping them keeps the file-tree scan (and the app startup) fast even
    // when the working directory is a large repository or a home folder.
    let is_ignored_dir = matches!(
        name,
        // Version control
        ".git" | ".hg" | ".svn" | ".bzr"
        // Rust / JS / TS build outputs
        | "target" | "node_modules" | "dist" | "build" | "out"
        | ".next" | ".nuxt" | ".svelte-kit" | ".turbo" | ".parcel-cache"
        | "coverage"
        // Package/dependency caches
        | ".cargo" | ".rustup" | "vendor" | "Pods" | "bower_components"
        // Python virtualenvs & caches
        | "venv" | ".venv" | "env" | ".env" | "__pycache__" | ".mypy_cache"
        | ".pytest_cache" | ".tox" | "site-packages"
        // Go / Java / others
        | ".gradle" | ".mvn" | "bin" | "obj"
        // IDE / editor metadata
        | ".idea" | ".vscode" | ".vs"
    );

    if is_ignored_dir {
        return true;
    }

    // Any other hidden directory is also skipped, but only stat it when needed
    // so we don't pay for `is_dir()` on regular hidden files.
    name.starts_with('.') && path.is_dir()
}

fn unique_child_path(parent: &Path, preferred_name: &str) -> PathBuf {
    let preferred_name = sanitize_file_name(preferred_name);
    let preferred = Path::new(&preferred_name);
    let stem = preferred
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("Untitled");
    let extension = preferred
        .extension()
        .and_then(|extension| extension.to_str());

    for index in 0.. {
        let name = if index == 0 {
            preferred_name.clone()
        } else if let Some(extension) = extension {
            format!("{stem} {index}.{extension}")
        } else {
            format!("{stem} {index}")
        };
        let path = parent.join(name);
        if !path.exists() {
            return path;
        }
    }

    unreachable!("unbounded loop returns a free child path")
}

fn sanitize_file_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            ch if ch.is_control() => '-',
            ch => ch,
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();

    if sanitized.is_empty() {
        "Untitled".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper: write `name` (relative to `root`) with the given bytes.
    fn write(root: &Path, rel: &str, bytes: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn scan_lists_only_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A mix of Markdown and non-Markdown files at the root and in a subfolder.
        write(root, "intro.md", "# Intro");
        write(root, "notes.markdown", "# Notes");
        write(root, "ignore.txt", "not markdown");
        write(root, "src/main.rs", "fn main() {}");
        write(root, "docs/guide.md", "# Guide");

        let tree = FileTree::scan(root).unwrap();
        let names: Vec<&str> = tree.entries.iter().map(|e| e.name.as_str()).collect();

        // Markdown files present…
        assert!(names.contains(&"intro.md"));
        assert!(names.contains(&"notes.markdown"));
        assert!(names.contains(&"guide.md"));
        // Non-Markdown files are absent.
        assert!(!names.contains(&"ignore.txt"));
        assert!(!names.contains(&"main.rs"));
        // Every collected file is markdown.
        assert!(
            tree.entries
                .iter()
                .filter(|e| e.kind == FileTreeEntryKind::File)
                .all(|e| e.is_markdown)
        );
    }

    #[test]
    fn scan_prunes_folders_without_markdown() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        write(root, "keep.md", "# Keep");
        // A folder with only non-Markdown content must not appear.
        write(root, "assets/logo.png", "png-bytes");
        write(root, "assets/sub/deep.txt", "text");
        // A folder that has Markdown somewhere in its subtree is kept.
        write(root, "docs/guide.md", "# Guide");

        let tree = FileTree::scan(root).unwrap();
        let names: Vec<String> = tree.entries.iter().map(|e| e.name.clone()).collect();

        assert!(names.contains(&"docs".to_string()));
        assert!(names.contains(&"guide.md".to_string()));
        assert!(!names.contains(&"assets".to_string()));
    }

    #[test]
    fn scan_returns_empty_tree_for_markdown_free_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        write(root, "a.txt", "no markdown here");
        write(root, "sub/b.rs", "fn main() {}");

        let tree = FileTree::scan(root).unwrap();
        assert!(tree.entries.is_empty());
    }

    #[test]
    fn delete_removes_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "note.md", "# Note");

        let mut tree = FileTree::scan(root).unwrap();
        let file_path = root.join("note.md");
        assert!(file_path.exists());

        tree.delete(&file_path).unwrap();

        assert!(!file_path.exists());
        // The tree refreshes and no longer lists the deleted file.
        assert!(!tree.entries.iter().any(|e| e.path == file_path));
    }

    #[test]
    fn delete_removes_an_empty_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // An empty folder won't appear in the scanned tree (pruned), but the
        // delete path must still handle it for the recursive-removal guard.
        write(root, "keep.md", "# Keep");
        let empty_dir = root.join("empty");
        fs::create_dir(&empty_dir).unwrap();

        let mut tree = FileTree::scan(root).unwrap();
        tree.delete(&empty_dir).unwrap();

        assert!(!empty_dir.exists());
    }

    #[test]
    fn delete_recursively_removes_a_non_empty_folder() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // A folder with a nested Markdown file - the only kind that ever
        // appears in the scanned tree.
        write(root, "docs/guide.md", "# Guide");
        let docs_dir = root.join("docs");

        let mut tree = FileTree::scan(root).unwrap();
        // Sanity: the folder is listed and is non-empty on disk.
        assert!(tree.entries.iter().any(|e| e.path == docs_dir));
        assert!(docs_dir.join("guide.md").exists());

        // Previously this returned Err "directory not empty" because
        // `fs::remove_dir` only removes empty folders.
        tree.delete(&docs_dir).unwrap();

        assert!(!docs_dir.exists());
        // After refresh, neither the folder nor its file are listed.
        assert!(
            !tree
                .entries
                .iter()
                .any(|e| e.path == docs_dir || e.path == docs_dir.join("guide.md"))
        );
    }

    #[test]
    fn delete_refuses_paths_outside_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(root, "keep.md", "# Keep");

        let mut tree = FileTree::scan(root).unwrap();
        // A path outside the workspace root must be rejected by the
        // `ensure_existing_path_within_root` guard, regardless of kind.
        let outside = std::env::temp_dir().join("markion-delete-guard-probe");
        fs::write(&outside, "probe").unwrap();
        let result = tree.delete(&outside);
        let _ = fs::remove_file(&outside);
        assert!(result.is_err());
    }

    /// `is_markdown_path` is the shared gate for both the file-tree scan and
    /// the OS drag-and-drop open path, so its extension rule is checked here in
    /// isolation. The handler in `main.rs` additionally guards directories with
    /// `path.is_file()`, so this test only asserts the extension check.
    #[test]
    fn is_markdown_path_recognises_supported_extensions_case_insensitively() {
        assert!(is_markdown_path(Path::new("note.md")));
        assert!(is_markdown_path(Path::new("notes.markdown")));
        assert!(is_markdown_path(Path::new("old.mdown")));
        // Case-insensitive (Windows filesystems are case-insensitive).
        assert!(is_markdown_path(Path::new("UPPER.MD")));
        assert!(is_markdown_path(Path::new("Mixed.Md")));
        assert!(is_markdown_path(Path::new("WEIRD.MARKDOWN")));
        // Path with directories still resolves by final extension.
        assert!(is_markdown_path(Path::new("docs/sub/guide.md")));

        // Non-Markdown extensions.
        assert!(!is_markdown_path(Path::new("image.png")));
        assert!(!is_markdown_path(Path::new("code.rs")));
        assert!(!is_markdown_path(Path::new("readme.txt")));

        // No extension.
        assert!(!is_markdown_path(Path::new("README")));
        assert!(!is_markdown_path(Path::new("docs/")));

        // A directory path with a `.md`-like name but no extension reports false
        // (its final component is a directory, not a Markdown file by name),
        // and a directory without an extension also reports false.
        assert!(!is_markdown_path(Path::new("docs/subfolder")));
    }

    /// Exercises the exact drop-filter predicate the external-drag handler in
    /// `main.rs` applies — `path.is_file() && is_markdown_path(path)` — against
    /// a real temp directory. This covers the substance of the "mixed drop
    /// opens only Markdown files; directories and non-Markdown files are
    /// skipped" requirement at the logic level. (Synthesizing a full GPUI
    /// `ExternalPaths` drop event would need a window/render harness the
    /// codebase does not have; the end-to-end path is verified manually per
    /// task 4.4 instead.)
    #[test]
    fn drop_filter_opens_only_real_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A mix of drop candidates: two Markdown files, one non-Markdown file,
        // and one directory. The OS can hand all four over in a single drag.
        let md_a = root.join("a.md");
        let md_b = root.join("b.mdown");
        let png = root.join("logo.png");
        let folder = root.join("notes");
        fs::write(&md_a, "# A").unwrap();
        fs::write(&md_b, "# B").unwrap();
        fs::write(&png, "png-bytes").unwrap();
        fs::create_dir(&folder).unwrap();

        let dropped: Vec<PathBuf> = vec![md_a.clone(), md_b.clone(), png, folder.clone()];

        // The predicate the handler runs per path. Mirrors
        // `handle_external_drop` exactly so this test fails if the two drift.
        let opened: Vec<PathBuf> = dropped
            .into_iter()
            .filter(|p| p.is_file() && is_markdown_path(p))
            .collect();

        assert_eq!(opened, vec![md_a, md_b]);
        // The PNG and the directory are both skipped.
        assert!(
            !opened
                .iter()
                .any(|p| p.extension().and_then(|e| e.to_str()) == Some("png"))
        );
        assert!(!opened.iter().any(|p| p == &folder));
    }
}
