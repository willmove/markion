use std::{env, fs, path::PathBuf, time::Duration};

use markion_plugin_protocol::{
    FrameLimits, HostMessage, PAGED_DOCUMENT_CAPABILITY, PackageLimits, PagedDocumentRequest,
    PagedDocumentResponse, PixelFormat, PluginMessage, ProcessPluginPeer, ProtocolVersion,
    TargetSpec, WorkerLaunch, inspect_package,
};
use semver::Version;

const REQUIRED_LOCALES: &[&str] = &["en", "zh-Hans", "zh-Hant", "ja", "fr", "de", "es"];
const SMOKE_MAX_SOURCE_BYTES: u64 = 512 * 1024 * 1024;
const SMOKE_MAX_PAGES: u32 = 10_000;
const SMOKE_MAX_RASTER_BYTES: u64 = 32 * 1024 * 1024;

fn main() {
    if let Err(error) = run() {
        eprintln!("plugin package inspection failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let archive = PathBuf::from(args.next().ok_or("package path is required")?);
    let public_key = PathBuf::from(args.next().ok_or("public-key path is required")?);
    let output = args.next().map(PathBuf::from);
    let smoke_document = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err("unexpected package inspector arguments".into());
    }
    if smoke_document.is_some() && output.is_none() {
        return Err("paged-document smoke requires an extraction path".into());
    }

    let archive_bytes = fs::read(&archive)?;
    let public_key_text = fs::read_to_string(&public_key)?;
    let package = inspect_package(
        &archive_bytes,
        &public_key_text,
        &Version::parse(env!("CARGO_PKG_VERSION"))?,
        ProtocolVersion::V1_0,
        &TargetSpec::current(),
        REQUIRED_LOCALES,
        PackageLimits::default(),
    )?;
    if let Some(output) = output {
        package.extract_to(&output)?;
        let manifest = &package.manifest;
        if smoke_document.is_some()
            && !manifest
                .capabilities
                .iter()
                .any(|capability| capability.id == PAGED_DOCUMENT_CAPABILITY)
        {
            return Err("package does not declare paged-document/v1".into());
        }
        let entry_point = output.join(&manifest.entry_point);
        let max_control_bytes = manifest
            .capabilities
            .iter()
            .map(|capability| capability.limits.max_control_bytes)
            .max()
            .ok_or("package has no capabilities")?;
        let max_body_bytes = manifest
            .capabilities
            .iter()
            .map(|capability| capability.limits.max_body_bytes)
            .max()
            .ok_or("package has no capabilities")?;
        let max_in_flight_requests = manifest
            .capabilities
            .iter()
            .map(|capability| usize::from(capability.limits.max_in_flight_requests))
            .min()
            .ok_or("package has no capabilities")?;
        let request_timeout_ms = manifest
            .capabilities
            .iter()
            .map(|capability| capability.limits.request_timeout_ms)
            .min()
            .ok_or("package has no capabilities")?;
        let launch = WorkerLaunch {
            executable: entry_point,
            working_directory: output,
            protocol: ProtocolVersion::V1_0,
            host_version: Version::parse(env!("CARGO_PKG_VERSION"))?,
            plugin_id: manifest.plugin_id.clone(),
            plugin_version: manifest.version.clone(),
            package_sha256: package.archive_sha256.clone(),
            requested_capabilities: manifest
                .capabilities
                .iter()
                .map(|capability| capability.id.clone())
                .collect(),
            expected_capabilities: manifest.capabilities.clone(),
            frame_limits: FrameLimits {
                max_header_bytes: max_control_bytes,
                max_body_bytes,
            },
            max_in_flight_requests,
            request_timeout: Duration::from_millis(u64::from(request_timeout_ms)),
            shutdown_timeout: Duration::from_secs(2),
            stderr_limit_bytes: 16 * 1024,
            environment: Vec::new(),
        };
        let peer = ProcessPluginPeer::launch(launch, 1)?;
        let response =
            peer.request_with_timeout(HostMessage::Ping, Vec::new(), Duration::from_secs(2))?;
        if response.message != PluginMessage::Pong || !response.body.is_empty() {
            peer.force_terminate();
            return Err("extracted plugin health check returned an invalid response".into());
        }
        if let Some(smoke_document) = smoke_document.as_deref()
            && let Err(error) = smoke_paged_document(&peer, smoke_document)
        {
            peer.force_terminate();
            return Err(error);
        }
        peer.shutdown()?;
    }
    println!(
        "{{\"plugin_id\":\"{}\",\"version\":\"{}\",\"archive_bytes\":{},\"installed_bytes\":{},\"archive_sha256\":\"{}\"}}",
        package.manifest.plugin_id,
        package.manifest.version,
        package.archive_size_bytes,
        package.installed_size_bytes,
        package.archive_sha256
    );
    Ok(())
}

fn smoke_paged_document(
    peer: &ProcessPluginPeer,
    document: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !document.is_file() {
        return Err(format!("smoke document is unavailable: {}", document.display()).into());
    }
    let opened = peer.request_with_timeout(
        HostMessage::PagedDocument(PagedDocumentRequest::Open {
            path: document.to_string_lossy().into_owned(),
            max_source_bytes: SMOKE_MAX_SOURCE_BYTES,
            max_pages: SMOKE_MAX_PAGES,
        }),
        Vec::new(),
        Duration::from_secs(30),
    )?;
    let PluginMessage::PagedDocument(PagedDocumentResponse::Opened {
        document_token,
        pages,
    }) = opened.message
    else {
        return Err("paged-document smoke did not open the fixture".into());
    };
    if pages.is_empty() {
        return Err("paged-document smoke opened a zero-page fixture".into());
    }

    let rendered = peer.request_with_timeout(
        HostMessage::PagedDocument(PagedDocumentRequest::Render {
            document_token,
            generation: 1,
            page_index: 0,
            width_px: 640,
            max_raster_bytes: SMOKE_MAX_RASTER_BYTES,
        }),
        Vec::new(),
        Duration::from_secs(30),
    )?;
    let PluginMessage::PagedDocument(PagedDocumentResponse::Rendered {
        document_token: rendered_token,
        generation,
        page_index,
        raster,
    }) = rendered.message
    else {
        return Err("paged-document smoke did not render the first page".into());
    };
    let expected_stride = raster
        .width
        .checked_mul(4)
        .ok_or("paged-document smoke raster stride overflow")?;
    let expected_body = u64::from(raster.stride)
        .checked_mul(u64::from(raster.height))
        .ok_or("paged-document smoke raster body overflow")?;
    if rendered_token != document_token
        || generation != 1
        || page_index != 0
        || raster.width == 0
        || raster.height == 0
        || raster.stride != expected_stride
        || raster.pixel_format != PixelFormat::Rgba8
        || raster.body_length != expected_body
        || raster.body_length != rendered.body.len() as u64
        || raster.body_length > SMOKE_MAX_RASTER_BYTES
    {
        return Err("paged-document smoke returned an invalid raster".into());
    }

    let closed = peer.request_with_timeout(
        HostMessage::PagedDocument(PagedDocumentRequest::Close { document_token }),
        Vec::new(),
        Duration::from_secs(5),
    )?;
    if !matches!(
        closed.message,
        PluginMessage::PagedDocument(PagedDocumentResponse::Closed {
            document_token: closed_token
        }) if closed_token == document_token
    ) || !closed.body.is_empty()
    {
        return Err("paged-document smoke did not close the fixture".into());
    }
    Ok(())
}
