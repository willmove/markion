//! Deny-by-default retrieval policy and real filesystem identity checks.
use crate::AiError;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    ops::Range,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileIdentity {
    pub volume: u64,
    pub file: u64,
    pub links: u64,
}
#[cfg(unix)]
pub fn identity(path: &Path) -> Result<FileIdentity, AiError> {
    use std::os::unix::fs::MetadataExt;
    let m = fs::symlink_metadata(path).map_err(|_| AiError::Scope)?;
    Ok(FileIdentity {
        volume: m.dev(),
        file: m.ino(),
        links: m.nlink(),
    })
}
#[cfg(windows)]
pub fn identity(path: &Path) -> Result<FileIdentity, AiError> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_READ_ATTRIBUTES, GetFileInformationByHandle,
    };
    let file = fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(7)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| AiError::Scope)?;
    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    // The open handle owns its lifetime and the output points to initialized writable storage.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, info.as_mut_ptr()) } == 0 {
        return Err(AiError::Scope);
    }
    let info = unsafe { info.assume_init() };
    Ok(FileIdentity {
        volume: info.dwVolumeSerialNumber as u64,
        file: ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64,
        links: info.nNumberOfLinks as u64,
    })
}
#[cfg(not(any(unix, windows)))]
pub fn identity(_: &Path) -> Result<FileIdentity, AiError> {
    Err(AiError::Scope)
}
fn ordinary(path: &Path) -> Result<(), AiError> {
    let m = fs::symlink_metadata(path).map_err(|_| AiError::Scope)?;
    if m.file_type().is_symlink() {
        return Err(AiError::Scope);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(AiError::Scope);
        }
    }
    if m.is_file() && identity(path)?.links != 1 {
        return Err(AiError::Scope);
    }
    if !m.is_file() && !m.is_dir() {
        return Err(AiError::Scope);
    }
    Ok(())
}
pub fn safe_name(name: &str) -> bool {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
    {
        return false;
    }
    let base = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    if matches!(base.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$") {
        return false;
    }
    if ["COM", "LPT"].iter().any(|p| {
        base.strip_prefix(p).is_some_and(|s| {
            matches!(
                s,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
    }) {
        return false;
    }
    true
}
pub fn supported(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "md" | "markdown" | "mdown" | "txt"
        )
    })
}
pub fn protected(path: &Path) -> bool {
    path.components().any(|c| {
        let Component::Normal(n) = c else {
            return false;
        };
        let s = n.to_string_lossy().to_ascii_lowercase();
        s.starts_with('.')
            || matches!(
                s.as_str(),
                "node_modules"
                    | "target"
                    | "vendor"
                    | "credentials"
                    | "secrets"
                    | "id_rsa"
                    | "id_ed25519"
            )
            || s.contains("api_key")
            || s.contains("api-key")
            || s.contains("password")
            || s.contains("credential")
            || s.contains("secret")
            || s == "token.txt"
    })
}
#[derive(Debug, Clone)]
pub struct ReadGrant {
    pub root: PathBuf,
    pub scope: PathBuf,
    pub root_identity: FileIdentity,
    pub scope_identity: FileIdentity,
}
impl ReadGrant {
    pub fn new(root: &Path, scope: &Path) -> Result<Self, AiError> {
        for path in root.ancestors() {
            ordinary(path)?;
        }
        for path in scope.ancestors() {
            ordinary(path)?;
        }
        let root = dunce::canonicalize(root).map_err(|_| AiError::Scope)?;
        let scope = dunce::canonicalize(scope).map_err(|_| AiError::Scope)?;
        if !scope.starts_with(&root) || !scope.is_dir() {
            return Err(AiError::Scope);
        }
        let relative = scope.strip_prefix(&root).map_err(|_| AiError::Scope)?;
        if protected(relative) {
            return Err(AiError::Scope);
        }
        Ok(Self {
            root_identity: identity(&root)?,
            scope_identity: identity(&scope)?,
            root,
            scope,
        })
    }
    pub fn validate(&self) -> Result<(), AiError> {
        ordinary(&self.root)?;
        ordinary(&self.scope)?;
        if identity(&self.root)? != self.root_identity
            || identity(&self.scope)? != self.scope_identity
        {
            return Err(AiError::Stale);
        }
        Ok(())
    }
    pub fn resolve(&self, relative: &str, existing: bool, file: bool) -> Result<PathBuf, AiError> {
        self.validate()?;
        let relative = Path::new(relative);
        if relative.as_os_str().is_empty()
            || relative
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
            || protected(relative)
        {
            return Err(AiError::Scope);
        }
        for c in relative.components() {
            if let Component::Normal(n) = c {
                if !n.to_str().is_some_and(safe_name) {
                    return Err(AiError::Scope);
                }
            }
        }
        if file && !supported(relative) {
            return Err(AiError::Scope);
        }
        let target = self.scope.join(relative);
        let mut cursor = self.scope.clone();
        let mut absent = false;
        for c in relative.components() {
            cursor.push(c);
            if !absent {
                match fs::symlink_metadata(&cursor) {
                    Ok(_) => {
                        ordinary(&cursor)?;
                        let real = dunce::canonicalize(&cursor).map_err(|_| AiError::Scope)?;
                        if !real.starts_with(&self.scope) {
                            return Err(AiError::Scope);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => absent = true,
                    Err(_) => return Err(AiError::Scope),
                }
            }
        }
        if existing && (absent || file && !target.is_file()) {
            return Err(AiError::Scope);
        }
        if !existing {
            if target.exists() {
                return Err(AiError::Stale);
            }
            let parent = target.parent().ok_or(AiError::Scope)?;
            if !parent.is_dir() {
                return Err(AiError::Scope);
            }
            let name = target
                .file_name()
                .ok_or(AiError::Scope)?
                .to_string_lossy()
                .to_lowercase();
            for entry in fs::read_dir(parent).map_err(|_| AiError::Scope)? {
                let entry = entry.map_err(|_| AiError::Scope)?;
                if entry.file_name().to_string_lossy().to_lowercase() == name {
                    return Err(AiError::Stale);
                }
            }
        }
        Ok(target)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub files: Vec<String>,
    pub next: Option<usize>,
    pub examined: usize,
}
pub fn list(grant: &ReadGrant, offset: usize, page: usize) -> Result<Listing, AiError> {
    grant.validate()?;
    if page == 0 || page > 200 || offset > 10000 {
        return Err(AiError::Limit);
    }
    let mut files = Vec::new();
    let mut stack = vec![grant.scope.clone()];
    let mut examined = 0;
    while let Some(dir) = stack.pop() {
        ordinary(&dir)?;
        let mut entries = fs::read_dir(&dir)
            .map_err(|_| AiError::Scope)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| AiError::Scope)?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            examined += 1;
            if examined > 20000 {
                return Err(AiError::Limit);
            }
            let path = entry.path();
            let relative = path
                .strip_prefix(&grant.scope)
                .map_err(|_| AiError::Scope)?;
            if protected(relative) || ordinary(&path).is_err() {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if supported(&path) {
                files.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    files.sort();
    let next = (offset + page < files.len()).then_some(offset + page);
    let files = files.into_iter().skip(offset).take(page).collect();
    Ok(Listing {
        files,
        next,
        examined,
    })
}
#[derive(Debug, Clone)]
pub struct BufferSnapshot {
    pub text: String,
    pub version: u64,
    pub document: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResult {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub dirty_buffer: bool,
    pub identity: FileIdentity,
    pub document: Option<u64>,
    pub version: Option<u64>,
    pub first_line: usize,
    pub last_line: usize,
}
pub fn read(
    grant: &ReadGrant,
    path: &str,
    range: Option<Range<usize>>,
    budget: usize,
    buffers: &BTreeMap<PathBuf, BufferSnapshot>,
) -> Result<ReadResult, AiError> {
    let resolved = grant.resolve(path, true, true)?;
    let buffer = buffers.get(&resolved);
    let text = match buffer {
        Some(b) => b.text.clone(),
        None => {
            let m = fs::metadata(&resolved).map_err(|_| AiError::Scope)?;
            if m.len() > 1024 * 1024 {
                return Err(AiError::Limit);
            }
            fs::read_to_string(&resolved).map_err(|_| AiError::Scope)?
        }
    };
    if text.contains('\0') {
        return Err(AiError::Scope);
    }
    let range = range.unwrap_or(0..text.len());
    let slice = text.get(range.clone()).ok_or(AiError::Scope)?;
    if slice.len() > budget {
        return Err(AiError::Limit);
    }
    Ok(ReadResult {
        path: path.into(),
        start: range.start,
        end: range.end,
        text: slice.into(),
        dirty_buffer: buffer.is_some(),
        identity: identity(&resolved)?,
        document: buffer.map(|b| b.document),
        version: buffer.map(|b| b.version),
        first_line: text[..range.start].bytes().filter(|b| *b == b'\n').count() + 1,
        last_line: text[..range.end].bytes().filter(|b| *b == b'\n').count() + 1,
    })
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub excerpt: String,
}
pub fn search(
    grant: &ReadGrant,
    query: &str,
    text: bool,
    offset: usize,
    budget: usize,
    buffers: &BTreeMap<PathBuf, BufferSnapshot>,
) -> Result<(Vec<SearchHit>, Option<usize>), AiError> {
    if query.is_empty() || query.len() > 1024 {
        return Err(AiError::Scope);
    }
    // An opaque integer cursor carries both file index and UTF-8 byte position.
    const STRIDE: usize = 1024 * 1024 + 1;
    let file_offset = offset / STRIDE;
    let byte_offset = offset % STRIDE;
    let listing = list(grant, file_offset, 100)?;
    let mut hits = Vec::new();
    let mut bytes = 0;
    for (index, path) in listing.files.into_iter().enumerate() {
        if text {
            let content = match read(grant, &path, None, 1024 * 1024, buffers) {
                Ok(v) => v,
                Err(AiError::Limit) => continue,
                Err(e) => return Err(e),
            };
            for (start, matched) in content.text.match_indices(query) {
                if index == 0 && start < byte_offset {
                    continue;
                }
                let end = start + matched.len();
                let excerpt = content.text[start..].chars().take(120).collect::<String>();
                let size = path.len() + excerpt.len() + 64;
                if bytes + size > budget || hits.len() >= 200 {
                    if hits.is_empty() {
                        return Err(AiError::Limit);
                    }
                    return Ok((hits, Some((file_offset + index) * STRIDE + start)));
                }
                bytes += size;
                hits.push(SearchHit {
                    path: path.clone(),
                    start,
                    end,
                    excerpt,
                });
            }
        } else if path.to_lowercase().contains(&query.to_lowercase()) {
            let size = path.len() + 64;
            if bytes + size > budget || hits.len() >= 200 {
                if hits.is_empty() {
                    return Err(AiError::Limit);
                }
                return Ok((hits, Some((file_offset + index) * STRIDE)));
            }
            bytes += size;
            hits.push(SearchHit {
                path,
                start: 0,
                end: 0,
                excerpt: String::new(),
            });
        }
    }
    Ok((hits, listing.next.map(|next| next * STRIDE)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policies_containment_and_exact_ranges() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir(&root).unwrap();
        fs::write(root.join("note.md"), "中文😀\r\ncontent").unwrap();
        fs::write(root.join("secret.md"), "private").unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        let grant = ReadGrant::new(&root, &root).unwrap();
        for path in [
            "../other.md",
            "/absolute.md",
            "x:ads.md",
            "CON.md",
            "note.md.",
            "note.md ",
            ".git/n.md",
            "secret.md",
            "binary.png",
        ] {
            assert!(grant.resolve(path, true, true).is_err(), "{path}");
        }
        assert_eq!(list(&grant, 0, 1).unwrap().files, vec!["note.md"]);
        assert!(grant.resolve("NOTE.MD", false, true).is_err());
        assert!(read(&grant, "note.md", Some(1..3), 100, &BTreeMap::new()).is_err());
        assert_eq!(
            read(&grant, "note.md", Some(0..6), 100, &BTreeMap::new())
                .unwrap()
                .text,
            "中文"
        );
        let mut buffers = BTreeMap::new();
        buffers.insert(
            dunce::canonicalize(root.join("note.md")).unwrap(),
            BufferSnapshot {
                text: "dirty".into(),
                version: 2,
                document: 1,
            },
        );
        assert_eq!(
            read(&grant, "note.md", None, 10, &buffers).unwrap().text,
            "dirty"
        );
        fs::hard_link(root.join("note.md"), root.join("alias.md")).unwrap();
        assert!(grant.resolve("note.md", true, true).is_err());
    }
    #[test]
    fn larger_input_budget_reads_complete_file_but_disk_ceiling_stays_finite() {
        let dir = tempfile::tempdir().unwrap();
        let text = "文".repeat(30_000);
        fs::write(dir.path().join("large.md"), &text).unwrap();
        let grant = ReadGrant::new(dir.path(), dir.path()).unwrap();
        assert!(matches!(
            read(&grant, "large.md", None, 65_536, &BTreeMap::new()),
            Err(AiError::Limit)
        ));
        let result = read(&grant, "large.md", None, 262_144, &BTreeMap::new()).unwrap();
        assert_eq!(result.text, text);
        assert_eq!(result.start, 0);
        assert_eq!(result.end, 90_000);
        fs::write(dir.path().join("huge.md"), "x".repeat(1024 * 1024 + 1)).unwrap();
        assert!(matches!(
            read(
                &grant,
                "huge.md",
                Some(0..100),
                1024 * 1024,
                &BTreeMap::new()
            ),
            Err(AiError::Limit)
        ));
    }
    #[test]
    fn replaced_scope_and_sibling_prefix_fail() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("a");
        let sibling = dir.path().join("ab");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&sibling).unwrap();
        assert!(ReadGrant::new(&root, &sibling).is_err());
        let grant = ReadGrant::new(&root, &root).unwrap();
        fs::rename(&root, dir.path().join("old")).unwrap();
        fs::create_dir(&root).unwrap();
        assert!(grant.validate().is_err());
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_are_denied() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("n.md"), "hi").unwrap();
        std::os::unix::fs::symlink(dir.path().join("n.md"), dir.path().join("alias.md")).unwrap();
        let grant = ReadGrant::new(dir.path(), dir.path()).unwrap();
        assert!(grant.resolve("alias.md", true, true).is_err());
    }
    #[test]
    fn pagination_resumes_inside_dense_dirty_file_and_listing_is_bounded() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..215 {
            fs::write(dir.path().join(format!("n{i:03}.md")), "ordinary").unwrap();
        }
        fs::write(dir.path().join("dense.md"), "hit ".repeat(501)).unwrap();
        let grant = ReadGrant::new(dir.path(), dir.path()).unwrap();
        let mut offset = 0;
        let mut files = Vec::new();
        loop {
            let page = list(&grant, offset, 37).unwrap();
            files.extend(page.files);
            match page.next {
                Some(n) => offset = n,
                None => break,
            }
        }
        assert_eq!(files.len(), 216);
        let mut cursor = 0;
        let mut positions = Vec::new();
        loop {
            let (hits, next) =
                search(&grant, "hit", true, cursor, 24000, &BTreeMap::new()).unwrap();
            positions.extend(hits.into_iter().map(|h| h.start));
            match next {
                Some(n) => {
                    assert!(n > cursor);
                    cursor = n
                }
                None => break,
            }
        }
        assert_eq!(positions, (0..501).map(|i| i * 4).collect::<Vec<_>>());
        let mut buffers = BTreeMap::new();
        buffers.insert(
            dunce::canonicalize(dir.path().join("dense.md")).unwrap(),
            BufferSnapshot {
                text: "first\n中文😀\r\nlast".into(),
                document: 44,
                version: 6,
            },
        );
        let r = read(&grant, "dense.md", Some(6..16), 100, &buffers).unwrap();
        assert_eq!(r.document, Some(44));
        assert_eq!(r.version, Some(6));
        assert_eq!(r.first_line, 2);
        assert_eq!(r.text, "中文😀");
    }
    #[cfg(windows)]
    #[test]
    fn windows_junctions_and_reparse_roots_are_denied() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("n.md"), "outside").unwrap();
        let link = dir.path().join("junction");
        let status = std::process::Command::new("cmd.exe")
            .args(["/c", "mklink", "/J"])
            .arg(&link)
            .arg(outside.path())
            .output()
            .unwrap();
        assert!(status.status.success());
        let grant = ReadGrant::new(dir.path(), dir.path()).unwrap();
        assert!(grant.resolve("junction/n.md", true, true).is_err());
        assert!(ReadGrant::new(&link, &link).is_err());
        assert!(list(&grant, 0, 100).unwrap().files.is_empty());
        fs::remove_dir(&link).unwrap();
    }
}
