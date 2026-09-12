use markion_pdf_viewer::{PDFIUM_BUILD, discover_runtime, probe_runtime};
use std::path::PathBuf;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(flag) = arguments.next() else {
        print_usage();
        std::process::exit(2);
    };
    let Some(value) = arguments.next() else {
        print_usage();
        std::process::exit(2);
    };
    if arguments.next().is_some() {
        print_usage();
        std::process::exit(2);
    }

    let runtime = if flag == "--runtime" {
        Ok(PathBuf::from(value))
    } else if flag == "--resource-root" {
        discover_runtime(PathBuf::from(value))
    } else {
        print_usage();
        std::process::exit(2);
    };
    let runtime = match runtime {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("PDFium {PDFIUM_BUILD} discovery failed: {error}");
            std::process::exit(1);
        }
    };
    match probe_runtime(&runtime) {
        Ok(()) => println!("PDFium {PDFIUM_BUILD} loaded from {}", runtime.display()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: pdf-runtime-probe (--runtime <path> | --resource-root <application-resource-root>)"
    );
}
