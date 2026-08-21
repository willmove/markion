//! Managed, document-relative local image resources.

use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
};

use super::atomic_write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedImage {
    pub stored_path: PathBuf,
    pub relative_url: String,
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
}
