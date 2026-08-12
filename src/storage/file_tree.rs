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

/// The curated plain-text file extensions the file tree lists alongside
/// Markdown. These are textual formats a user would reasonably want to open
/// and edit in a writing-focused editor: notes, logs, lightweight markup, and
/// delimited data. Like `MARKDOWN_EXTENSIONS`, comparison is case-insensitive
/// (ASCII-lowercased). Non-text files (binaries, images, source code, …) are
/// intentionally excluded so the tree stays low-noise.
pub const TEXT_EXTENSIONS: &[&str] = &[
    "txt", "text", "log", "csv", "tsv", "org", "rst", "adoc", "asciidoc",
];

/// Returns `true` when `path` has one of the curated plain-text extensions
/// (`txt`/`text`/`log`/`csv`/…), compared case-insensitively. This governs
/// which non-Markdown files appear in the file tree. It is intentionally
/// separate from `is_markdown_path` (which is still the gate for the
/// drag-and-drop open path and for marking a file as Markdown).
pub fn is_text_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            TEXT_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
        })
}

/// The category of a regular file listed in the tree. Only meaningful when
/// `FileTreeEntry::kind == File`; directory entries carry `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeFileKind {
    Markdown,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeEntry {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub kind: FileTreeEntryKind,
    /// `None` for directories; `Some(Markdown)` or `Some(Text)` for files.
    pub file_kind: Option<FileTreeFileKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTree {
    pub root: PathBuf,
    pub entries: Vec<FileTreeEntry>,
    /// Whether the scan includes hidden entries (dotfile names on every
    /// platform, plus Windows-hidden-attribute entries on Windows). Captured
    /// at scan time so `refresh` (called by create/rename/move/delete) keeps
    /// the same visibility rule without the caller having to pass it back in.
    pub show_hidden: bool,
}

impl FileTree {
    pub fn scan(root: impl AsRef<Path>) -> io::Result<Self> {
        Self::scan_with_options(root, false)
    }

    /// Scans `root` with an explicit hidden-entry visibility rule. When
    /// `show_hidden` is `false`, hidden files and folders are omitted (the
    /// default, preserving the historical behavior); when `true`, they are
    /// included subject to the Markdown/text extension filter and the
    /// always-excluded noise list. `scan` is a thin wrapper over this.
    pub fn scan_with_options(root: impl AsRef<Path>, show_hidden: bool) -> io::Result<Self> {
        let root = root.as_ref().to_path_buf();
        let mut entries = Vec::new();
        collect_file_tree_entries(&root, 0, &mut entries, show_hidden)?;
        Ok(Self {
            root,
            entries,
            show_hidden,
        })
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
            // `remove_dir_all` (not `remove_dir`): folders shown in the tree
            // may be non-empty (the scan no longer prunes by content) and the
            // user can right-click any directory the tree lists.
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        self.refresh()
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        self.entries.clear();
        // An empty tree is a valid result for a content-free root.
        collect_file_tree_entries(&self.root, 0, &mut self.entries, self.show_hidden)?;
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

/// Recursively collects Markdown and curated plain-text files, plus the
/// folders that contain them, into `entries`.
///
/// Regular files are classified once by extension: Markdown
/// (`md`/`markdown`/`mdown`) or curated plain text (`txt`/`text`/`log`/…).
/// Everything else (`.rs`, `.toml`, images, …) is skipped so the sidebar stays
/// low-noise. Directories are kept as nesting rows whenever they exist on disk
/// — empty folders (and folders containing only non-text files) are **not**
/// pruned, so the tree mirrors real workspace structure. The hard-coded
/// directory blacklist is always excluded by `should_skip_file_tree_path`, and
/// hidden entries (dotfile names, or Windows-hidden-attribute entries) are
/// excluded unless `show_hidden` is `true`.
fn collect_file_tree_entries(
    root: &Path,
    depth: usize,
    entries: &mut Vec<FileTreeEntry>,
    show_hidden: bool,
) -> io::Result<()> {
    let mut children = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| !should_skip_file_tree_path(&entry.path(), show_hidden))
        .collect::<Vec<_>>();
    children.sort_by(|a, b| {
        let a_path = a.path();
        let b_path = b.path();
        b_path
            .is_dir()
            .cmp(&a_path.is_dir())
            .then_with(|| a.file_name().cmp(&b.file_name()))
    });

    for entry in children {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            entries.push(FileTreeEntry {
                path: path.clone(),
                name: name.clone(),
                depth,
                kind: FileTreeEntryKind::Directory,
                file_kind: None,
            });
            collect_file_tree_entries(&path, depth + 1, entries, show_hidden)?;
            continue;
        }

        // Regular file: classify once by extension. Markdown and curated
        // plain-text files are collected; everything else is skipped.
        let file_kind = if is_markdown_path(&path) {
            FileTreeFileKind::Markdown
        } else if is_text_path(&path) {
            FileTreeFileKind::Text
        } else {
            continue;
        };

        entries.push(FileTreeEntry {
            path: path.clone(),
            name,
            depth,
            kind: FileTreeEntryKind::File,
            file_kind: Some(file_kind),
        });
    }

    Ok(())
}

fn should_skip_file_tree_path(path: &Path, show_hidden: bool) -> bool {
    is_always_excluded(path) || is_hidden_entry(path, show_hidden)
}

/// Directories that are commonly huge or irrelevant to a Markdown workspace.
/// Skipping them keeps the file-tree scan (and the app startup) fast even when
/// the working directory is a large repository or a home folder. This layer is
/// always applied regardless of the show-hidden preference — these entries are
/// treated as build/dependency noise, not OS-hidden files.
fn is_always_excluded(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
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
    )
}

/// OS-hidden entries: dotfile names on every platform (files **and** folders),
/// plus the Windows hidden file attribute on Windows. Gated by `show_hidden` —
/// when the preference is on, this layer is a no-op so hidden entries pass
/// through (still subject to `is_always_excluded` and the extension filter).
fn is_hidden_entry(path: &Path, show_hidden: bool) -> bool {
    if show_hidden {
        return false;
    }
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if name.starts_with('.') {
        return true;
    }
    // Best-effort Windows hidden-attribute check. On `metadata` failure (broken
    // symlink, permission, …) treat the attribute as not-set; the dotfile check
    // above still applies independently. Non-Windows builds pay nothing here.
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
                return true;
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = path;
    }
    false
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
    fn scan_lists_markdown_and_plain_text_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A mix of Markdown, plain-text, and non-text files at the root and in a subfolder.
        write(root, "intro.md", "# Intro");
        write(root, "notes.markdown", "# Notes");
        write(root, "todo.txt", "- buy milk");
        write(root, "run.csv", "a,b\n1,2");
        write(root, "image.png", "png-bytes");
        write(root, "src/main.rs", "fn main() {}");
        write(root, "docs/guide.md", "# Guide");
        write(root, "docs/debug.log", "trace");

        let tree = FileTree::scan(root).unwrap();
        let names: Vec<&str> = tree.entries.iter().map(|e| e.name.as_str()).collect();

        // Markdown files present.
        assert!(names.contains(&"intro.md"));
        assert!(names.contains(&"notes.markdown"));
        assert!(names.contains(&"guide.md"));
        // Curated plain-text files are now also listed.
        assert!(names.contains(&"todo.txt"));
        assert!(names.contains(&"run.csv"));
        assert!(names.contains(&"debug.log"));
        // Non-text files are still absent.
        assert!(!names.contains(&"image.png"));
        assert!(!names.contains(&"main.rs"));
        // Every collected file is classified, Markdown distinguished from Text.
        for entry in tree.entries.iter().filter(|e| e.kind == FileTreeEntryKind::File) {
            match entry.file_kind {
                Some(FileTreeFileKind::Markdown) => assert!(is_markdown_path(&entry.path)),
                Some(FileTreeFileKind::Text) => assert!(is_text_path(&entry.path)),
                None => panic!("file entry missing file_kind: {:?}", entry.path),
            }
        }
        assert!(tree.entries.iter().any(|e| e.name == "intro.md"
            && e.file_kind == Some(FileTreeFileKind::Markdown)));
        assert!(tree.entries.iter().any(|e| e.name == "todo.txt"
            && e.file_kind == Some(FileTreeFileKind::Text)));
    }

    #[test]
    fn scan_keeps_empty_folders() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        write(root, "keep.md", "# Keep");
        // An empty folder now appears (it used to be pruned).
        fs::create_dir(root.join("empty")).unwrap();
        // A folder whose subtree contains only non-text files also appears.
        write(root, "assets/logo.png", "png-bytes");
        // A folder with a Markdown descendant is kept (unchanged).
        write(root, "docs/guide.md", "# Guide");

        let tree = FileTree::scan(root).unwrap();
        let names: Vec<String> = tree.entries.iter().map(|e| e.name.clone()).collect();

        assert!(names.contains(&"docs".to_string()));
        assert!(names.contains(&"guide.md".to_string()));
        // Empty and asset-only folders are now listed as nesting rows.
        assert!(names.contains(&"empty".to_string()));
        assert!(names.contains(&"assets".to_string()));
        // The non-text file inside `assets` is still hidden.
        assert!(!names.contains(&"logo.png".to_string()));
    }

    #[test]
    fn scan_excludes_blacklisted_and_hidden_directories() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        write(root, "keep.md", "# Keep");
        // Blacklisted build/VCS dirs and hidden dirs stay excluded even when
        // they contain text/markdown content.
        write(root, "target/build.log", "log");
        write(root, ".hidden/secret.md", "# Secret");

        let tree = FileTree::scan(root).unwrap();
        let names: Vec<String> = tree.entries.iter().map(|e| e.name.clone()).collect();

        assert!(!names.contains(&"target".to_string()));
        assert!(!names.contains(&"build.log".to_string()));
        assert!(!names.contains(&".hidden".to_string()));
        assert!(!names.contains(&"secret.md".to_string()));
    }

    #[test]
    fn scan_hides_dotfile_files_unless_show_hidden() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A dotfile Markdown file plus a regular Markdown file at the root.
        write(root, ".secret.md", "# Secret");
        write(root, "keep.md", "# Keep");

        let hidden = FileTree::scan_with_options(root, false).unwrap();
        let names: Vec<&str> = hidden.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(!names.contains(&".secret.md"), "dotfile file must be hidden by default");
        assert!(names.contains(&"keep.md"));

        let shown = FileTree::scan_with_options(root, true).unwrap();
        let names: Vec<&str> = shown.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&".secret.md"), "dotfile file must appear when show_hidden is on");
        assert!(shown.show_hidden, "scanned tree must record the show_hidden flag");
    }

    #[test]
    fn scan_hides_dotfile_folders_and_their_children_unless_show_hidden() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A dotfile folder containing a Markdown file. When hidden, the whole
        // subtree is omitted; when revealed, both the folder and the child
        // appear (folders are not content-pruned, so the folder shows once the
        // skip predicate lets it through).
        write(root, ".notes/inside.md", "# Inside");
        write(root, "keep.md", "# Keep");

        let hidden = FileTree::scan_with_options(root, false).unwrap();
        let names: Vec<&str> = hidden.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(!names.contains(&".notes"));
        assert!(!names.contains(&"inside.md"));

        let shown = FileTree::scan_with_options(root, true).unwrap();
        let names: Vec<&str> = shown.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&".notes"));
        assert!(names.contains(&"inside.md"));
    }

    #[test]
    fn scan_always_excludes_noise_list_regardless_of_show_hidden() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Noise-list directories stay excluded even when hidden entries are
        // revealed. `target` is on the always-excluded list (not OS-hidden).
        write(root, "target/build.log", "log");
        write(root, "target/guide.md", "# Guide");
        write(root, "node_modules/pkg/index.md", "# Pkg");
        write(root, "keep.md", "# Keep");

        for show_hidden in [false, true] {
            let tree = FileTree::scan_with_options(root, show_hidden).unwrap();
            let names: Vec<String> = tree.entries.iter().map(|e| e.name.clone()).collect();
            assert!(!names.contains(&"target".to_string()), "target excluded (show_hidden={show_hidden})");
            assert!(
                !names.contains(&"node_modules".to_string()),
                "node_modules excluded (show_hidden={show_hidden})"
            );
            assert!(
                !names.iter().any(|n| n == "build.log" || n == "guide.md" || n == "index.md"),
                "noise-list children excluded (show_hidden={show_hidden})"
            );
            assert!(names.contains(&"keep.md".to_string()));
        }
    }

    #[test]
    fn scan_keeps_markdown_text_filter_when_show_hidden() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Hidden entries still pass through the Markdown/text extension filter.
        // A hidden non-text file (`.env`) never appears in either state; a
        // hidden Markdown file (`.draft.md`) appears only when revealed.
        write(root, ".env", "SECRET=1");
        write(root, ".draft.md", "# Draft");
        write(root, "keep.md", "# Keep");

        let hidden = FileTree::scan_with_options(root, false).unwrap();
        let names: Vec<&str> = hidden.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(!names.contains(&".env"));
        assert!(!names.contains(&".draft.md"));

        let shown = FileTree::scan_with_options(root, true).unwrap();
        let names: Vec<&str> = shown.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(!names.contains(&".env"), "hidden non-text file stays excluded even when show_hidden is on");
        assert!(names.contains(&".draft.md"));
    }

    #[test]
    fn scan_returns_empty_tree_for_truly_empty_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A root with only non-text files lists the folders but no file rows.
        write(root, "sub/b.rs", "fn main() {}");
        let tree = FileTree::scan(root).unwrap();
        let names: Vec<String> = tree.entries.iter().map(|e| e.name.clone()).collect();
        assert!(names.contains(&"sub".to_string()));
        assert_eq!(
            tree.entries
                .iter()
                .filter(|e| e.kind == FileTreeEntryKind::File)
                .count(),
            0
        );

        // A truly empty root (no children at all) yields zero entries.
        let empty_root = tempfile::tempdir().unwrap();
        let empty_tree = FileTree::scan(empty_root.path()).unwrap();
        assert!(empty_tree.entries.is_empty());
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
        // Empty folders now appear in the scanned tree; the delete path must
        // still handle them (and uses `remove_dir_all`, which tolerates empty
        // dirs as well as non-empty ones).
        write(root, "keep.md", "# Keep");
        let empty_dir = root.join("empty");
        fs::create_dir(&empty_dir).unwrap();

        let mut tree = FileTree::scan(root).unwrap();
        // Sanity: the empty folder is listed before deletion.
        assert!(tree.entries.iter().any(|e| e.path == empty_dir));
        tree.delete(&empty_dir).unwrap();

        assert!(!empty_dir.exists());
        assert!(!tree.entries.iter().any(|e| e.path == empty_dir));
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

    /// `is_text_path` is the gate that admits curated plain-text files into the
    /// tree (separate from `is_markdown_path`, which still governs drag-and-drop
    /// and the Markdown classification). Checks its extension rule in isolation.
    #[test]
    fn is_text_path_recognises_curated_extensions_case_insensitively() {
        for ext in ["txt", "text", "log", "csv", "tsv", "org", "rst", "adoc", "asciidoc"] {
            assert!(is_text_path(Path::new(&format!("notes.{ext}"))), "{ext} should count");
        }
        // Case-insensitive (Windows filesystems are case-insensitive).
        assert!(is_text_path(Path::new("UPPER.TXT")));
        assert!(is_text_path(Path::new("Mixed.Csv")));
        // Path with directories still resolves by final extension.
        assert!(is_text_path(Path::new("docs/sub/debug.log")));

        // Markdown extensions are NOT text-tree-curated here (they are handled
        // by `is_markdown_path`); is_text_path reports false for them so the
        // two classifications stay disjoint.
        assert!(!is_text_path(Path::new("note.md")));
        assert!(!is_text_path(Path::new("notes.markdown")));

        // Non-text extensions.
        assert!(!is_text_path(Path::new("image.png")));
        assert!(!is_text_path(Path::new("code.rs")));
        assert!(!is_text_path(Path::new("config.toml")));

        // No extension.
        assert!(!is_text_path(Path::new("README")));
        assert!(!is_text_path(Path::new("docs/")));
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

        // A mix of drop candidates: two Markdown files, non-Markdown files
        // (an image and now plain-text types), and a directory. The OS can hand
        // all of them over in a single drag.
        let md_a = root.join("a.md");
        let md_b = root.join("b.mdown");
        let png = root.join("logo.png");
        let txt = root.join("readme.txt");
        let csv = root.join("data.csv");
        let folder = root.join("notes");
        fs::write(&md_a, "# A").unwrap();
        fs::write(&md_b, "# B").unwrap();
        fs::write(&png, "png-bytes").unwrap();
        fs::write(&txt, "plain text").unwrap();
        fs::write(&csv, "a,b\n1,2").unwrap();
        fs::create_dir(&folder).unwrap();

        let dropped: Vec<PathBuf> = vec![
            md_a.clone(),
            md_b.clone(),
            png,
            txt.clone(),
            csv.clone(),
            folder.clone(),
        ];

        // The predicate the handler runs per path. Mirrors
        // `handle_external_drop` exactly so this test fails if the two drift.
        // Drag-and-drop stays Markdown-only even though `.txt`/`.csv` now appear
        // in (and are openable from) the file tree.
        let opened: Vec<PathBuf> = dropped
            .into_iter()
            .filter(|p| p.is_file() && is_markdown_path(p))
            .collect();

        assert_eq!(opened, vec![md_a, md_b]);
        // The PNG, the plain-text files, and the directory are all skipped.
        assert!(
            !opened
                .iter()
                .any(|p| p.extension().and_then(|e| e.to_str()) == Some("png"))
        );
        assert!(!opened.iter().any(|p| p == &txt));
        assert!(!opened.iter().any(|p| p == &csv));
        assert!(!opened.iter().any(|p| p == &folder));
    }
}
