use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use percent_encoding::percent_decode_str;
use sha2::{Digest, Sha256};
use thiserror::Error;

const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "svg",
];

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("the resource reference is not an allowed relative image path")]
    InvalidReference,
    #[error("the resource is outside the document asset directory")]
    OutsideAssetDirectory,
    #[error("the resource is missing or is not a regular file")]
    Unavailable,
    #[error("the resource image type is unsupported")]
    UnsupportedType,
    #[error("the resource could not be read")]
    Io(#[source] io::Error),
}

/// An immutable, opaque description of a single image that a publishing
/// session may read. Filesystem paths deliberately remain private.
#[derive(Debug, Clone)]
pub struct PublishingResource {
    authored_url: String,
    id: String,
    canonical_asset_root: PathBuf,
    canonical_path: PathBuf,
}

impl PublishingResource {
    pub fn from_path(
        authored_url: impl Into<String>,
        asset_root: &Path,
        candidate: &Path,
    ) -> Result<Self, ResourceError> {
        let authored_url = authored_url.into();
        validate_authored_reference(&authored_url)?;
        validate_image_extension(candidate)?;

        let canonical_asset_root = fs::canonicalize(asset_root).map_err(map_unavailable)?;
        let canonical_path = fs::canonicalize(candidate).map_err(map_unavailable)?;
        if !canonical_path.starts_with(&canonical_asset_root) {
            return Err(ResourceError::OutsideAssetDirectory);
        }
        let metadata = fs::metadata(&canonical_path).map_err(map_unavailable)?;
        if !metadata.is_file() {
            return Err(ResourceError::Unavailable);
        }

        let mut digest = Sha256::new();
        digest.update(authored_url.as_bytes());
        digest.update([0]);
        digest.update(canonical_path.to_string_lossy().as_bytes());
        let id = format!("{:x}", digest.finalize());

        Ok(Self {
            authored_url,
            id,
            canonical_asset_root,
            canonical_path,
        })
    }

    pub fn authored_url(&self) -> &str {
        &self.authored_url
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn read(&self) -> Result<ResourceBytes, ResourceError> {
        let root = fs::canonicalize(&self.canonical_asset_root).map_err(map_unavailable)?;
        let path = fs::canonicalize(&self.canonical_path).map_err(map_unavailable)?;
        if !path.starts_with(&root) {
            return Err(ResourceError::OutsideAssetDirectory);
        }
        validate_image_extension(&path)?;
        let metadata = fs::metadata(&path).map_err(map_unavailable)?;
        if !metadata.is_file() {
            return Err(ResourceError::Unavailable);
        }
        let bytes = fs::read(&path).map_err(ResourceError::Io)?;
        let mime = mime_guess::from_path(&path)
            .first_raw()
            .unwrap_or("application/octet-stream")
            .to_owned();
        Ok(ResourceBytes { bytes, mime })
    }
}

#[derive(Debug)]
pub struct ResourceBytes {
    pub bytes: Vec<u8>,
    pub mime: String,
}

fn validate_authored_reference(reference: &str) -> Result<(), ResourceError> {
    let path_part = reference.split(['?', '#']).next().unwrap_or_default();
    let decoded = percent_decode_str(path_part)
        .decode_utf8()
        .map_err(|_| ResourceError::InvalidReference)?;
    if decoded.is_empty()
        || decoded.contains('\0')
        || decoded.contains("://")
        || decoded.starts_with(['/', '\\'])
        || decoded.as_bytes().get(1) == Some(&b':')
    {
        return Err(ResourceError::InvalidReference);
    }
    let normalized = decoded.replace('\\', "/");
    if Path::new(&normalized).components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(ResourceError::InvalidReference);
    }
    Ok(())
}

fn validate_image_extension(path: &Path) -> Result<(), ResourceError> {
    let supported = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|extension| SUPPORTED_IMAGE_EXTENSIONS.contains(&extension.as_str()));
    if supported {
        Ok(())
    } else {
        Err(ResourceError::UnsupportedType)
    }
}

fn map_unavailable(error: io::Error) -> ResourceError {
    match error.kind() {
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied => ResourceError::Unavailable,
        _ => ResourceError::Io(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn allows_only_regular_supported_images_below_the_canonical_root() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("note.assets");
        fs::create_dir(&root).unwrap();
        let image = root.join("sample.PNG");
        fs::write(&image, b"png").unwrap();

        let resource = PublishingResource::from_path("note.assets/sample.PNG", &root, &image)
            .expect("managed image should be accepted");
        let read = resource.read().unwrap();
        assert_eq!(read.bytes, b"png");
        assert_eq!(read.mime, "image/png");

        let outside = temp.path().join("outside.png");
        fs::write(&outside, b"outside").unwrap();
        assert!(matches!(
            PublishingResource::from_path("outside.png", &root, &outside),
            Err(ResourceError::OutsideAssetDirectory)
        ));
        assert!(matches!(
            PublishingResource::from_path("../outside.png", &root, &image),
            Err(ResourceError::InvalidReference)
        ));
        assert!(matches!(
            PublishingResource::from_path("%2e%2e/outside.png", &root, &image),
            Err(ResourceError::InvalidReference)
        ));
        assert!(matches!(
            PublishingResource::from_path("C:\\outside.png", &root, &image),
            Err(ResourceError::InvalidReference)
        ));
        assert!(matches!(
            PublishingResource::from_path("note.assets/file.exe", &root, &root.join("file.exe")),
            Err(ResourceError::UnsupportedType)
        ));
        assert!(matches!(
            PublishingResource::from_path(
                "note.assets/missing.png",
                &root,
                &root.join("missing.png")
            ),
            Err(ResourceError::Unavailable)
        ));
        assert!(PublishingResource::from_path("note.assets\\sample.PNG", &root, &image).is_ok());
        assert!(PublishingResource::from_path("note.assets/%73ample.PNG", &root, &image).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn rechecks_symlink_containment_when_serving() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("note.assets");
        fs::create_dir(&root).unwrap();
        let original = root.join("sample.png");
        let outside = temp.path().join("outside.png");
        fs::write(&original, b"inside").unwrap();
        fs::write(&outside, b"outside").unwrap();
        let resource =
            PublishingResource::from_path("note.assets/sample.png", &root, &original).unwrap();
        fs::remove_file(&original).unwrap();
        symlink(&outside, &original).unwrap();

        assert!(matches!(
            resource.read(),
            Err(ResourceError::OutsideAssetDirectory)
        ));
    }

    #[cfg(windows)]
    #[test]
    fn rechecks_windows_symlink_containment_when_supported() {
        use std::os::windows::fs::symlink_file;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("note.assets");
        fs::create_dir(&root).unwrap();
        let original = root.join("sample.png");
        let outside = temp.path().join("outside.png");
        fs::write(&original, b"inside").unwrap();
        fs::write(&outside, b"outside").unwrap();
        let resource =
            PublishingResource::from_path("note.assets/sample.png", &root, &original).unwrap();
        fs::remove_file(&original).unwrap();
        if let Err(error) = symlink_file(&outside, &original) {
            if error.kind() == io::ErrorKind::PermissionDenied || error.raw_os_error() == Some(1314)
            {
                return;
            }
            panic!("creating test symlink failed: {error}");
        }
        assert!(matches!(
            resource.read(),
            Err(ResourceError::OutsideAssetDirectory)
        ));
    }
}
