use std::{
    env, fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use markion_plugin_protocol::{
    Frame, FrameKind, FrameLimits, HandshakeHello, HostMessage, PluginMessage, ProtocolVersion,
    read_frame, write_frame,
};
use semver::Version;

fn main() {
    if let Err(error) = run() {
        eprintln!("fixture host failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let worker = PathBuf::from(args.next().ok_or("worker path is required")?);
    let scratch = PathBuf::from(args.next().ok_or("scratch path is required")?);
    if args.next().is_some() {
        return Err("unexpected fixture host arguments".into());
    }
    if !worker.is_file() {
        return Err("worker path is not a file".into());
    }
    fs::create_dir_all(&scratch)?;
    verify_handshake(&worker)?;
    verify_forced_parent_cleanup(&worker, &scratch)?;
    println!(
        "{{\"handshake\":true,\"forced_parent_cleanup\":true,\"worker\":\"{}\"}}",
        worker.file_name().unwrap_or_default().to_string_lossy()
    );
    Ok(())
}

fn verify_handshake(worker: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(worker)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("worker stdin unavailable")?;
    let mut stdout = child.stdout.take().ok_or("worker stdout unavailable")?;
    let limits = FrameLimits::default();
    let hello = HostMessage::Handshake(HandshakeHello {
        protocol: ProtocolVersion::V1_0,
        host_version: Version::new(0, 3, 9),
        expected_plugin_id: "dev.markion.fixture".to_owned(),
        expected_plugin_version: Version::new(1, 0, 0),
        package_sha256: "fixture".to_owned(),
        requested_capabilities: Vec::new(),
    });
    write_frame(
        &mut stdin,
        &Frame::from_header(
            ProtocolVersion::V1_0,
            FrameKind::Request,
            1,
            &hello,
            Vec::new(),
        )?,
        limits,
    )?;
    let response = read_frame(&mut stdout, limits)?.ok_or("worker ended before handshake")?;
    match response.decode_header::<PluginMessage>()? {
        PluginMessage::HandshakeAccepted(accepted)
            if accepted.plugin_id == "dev.markion.fixture"
                && accepted.protocol == ProtocolVersion::V1_0 => {}
        _ => return Err("worker returned an invalid handshake".into()),
    }
    write_frame(
        &mut stdin,
        &Frame::from_header(
            ProtocolVersion::V1_0,
            FrameKind::Shutdown,
            2,
            &HostMessage::Shutdown,
            Vec::new(),
        )?,
        limits,
    )?;
    let response = read_frame(&mut stdout, limits)?.ok_or("worker ended before shutdown ack")?;
    if response.decode_header::<PluginMessage>()? != PluginMessage::ShutdownAccepted {
        return Err("worker did not acknowledge shutdown".into());
    }
    drop(stdin);
    if !child.wait()?.success() {
        return Err("worker exited unsuccessfully".into());
    }
    Ok(())
}

fn verify_forced_parent_cleanup(
    worker: &Path,
    scratch: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let lifecycle = scratch.join("lifecycle");
    if lifecycle.exists() {
        fs::remove_dir_all(&lifecycle)?;
    }
    fs::create_dir_all(&lifecycle)?;
    let mut child = Command::new(worker)
        .arg("--lifecycle")
        .arg(&lifecycle)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("lifecycle stdout unavailable")?;
    let mut reader = BufReader::new(stdout);
    let mut ready = String::new();
    reader.read_line(&mut ready)?;
    if !ready.starts_with("child_pid=") {
        return Err("lifecycle child did not become ready".into());
    }
    child.kill()?;
    child.wait()?;

    let marker = lifecycle.join("child-exited");
    let deadline = Instant::now() + Duration::from_secs(10);
    while !marker.is_file() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    if !marker.is_file() {
        return Err("worker child survived forced parent termination".into());
    }
    Ok(())
}
