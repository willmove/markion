//! Preferences panel Export tab: immediate-apply setters over
//! `export_preferences`, the background pandoc-availability probe, and the
//! native pickers for the pandoc binary / DOCX reference template.

use super::*;

/// PDF margin stepper bounds in millimetres (sane printable range around the
/// 25 mm default; the config file itself accepts any u32).
const MIN_PDF_MARGIN_MM: u32 = 10;
const MAX_PDF_MARGIN_MM: u32 = 50;

impl MarkionApp {
    pub(super) fn set_export_backend(
        &mut self,
        backend: ExportBackendPreference,
        cx: &mut Context<Self>,
    ) {
        if self.export_preferences.backend == backend {
            return;
        }
        self.export_preferences.backend = backend;
        self.status = t(
            self.language,
            match backend {
                ExportBackendPreference::BuiltIn => Msg::StatusExportBackendBuiltin,
                ExportBackendPreference::Pandoc => Msg::StatusExportBackendPandoc,
            },
        )
        .into();
        if backend == ExportBackendPreference::Pandoc {
            self.refresh_pandoc_availability(cx);
        }
        self.persist_preferences();
        cx.notify();
    }

    /// Probes `pandoc --version` on a background executor so panel rendering
    /// never spawns processes; the cached result drives the availability
    /// line. Re-run when the Export tab opens, the backend switches to
    /// pandoc, or the pandoc path changes.
    pub(super) fn refresh_pandoc_availability(&mut self, cx: &mut Context<Self>) {
        let pandoc_path = self.export_preferences.pandoc_path.clone();
        self.pandoc_available_cached = None;
        cx.notify();
        cx.spawn(async move |this, cx| {
            let available = cx
                .background_spawn(async move { pandoc_available(pandoc_path.as_deref()) })
                .await;
            let _ = this.update(cx, |app, cx| {
                app.pandoc_available_cached = Some(available);
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn choose_pandoc_path(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let language = self.language;
        let picked = prompt_for_open_file(window, language, Msg::PrefExportPickPandocTitle, None);
        cx.spawn(async move |this, cx| {
            if let Some(path) = picked.await {
                let display_path = path.display().to_string();
                let _ = this.update(cx, |app, cx| {
                    app.export_preferences.pandoc_path = Some(display_path.clone());
                    app.status = tf(
                        app.language,
                        Msg::StatusExportPandocPathSet,
                        &[&display_path],
                    )
                    .into();
                    app.persist_preferences();
                    app.refresh_pandoc_availability(cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn reset_pandoc_path(&mut self, cx: &mut Context<Self>) {
        self.export_preferences.pandoc_path = None;
        self.status = t(self.language, Msg::StatusExportPandocPathReset).into();
        self.persist_preferences();
        self.refresh_pandoc_availability(cx);
    }

    pub(super) fn choose_reference_doc(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let language = self.language;
        let picked = prompt_for_open_file(
            window,
            language,
            Msg::PrefExportPickReferenceTitle,
            Some((Msg::PrefExportPickReferenceFilter, &["docx"])),
        );
        cx.spawn(async move |this, cx| {
            if let Some(path) = picked.await {
                let _ = this.update(cx, |app, cx| {
                    app.export_preferences.reference_doc = Some(path.display().to_string());
                    app.status = t(app.language, Msg::StatusExportReferenceDocSet).into();
                    app.persist_preferences();
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub(super) fn reset_reference_doc(&mut self, cx: &mut Context<Self>) {
        self.export_preferences.reference_doc = None;
        self.status = t(self.language, Msg::StatusExportReferenceDocReset).into();
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_pandoc_pdf_engine(&mut self, engine: &str, cx: &mut Context<Self>) {
        self.export_preferences.pdf_engine = engine.to_string();
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_docx_page_size(&mut self, page_size: DocxPageSize, cx: &mut Context<Self>) {
        self.export_preferences.docx.page_size = page_size;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn toggle_docx_toc(&mut self, cx: &mut Context<Self>) {
        self.export_preferences.docx.toc = !self.export_preferences.docx.toc;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_docx_image_policy(
        &mut self,
        policy: DocxImagePolicy,
        cx: &mut Context<Self>,
    ) {
        self.export_preferences.docx.image_policy = policy;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_pdf_page_size(&mut self, page_size: PdfPageSize, cx: &mut Context<Self>) {
        self.export_preferences.pdf.page_size = page_size;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn step_pdf_margin(&mut self, delta: i32, cx: &mut Context<Self>) {
        let current = self.export_preferences.pdf.margin_mm as i32;
        let stepped =
            (current + delta).clamp(MIN_PDF_MARGIN_MM as i32, MAX_PDF_MARGIN_MM as i32) as u32;
        self.export_preferences.pdf.margin_mm = stepped;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn toggle_pdf_toc(&mut self, cx: &mut Context<Self>) {
        self.export_preferences.pdf.toc = !self.export_preferences.pdf.toc;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn toggle_pdf_page_numbers(&mut self, cx: &mut Context<Self>) {
        self.export_preferences.pdf.page_numbers = !self.export_preferences.pdf.page_numbers;
        self.persist_preferences();
        cx.notify();
    }
}
