use markion_pdf_viewer::{DocumentId, NativePdfBackend, PdfRendererBackend, packaged_runtime_path};
use std::path::PathBuf;

fn usage() -> ! {
    eprintln!("usage: pdf-packaged-smoke --resource-root <path> --fixture <pdf>");
    std::process::exit(2);
}

fn main() {
    let mut arguments = std::env::args_os().skip(1);
    let Some(resource_flag) = arguments.next() else {
        usage();
    };
    let Some(resource_root) = arguments.next() else {
        usage();
    };
    let Some(fixture_flag) = arguments.next() else {
        usage();
    };
    let Some(fixture) = arguments.next() else {
        usage();
    };
    if resource_flag != "--resource-root"
        || fixture_flag != "--fixture"
        || arguments.next().is_some()
    {
        usage();
    }

    let resource_root = PathBuf::from(resource_root);
    let fixture = PathBuf::from(fixture);
    let runtime = packaged_runtime_path(&resource_root);
    let result = (|| {
        let mut backend = NativePdfBackend::load(&runtime)?;
        let document = DocumentId(1);
        let pages = backend.open(document, &fixture)?;
        let raster = backend.render(document, 0, 256)?;
        backend.close(document);
        if pages.len() != 1
            || raster.width_px == 0
            || raster.height_px == 0
            || raster.rgba.is_empty()
        {
            return Err(markion_pdf_viewer::PdfErrorKind::InvalidRaster);
        }
        Ok::<_, markion_pdf_viewer::PdfErrorKind>((pages.len(), raster.byte_len()))
    })();

    match result {
        Ok((pages, bytes)) => println!(
            "Packaged PDF smoke passed: runtime={} pages={pages} raster_bytes={bytes}",
            runtime.display()
        ),
        Err(error) => {
            eprintln!(
                "Packaged PDF smoke failed for runtime {}: {error}",
                runtime.display()
            );
            std::process::exit(1);
        }
    }
}
