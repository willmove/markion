use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use markion_pdf_viewer::{DEV_RUNTIME_ENV, MAX_PAGE_COUNT, MAX_RASTER_BYTES, MAX_SOURCE_BYTES};
use markion_plugin_protocol::{
    CapabilityDeclaration, HostMessage, PAGED_DOCUMENT_CAPABILITY, PagedDocumentRequest,
    PagedDocumentResponse, PluginErrorCode, PluginMessage, ResourceLimits,
};
use markion_plugin_protocol::{ProcessPluginPeer, WorkerLaunch};
use semver::Version;

fn fixture_bytes(name: &str) -> Vec<u8> {
    let encoded = match name {
        "one-page-embedded.pdf" => {
            include_str!("../../pdf-viewer/tests/fixtures/one-page-embedded.pdf.b64")
        }
        "encrypted.pdf" => include_str!("../../pdf-viewer/tests/fixtures/encrypted.pdf.b64"),
        "corrupt.pdf" => include_str!("../../pdf-viewer/tests/fixtures/corrupt.pdf.b64"),
        _ => panic!("unknown fixture"),
    };
    STANDARD
        .decode(
            encoded
                .chars()
                .filter(|character| !character.is_whitespace())
                .collect::<String>(),
        )
        .unwrap()
}

fn stage_worker(root: &Path, runtime: &Path) -> (PathBuf, PathBuf) {
    let plugin_root = root.join("插件 PDF worker");
    let bin = plugin_root.join("bin");
    let resources = plugin_root.join("resources");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&resources).unwrap();
    let source = PathBuf::from(env!("CARGO_BIN_EXE_markion-plugin-pdf"));
    let worker = bin.join(source.file_name().unwrap());
    fs::copy(source, &worker).unwrap();
    make_executable(&worker);
    fs::copy(
        runtime,
        resources.join(markion_pdf_viewer::runtime_library_name()),
    )
    .unwrap();
    (plugin_root, worker)
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_: &Path) {}

fn launch(plugin_root: PathBuf, worker: PathBuf) -> ProcessPluginPeer {
    let capability = CapabilityDeclaration {
        id: PAGED_DOCUMENT_CAPABILITY.to_owned(),
        limits: ResourceLimits::default(),
    };
    let mut config = WorkerLaunch::fixture(worker, plugin_root);
    config.plugin_id = "dev.markion.pdf".to_owned();
    config.plugin_version = Version::new(0, 1, 0);
    config.package_sha256 = "native-worker-fixture".to_owned();
    config.requested_capabilities = vec![PAGED_DOCUMENT_CAPABILITY.to_owned()];
    config.expected_capabilities = vec![capability];
    config.request_timeout = Duration::from_secs(30);
    ProcessPluginPeer::launch(config, 41).unwrap()
}

#[test]
#[ignore = "requires an explicitly staged PDFium 7881 runtime"]
fn native_worker_opens_renders_rejects_bad_documents_and_shuts_down() {
    let runtime = std::env::var_os(DEV_RUNTIME_ENV)
        .map(PathBuf::from)
        .expect("MARKION_PDFIUM_RUNTIME must name the staged runtime");
    let temporary = tempfile::tempdir().unwrap();
    let (plugin_root, worker) = stage_worker(temporary.path(), &runtime);
    let one_page = temporary.path().join("one-page.pdf");
    let encrypted = temporary.path().join("encrypted.pdf");
    let corrupt = temporary.path().join("corrupt.pdf");
    fs::write(&one_page, fixture_bytes("one-page-embedded.pdf")).unwrap();
    fs::write(&encrypted, fixture_bytes("encrypted.pdf")).unwrap();
    fs::write(&corrupt, fixture_bytes("corrupt.pdf")).unwrap();

    let peer = launch(plugin_root, worker);
    let opened = peer
        .request(
            HostMessage::PagedDocument(PagedDocumentRequest::Open {
                path: one_page.display().to_string(),
                max_source_bytes: MAX_SOURCE_BYTES,
                max_pages: MAX_PAGE_COUNT,
            }),
            Vec::new(),
        )
        .unwrap();
    let PluginMessage::PagedDocument(PagedDocumentResponse::Opened {
        document_token,
        pages,
    }) = opened.message
    else {
        panic!("worker did not open fixture");
    };
    assert_eq!(pages.len(), 1);

    let rendered = peer
        .request(
            HostMessage::PagedDocument(PagedDocumentRequest::Render {
                document_token,
                generation: 12,
                page_index: 0,
                width_px: 640,
                max_raster_bytes: MAX_RASTER_BYTES as u64,
            }),
            Vec::new(),
        )
        .unwrap();
    let PluginMessage::PagedDocument(PagedDocumentResponse::Rendered { raster, .. }) =
        rendered.message
    else {
        panic!("worker did not render fixture");
    };
    assert_eq!(rendered.body.len() as u64, raster.body_length);
    assert_eq!(raster.stride, raster.width * 4);

    for (path, expected) in [
        (&encrypted, PluginErrorCode::EncryptedDocument),
        (&corrupt, PluginErrorCode::CorruptDocument),
    ] {
        let response = peer
            .request(
                HostMessage::PagedDocument(PagedDocumentRequest::Open {
                    path: path.display().to_string(),
                    max_source_bytes: MAX_SOURCE_BYTES,
                    max_pages: MAX_PAGE_COUNT,
                }),
                Vec::new(),
            )
            .unwrap();
        assert!(matches!(
            response.message,
            PluginMessage::Error(error) if error.code == expected
        ));
    }

    assert!(matches!(
        peer.request(
            HostMessage::PagedDocument(PagedDocumentRequest::Close { document_token }),
            Vec::new(),
        )
        .unwrap()
        .message,
        PluginMessage::PagedDocument(PagedDocumentResponse::Closed { .. })
    ));
    peer.shutdown().unwrap();
}

#[test]
fn missing_runtime_fails_before_handshake() {
    let temporary = tempfile::tempdir().unwrap();
    let plugin_root = temporary.path().join("plugin");
    let bin = plugin_root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let source = PathBuf::from(env!("CARGO_BIN_EXE_markion-plugin-pdf"));
    let worker = bin.join(source.file_name().unwrap());
    fs::copy(source, &worker).unwrap();
    make_executable(&worker);
    assert!(
        ProcessPluginPeer::launch(
            {
                let mut config = WorkerLaunch::fixture(worker, plugin_root);
                config.plugin_id = "dev.markion.pdf".to_owned();
                config.plugin_version = Version::new(0, 1, 0);
                config.requested_capabilities = vec![PAGED_DOCUMENT_CAPABILITY.to_owned()];
                config.expected_capabilities = vec![CapabilityDeclaration {
                    id: PAGED_DOCUMENT_CAPABILITY.to_owned(),
                    limits: ResourceLimits::default(),
                }];
                config
            },
            42,
        )
        .is_err()
    );
}
