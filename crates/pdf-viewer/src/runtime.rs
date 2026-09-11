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
    let candidate = match (build_mode, developer_override) {
        (RuntimeBuildMode::Release, Some(_)) => {
            return Err(RuntimeDiscoveryError::DeveloperOverrideForbidden);
        }
        (RuntimeBuildMode::Development, Some(path)) => path.to_path_buf(),
        (_, None) => packaged_runtime_path(resource_root),
    };
    if !candidate.exists() {
        return Err(RuntimeDiscoveryError::Missing(candidate));
    }
    if !candidate.is_file() {
        return Err(RuntimeDiscoveryError::NotAFile(candidate));
    }
    Ok(candidate)
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

    resolve_runtime_path(resource_root, build_mode, developer_override.as_deref())
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
    }

    #[test]
    fn development_mode_accepts_only_an_explicit_existing_file() {
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
}
