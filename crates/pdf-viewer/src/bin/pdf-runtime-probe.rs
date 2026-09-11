use markion_pdf_viewer::{PDFIUM_BUILD, probe_runtime};
use std::path::PathBuf;

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(flag) = arguments.next() else {
        eprintln!("usage: pdf-runtime-probe --runtime <path>");
        std::process::exit(2);
    };
    let Some(runtime) = arguments.next() else {
        eprintln!("usage: pdf-runtime-probe --runtime <path>");
        std::process::exit(2);
    };
    if flag != "--runtime" || arguments.next().is_some() {
        eprintln!("usage: pdf-runtime-probe --runtime <path>");
        std::process::exit(2);
    }

    let runtime = PathBuf::from(runtime);
    match probe_runtime(&runtime) {
        Ok(()) => println!("PDFium {PDFIUM_BUILD} loaded from {}", runtime.display()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
