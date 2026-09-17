use std::{
    env, fs,
    io::{self, BufRead, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

use markion_plugin_protocol::{
    Frame, FrameKind, FrameLimits, HandshakeAccepted, HostMessage, PluginMessage, ProtocolVersion,
    TargetSpec, read_frame, write_frame,
};
use semver::Version;

const FIXTURE_ID: &str = "dev.markion.fixture";

fn main() {
    if let Err(error) = run() {
        eprintln!("fixture worker failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if args.first().is_some_and(|arg| arg == "--probe") {
        println!(
            "{{\"plugin_id\":\"{FIXTURE_ID}\",\"protocol\":\"1.0\",\"os\":\"{}\",\"arch\":\"{}\",\"pid\":{}}}",
            env::consts::OS,
            env::consts::ARCH,
            std::process::id()
        );
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--lifecycle") {
        let directory = args.get(1).ok_or("--lifecycle requires a directory")?;
        return lifecycle_parent(Path::new(directory));
    }
    if args.first().is_some_and(|arg| arg == "--lifecycle-child") {
        let marker = args
            .get(1)
            .ok_or("--lifecycle-child requires an exit marker")?;
        return lifecycle_child(Path::new(marker));
    }
    framed_worker()
}

fn framed_worker() -> Result<(), Box<dyn std::error::Error>> {
    let limits = FrameLimits::default();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    while let Some(frame) = read_frame(&mut reader, limits)? {
        let message: HostMessage = frame.decode_header()?;
        let (kind, response, should_exit) = match message {
            HostMessage::Handshake(hello) => {
                if hello.protocol != ProtocolVersion::V1_0
                    || hello.expected_plugin_id != FIXTURE_ID
                    || hello.expected_plugin_version != Version::new(1, 0, 0)
                {
                    return Err("incompatible fixture handshake".into());
                }
                (
                    FrameKind::Response,
                    PluginMessage::HandshakeAccepted(HandshakeAccepted {
                        protocol: ProtocolVersion::V1_0,
                        plugin_id: FIXTURE_ID.to_owned(),
                        plugin_version: Version::new(1, 0, 0),
                        target: TargetSpec::current(),
                        package_sha256: hello.package_sha256,
                        capabilities: Vec::new(),
                    }),
                    false,
                )
            }
            HostMessage::Ping => (FrameKind::Response, PluginMessage::Pong, false),
            HostMessage::Cancel { request_id } => (
                FrameKind::Response,
                PluginMessage::Cancelled { request_id },
                false,
            ),
            HostMessage::Shutdown => (FrameKind::Shutdown, PluginMessage::ShutdownAccepted, true),
            HostMessage::PagedDocument(_) => return Err("fixture has no document".into()),
        };
        let response = Frame::from_header(
            ProtocolVersion::V1_0,
            kind,
            frame.request_id,
            &response,
            Vec::new(),
        )?;
        write_frame(&mut writer, &response, limits)?;
        if should_exit {
            break;
        }
    }
    Ok(())
}

fn lifecycle_parent(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(directory)?;
    let marker = directory.join("child-exited");
    let mut child = Command::new(env::current_exe()?)
        .arg("--lifecycle-child")
        .arg(&marker)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    println!("child_pid={}", child.id());
    io::stdout().flush()?;

    let mut input = io::stdin().lock();
    let mut sink = Vec::new();
    input.read_to_end(&mut sink)?;
    drop(child.stdin.take());
    let status = child.wait()?;
    if !status.success() {
        return Err("lifecycle child failed".into());
    }
    Ok(())
}

fn lifecycle_child(marker: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = io::stdin().lock();
    let mut sink = Vec::new();
    input.read_to_end(&mut sink)?;
    fs::write(marker, b"exited")?;
    Ok(())
}

#[allow(dead_code)]
fn read_ready_line<R: BufRead>(reader: &mut R) -> io::Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}
