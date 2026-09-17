use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use semver::Version;
use sha2::{Digest, Sha256};
use thiserror::Error;
use zip::ZipArchive;

use crate::{
    PLUGIN_MANIFEST_NAME, PLUGIN_SIGNATURE_NAME, PluginManifest, ProtocolVersion, TargetSpec,
    ValidationError, canonical_json, verify_minisign,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackageLimits {
    pub max_archive_bytes: u64,
    pub max_installed_bytes: u64,
    pub max_member_bytes: u64,
    pub max_members: usize,
    pub max_manifest_bytes: usize,
    pub max_signature_bytes: usize,
}

impl Default for PackageLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 6 * 1024 * 1024,
            max_installed_bytes: 10 * 1024 * 1024,
            max_member_bytes: 10 * 1024 * 1024,
            max_members: 64,
            max_manifest_bytes: 256 * 1024,
            max_signature_bytes: 64 * 1024,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedMember {
    pub path: String,
    pub bytes: Vec<u8>,
    pub executable: bool,
}

#[derive(Clone, Debug)]
pub struct VerifiedPackage {
    pub manifest: PluginManifest,
    pub archive_sha256: String,
    pub archive_size_bytes: u64,
    pub installed_size_bytes: u64,
    pub members: Vec<VerifiedMember>,
}

impl VerifiedPackage {
    pub fn extract_to(&self, root: &Path) -> Result<(), PackageError> {
        let root = absolute_path(root)?;
        if root.exists() {
            let mut entries = fs::read_dir(&root)?;
            if entries.next().transpose()?.is_some() {
                return Err(PackageError::ExtractionRootNotEmpty(root));
            }
        } else {
            fs::create_dir_all(&root)?;
        }

        for member in &self.members {
            let destination = root.join(&member.path);
            if !destination.starts_with(&root) {
                return Err(PackageError::UnsafePath(member.path.clone()));
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&destination)?;
            file.write_all(&member.bytes)?;
            file.sync_all()?;
            set_executable(&destination, member.executable)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("package I/O failure")]
    Io(#[from] std::io::Error),
    #[error("invalid ZIP package")]
    Zip(#[from] zip::result::ZipError),
    #[error("package exceeds the archive-size limit")]
    ArchiveTooLarge,
    #[error("package contains too many members")]
    TooManyMembers,
    #[error("package expanded bytes exceed the installed-size limit")]
    InstalledTooLarge,
    #[error("package member exceeds its size limit")]
    MemberTooLarge,
    #[error("package path is unsafe: {0}")]
    UnsafePath(String),
    #[error("package contains a duplicate or case-colliding path: {0}")]
    DuplicatePath(String),
    #[error("package contains a symbolic link or unsupported member: {0}")]
    UnsupportedMember(String),
    #[error("package lacks {0}")]
    MissingRequired(&'static str),
    #[error("package manifest is too large")]
    ManifestTooLarge,
    #[error("package signature is too large")]
    SignatureTooLarge,
    #[error("package manifest is invalid JSON")]
    ManifestJson(#[source] serde_json::Error),
    #[error("package manifest is not canonical JSON")]
    NonCanonicalManifest,
    #[error("package manifest validation failed")]
    Manifest(#[from] ValidationError),
    #[error("package signature verification failed")]
    Signature,
    #[error("package member set does not match the signed manifest")]
    ManifestClosure,
    #[error("package member length differs from the signed manifest: {0}")]
    LengthMismatch(String),
    #[error("package member digest differs from the signed manifest: {0}")]
    DigestMismatch(String),
    #[error("declared package size exceeds configured limits")]
    DeclaredSizeTooLarge,
    #[error("extraction root is not empty: {0}")]
    ExtractionRootNotEmpty(PathBuf),
}

pub fn inspect_package(
    archive_bytes: &[u8],
    public_key: &str,
    host_version: &Version,
    protocol: ProtocolVersion,
    target: &TargetSpec,
    required_locales: &[&str],
    limits: PackageLimits,
) -> Result<VerifiedPackage, PackageError> {
    inspect_package_with(
        archive_bytes,
        |manifest, signature| {
            let signature = std::str::from_utf8(signature).map_err(|_| ())?;
            verify_minisign(public_key, signature, manifest).map_err(|_| ())
        },
        host_version,
        protocol,
        target,
        required_locales,
        limits,
    )
}

pub fn inspect_package_with<F>(
    archive_bytes: &[u8],
    verifier: F,
    host_version: &Version,
    protocol: ProtocolVersion,
    target: &TargetSpec,
    required_locales: &[&str],
    limits: PackageLimits,
) -> Result<VerifiedPackage, PackageError>
where
    F: FnOnce(&[u8], &[u8]) -> Result<(), ()>,
{
    if archive_bytes.len() as u64 > limits.max_archive_bytes {
        return Err(PackageError::ArchiveTooLarge);
    }
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes))?;
    if archive.len() > limits.max_members {
        return Err(PackageError::TooManyMembers);
    }

    let mut members = BTreeMap::<String, Vec<u8>>::new();
    let mut folded_paths = BTreeSet::new();
    let mut installed_size = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let path = entry.name().to_owned();
        validate_archive_path(&path)?;
        if entry.is_dir() || is_symlink(entry.unix_mode()) {
            return Err(PackageError::UnsupportedMember(path));
        }
        if !folded_paths.insert(path.to_ascii_lowercase()) {
            return Err(PackageError::DuplicatePath(path));
        }
        let declared = entry.size();
        if declared > limits.max_member_bytes {
            return Err(PackageError::MemberTooLarge);
        }
        installed_size = installed_size
            .checked_add(declared)
            .ok_or(PackageError::InstalledTooLarge)?;
        if installed_size > limits.max_installed_bytes {
            return Err(PackageError::InstalledTooLarge);
        }
        let capacity = usize::try_from(declared).map_err(|_| PackageError::MemberTooLarge)?;
        let mut bytes = Vec::with_capacity(capacity);
        entry
            .by_ref()
            .take(limits.max_member_bytes + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != declared {
            return Err(PackageError::LengthMismatch(path));
        }
        members.insert(path, bytes);
    }

    let manifest_bytes = members
        .get(PLUGIN_MANIFEST_NAME)
        .ok_or(PackageError::MissingRequired(PLUGIN_MANIFEST_NAME))?;
    if manifest_bytes.len() > limits.max_manifest_bytes {
        return Err(PackageError::ManifestTooLarge);
    }
    let signature_bytes = members
        .get(PLUGIN_SIGNATURE_NAME)
        .ok_or(PackageError::MissingRequired(PLUGIN_SIGNATURE_NAME))?;
    if signature_bytes.len() > limits.max_signature_bytes {
        return Err(PackageError::SignatureTooLarge);
    }
    let manifest: PluginManifest =
        serde_json::from_slice(manifest_bytes).map_err(PackageError::ManifestJson)?;
    if canonical_json(&manifest)? != *manifest_bytes {
        return Err(PackageError::NonCanonicalManifest);
    }
    verifier(manifest_bytes, signature_bytes).map_err(|_| PackageError::Signature)?;
    manifest.validate(host_version, protocol, target, required_locales)?;
    if manifest.archive_size_bytes > limits.max_archive_bytes
        || manifest.installed_size_bytes > limits.max_installed_bytes
    {
        return Err(PackageError::DeclaredSizeTooLarge);
    }

    let expected = manifest
        .members
        .iter()
        .map(|member| member.path.as_str())
        .chain([PLUGIN_MANIFEST_NAME, PLUGIN_SIGNATURE_NAME])
        .collect::<BTreeSet<_>>();
    let actual = members.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if expected != actual {
        return Err(PackageError::ManifestClosure);
    }

    for declared in &manifest.members {
        let bytes = members
            .get(&declared.path)
            .ok_or(PackageError::ManifestClosure)?;
        if bytes.len() as u64 != declared.length {
            return Err(PackageError::LengthMismatch(declared.path.clone()));
        }
        let digest = format!("{:x}", Sha256::digest(bytes));
        if !digest.eq_ignore_ascii_case(&declared.sha256) {
            return Err(PackageError::DigestMismatch(declared.path.clone()));
        }
    }

    let executable_paths = manifest
        .members
        .iter()
        .filter(|member| member.executable)
        .map(|member| member.path.as_str())
        .collect::<BTreeSet<_>>();
    let verified_members = members
        .into_iter()
        .map(|(path, bytes)| VerifiedMember {
            executable: executable_paths.contains(path.as_str()),
            path,
            bytes,
        })
        .collect();

    Ok(VerifiedPackage {
        manifest,
        archive_sha256: format!("{:x}", Sha256::digest(archive_bytes)),
        archive_size_bytes: archive_bytes.len() as u64,
        installed_size_bytes: installed_size,
        members: verified_members,
    })
}

fn validate_archive_path(path: &str) -> Result<(), PackageError> {
    let invalid = path.is_empty()
        || path.len() > 240
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..");
    if invalid {
        Err(PackageError::UnsafePath(path.to_owned()))
    } else {
        Ok(())
    }
}

fn is_symlink(mode: Option<u32>) -> bool {
    mode.is_some_and(|mode| mode & 0o170000 == 0o120000)
}

fn absolute_path(path: &Path) -> Result<PathBuf, PackageError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

#[cfg(unix)]
fn set_executable(path: &Path, executable: bool) -> Result<(), PackageError> {
    use std::os::unix::fs::PermissionsExt;

    let mode = if executable { 0o755 } else { 0o644 };
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path, _executable: bool) -> Result<(), PackageError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use semver::VersionReq;
    use zip::{CompressionMethod, ZipWriter, write::FileOptions};

    use super::*;
    use crate::{
        CapabilityDeclaration, LocalizedIdentity, MemberManifest, ProtocolRange, ResourceLimits,
    };

    fn package(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
            for (name, bytes) in entries {
                writer.start_file(*name, options).unwrap();
                writer.write_all(bytes).unwrap();
            }
            writer.finish().unwrap();
        }
        cursor.into_inner()
    }

    fn manifest(worker: &[u8]) -> Vec<u8> {
        canonical_json(&PluginManifest {
            schema_version: 1,
            plugin_id: "dev.markion.fixture".to_owned(),
            version: Version::new(1, 0, 0),
            publisher: "Markion".to_owned(),
            host_version: VersionReq::parse(">=0.3.9, <0.4.0").unwrap(),
            protocol: ProtocolRange::V1,
            target: TargetSpec::current(),
            entry_point: "bin/worker".to_owned(),
            identities: BTreeMap::from([(
                "en".to_owned(),
                LocalizedIdentity {
                    name: "Fixture".to_owned(),
                    description: "Fixture worker".to_owned(),
                },
            )]),
            permissions: Vec::new(),
            capabilities: vec![CapabilityDeclaration {
                id: "paged-document/v1".to_owned(),
                limits: ResourceLimits::default(),
            }],
            file_handlers: Vec::new(),
            archive_size_bytes: 1024,
            installed_size_bytes: 1024,
            members: vec![MemberManifest {
                path: "bin/worker".to_owned(),
                length: worker.len() as u64,
                sha256: format!("{:x}", Sha256::digest(worker)),
                executable: true,
            }],
        })
        .unwrap()
    }

    #[test]
    fn package_closure_and_digest_are_verified_before_extraction() {
        let worker = b"worker";
        let manifest = manifest(worker);
        let archive = package(&[
            (PLUGIN_MANIFEST_NAME, &manifest),
            (PLUGIN_SIGNATURE_NAME, b"fixture-signature"),
            ("bin/worker", worker),
        ]);
        let verified = inspect_package_with(
            &archive,
            |_, signature| (signature == b"fixture-signature").then_some(()).ok_or(()),
            &Version::new(0, 3, 9),
            ProtocolVersion::V1_0,
            &TargetSpec::current(),
            &["en"],
            PackageLimits::default(),
        )
        .unwrap();
        let output = tempfile::tempdir().unwrap();
        verified.extract_to(output.path()).unwrap();
        assert_eq!(fs::read(output.path().join("bin/worker")).unwrap(), worker);
    }

    #[test]
    fn traversal_case_collision_extra_member_and_digest_mismatch_fail() {
        let worker = b"worker";
        let manifest = manifest(worker);
        let traversal = package(&[("../worker", worker)]);
        assert!(matches!(
            inspect_package_with(
                &traversal,
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                PackageLimits::default(),
            ),
            Err(PackageError::UnsafePath(_))
        ));

        let collision = package(&[("bin/worker", worker), ("BIN/WORKER", worker)]);
        assert!(matches!(
            inspect_package_with(
                &collision,
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                PackageLimits::default(),
            ),
            Err(PackageError::DuplicatePath(_))
        ));

        let extra = package(&[
            (PLUGIN_MANIFEST_NAME, &manifest),
            (PLUGIN_SIGNATURE_NAME, b"fixture-signature"),
            ("bin/worker", worker),
            ("extra", b"extra"),
        ]);
        assert!(matches!(
            inspect_package_with(
                &extra,
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                PackageLimits::default(),
            ),
            Err(PackageError::ManifestClosure)
        ));

        let mismatch = package(&[
            (PLUGIN_MANIFEST_NAME, &manifest),
            (PLUGIN_SIGNATURE_NAME, b"fixture-signature"),
            ("bin/worker", b"changed"),
        ]);
        assert!(matches!(
            inspect_package_with(
                &mismatch,
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                PackageLimits::default(),
            ),
            Err(PackageError::LengthMismatch(_)) | Err(PackageError::DigestMismatch(_))
        ));
    }

    #[test]
    fn absolute_symlink_member_count_and_expanded_size_limits_fail() {
        let absolute = package(&[("/absolute", b"x")]);
        assert!(matches!(
            inspect_package_with(
                &absolute,
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                PackageLimits::default(),
            ),
            Err(PackageError::UnsafePath(_))
        ));

        let mut symlink_cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut symlink_cursor);
            writer
                .add_symlink("bin/link", "target", FileOptions::default())
                .unwrap();
            writer.finish().unwrap();
        }
        assert!(matches!(
            inspect_package_with(
                &symlink_cursor.into_inner(),
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                PackageLimits::default(),
            ),
            Err(PackageError::UnsupportedMember(_))
        ));

        let many_entries = (0..65)
            .map(|index| (format!("entry-{index}"), vec![b'x']))
            .collect::<Vec<_>>();
        let mut many_cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut many_cursor);
            for (name, bytes) in &many_entries {
                writer.start_file(name, FileOptions::default()).unwrap();
                writer.write_all(bytes).unwrap();
            }
            writer.finish().unwrap();
        }
        assert!(matches!(
            inspect_package_with(
                &many_cursor.into_inner(),
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                PackageLimits::default(),
            ),
            Err(PackageError::TooManyMembers)
        ));

        let expanded = package(&[("large", b"expanded")]);
        let limits = PackageLimits {
            max_installed_bytes: 2,
            ..PackageLimits::default()
        };
        assert!(matches!(
            inspect_package_with(
                &expanded,
                |_, _| Ok(()),
                &Version::new(0, 3, 9),
                ProtocolVersion::V1_0,
                &TargetSpec::current(),
                &["en"],
                limits,
            ),
            Err(PackageError::InstalledTooLarge)
        ));
    }
}
