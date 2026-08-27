use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    env, fs, io,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const BUNDLE_MANIFEST: &str = "bundle-manifest.json";

const MARKNICE_SOURCE_REPOSITORY: &str = "https://github.com/willmove/marknice";
const REQUIRED_EXPORT_FILES: &[&str] = &[
    "static/export-runtime.js",
    "static/marknice-format-runtime.js",
    "static/marknice-word-runtime.js",
    "static/vendor/html-docx.js",
    "LICENSE.html-docx-js.txt",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleManifest {
    pub import_format_version: u32,
    pub source_repository: String,
    pub source_commit: String,
    #[serde(default)]
    pub third_party: Vec<ThirdPartyComponent>,
    pub files: Vec<BundleFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThirdPartyComponent {
    pub name: String,
    pub version: String,
    pub license: String,
    pub license_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BundleFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleVerification {
    pub file_count: usize,
    pub total_bytes: u64,
    pub source_commit: String,
}

#[derive(Debug, Error)]
pub enum BundleError {
    #[error("the local publishing workspace bundle is unavailable")]
    Unavailable,
    #[error("the local publishing workspace manifest could not be read")]
    ManifestIo(#[source] io::Error),
    #[error("the local publishing workspace manifest is invalid")]
    InvalidManifest(#[source] serde_json::Error),
    #[error("the local publishing workspace manifest has an unsafe or duplicate path")]
    InvalidPath,
    #[error("a required local publishing workspace file is missing")]
    MissingFile,
    #[error("the local publishing workspace contains an unlisted runtime file")]
    UnlistedFile,
    #[error("a local publishing workspace file failed its integrity check: {path}")]
    DigestMismatch { path: String },
    #[error("the local publishing workspace references a remote runtime dependency")]
    RemoteRuntimeDependency,
    #[error("a local publishing workspace dependency is not bundled")]
    MissingLocalDependency,
    #[error("the local publishing workspace provenance or license data is incomplete")]
    IncompleteProvenance,
    #[error("a local publishing workspace contains a prohibited export artifact: {path}")]
    ProhibitedExportArtifact { path: String },
    #[error("the local publishing workspace could not be read")]
    Io(#[source] io::Error),
}

/// Finds a launchable bundle in an explicit development override, a
/// source-tree layout, or one of the native packaged-resource layouts. A
/// candidate is accepted through the minimal runtime gate, so in-place
/// package upgrades that leave unlisted files behind do not block publishing.
pub fn discover_workspace_assets() -> Result<PathBuf, BundleError> {
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os("MARKION_MARKNICE_WORKSPACE_DIR") {
        candidates.push(PathBuf::from(path));
    }
    if let Ok(current) = env::current_dir() {
        candidates.push(current.join("assets").join("marknice-workspace"));
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
            .join("marknice-workspace"),
    );
    if let Ok(executable) = env::current_exe()
        && let Some(binary_dir) = executable.parent()
    {
        candidates.push(binary_dir.join("assets").join("marknice-workspace"));
        candidates.push(
            binary_dir
                .join("resources")
                .join("assets")
                .join("marknice-workspace"),
        );
        if let Some(contents_dir) = binary_dir.parent() {
            candidates.push(
                contents_dir
                    .join("Resources")
                    .join("assets")
                    .join("marknice-workspace"),
            );
        }
    }

    let mut first_error = None;
    let mut seen = HashSet::new();
    for candidate in candidates {
        let candidate = candidate.canonicalize().unwrap_or(candidate);
        if !seen.insert(candidate.clone()) || !candidate.is_dir() {
            continue;
        }
        match verify_launch_gate(&candidate) {
            Ok(_) => return Ok(candidate),
            Err(error) => first_error.get_or_insert(error),
        };
    }
    Err(first_error.unwrap_or(BundleError::Unavailable))
}

pub fn verify_bundle(root: &Path) -> Result<BundleVerification, BundleError> {
    if !root.is_dir() {
        return Err(BundleError::Unavailable);
    }
    let manifest_bytes = fs::read(root.join(BUNDLE_MANIFEST)).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            BundleError::Unavailable
        } else {
            BundleError::ManifestIo(error)
        }
    })?;
    let manifest: BundleManifest =
        serde_json::from_slice(&manifest_bytes).map_err(BundleError::InvalidManifest)?;
    validate_provenance(&manifest)?;
    validate_marknice_export_artifacts(&manifest)?;

    let mut listed = HashMap::new();
    let mut total_bytes = 0_u64;
    for file in &manifest.files {
        let relative = safe_relative_path(&file.path)?;
        if listed.insert(file.path.clone(), relative.clone()).is_some() {
            return Err(BundleError::InvalidPath);
        }
        let path = root.join(&relative);
        let metadata = fs::metadata(&path).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                BundleError::MissingFile
            } else {
                BundleError::Io(error)
            }
        })?;
        if !metadata.is_file() {
            return Err(BundleError::MissingFile);
        }
        let bytes = fs::read(&path).map_err(BundleError::Io)?;
        let digest_input: Cow<'_, [u8]> = if is_text_extension(&relative) {
            Cow::Owned(normalize_line_endings(&bytes))
        } else {
            Cow::Borrowed(bytes.as_slice())
        };
        let actual = format!("{:x}", Sha256::digest(digest_input.as_ref()));
        if !actual.eq_ignore_ascii_case(&file.sha256) {
            return Err(BundleError::DigestMismatch {
                path: file.path.clone(),
            });
        }
        total_bytes = total_bytes.saturating_add(metadata.len());
        verify_runtime_references(root, &relative, &bytes, &listed, &manifest.files)?;
    }
    if !listed.contains_key("index.html") {
        return Err(BundleError::MissingFile);
    }

    let expected: HashSet<PathBuf> = listed.values().cloned().collect();
    let mut actual = Vec::new();
    collect_files(root, root, &mut actual)?;
    if actual
        .into_iter()
        .any(|path| path != Path::new(BUNDLE_MANIFEST) && !expected.contains(&path))
    {
        return Err(BundleError::UnlistedFile);
    }

    Ok(BundleVerification {
        file_count: manifest.files.len(),
        total_bytes,
        source_commit: manifest.source_commit,
    })
}

/// Minimal launch-time gate for the runtime call sites: the manifest parses,
/// its provenance is valid, and the entry shell matches its recorded digest.
/// Files on disk that the manifest does not list — in-place-upgrade
/// leftovers, OS metadata — are deliberately tolerated, because the release
/// pipeline already verified the complete bundle at package time. Full
/// `verify_bundle` remains the exhaustive release-time check.
pub fn verify_launch_gate(root: &Path) -> Result<BundleVerification, BundleError> {
    let manifest_bytes = fs::read(root.join(BUNDLE_MANIFEST)).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            BundleError::Unavailable
        } else {
            BundleError::ManifestIo(error)
        }
    })?;
    let manifest: BundleManifest =
        serde_json::from_slice(&manifest_bytes).map_err(BundleError::InvalidManifest)?;
    validate_provenance(&manifest)?;

    let entry = manifest
        .files
        .iter()
        .find(|file| file.path == "index.html")
        .ok_or(BundleError::MissingFile)?;
    let path = root.join("index.html");
    let metadata = fs::metadata(&path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            BundleError::MissingFile
        } else {
            BundleError::Io(error)
        }
    })?;
    if !metadata.is_file() {
        return Err(BundleError::MissingFile);
    }
    let bytes = fs::read(&path).map_err(BundleError::Io)?;
    let actual = format!(
        "{:x}",
        Sha256::digest(normalize_line_endings(&bytes).as_slice())
    );
    if !actual.eq_ignore_ascii_case(&entry.sha256) {
        return Err(BundleError::DigestMismatch {
            path: entry.path.clone(),
        });
    }
    Ok(BundleVerification {
        file_count: manifest.files.len(),
        total_bytes: metadata.len(),
        source_commit: manifest.source_commit,
    })
}

fn validate_provenance(manifest: &BundleManifest) -> Result<(), BundleError> {
    let valid_commit = manifest.source_commit.len() == 40
        && manifest
            .source_commit
            .chars()
            .all(|character| character.is_ascii_hexdigit());
    if manifest.import_format_version == 0
        || manifest.source_repository.trim().is_empty()
        || !valid_commit
        || manifest.files.is_empty()
        || manifest.third_party.iter().any(|component| {
            component.name.trim().is_empty()
                || component.version.trim().is_empty()
                || component.license.trim().is_empty()
                || component.license_file.trim().is_empty()
                || !manifest
                    .files
                    .iter()
                    .any(|file| file.path == component.license_file)
        })
    {
        return Err(BundleError::IncompleteProvenance);
    }
    Ok(())
}

/// The browser export is intentionally a checked-in static bundle: npm
/// metadata, credentials, and generated documents must never become release
/// assets. This applies only to MarkNice; generic test bundles retain their
/// focused local-closure coverage.
fn validate_marknice_export_artifacts(manifest: &BundleManifest) -> Result<(), BundleError> {
    if manifest.source_repository != MARKNICE_SOURCE_REPOSITORY {
        return Ok(());
    }

    let converter = manifest
        .third_party
        .iter()
        .find(|component| component.name == "html-docx-js")
        .filter(|component| {
            component.version == "0.3.1"
                && component.license == "MIT"
                && component.license_file == "LICENSE.html-docx-js.txt"
        });
    if converter.is_none()
        || REQUIRED_EXPORT_FILES
            .iter()
            .any(|required| !manifest.files.iter().any(|file| file.path == *required))
    {
        return Err(BundleError::IncompleteProvenance);
    }

    for file in &manifest.files {
        if is_prohibited_export_artifact(&file.path) {
            return Err(BundleError::ProhibitedExportArtifact {
                path: file.path.clone(),
            });
        }
    }
    Ok(())
}

fn is_prohibited_export_artifact(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();
    let file_name = lower.rsplit('/').next().unwrap_or_default();
    lower.split('/').any(|part| part == "node_modules")
        || matches!(
            file_name,
            "package.json"
                | "package-lock.json"
                | "npm-shrinkwrap.json"
                | ".npmrc"
                | ".env"
                | "id_rsa"
                | "credentials"
        )
        || [".tgz", ".npm", ".docx", ".mht", ".pem", ".key", ".p12"]
            .iter()
            .any(|extension| file_name.ends_with(extension))
}

const TEXT_EXTENSIONS: &[&str] = &[
    "html", "htm", "css", "js", "mjs", "json", "txt", "md", "map",
];

fn is_text_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| TEXT_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()))
}

/// Normalizes CRLF and lone CR line endings to LF so a workspace checked out
/// with platform line endings verifies identically to its canonical LF bytes.
/// Binary files are never normalized; callers gate this with is_text_extension.
fn normalize_line_endings(bytes: &[u8]) -> Vec<u8> {
    if !bytes.contains(&b'\r') {
        return bytes.to_vec();
    }
    let mut normalized = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' {
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 2;
            } else {
                index += 1;
            }
            normalized.push(b'\n');
        } else {
            normalized.push(bytes[index]);
            index += 1;
        }
    }
    normalized
}

fn safe_relative_path(value: &str) -> Result<PathBuf, BundleError> {
    if value.is_empty() || value.contains('\\') || value.as_bytes().get(1) == Some(&b':') {
        return Err(BundleError::InvalidPath);
    }
    let path = Path::new(value);
    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(BundleError::InvalidPath);
    }
    Ok(path.to_path_buf())
}

fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), BundleError> {
    for entry in fs::read_dir(directory).map_err(BundleError::Io)? {
        let entry = entry.map_err(BundleError::Io)?;
        let file_type = entry.file_type().map_err(BundleError::Io)?;
        if file_type.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else if file_type.is_file() {
            files.push(
                entry
                    .path()
                    .strip_prefix(root)
                    .map_err(|_| BundleError::InvalidPath)?
                    .to_path_buf(),
            );
        } else {
            return Err(BundleError::InvalidPath);
        }
    }
    Ok(())
}

fn verify_runtime_references(
    root: &Path,
    relative: &Path,
    bytes: &[u8],
    _already_listed: &HashMap<String, PathBuf>,
    manifest_files: &[BundleFile],
) -> Result<(), BundleError> {
    let extension = relative
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !matches!(extension, "html" | "css" | "js" | "mjs") {
        return Ok(());
    }
    let text = std::str::from_utf8(bytes).map_err(|_| BundleError::MissingLocalDependency)?;
    let lower = text.to_ascii_lowercase();
    let remote_patterns: &[&str] = if extension == "css" {
        &[
            "url(http://",
            "url(https://",
            "@import http://",
            "@import https://",
        ]
    } else if extension == "html" {
        &[
            "src=\"http://",
            "src=\"https://",
            "href=\"http://",
            "href=\"https://",
            "src='http://",
            "src='https://",
            "href='http://",
            "href='https://",
        ]
    } else {
        &[
            "fetch(\"http://",
            "fetch(\"https://",
            "fetch('http://",
            "fetch('https://",
            "import(\"http://",
            "import(\"https://",
        ]
    };
    if remote_patterns
        .iter()
        .any(|pattern| lower.contains(pattern))
    {
        return Err(BundleError::RemoteRuntimeDependency);
    }

    if extension == "html" {
        for reference in quoted_attribute_values(text, &["src", "href"]) {
            if reference.starts_with(['#', '/'])
                || reference.starts_with("data:")
                || reference.starts_with("blob:")
            {
                continue;
            }
            let dependency = relative
                .parent()
                .unwrap_or(Path::new(""))
                .join(reference.split(['?', '#']).next().unwrap_or_default());
            let normalized = normalize_relative(&dependency)?;
            let normalized_string = normalized.to_string_lossy().replace('\\', "/");
            if !root.join(&normalized).is_file()
                || !manifest_files
                    .iter()
                    .any(|file| file.path == normalized_string)
            {
                return Err(BundleError::MissingLocalDependency);
            }
        }
    }
    if extension == "css" {
        for reference in css_url_values(text) {
            if reference.starts_with(['#', '/'])
                || reference.starts_with("data:")
                || reference.starts_with("blob:")
            {
                continue;
            }
            verify_local_dependency(root, relative, reference, manifest_files)?;
        }
    }
    if matches!(extension, "js" | "mjs") {
        for reference in literal_call_values(text, "fetch(") {
            if reference.starts_with('/') || reference.contains("${") {
                continue;
            }
            verify_local_dependency(root, relative, reference, manifest_files)?;
        }
    }
    Ok(())
}

fn verify_local_dependency(
    root: &Path,
    owner: &Path,
    reference: &str,
    manifest_files: &[BundleFile],
) -> Result<(), BundleError> {
    let dependency = owner
        .parent()
        .unwrap_or(Path::new(""))
        .join(reference.split(['?', '#']).next().unwrap_or_default());
    let normalized = normalize_relative(&dependency)?;
    let normalized_string = normalized.to_string_lossy().replace('\\', "/");
    if !root.join(&normalized).is_file()
        || !manifest_files
            .iter()
            .any(|file| file.path == normalized_string)
    {
        return Err(BundleError::MissingLocalDependency);
    }
    Ok(())
}

fn css_url_values(text: &str) -> Vec<&str> {
    let mut values = Vec::new();
    let lower = text.to_ascii_lowercase();
    let mut offset = 0;
    while let Some(index) = lower[offset..].find("url(") {
        let start = offset + index + 4;
        let Some(end) = text[start..].find(')') else {
            break;
        };
        values.push(text[start..start + end].trim().trim_matches(['\'', '"']));
        offset = start + end + 1;
    }
    values
}

fn literal_call_values<'a>(text: &'a str, call: &str) -> Vec<&'a str> {
    let mut values = Vec::new();
    let mut offset = 0;
    while let Some(index) = text[offset..].find(call) {
        let start = offset + index + call.len();
        let remainder = text[start..].trim_start();
        if let Some(quote) = remainder
            .chars()
            .next()
            .filter(|ch| matches!(ch, '\'' | '"'))
            && let Some(end) = remainder[1..].find(quote)
        {
            values.push(&remainder[1..1 + end]);
        }
        offset = start;
    }
    values
}

fn quoted_attribute_values<'a>(text: &'a str, names: &[&str]) -> Vec<&'a str> {
    let mut values = Vec::new();
    let lower = text.to_ascii_lowercase();
    for name in names {
        let mut offset = 0;
        while let Some(index) = lower[offset..].find(name) {
            let start = offset + index + name.len();
            let remainder = &text[start..];
            let trimmed = remainder.trim_start();
            if let Some(after_equals) = trimmed.strip_prefix('=') {
                let value = after_equals.trim_start();
                if let Some(quote) = value.chars().next().filter(|ch| matches!(ch, '\'' | '"'))
                    && let Some(end) = value[1..].find(quote)
                {
                    values.push(&value[1..1 + end]);
                }
            }
            offset = start;
        }
    }
    values
}

fn normalize_relative(path: &Path) -> Result<PathBuf, BundleError> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(BundleError::InvalidPath);
                }
            }
            Component::RootDir | Component::Prefix(_) => return Err(BundleError::InvalidPath),
        }
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_bundle(root: &Path, html: &str) {
        fs::create_dir_all(root.join("static")).unwrap();
        fs::write(root.join("index.html"), html).unwrap();
        fs::write(root.join("static/app.js"), "window.ready = true;").unwrap();
        fs::write(root.join("LICENSE.marked.txt"), "MIT").unwrap();
        let paths = ["index.html", "static/app.js", "LICENSE.marked.txt"];
        let files = paths
            .iter()
            .map(|path| BundleFile {
                path: (*path).to_owned(),
                sha256: format!("{:x}", Sha256::digest(fs::read(root.join(path)).unwrap())),
            })
            .collect();
        let manifest = BundleManifest {
            import_format_version: 1,
            source_repository: "https://example.invalid/marknice".into(),
            source_commit: "c009c1ec7e7c92f89afa5a32edcb126b5296bda7".into(),
            third_party: vec![ThirdPartyComponent {
                name: "marked".into(),
                version: "1".into(),
                license: "MIT".into(),
                license_file: "LICENSE.marked.txt".into(),
            }],
            files,
        };
        fs::write(
            root.join(BUNDLE_MANIFEST),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn verifies_digests_local_closure_and_provenance() {
        let temp = tempfile::tempdir().unwrap();
        create_bundle(temp.path(), r#"<script src="static/app.js"></script>"#);
        let verified = verify_bundle(temp.path()).unwrap();
        assert_eq!(verified.file_count, 3);

        fs::write(temp.path().join("static/app.js"), "changed").unwrap();
        assert!(matches!(
            verify_bundle(temp.path()),
            Err(BundleError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn verifies_text_files_after_crlf_checkout_conversion() {
        let temp = tempfile::tempdir().unwrap();
        create_bundle(temp.path(), r#"<script src="static/app.js"></script>"#);
        for path in ["index.html", "static/app.js", "LICENSE.marked.txt"] {
            let text = fs::read_to_string(temp.path().join(path)).unwrap();
            let crlf = text.replace('\n', "\r\n");
            fs::write(temp.path().join(path), crlf).unwrap();
        }
        assert!(verify_bundle(temp.path()).is_ok());

        // A genuine content change must still fail even in a CRLF working tree.
        fs::write(
            temp.path().join("static/app.js"),
            "window.ready = true;\r\nwindow.tampered = true;\r\n",
        )
        .unwrap();
        assert!(matches!(
            verify_bundle(temp.path()),
            Err(BundleError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn rejects_remote_and_unlisted_runtime_files() {
        let temp = tempfile::tempdir().unwrap();
        create_bundle(
            temp.path(),
            r#"<script src="https://cdn.invalid/app.js"></script>"#,
        );
        assert!(matches!(
            verify_bundle(temp.path()),
            Err(BundleError::RemoteRuntimeDependency)
        ));

        create_bundle(temp.path(), r#"<script src="static/app.js"></script>"#);
        fs::write(temp.path().join("extra.js"), "extra").unwrap();
        assert!(matches!(
            verify_bundle(temp.path()),
            Err(BundleError::UnlistedFile)
        ));
    }

    #[test]
    fn launch_gate_pins_the_entry_shell_and_tolerates_everything_else() {
        let temp = tempfile::tempdir().unwrap();
        create_bundle(temp.path(), r#"<script src="static/app.js"></script>"#);

        // Unlisted files, including nested ones, do not block the launch gate.
        fs::create_dir_all(temp.path().join("static/vendor/fonts")).unwrap();
        fs::write(temp.path().join("static/vendor/orphan.js"), "leftover").unwrap();
        fs::write(
            temp.path()
                .join("static/vendor/fonts/KaTeX_Main-Regular.ttf"),
            b"font",
        )
        .unwrap();
        fs::write(temp.path().join("LICENSE.orphan.txt"), "leftover").unwrap();
        assert!(verify_launch_gate(temp.path()).is_ok());

        // The entry shell stays pinned: tampering fails with the file named.
        fs::write(
            temp.path().join("index.html"),
            r#"<script src="static/app.js"></script><meta name="tampered">"#,
        )
        .unwrap();
        assert!(matches!(
            verify_launch_gate(temp.path()),
            Err(BundleError::DigestMismatch { path }) if path == "index.html"
        ));

        fs::remove_file(temp.path().join("index.html")).unwrap();
        assert!(matches!(
            verify_launch_gate(temp.path()),
            Err(BundleError::MissingFile)
        ));
    }

    #[test]
    fn launch_gate_rejects_missing_entry_listing_and_invalid_manifests() {
        let temp = tempfile::tempdir().unwrap();
        create_bundle(temp.path(), r#"<script src="static/app.js"></script>"#);
        let manifest: BundleManifest =
            serde_json::from_slice(&fs::read(temp.path().join(BUNDLE_MANIFEST)).unwrap()).unwrap();

        // A manifest that no longer lists the entry shell is not launchable.
        let mut without_entry = manifest.clone();
        without_entry.files.retain(|file| file.path != "index.html");
        fs::write(
            temp.path().join(BUNDLE_MANIFEST),
            serde_json::to_vec_pretty(&without_entry).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            verify_launch_gate(temp.path()),
            Err(BundleError::MissingFile)
        ));

        fs::write(temp.path().join(BUNDLE_MANIFEST), b"not json").unwrap();
        assert!(matches!(
            verify_launch_gate(temp.path()),
            Err(BundleError::InvalidManifest(_))
        ));

        fs::remove_file(temp.path().join(BUNDLE_MANIFEST)).unwrap();
        assert!(matches!(
            verify_launch_gate(temp.path()),
            Err(BundleError::Unavailable)
        ));

        // Invalid provenance still blocks launching.
        create_bundle(temp.path(), r#"<script src="static/app.js"></script>"#);
        let mut bad_provenance = manifest;
        bad_provenance.source_commit = "short".into();
        fs::write(
            temp.path().join(BUNDLE_MANIFEST),
            serde_json::to_vec_pretty(&bad_provenance).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            verify_launch_gate(temp.path()),
            Err(BundleError::IncompleteProvenance)
        ));
    }

    #[test]
    fn launch_gate_accepts_an_upgraded_install_with_upgrade_leftovers() {
        // Regression shape of the v0.1.24 -> v0.2.2 in-place NSIS upgrade:
        // the MathJax-era manifest replaced KaTeX, but the installer never
        // removed the old KaTeX assets from the install directory.
        let temp = tempfile::tempdir().unwrap();
        create_bundle(temp.path(), r#"<script src="static/app.js"></script>"#);
        fs::create_dir_all(temp.path().join("static/vendor/fonts")).unwrap();
        fs::write(temp.path().join("static/vendor/katex.min.js"), "katex").unwrap();
        fs::write(temp.path().join("static/vendor/katex.min.css"), "katex").unwrap();
        fs::write(temp.path().join("LICENSE.katex.txt"), "MIT").unwrap();
        fs::write(
            temp.path()
                .join("static/vendor/fonts/KaTeX_Main-Regular.ttf"),
            b"font",
        )
        .unwrap();

        assert!(verify_launch_gate(temp.path()).is_ok());
        assert!(matches!(
            verify_bundle(temp.path()),
            Err(BundleError::UnlistedFile)
        ));
    }

    #[test]
    fn verifies_the_checked_in_workspace() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
            .join("marknice-workspace");
        let verification = verify_bundle(&root).expect("checked-in workspace must be reproducible");
        assert!(verification.file_count > 10);
        assert_eq!(
            verification.source_commit,
            "c009c1ec7e7c92f89afa5a32edcb126b5296bda7"
        );

        let manifest: BundleManifest = serde_json::from_slice(
            &fs::read(root.join(BUNDLE_MANIFEST)).expect("checked-in workspace manifest"),
        )
        .expect("checked-in workspace manifest is valid");
        assert!(
            REQUIRED_EXPORT_FILES
                .iter()
                .all(|required| { manifest.files.iter().any(|file| file.path == *required) })
        );
        assert!(manifest.third_party.iter().any(|component| {
            component.name == "html-docx-js"
                && component.version == "0.3.1"
                && component.license == "MIT"
                && component.license_file == "LICENSE.html-docx-js.txt"
        }));
    }

    #[test]
    fn canonical_bundle_rejects_npm_credentials_and_generated_documents() {
        for path in [
            "node_modules/html-docx-js/index.js",
            "package-lock.json",
            "static/runtime.tgz",
            "export.docx",
            "token.pem",
        ] {
            assert!(
                is_prohibited_export_artifact(path),
                "{path} must be rejected"
            );
        }
        assert!(!is_prohibited_export_artifact("static/vendor/html-docx.js"));
    }
}
