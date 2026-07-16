use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};

use gpui::Window;
use markion::{ExportFormat, Language, Msg, t};
#[cfg(target_os = "windows")]
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle, WindowHandle,
};
use rfd::AsyncFileDialog;

const KNOWN_OUTPUT_EXTENSIONS: &[&str] = &[
    "md", "markdown", "mdown", "html", "htm", "pdf", "tex", "latex", "docx", "png", "jpg", "jpeg",
];

#[cfg(target_os = "windows")]
struct WindowsDialogParent(RawWindowHandle);

#[cfg(target_os = "windows")]
impl HasWindowHandle for WindowsDialogParent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // SAFETY: The handle is borrowed from the live GPUI window for the
        // duration of dialog construction. `rfd` copies the raw handle.
        Ok(unsafe { WindowHandle::borrow_raw(self.0) })
    }
}

#[cfg(target_os = "windows")]
impl HasDisplayHandle for WindowsDialogParent {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::windows())
    }
}

#[cfg(target_os = "windows")]
fn set_dialog_parent(dialog: AsyncFileDialog, window: &Window) -> AsyncFileDialog {
    match HasWindowHandle::window_handle(window) {
        Ok(handle) => dialog.set_parent(&WindowsDialogParent(handle.as_raw())),
        Err(_) => dialog,
    }
}

#[cfg(not(target_os = "windows"))]
fn set_dialog_parent(dialog: AsyncFileDialog, window: &Window) -> AsyncFileDialog {
    dialog.set_parent(window)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SaveTarget {
    Markdown,
    Export(ExportFormat),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SaveTargetProfile {
    pub title: Msg,
    pub filter_label: Msg,
    pub accepted_extensions: &'static [&'static str],
    pub canonical_extension: &'static str,
    pub suggested_suffix: &'static str,
}

impl SaveTarget {
    pub(super) const fn profile(self) -> SaveTargetProfile {
        match self {
            Self::Markdown | Self::Export(ExportFormat::Markdown) => SaveTargetProfile {
                title: Msg::ItemSaveAs,
                filter_label: Msg::FileTypeMarkdown,
                accepted_extensions: &["md", "markdown", "mdown"],
                canonical_extension: "md",
                suggested_suffix: "md",
            },
            Self::Export(ExportFormat::Html) => SaveTargetProfile {
                title: Msg::ItemExportHtml,
                filter_label: Msg::FileTypeStyledHtml,
                accepted_extensions: &["html", "htm"],
                canonical_extension: "html",
                suggested_suffix: "html",
            },
            Self::Export(ExportFormat::PlainHtml) => SaveTargetProfile {
                title: Msg::ItemExportPlainHtml,
                filter_label: Msg::FileTypePlainHtml,
                accepted_extensions: &["html", "htm"],
                canonical_extension: "html",
                suggested_suffix: "plain.html",
            },
            Self::Export(ExportFormat::Pdf) => SaveTargetProfile {
                title: Msg::ItemExportPdf,
                filter_label: Msg::FileTypePdf,
                accepted_extensions: &["pdf"],
                canonical_extension: "pdf",
                suggested_suffix: "pdf",
            },
            Self::Export(ExportFormat::Latex) => SaveTargetProfile {
                title: Msg::ItemExportLatex,
                filter_label: Msg::FileTypeLatex,
                accepted_extensions: &["tex", "latex"],
                canonical_extension: "tex",
                suggested_suffix: "tex",
            },
            Self::Export(ExportFormat::Docx) => SaveTargetProfile {
                title: Msg::ItemExportDocx,
                filter_label: Msg::FileTypeDocx,
                accepted_extensions: &["docx"],
                canonical_extension: "docx",
                suggested_suffix: "docx",
            },
            Self::Export(ExportFormat::Png) => SaveTargetProfile {
                title: Msg::ItemExportPng,
                filter_label: Msg::FileTypePng,
                accepted_extensions: &["png"],
                canonical_extension: "png",
                suggested_suffix: "png",
            },
            Self::Export(ExportFormat::Jpeg) => SaveTargetProfile {
                title: Msg::ItemExportJpeg,
                filter_label: Msg::FileTypeJpeg,
                accepted_extensions: &["jpg", "jpeg"],
                canonical_extension: "jpg",
                suggested_suffix: "jpg",
            },
        }
    }

    pub(super) fn normalize_path(self, path: impl AsRef<Path>) -> PathBuf {
        let profile = self.profile();
        let path = path.as_ref();
        let accepted = path
            .extension()
            .and_then(|extension| extension.to_str())
            .filter(|extension| !extension.is_empty())
            .is_some_and(|extension| {
                profile
                    .accepted_extensions
                    .iter()
                    .any(|accepted| extension.eq_ignore_ascii_case(accepted))
            });

        if accepted && !has_appended_incompatible_extension(path, profile) {
            return path.to_path_buf();
        }

        let mut normalized = if accepted {
            path.with_extension("")
        } else {
            path.to_path_buf()
        };
        normalized.set_extension(profile.canonical_extension);
        normalized
    }

    pub(super) fn suggested_name(self, stem: &str) -> String {
        format!("{stem}.{}", self.profile().suggested_suffix)
    }
}

fn has_appended_incompatible_extension(path: &Path, profile: SaveTargetProfile) -> bool {
    path.file_stem()
        .map(Path::new)
        .and_then(Path::extension)
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            KNOWN_OUTPUT_EXTENSIONS
                .iter()
                .any(|known| extension.eq_ignore_ascii_case(known))
                && !profile
                    .accepted_extensions
                    .iter()
                    .any(|accepted| extension.eq_ignore_ascii_case(accepted))
        })
}

pub(super) fn prompt_for_save_path(
    window: &Window,
    directory: &Path,
    suggested_name: &str,
    language: Language,
    target: SaveTarget,
) -> Pin<Box<dyn Future<Output = Option<PathBuf>> + 'static>> {
    let profile = target.profile();
    let dialog = set_dialog_parent(AsyncFileDialog::new(), window)
        .set_directory(directory)
        .set_file_name(suggested_name)
        .set_title(t(language, profile.title))
        .add_filter(
            t(language, profile.filter_label),
            profile.accepted_extensions,
        );

    Box::pin(async move {
        dialog
            .save_file()
            .await
            .map(|handle| target.normalize_path(handle.path()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use markion::MarkdownDocument;

    #[test]
    fn save_target_profiles_cover_markdown_and_every_export_format() {
        let cases = [
            (
                SaveTarget::Markdown,
                Msg::ItemSaveAs,
                Msg::FileTypeMarkdown,
                &["md", "markdown", "mdown"][..],
                "md",
                "md",
            ),
            (
                SaveTarget::Export(ExportFormat::Html),
                Msg::ItemExportHtml,
                Msg::FileTypeStyledHtml,
                &["html", "htm"][..],
                "html",
                "html",
            ),
            (
                SaveTarget::Export(ExportFormat::PlainHtml),
                Msg::ItemExportPlainHtml,
                Msg::FileTypePlainHtml,
                &["html", "htm"][..],
                "html",
                "plain.html",
            ),
            (
                SaveTarget::Export(ExportFormat::Pdf),
                Msg::ItemExportPdf,
                Msg::FileTypePdf,
                &["pdf"][..],
                "pdf",
                "pdf",
            ),
            (
                SaveTarget::Export(ExportFormat::Latex),
                Msg::ItemExportLatex,
                Msg::FileTypeLatex,
                &["tex", "latex"][..],
                "tex",
                "tex",
            ),
            (
                SaveTarget::Export(ExportFormat::Docx),
                Msg::ItemExportDocx,
                Msg::FileTypeDocx,
                &["docx"][..],
                "docx",
                "docx",
            ),
            (
                SaveTarget::Export(ExportFormat::Png),
                Msg::ItemExportPng,
                Msg::FileTypePng,
                &["png"][..],
                "png",
                "png",
            ),
            (
                SaveTarget::Export(ExportFormat::Jpeg),
                Msg::ItemExportJpeg,
                Msg::FileTypeJpeg,
                &["jpg", "jpeg"][..],
                "jpg",
                "jpg",
            ),
        ];

        for (target, title, filter, extensions, canonical, suffix) in cases {
            let profile = target.profile();
            assert_eq!(profile.title, title);
            assert_eq!(profile.filter_label, filter);
            assert_eq!(profile.accepted_extensions, extensions);
            assert_eq!(profile.canonical_extension, canonical);
            assert_eq!(profile.suggested_suffix, suffix);
        }
    }

    #[test]
    fn path_normalization_preserves_aliases_and_repairs_other_extensions() {
        let markdown = SaveTarget::Markdown;
        assert_eq!(markdown.normalize_path("notes"), PathBuf::from("notes.md"));
        assert_eq!(markdown.normalize_path("notes."), PathBuf::from("notes.md"));
        assert_eq!(
            markdown.normalize_path("notes.txt"),
            PathBuf::from("notes.md")
        );
        assert_eq!(
            markdown.normalize_path("NOTES.MarkDown"),
            PathBuf::from("NOTES.MarkDown")
        );
        assert_eq!(
            markdown.normalize_path("notes.pdf.md"),
            PathBuf::from("notes.md")
        );

        let latex = SaveTarget::Export(ExportFormat::Latex);
        assert_eq!(
            latex.normalize_path("论文 final.LATEX"),
            PathBuf::from("论文 final.LATEX")
        );
        assert_eq!(
            latex.normalize_path("论文 final.docx"),
            PathBuf::from("论文 final.tex")
        );

        let jpeg = SaveTarget::Export(ExportFormat::Jpeg);
        assert_eq!(
            jpeg.normalize_path("snapshot.JPEG"),
            PathBuf::from("snapshot.JPEG")
        );

        let html = SaveTarget::Export(ExportFormat::Html);
        assert_eq!(
            html.normalize_path("report.docx.html"),
            PathBuf::from("report.html")
        );
        assert_eq!(
            html.normalize_path("report.v1.html"),
            PathBuf::from("report.v1.html")
        );
        assert_eq!(
            SaveTarget::Export(ExportFormat::PlainHtml).normalize_path("report.plain.html"),
            PathBuf::from("report.plain.html")
        );
    }

    #[test]
    fn selected_export_target_remains_authoritative_over_typed_extension() {
        let target = SaveTarget::Export(ExportFormat::Pdf);
        assert_eq!(
            target.normalize_path("report.docx"),
            PathBuf::from("report.pdf")
        );
        assert_eq!(target, SaveTarget::Export(ExportFormat::Pdf));
        assert_eq!(
            SaveTarget::Export(ExportFormat::PlainHtml).suggested_name("report"),
            "report.plain.html"
        );
    }

    #[test]
    fn preparing_save_metadata_does_not_mutate_document_or_versioned_cache() {
        let document = MarkdownDocument::from_text("# Title\n\nBody");
        let version = document.version();
        let line_count = document.line_count();
        let text = document.text().to_owned();
        let dirty = document.is_dirty();

        let target = SaveTarget::Export(ExportFormat::Html);
        let _profile = target.profile();
        let _suggested_name = target.suggested_name("Untitled");

        assert_eq!(document.version(), version);
        assert_eq!(document.line_count(), line_count);
        assert_eq!(document.text(), text);
        assert_eq!(document.is_dirty(), dirty);
        assert!(document.path().is_none());
    }
}
