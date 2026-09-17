use std::{env, fs, path::PathBuf, process};

use markion_plugin_protocol::{PluginCatalog, canonical_json, verify_minisign};

fn main() {
    if let Err(error) = run() {
        eprintln!("plugin catalog tool failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let command = args.next().ok_or("missing command")?;
    match command.to_string_lossy().as_ref() {
        "canonicalize" => {
            let input = PathBuf::from(args.next().ok_or("missing input catalog")?);
            let output = PathBuf::from(args.next().ok_or("missing output catalog")?);
            reject_extra_arguments(args)?;
            let catalog: PluginCatalog = serde_json::from_slice(&fs::read(input)?)?;
            fs::write(output, canonical_json(&catalog)?)?;
        }
        "verify" => {
            let catalog_path = PathBuf::from(args.next().ok_or("missing input catalog")?);
            let signature_path = PathBuf::from(args.next().ok_or("missing signature")?);
            let public_key_path = PathBuf::from(args.next().ok_or("missing public key")?);
            reject_extra_arguments(args)?;
            let catalog_bytes = fs::read(catalog_path)?;
            let catalog: PluginCatalog = serde_json::from_slice(&catalog_bytes)?;
            let canonical = canonical_json(&catalog)?;
            if canonical != catalog_bytes {
                return Err("catalog is not canonical JSON".into());
            }
            let signature = fs::read_to_string(signature_path)?;
            let public_key = fs::read_to_string(public_key_path)?;
            verify_minisign(&public_key, &signature, &canonical)?;
        }
        _ => return Err("expected `canonicalize` or `verify`".into()),
    }
    Ok(())
}

fn reject_extra_arguments(
    mut args: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(), String> {
    if args.next().is_some() {
        Err("unexpected extra argument".to_owned())
    } else {
        Ok(())
    }
}
