//! Managed, document-relative local image resources.

use std::{
    collections::HashMap,
    collections::hash_map::DefaultHasher,
    fs::{self, OpenOptions},
    hash::{Hash, Hasher},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use markion_docx_import::{AssetId, PreparedImport};
use percent_encoding::percent_decode_str;

use super::atomic_write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedImage {
    pub stored_path: PathBuf,
    pub relative_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedDocxImport {
    pub markdown_path: PathBuf,
    pub asset_paths: Vec<PathBuf>,
    /// Import-owned paths that could not be removed after commit. The main
    /// output remains valid; callers can surface these exact cleanup remnants.
    pub retained_paths: Vec<PathBuf>,
}

static NEXT_DOCX_IMPORT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishFailpoint {
    AfterStaging,
    AfterAssetDirectory,
    AfterFirstAsset,
    SubstituteFirstAssetBeforeRollback,
    BeforeMarkdown,
    SubstituteMarkdownBeforeCommit,
    SimulatedTerminationBeforeMarkdown,
}

/// A local image reference outside the publishing image scope that the
/// organize action can copy into the document's asset directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrganizeCandidate {
    pub authored_url: String,
    pub source_path: PathBuf,
}

/// Resolves out-of-scope local image references to readable, supported image
/// files for the user-confirmed organize action. References that already
/// resolve inside the publishing image scope are not offered (they preview
/// without help); `file:` URLs, unsupported extensions, and missing or
/// unreadable files are skipped.
pub fn organize_candidates(
    document_path: &Path,
    references: impl IntoIterator<Item = impl AsRef<str>>,
) -> Vec<OrganizeCandidate> {
    let document_dir = document_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let scope_root = document_scope_root(document_path);
    let canonical_scope = fs::canonicalize(&scope_root).unwrap_or(scope_root);
    let mut candidates = Vec::new();
    for reference in references {
        let reference = reference.as_ref();
        if !is_local_reference(reference)
            || reference
                .trim_start()
                .to_ascii_lowercase()
                .starts_with("file:")
        {
            continue;
        }
        let Some(candidate) = resolve_local_reference(document_dir, reference) else {
            continue;
        };
        let Ok(canonical) = fs::canonicalize(&candidate) else {
            continue;
        };
        if canonical.starts_with(&canonical_scope) {
            continue;
        }
        if !image_extension_supported(&canonical) || !canonical.is_file() {
            continue;
        }
        candidates.push(OrganizeCandidate {
            authored_url: reference.to_owned(),
            source_path: canonical,
        });
    }
    candidates
}

/// True for references that are neither remote URLs nor embedded payloads.
pub(crate) fn is_local_reference(reference: &str) -> bool {
    let lower = reference.trim_start().to_ascii_lowercase();
    !(lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("blob:")
        || lower.starts_with("//"))
}

/// Resolves an authored local reference against the document directory.
/// Absolute references (a leading separator or a Windows drive prefix)
/// resolve to themselves; percent-encoding and backslashes are decoded
/// first.
pub(crate) fn resolve_local_reference(document_dir: &Path, reference: &str) -> Option<PathBuf> {
    let path = reference.split(['?', '#']).next().unwrap_or_default();
    let decoded = percent_decode_str(path).decode_utf8().ok()?;
    let normalized = decoded.replace('\\', "/");
    let bytes = normalized.as_bytes();
    let has_drive_prefix = bytes.len() >= 2
        && bytes.first().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.get(1) == Some(&b':');
    if normalized.starts_with('/') || has_drive_prefix {
        return Some(PathBuf::from(normalized));
    }
    let mut candidate = document_dir.to_path_buf();
    for component in normalized.split('/') {
        candidate.push(component);
    }
    Some(candidate)
}

/// Local image extensions supported consistently by import, workspace scans,
/// file icons, and the read-only image viewer.
pub const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "svg",
];

pub fn image_extension_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| canonical_extension(extension).is_some())
}

/// Returns the stable managed-image directory associated with a saved
/// document. The directory is not created by this observational helper.
pub fn document_asset_dir(document_path: &Path) -> PathBuf {
    let parent = document_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let document_stem = document_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(sanitize_stem)
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "document".into());
    parent.join(format!("{document_stem}.assets"))
}

/// Returns the publishing image scope for a saved document: the parent of
/// the document's directory, covering the document's own directory tree at
/// any depth and exactly one directory level above it. A document without a
/// grandparent degenerates to its own directory tree, and a bare relative
/// document falls back to the working directory.
pub fn document_scope_root(document_path: &Path) -> PathBuf {
    let document_dir = document_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    document_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(document_dir)
        .to_path_buf()
}

pub fn import_image_file(document_path: &Path, source_path: &Path) -> io::Result<ImportedImage> {
    let extension = source_path
        .extension()
        .and_then(|extension| extension.to_str())
        .and_then(canonical_extension)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unsupported image format"))?;
    let bytes = fs::read(source_path)?;
    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");
    import_image_bytes(document_path, stem, extension, &bytes)
}

pub fn import_image_bytes(
    document_path: &Path,
    suggested_stem: &str,
    extension: &str,
    bytes: &[u8],
) -> io::Result<ImportedImage> {
    let extension = canonical_extension(extension)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "unsupported image format"))?;
    if bytes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "image is empty",
        ));
    }
    let asset_dir = document_asset_dir(document_path);
    let asset_name = asset_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document.assets")
        .to_owned();
    fs::create_dir_all(&asset_dir)?;

    let stem = sanitize_stem(suggested_stem);
    let stem = if stem.is_empty() {
        "image".to_string()
    } else {
        stem
    };
    let digest = digest(bytes);
    let base = format!("{stem}-{digest:016x}");
    let mut suffix = 0usize;
    loop {
        let file_name = if suffix == 0 {
            format!("{base}.{extension}")
        } else {
            format!("{base}-{suffix}.{extension}")
        };
        let stored_path = asset_dir.join(&file_name);
        if stored_path.exists() {
            if fs::read(&stored_path)? == bytes {
                return Ok(imported(stored_path, &asset_name, &file_name));
            }
            suffix += 1;
            continue;
        }
        atomic_write(&stored_path, bytes)?;
        return Ok(imported(stored_path, &asset_name, &file_name));
    }
}

/// Publishes one prepared DOCX import without overwriting an existing
/// Markdown file or asset directory. Files are completed in a private sibling
/// directory first; assets become visible before the Markdown commit point.
pub fn publish_docx_import(
    prepared: &PreparedImport,
    destination: &Path,
) -> io::Result<PublishedDocxImport> {
    publish_docx_import_impl(prepared, destination, None)
}

fn publish_docx_import_impl(
    prepared: &PreparedImport,
    destination: &Path,
    failpoint: Option<PublishFailpoint>,
) -> io::Result<PublishedDocxImport> {
    if destination
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("md"))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "DOCX imports require a new .md destination",
        ));
    }
    if destination.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Markdown destination already exists",
        ));
    }
    let asset_dir = document_asset_dir(destination);
    if asset_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "managed asset destination already exists",
        ));
    }
    let parent = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let canonical_parent = fs::canonicalize(parent)?;
    let id = NEXT_DOCX_IMPORT_ID.fetch_add(1, Ordering::Relaxed);
    let destination_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document.md");
    let staging = parent.join(format!(
        ".{destination_name}.markion-import-{}-{id}.tmp",
        std::process::id()
    ));
    fs::create_dir(&staging)?;

    let mut result = publish_docx_import_staged(
        prepared,
        destination,
        &asset_dir,
        &staging,
        &canonical_parent,
        failpoint,
    );
    if let Err(cleanup_error) = fs::remove_dir_all(&staging) {
        if staging.exists() {
            match &mut result {
                Ok(published) => published.retained_paths.push(staging.clone()),
                Err(error) => {
                    *error = error_with_retained(
                        io::Error::new(
                            error.kind(),
                            format!("{error}; cleanup failed: {cleanup_error}"),
                        ),
                        &[staging.clone()],
                    );
                }
            }
        }
    }
    result
}

fn publish_docx_import_staged(
    prepared: &PreparedImport,
    destination: &Path,
    asset_dir: &Path,
    staging: &Path,
    canonical_parent: &Path,
    failpoint: Option<PublishFailpoint>,
) -> io::Result<PublishedDocxImport> {
    let staged_assets = staging.join("assets");
    let staged_owner = staging.join("asset-owner");
    if !prepared.assets.is_empty() {
        fs::create_dir(&staged_assets)?;
        create_new_synced(&staged_owner, b"markion-docx-import")?;
    }
    let asset_dir_name = asset_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("document.assets");
    let mut asset_urls = HashMap::new();
    let mut staged_paths = Vec::new();
    for asset in &prepared.assets {
        let extension = canonical_extension(&asset.extension).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsupported prepared image format",
            )
        })?;
        let stem = sanitize_stem(&asset.suggested_stem);
        let stem = if stem.is_empty() {
            "image".to_owned()
        } else {
            stem
        };
        let name = format!("{stem}-{:016x}.{extension}", digest(&asset.bytes));
        let staged_path = staged_assets.join(&name);
        create_new_synced(&staged_path, &asset.bytes)?;
        asset_urls.insert(asset.id, format!("{asset_dir_name}/{name}"));
        staged_paths.push((staged_path, name));
    }
    let markdown = prepared
        .render_markdown(|id: AssetId| asset_urls.get(&id).cloned())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
    let staged_markdown = staging.join("document.md");
    create_new_synced(&staged_markdown, markdown.as_bytes())?;
    inject_publish_failure(failpoint, PublishFailpoint::AfterStaging)?;

    let mut published_assets = Vec::new();
    if !staged_paths.is_empty() {
        fs::create_dir(asset_dir)?;
        let owner = asset_dir.join(".markion-import-owner");
        if let Err(error) = fs::hard_link(&staged_owner, &owner) {
            return Err(error_with_retained(error, &[asset_dir.to_path_buf()]));
        }
        if let Err(error) = validate_owned_asset_dir(asset_dir, canonical_parent, &staged_owner) {
            let retained =
                remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
            return Err(error_with_retained(error, &retained));
        }
        if failpoint == Some(PublishFailpoint::AfterAssetDirectory) {
            let error = io::Error::other("injected failure after asset directory creation");
            let retained =
                remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
            return Err(error_with_retained(error, &retained));
        }
        for (staged_path, name) in &staged_paths {
            if let Err(error) = validate_owned_asset_dir(asset_dir, canonical_parent, &staged_owner)
            {
                let retained =
                    remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
                return Err(error_with_retained(error, &retained));
            }
            let published = asset_dir.join(name);
            if let Err(error) = fs::hard_link(staged_path, &published) {
                let retained =
                    remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
                return Err(error_with_retained(error, &retained));
            }
            published_assets.push(published);
            if failpoint == Some(PublishFailpoint::SubstituteFirstAssetBeforeRollback) {
                let substituted = published_assets.last().expect("published asset");
                fs::remove_file(substituted)?;
                fs::write(substituted, b"unrelated replacement")?;
                let error = io::Error::other("injected substituted asset race");
                let retained =
                    remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
                return Err(error_with_retained(error, &retained));
            }
            if failpoint == Some(PublishFailpoint::AfterFirstAsset) {
                let error = io::Error::other("injected failure after first asset publication");
                let retained =
                    remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
                return Err(error_with_retained(error, &retained));
            }
        }
    }
    if failpoint == Some(PublishFailpoint::SimulatedTerminationBeforeMarkdown) {
        return Err(io::Error::other(
            "simulated process termination before Markdown commit",
        ));
    }
    if failpoint == Some(PublishFailpoint::BeforeMarkdown) {
        let error = io::Error::other("injected failure before Markdown commit");
        let retained =
            remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
        return Err(error_with_retained(error, &retained));
    }
    if failpoint == Some(PublishFailpoint::SubstituteMarkdownBeforeCommit) {
        fs::write(destination, b"unrelated destination")?;
    }
    if let Err(error) = fs::hard_link(&staged_markdown, destination) {
        let retained =
            remove_owned_import_outputs(destination, asset_dir, &published_assets, staging);
        return Err(error_with_retained(error, &retained));
    }
    let mut retained_paths = Vec::new();
    if !staged_paths.is_empty() {
        let owner = asset_dir.join(".markion-import-owner");
        if same_file::is_same_file(&owner, &staged_owner).unwrap_or(false) {
            let _ = fs::remove_file(&owner);
        }
        if owner.exists() {
            retained_paths.push(owner);
        }
    }
    sync_parent_directory(destination);
    Ok(PublishedDocxImport {
        markdown_path: destination.to_path_buf(),
        asset_paths: published_assets,
        retained_paths,
    })
}

fn inject_publish_failure(
    selected: Option<PublishFailpoint>,
    current: PublishFailpoint,
) -> io::Result<()> {
    if selected == Some(current) {
        Err(io::Error::other(format!(
            "injected DOCX publication failure at {current:?}"
        )))
    } else {
        Ok(())
    }
}

fn create_new_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn validate_owned_asset_dir(
    asset_dir: &Path,
    canonical_parent: &Path,
    staged_owner: &Path,
) -> io::Result<()> {
    let metadata = fs::symlink_metadata(asset_dir)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::other(
            "managed asset destination was replaced during import",
        ));
    }
    let canonical = fs::canonicalize(asset_dir)?;
    if canonical.parent() != Some(canonical_parent) {
        return Err(io::Error::other(
            "managed asset destination escaped its selected parent",
        ));
    }
    if !same_file::is_same_file(asset_dir.join(".markion-import-owner"), staged_owner)
        .unwrap_or(false)
    {
        return Err(io::Error::other(
            "managed asset destination ownership changed during import",
        ));
    }
    Ok(())
}

fn remove_owned_import_outputs(
    markdown: &Path,
    asset_dir: &Path,
    published_assets: &[PathBuf],
    staging: &Path,
) -> Vec<PathBuf> {
    let owner = asset_dir.join(".markion-import-owner");
    let owned_directory =
        same_file::is_same_file(&owner, staging.join("asset-owner")).unwrap_or(false);
    let staged_markdown = staging.join("document.md");
    if markdown.is_file() && same_file::is_same_file(markdown, &staged_markdown).unwrap_or(false) {
        let _ = fs::remove_file(markdown);
    }
    for path in published_assets {
        let staged = path
            .file_name()
            .map(|name| staging.join("assets").join(name));
        if path.is_file()
            && staged
                .as_deref()
                .is_some_and(|staged| same_file::is_same_file(path, staged).unwrap_or(false))
        {
            let _ = fs::remove_file(path);
        }
    }
    if owned_directory {
        let _ = fs::remove_file(&owner);
        let _ = fs::remove_dir(asset_dir);
    }
    let mut retained = Vec::new();
    if markdown.exists() {
        retained.push(markdown.to_path_buf());
    }
    for path in published_assets {
        if path.exists() {
            retained.push(path.clone());
        }
    }
    if asset_dir.exists() && !retained.iter().any(|path| path == asset_dir) {
        retained.push(asset_dir.to_path_buf());
    }
    retained
}

fn error_with_retained(error: io::Error, retained: &[PathBuf]) -> io::Error {
    if retained.is_empty() {
        return error;
    }
    let paths = retained
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    io::Error::new(
        error.kind(),
        format!("{error}; retained import paths: {paths}"),
    )
}

fn sync_parent_directory(path: &Path) {
    #[cfg(not(unix))]
    let _ = path;
    #[cfg(unix)]
    if let Some(parent) = path.parent()
        && let Ok(directory) = fs::File::open(parent)
    {
        let _ = directory.sync_all();
    }
}

fn imported(stored_path: PathBuf, asset_name: &str, file_name: &str) -> ImportedImage {
    ImportedImage {
        stored_path,
        // Generated components contain only portable URL-safe ASCII.
        relative_url: format!("{asset_name}/{file_name}"),
    }
}

fn canonical_extension(extension: &str) -> Option<&'static str> {
    let extension = extension.trim_start_matches('.').to_ascii_lowercase();
    let supported = IMAGE_EXTENSIONS
        .iter()
        .copied()
        .find(|candidate| *candidate == extension)?;
    Some(match supported {
        "jpeg" => "jpg",
        "tif" => "tiff",
        canonical => canonical,
    })
}

fn sanitize_stem(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut previous_dash = false;
    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            Some(ch.to_ascii_lowercase())
        } else if ch.is_alphanumeric() {
            Some(ch)
        } else {
            Some('-')
        };
        if let Some(ch) = normalized {
            if ch == '-' {
                if previous_dash {
                    continue;
                }
                previous_dash = true;
            } else {
                previous_dash = false;
            }
            output.push(ch);
        }
    }
    output
        .trim_matches(['-', '.', ' '])
        .chars()
        .take(64)
        .collect()
}

fn digest(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markion_docx_import::{
        AssetId, ImportLimits, ImportSummary, MarkdownChunk, PackageStats, PreparedAsset,
        PreparedImport,
    };

    fn prepared_with_asset() -> PreparedImport {
        PreparedImport {
            chunks: vec![
                MarkdownChunk::Text("before __MARKION_ASSET_0__ ![alt](".into()),
                MarkdownChunk::AssetUrl(AssetId(0)),
                MarkdownChunk::Text(") after\n".into()),
            ],
            assets: vec![PreparedAsset {
                id: AssetId(0),
                suggested_stem: "../Screen shot [1]".into(),
                extension: "png".into(),
                bytes: b"prepared image bytes".to_vec(),
                width: 1,
                height: 1,
            }],
            diagnostics: vec![],
            summary: ImportSummary {
                paragraphs: 1,
                tables: 0,
                images: 1,
                footnotes: 0,
                revisions_accepted: false,
            },
            package: PackageStats {
                entries: 3,
                decompressed_bytes: 100,
                limits: ImportLimits::default(),
            },
        }
    }

    #[test]
    fn import_generates_safe_relative_link_and_reuses_identical_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let document = dir.path().join("My Note.md");
        let first =
            import_image_bytes(&document, "../Screen shot [1]", ".PNG", b"png bytes").unwrap();
        let second =
            import_image_bytes(&document, "../Screen shot [1]", "png", b"png bytes").unwrap();
        assert_eq!(first, second);
        assert!(
            first
                .stored_path
                .starts_with(dir.path().join("my-note.assets"))
        );
        assert!(
            first
                .relative_url
                .starts_with("my-note.assets/screen-shot-1-")
        );
        assert!(!first.relative_url.contains(".."));
        assert_eq!(fs::read(first.stored_path).unwrap(), b"png bytes");
    }

    #[test]
    fn same_name_different_content_gets_distinct_content_name() {
        let dir = tempfile::tempdir().unwrap();
        let document = dir.path().join("note.md");
        let a = import_image_bytes(&document, "image", "jpg", b"one").unwrap();
        let b = import_image_bytes(&document, "image", "jpg", b"two").unwrap();
        assert_ne!(a.stored_path, b.stored_path);
    }

    #[test]
    fn rejects_unknown_or_empty_images() {
        let dir = tempfile::tempdir().unwrap();
        let document = dir.path().join("note.md");
        assert!(import_image_bytes(&document, "image", "exe", b"x").is_err());
        assert!(import_image_bytes(&document, "image", "png", b"").is_err());
    }

    #[test]
    fn supported_image_extensions_are_shared_and_case_insensitive() {
        for extension in IMAGE_EXTENSIONS {
            assert!(image_extension_supported(Path::new(&format!(
                "asset.{extension}"
            ))));
            assert!(image_extension_supported(Path::new(&format!(
                "asset.{}",
                extension.to_ascii_uppercase()
            ))));
        }
        assert!(!image_extension_supported(Path::new("asset.exe")));
        assert!(!image_extension_supported(Path::new("asset")));
    }

    #[test]
    fn organize_candidates_offer_only_out_of_scope_readable_images() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        let docs = root.join("docs");
        fs::create_dir_all(docs.join("img")).unwrap();
        fs::create_dir_all(temp.path().join("outside")).unwrap();
        fs::create_dir_all(temp.path().join("elsewhere")).unwrap();
        let document = docs.join("note.md");
        fs::write(&document, b"note").unwrap();
        fs::write(docs.join("sibling.png"), b"sibling").unwrap();
        fs::write(docs.join("img/nested.png"), b"nested").unwrap();
        fs::write(root.join("banner.png"), b"banner").unwrap();
        fs::write(temp.path().join("outside/escape.png"), b"escape").unwrap();
        fs::write(temp.path().join("elsewhere/deep.png"), b"deep").unwrap();
        fs::write(temp.path().join("bad.exe"), b"exe").unwrap();

        let absolute = temp.path().join("elsewhere/deep.png");
        let absolute_reference = absolute.to_string_lossy().to_string();
        let references = vec![
            "sibling.png".to_owned(),
            "img/nested.png".to_owned(),
            "../banner.png".to_owned(),
            "../../outside/escape.png".to_owned(),
            absolute_reference.clone(),
            "../../bad.exe".to_owned(),
            "../../missing.png".to_owned(),
            "file:///elsewhere/deep.png".to_owned(),
            "https://example.com/a.png".to_owned(),
        ];

        let candidates = organize_candidates(&document, references);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].authored_url, "../../outside/escape.png");
        assert!(candidates[0].source_path.ends_with("escape.png"));
        assert_eq!(candidates[1].authored_url, absolute_reference);
        assert!(candidates[1].source_path.ends_with("deep.png"));
    }

    #[test]
    fn document_scope_root_covers_document_tree_and_one_level_above() {
        let temp = tempfile::tempdir().unwrap();
        let docs = temp.path().join("docs");
        let document = docs.join("note.md");
        // The scope root is exactly one level above the document's directory.
        assert_eq!(document_scope_root(&document), temp.path());
        // A bare relative document falls back to the working directory.
        assert_eq!(document_scope_root(Path::new("note.md")), Path::new("."));
    }

    #[test]
    fn docx_import_publishes_assets_before_portable_markdown() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("中文 notes").join("article.md");
        let published = publish_docx_import(&prepared_with_asset(), &destination).unwrap();
        assert_eq!(published.markdown_path, destination);
        assert_eq!(published.asset_paths.len(), 1);
        assert!(published.asset_paths[0].is_file());
        assert!(
            !document_asset_dir(&published.markdown_path)
                .join(".markion-import-owner")
                .exists()
        );
        let markdown = fs::read_to_string(&published.markdown_path).unwrap();
        assert!(markdown.contains("article.assets/screen-shot-1-"));
        assert!(markdown.contains("__MARKION_ASSET_0__"));
        assert!(!markdown.contains("markion-import"));
        assert_eq!(
            fs::read(&published.asset_paths[0]).unwrap(),
            b"prepared image bytes"
        );
    }

    #[test]
    fn docx_import_is_create_only_and_preserves_conflicts() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("article.md");
        fs::write(&destination, "existing").unwrap();
        let error = publish_docx_import(&prepared_with_asset(), &destination).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(destination).unwrap(), "existing");

        let destination = temp.path().join("other.md");
        let asset_dir = document_asset_dir(&destination);
        fs::create_dir(&asset_dir).unwrap();
        fs::write(asset_dir.join("keep.txt"), "existing").unwrap();
        let error = publish_docx_import(&prepared_with_asset(), &destination).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(asset_dir.join("keep.txt")).unwrap(),
            "existing"
        );
    }

    #[test]
    fn text_only_docx_import_creates_no_empty_asset_directory() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("text.md");
        let mut prepared = prepared_with_asset();
        prepared.assets.clear();
        prepared.chunks = vec![MarkdownChunk::Text("# Text\n".into())];
        publish_docx_import(&prepared, &destination).unwrap();
        assert_eq!(fs::read_to_string(&destination).unwrap(), "# Text\n");
        assert!(!document_asset_dir(&destination).exists());
    }

    #[test]
    fn docx_import_rolls_back_every_injected_precommit_failure() {
        for failpoint in [
            PublishFailpoint::AfterStaging,
            PublishFailpoint::AfterAssetDirectory,
            PublishFailpoint::AfterFirstAsset,
            PublishFailpoint::BeforeMarkdown,
        ] {
            let temp = tempfile::tempdir().unwrap();
            let destination = temp.path().join("中文 parent").join("article.md");
            assert!(
                publish_docx_import_impl(&prepared_with_asset(), &destination, Some(failpoint))
                    .is_err()
            );
            assert!(!destination.exists(), "{failpoint:?}");
            assert!(!document_asset_dir(&destination).exists(), "{failpoint:?}");
            let parent = destination.parent().unwrap();
            assert!(
                fs::read_dir(parent).unwrap().all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains("markion-import")),
                "{failpoint:?}"
            );
        }
    }

    #[test]
    fn simulated_termination_never_exposes_markdown_before_assets() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("article.md");
        let error = publish_docx_import_impl(
            &prepared_with_asset(),
            &destination,
            Some(PublishFailpoint::SimulatedTerminationBeforeMarkdown),
        )
        .unwrap_err();
        assert!(error.to_string().contains("termination"));
        assert!(!destination.exists());
        let assets = document_asset_dir(&destination);
        assert!(assets.is_dir());
        assert_eq!(fs::read_dir(assets).unwrap().count(), 2);
    }

    #[test]
    fn rollback_preserves_substituted_files_and_reports_the_retained_path() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("article.md");
        let error = publish_docx_import_impl(
            &prepared_with_asset(),
            &destination,
            Some(PublishFailpoint::SubstituteFirstAssetBeforeRollback),
        )
        .unwrap_err();
        let assets = document_asset_dir(&destination);
        let retained = fs::read_dir(&assets)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(fs::read(&retained).unwrap(), b"unrelated replacement");
        assert!(error.to_string().contains(&retained.display().to_string()));
        assert!(!destination.exists());
    }

    #[test]
    fn markdown_commit_race_never_overwrites_or_deletes_the_winner() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("article.md");
        let error = publish_docx_import_impl(
            &prepared_with_asset(),
            &destination,
            Some(PublishFailpoint::SubstituteMarkdownBeforeCommit),
        )
        .unwrap_err();
        assert_eq!(fs::read(&destination).unwrap(), b"unrelated destination");
        assert!(
            error
                .to_string()
                .contains(&destination.display().to_string())
        );
        assert!(!document_asset_dir(&destination).exists());
    }
}
