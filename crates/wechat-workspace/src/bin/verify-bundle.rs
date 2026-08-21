use std::{env, path::PathBuf, process::ExitCode};

use wechat_workspace::{discover_workspace_assets, verify_bundle};

fn main() -> ExitCode {
    let root = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(discover_workspace_assets);
    let root = match root {
        Ok(root) => root,
        Err(error) => {
            eprintln!("workspace discovery failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    match verify_bundle(&root) {
        Ok(result) => {
            println!(
                "verified {} files ({} bytes), MarkNice {}",
                result.file_count, result.total_bytes, result.source_commit
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("workspace verification failed: {error}");
            ExitCode::FAILURE
        }
    }
}
