use pdfium_render::prelude::Pdfium;
use std::fmt;
use std::path::{Path, PathBuf};

pub const PDFIUM_BUILD: u32 = 7881;
pub const DEV_RUNTIME_ENV: &str = "MARKION_PDFIUM_RUNTIME";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeBuildMode {
    Development,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeDiscoveryError {
    Missing(PathBuf),
    NotAFile(PathBuf),
    DeveloperOverrideForbidden,
    LoadFailed,
}

impl fmt::Display for RuntimeDiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(path) => {
                write!(formatter, "PDFium runtime is missing: {}", path.display())
            }
            Self::NotAFile(path) => {
                write!(
                    formatter,
                    "PDFium runtime path is not a file: {}",
                    path.display()
                )
            }
            Self::DeveloperOverrideForbidden => {
                formatter.write_str("PDFium developer override is forbidden in release builds")
            }
            Self::LoadFailed => formatter.write_str("PDFium runtime could not be loaded"),
        }
    }
}

impl std::error::Error for RuntimeDiscoveryError {}

pub const fn runtime_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    }
}

pub fn packaged_runtime_path(resource_root: impl AsRef<Path>) -> PathBuf {
    resource_root
        .as_ref()
        .join("assets")
        .join("pdfium")
        .join(runtime_library_name())
}

pub fn resolve_runtime_path(
    resource_root: impl AsRef<Path>,
    build_mode: RuntimeBuildMode,
    developer_override: Option<&Path>,
) -> Result<PathBuf, RuntimeDiscoveryError> {
    resolve_runtime_path_with_staged(resource_root, build_mode, developer_override, None)
}

fn resolve_runtime_path_with_staged(
    resource_root: impl AsRef<Path>,
    build_mode: RuntimeBuildMode,
    developer_override: Option<&Path>,
    staged_runtime: Option<&Path>,
) -> Result<PathBuf, RuntimeDiscoveryError> {
    if build_mode == RuntimeBuildMode::Release && developer_override.is_some() {
        return Err(RuntimeDiscoveryError::DeveloperOverrideForbidden);
    }

    if let Some(path) = developer_override {
        return require_runtime_file(path.to_path_buf());
    }

    let packaged = packaged_runtime_path(resource_root);
    if packaged.exists() || build_mode == RuntimeBuildMode::Release {
        return require_runtime_file(packaged);
    }

    if let Some(path) = staged_runtime
        && path.exists()
    {
        return require_runtime_file(path.to_path_buf());
    }

    Err(RuntimeDiscoveryError::Missing(packaged))
}

fn require_runtime_file(candidate: PathBuf) -> Result<PathBuf, RuntimeDiscoveryError> {
    if !candidate.exists() {
        return Err(RuntimeDiscoveryError::Missing(candidate));
    }
    if !candidate.is_file() {
        return Err(RuntimeDiscoveryError::NotAFile(candidate));
    }
    Ok(candidate)
}

#[cfg(debug_assertions)]
fn staged_development_runtime_path() -> Option<PathBuf> {
    let target = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else {
        return None;
    };
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.parent()?;
    Some(
        repository_root
            .join("target")
            .join("pdfium-runtime")
            .join(target)
            .join(runtime_library_name()),
    )
}

pub fn discover_runtime(resource_root: impl AsRef<Path>) -> Result<PathBuf, RuntimeDiscoveryError> {
    #[cfg(debug_assertions)]
    let build_mode = RuntimeBuildMode::Development;
    #[cfg(not(debug_assertions))]
    let build_mode = RuntimeBuildMode::Release;

    #[cfg(debug_assertions)]
    let developer_override = std::env::var_os(DEV_RUNTIME_ENV).map(PathBuf::from);
    #[cfg(not(debug_assertions))]
    let developer_override: Option<PathBuf> = None;

    #[cfg(debug_assertions)]
    let staged_runtime = staged_development_runtime_path();
    #[cfg(not(debug_assertions))]
    let staged_runtime: Option<PathBuf> = None;

    resolve_runtime_path_with_staged(
        resource_root,
        build_mode,
        developer_override.as_deref(),
        staged_runtime.as_deref(),
    )
}

/// One-shot packaging probe. PDFium bindings can be initialized once per process.
pub fn probe_runtime(runtime_path: impl AsRef<Path>) -> Result<(), RuntimeDiscoveryError> {
    let runtime_path = runtime_path.as_ref();
    if !runtime_path.exists() {
        return Err(RuntimeDiscoveryError::Missing(runtime_path.to_path_buf()));
    }
    if !runtime_path.is_file() {
        return Err(RuntimeDiscoveryError::NotAFile(runtime_path.to_path_buf()));
    }
    let bindings =
        Pdfium::bind_to_library(runtime_path).map_err(|_| RuntimeDiscoveryError::LoadFailed)?;
    let _pdfium = Pdfium::new(bindings);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_mode_uses_only_the_packaged_path() {
        let temporary = tempfile::tempdir().unwrap();
        let override_path = temporary.path().join(runtime_library_name());
        std::fs::write(&override_path, b"not loaded during discovery").unwrap();
        assert_eq!(
            resolve_runtime_path(
                temporary.path(),
                RuntimeBuildMode::Release,
                Some(&override_path)
            ),
            Err(RuntimeDiscoveryError::DeveloperOverrideForbidden)
        );
        assert_eq!(
            resolve_runtime_path(temporary.path(), RuntimeBuildMode::Release, None),
            Err(RuntimeDiscoveryError::Missing(packaged_runtime_path(
                temporary.path()
            )))
        );
        assert_eq!(
            resolve_runtime_path_with_staged(
                temporary.path(),
                RuntimeBuildMode::Release,
                None,
                Some(&override_path),
            ),
            Err(RuntimeDiscoveryError::Missing(packaged_runtime_path(
                temporary.path()
            )))
        );
    }

    #[test]
    fn development_mode_accepts_an_explicit_existing_file() {
        let temporary = tempfile::tempdir().unwrap();
        let runtime = temporary.path().join("explicit-runtime");
        std::fs::write(&runtime, b"not loaded during discovery").unwrap();
        assert_eq!(
            resolve_runtime_path(
                temporary.path(),
                RuntimeBuildMode::Development,
                Some(&runtime)
            ),
            Ok(runtime)
        );
    }

    #[test]
    fn development_mode_falls_back_to_the_checksum_staged_runtime() {
        let temporary = tempfile::tempdir().unwrap();
        let resource_root = temporary.path().join("resource-root");
        let staged_runtime = temporary.path().join("pdfium-runtime");
        std::fs::write(&staged_runtime, b"not loaded during discovery").unwrap();

        assert_eq!(
            resolve_runtime_path_with_staged(
                &resource_root,
                RuntimeBuildMode::Development,
                None,
                Some(&staged_runtime),
            ),
            Ok(staged_runtime)
        );
    }

    #[test]
    fn development_runtime_precedence_is_override_then_packaged_then_staged() {
        let temporary = tempfile::tempdir().unwrap();
        let resource_root = temporary.path().join("resource-root");
        let packaged_runtime = packaged_runtime_path(&resource_root);
        std::fs::create_dir_all(packaged_runtime.parent().unwrap()).unwrap();
        std::fs::write(&packaged_runtime, b"packaged").unwrap();
        let staged_runtime = temporary.path().join("staged-runtime");
        std::fs::write(&staged_runtime, b"staged").unwrap();
        let developer_override = temporary.path().join("developer-override");
        std::fs::write(&developer_override, b"override").unwrap();

        assert_eq!(
            resolve_runtime_path_with_staged(
                &resource_root,
                RuntimeBuildMode::Development,
                Some(&developer_override),
                Some(&staged_runtime),
            ),
            Ok(developer_override)
        );
        assert_eq!(
            resolve_runtime_path_with_staged(
                &resource_root,
                RuntimeBuildMode::Development,
                None,
                Some(&staged_runtime),
            ),
            Ok(packaged_runtime)
        );
    }
}
