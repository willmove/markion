use std::{
    env, fs,
    io::{self, BufRead, Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use markion_plugin_protocol::{
    CapabilityDeclaration, Frame, FrameKind, FrameLimits, HandshakeAccepted, HostMessage,
    PAGED_DOCUMENT_CAPABILITY, PluginMessage, ProtocolVersion, ResourceLimits, TargetSpec,
    read_frame, write_frame,
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
    if args.first().is_some_and(|arg| arg == "--heartbeat-child") {
        let marker = args
            .get(1)
            .ok_or("--heartbeat-child requires a heartbeat path")?;
        return heartbeat_child(Path::new(marker));
    }
    framed_worker()
}

fn framed_worker() -> Result<(), Box<dyn std::error::Error>> {
    let mode = env::var("MARKION_PLUGIN_FIXTURE_MODE").unwrap_or_default();
    let limits = FrameLimits::default();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    let mut heartbeat = None;
    let mut ping_count = 0u32;
    while let Some(frame) = read_frame(&mut reader, limits)? {
        let message: HostMessage = frame.decode_header()?;
        let (kind, response, should_exit) = match message {
            HostMessage::Handshake(hello) => {
                if hello.protocol != ProtocolVersion::V1_0
                    || hello.expected_plugin_id != FIXTURE_ID
                    || hello.expected_plugin_version != Version::new(1, 0, 0)
                    || !hello
                        .requested_capabilities
                        .iter()
                        .all(|capability| capability == PAGED_DOCUMENT_CAPABILITY)
                {
                    return Err("incompatible fixture handshake".into());
                }
                let capabilities = hello
                    .requested_capabilities
                    .iter()
                    .map(|capability| CapabilityDeclaration {
                        id: capability.clone(),
                        limits: ResourceLimits::default(),
                    })
                    .collect();
                (
                    FrameKind::Response,
                    PluginMessage::HandshakeAccepted(HandshakeAccepted {
                        protocol: ProtocolVersion::V1_0,
                        plugin_id: FIXTURE_ID.to_owned(),
                        plugin_version: Version::new(1, 0, 0),
                        target: TargetSpec::current(),
                        package_sha256: if mode == "bad-handshake" {
                            "wrong-digest".to_owned()
                        } else {
                            hello.package_sha256
                        },
                        capabilities,
                    }),
                    false,
                )
            }
            HostMessage::Ping => {
                ping_count = ping_count.saturating_add(1);
                if mode == "hang-ping" || (mode == "hang-first-ping" && ping_count == 1) {
                    thread::sleep(Duration::from_secs(2));
                } else if mode == "crash-ping" {
                    std::process::exit(23);
                } else if mode == "malformed-ping" {
                    eprint!("{}", "diagnostic".repeat(8192));
                    io::stderr().flush()?;
                    writer.write_all(b"NOPE")?;
                    writer.flush()?;
                    return Ok(());
                } else if mode == "spawn-child-ping" && heartbeat.is_none() {
                    let scratch = env::var_os("MARKION_PLUGIN_FIXTURE_SCRATCH")
                        .ok_or("fixture scratch path is missing")?;
                    let scratch = Path::new(&scratch);
                    fs::create_dir_all(scratch)?;
                    let heartbeat_path = scratch.join("heartbeat");
                    let child = Command::new(env::current_exe()?)
                        .arg("--heartbeat-child")
                        .arg(&heartbeat_path)
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;
                    fs::write(scratch.join("child.pid"), child.id().to_string())?;
                    heartbeat = Some(child);
                }
                (FrameKind::Response, PluginMessage::Pong, false)
            }
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
    if let Some(mut child) = heartbeat {
        let _ = child.kill();
        let _ = child.wait();
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

fn heartbeat_child(marker: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;

    loop {
        let mut file = OpenOptions::new().create(true).append(true).open(marker)?;
        file.write_all(b".")?;
        file.flush()?;
        thread::sleep(Duration::from_millis(25));
    }
}

#[allow(dead_code)]
fn read_ready_line<R: BufRead>(reader: &mut R) -> io::Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line)
}
