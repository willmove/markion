use std::{
    collections::BTreeSet,
    env,
    io::{self},
};

use markion_pdf_plugin::{PdfWorker, runtime_path_from_executable};
use markion_pdf_viewer::NativePdfBackend;
use markion_plugin_protocol::{
    CapabilityDeclaration, Frame, FrameKind, FrameLimits, HandshakeAccepted, HostMessage,
    PluginError, PluginErrorCode, PluginMessage, ProtocolVersion, ResourceLimits, TargetSpec,
    read_frame, write_frame,
};
use semver::Version;

const PLUGIN_ID: &str = "dev.markion.pdf";
const PLUGIN_VERSION: &str = "0.1.0";

fn main() {
    if let Err(error) = run() {
        eprintln!("PDF plugin stopped: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let executable = env::current_exe()?;
    let runtime = runtime_path_from_executable(&executable)?;
    let backend = NativePdfBackend::load(&runtime)?;
    let mut worker = PdfWorker::new(backend);
    let limits = FrameLimits::default();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let mut handshaken = false;
    let mut seen_request_ids = BTreeSet::new();

    while let Some(frame) = read_frame(&mut reader, limits)? {
        if frame.request_id == 0 || !seen_request_ids.insert(frame.request_id) {
            return Err("invalid or duplicate request id".into());
        }
        if frame.protocol != ProtocolVersion::V1_0 {
            return Err("unsupported protocol version".into());
        }
        let message: HostMessage = frame.decode_header()?;
        let (kind, response, body, stop) = match message {
            HostMessage::Handshake(hello) if !handshaken => {
                let version = Version::parse(PLUGIN_VERSION)?;
                if hello.protocol != ProtocolVersion::V1_0
                    || hello.expected_plugin_id != PLUGIN_ID
                    || hello.expected_plugin_version != version
                    || !hello
                        .requested_capabilities
                        .iter()
                        .all(|capability| capability == "paged-document/v1")
                {
                    return Err("incompatible PDF plugin handshake".into());
                }
                handshaken = true;
                (
                    FrameKind::Response,
                    PluginMessage::HandshakeAccepted(HandshakeAccepted {
                        protocol: ProtocolVersion::V1_0,
                        plugin_id: PLUGIN_ID.to_owned(),
                        plugin_version: version,
                        target: TargetSpec::current(),
                        package_sha256: hello.package_sha256,
                        capabilities: vec![CapabilityDeclaration {
                            id: "paged-document/v1".to_owned(),
                            limits: ResourceLimits::default(),
                        }],
                    }),
                    Vec::new(),
                    false,
                )
            }
            _ if !handshaken => return Err("handshake required".into()),
            HostMessage::Ping => (FrameKind::Response, PluginMessage::Pong, Vec::new(), false),
            HostMessage::Cancel { request_id } => (
                FrameKind::Response,
                PluginMessage::Cancelled { request_id },
                Vec::new(),
                false,
            ),
            HostMessage::Shutdown => (
                FrameKind::Shutdown,
                PluginMessage::ShutdownAccepted,
                Vec::new(),
                true,
            ),
            HostMessage::PagedDocument(request) => {
                let reply = worker.handle(request);
                (FrameKind::Response, reply.message, reply.body, false)
            }
            HostMessage::Handshake(_) => (
                FrameKind::Response,
                PluginMessage::Error(PluginError {
                    code: PluginErrorCode::InvalidRequest,
                    retryable: false,
                }),
                Vec::new(),
                false,
            ),
        };
        let response = Frame::from_header(
            ProtocolVersion::V1_0,
            kind,
            frame.request_id,
            &response,
            body,
        )?;
        write_frame(&mut writer, &response, limits)?;
        if stop {
            break;
        }
    }
    worker.close_all();
    Ok(())
}
