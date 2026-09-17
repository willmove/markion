use std::{env, fs, path::PathBuf};

use markion_plugin_protocol::{PackageLimits, ProtocolVersion, TargetSpec, inspect_package};
use semver::Version;

const REQUIRED_LOCALES: &[&str] = &["en", "zh-Hans", "zh-Hant", "ja", "fr", "de", "es"];

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
    if args.next().is_some() {
        return Err("unexpected package inspector arguments".into());
    }

    let archive_bytes = fs::read(&archive)?;
    let public_key_text = fs::read_to_string(&public_key)?;
    let package = inspect_package(
        &archive_bytes,
        &public_key_text,
        &Version::new(0, 3, 9),
        ProtocolVersion::V1_0,
        &TargetSpec::current(),
        REQUIRED_LOCALES,
        PackageLimits::default(),
    )?;
    if let Some(output) = output {
        package.extract_to(&output)?;
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
