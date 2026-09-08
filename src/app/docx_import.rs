use super::*;

use markion_docx_import::{
    CancellationToken, DiagnosticCode, DiagnosticSeverity, ImportError, ImportLimits,
    PreparedImport,
};

#[derive(Default)]
pub(super) struct DocxImportCoordinator {
    generation: u64,
    cancellation: Option<CancellationToken>,
    active: bool,
    saving: bool,
}

impl DocxImportCoordinator {
    fn begin(&mut self) -> Option<u64> {
        if self.saving {
            return None;
        }
        self.cancel_worker();
        self.generation = self.generation.wrapping_add(1);
        self.active = true;
        Some(self.generation)
    }

    fn install_worker(&mut self, generation: u64, token: CancellationToken) -> bool {
        if !self.accepts(generation) || self.saving {
            token.cancel();
            return false;
        }
        self.cancellation = Some(token);
        true
    }

    fn accepts(&self, generation: u64) -> bool {
        self.active && self.generation == generation
    }

    fn finish_conversion(&mut self, generation: u64) -> bool {
        if !self.accepts(generation) {
            return false;
        }
        self.cancellation = None;
        true
    }

    fn start_saving(&mut self, generation: u64) -> bool {
        if !self.accepts(generation) || self.saving {
            return false;
        }
        self.cancellation = None;
        self.saving = true;
        true
    }

    fn complete(&mut self, generation: u64) -> bool {
        if !self.accepts(generation) || self.saving {
            return false;
        }
        self.cancellation = None;
        self.active = false;
        true
    }

    fn finish_saving(&mut self, generation: u64) -> bool {
        if self.generation != generation || !self.saving {
            return false;
        }
        self.saving = false;
        self.active = false;
        true
    }

    fn cancel(&mut self) -> bool {
        if !self.active || self.saving {
            return false;
        }
        self.cancel_worker();
        self.active = false;
        self.generation = self.generation.wrapping_add(1);
        true
    }

    fn cancel_worker(&mut self) -> bool {
        let Some(token) = self.cancellation.take() else {
            return false;
        };
        token.cancel();
        true
    }

    pub(super) fn is_running(&self) -> bool {
        self.active && !self.saving
    }
}

impl Drop for DocxImportCoordinator {
    fn drop(&mut self) {
        self.cancel_worker();
    }
}

enum DocxImportFailure {
    Read(String),
    Convert(ImportError),
}

impl MarkionApp {
    pub(super) fn import_docx(
        &mut self,
        _: &ImportDocx,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(generation) = self.docx_import.begin() else {
            self.status = self.tr(Msg::StatusDocxImportSaving).into();
            self.active_menu = None;
            cx.notify();
            return;
        };
        let picker = prompt_for_open_file(
            window,
            self.language,
            Msg::PromptImportDocx,
            Some((Msg::FileTypeDocx, &["docx"])),
        );
        let window_handle = window.window_handle();
        self.active_menu = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let Some(source) = picker.await else {
                let _ = this.update(cx, |app, cx| {
                    if app.docx_import.complete(generation) {
                        app.status = app.tr(Msg::StatusDocxImportCanceled).into();
                        cx.notify();
                    }
                });
                return;
            };

            let cancellation = CancellationToken::default();
            let worker_token = cancellation.clone();
            let accepted = this
                .update(cx, |app, cx| {
                    if !app.docx_import.install_worker(generation, cancellation) {
                        return false;
                    }
                    app.status = app.tr(Msg::StatusDocxImportReading).into();
                    cx.notify();
                    true
                })
                .unwrap_or(false);
            if !accepted {
                return;
            }

            let source_for_worker = source.clone();
            let outcome = cx
                .background_executor()
                .spawn(async move {
                    let limits = ImportLimits::default();
                    let metadata = fs::metadata(&source_for_worker)
                        .map_err(|error| DocxImportFailure::Read(error.to_string()))?;
                    if metadata.len() > limits.max_source_bytes as u64 {
                        return Err(DocxImportFailure::Convert(ImportError::SourceTooLarge));
                    }
                    let bytes = fs::read(&source_for_worker)
                        .map_err(|error| DocxImportFailure::Read(error.to_string()))?;
                    markion_docx_import::import_docx(&bytes, limits, worker_token)
                        .map_err(DocxImportFailure::Convert)
                })
                .await;

            let _ = window_handle.update(cx, |_, window, cx| {
                let _ = this.update(cx, |app, cx| {
                    if !app.docx_import.finish_conversion(generation) {
                        return;
                    }
                    match outcome {
                        Ok(prepared) => {
                            app.prompt_docx_import_report(generation, source, prepared, window, cx);
                        }
                        Err(error) => {
                            app.docx_import.complete(generation);
                            let detail = localized_import_error(app.language, &error);
                            app.status = app.trf(Msg::StatusDocxImportFailed, &[&detail]);
                            cx.notify();
                        }
                    }
                });
            });
        })
        .detach();
    }

    pub(super) fn cancel_docx_import(
        &mut self,
        _: &CancelDocxImport,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.docx_import.cancel() {
            self.status = self.tr(Msg::StatusDocxImportCanceled).into();
            self.active_menu = None;
            cx.notify();
        }
    }

    fn prompt_docx_import_report(
        &mut self,
        generation: u64,
        source: PathBuf,
        prepared: PreparedImport,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let has_content_loss = prepared.has_content_loss();
        let detail = docx_import_report(self.language, &prepared);
        let (buttons, accepted_index) = docx_report_buttons(self.language, has_content_loss);
        let source_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".docx");
        let title = format!("{} — {source_name}", self.tr(Msg::DialogDocxImportTitle));
        let answer = window.prompt(
            if has_content_loss {
                PromptLevel::Warning
            } else {
                PromptLevel::Info
            },
            &title,
            Some(&detail),
            &buttons,
            cx,
        );
        let window_handle = window.window_handle();
        cx.spawn(async move |this, cx| {
            if !matches!(answer.await, Ok(index) if index == accepted_index) {
                let _ = this.update(cx, |app, cx| {
                    if app.docx_import.complete(generation) {
                        app.status = app.tr(Msg::StatusDocxImportCanceled).into();
                        cx.notify();
                    }
                });
                return;
            }
            let _ = window_handle.update(cx, |_, window, cx| {
                let _ = this.update(cx, |app, cx| {
                    if app.docx_import.accepts(generation) {
                        app.choose_docx_import_destination(
                            generation, source, prepared, window, cx,
                        );
                    }
                });
            });
        })
        .detach();
    }

    fn choose_docx_import_destination(
        &mut self,
        generation: u64,
        source: PathBuf,
        prepared: PreparedImport,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let directory = source
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(&self.workspace_root);
        let stem = source
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("imported");
        let suggested = SaveTarget::DocxImportMarkdown.suggested_name(stem);
        let picker = prompt_for_save_path(
            window,
            directory,
            &suggested,
            self.language,
            SaveTarget::DocxImportMarkdown,
        );
        let window_handle = window.window_handle();
        self.status = self.tr(Msg::StatusChoosingSaveLocation).into();
        cx.notify();

        cx.spawn(async move |this, cx| {
            let Some(destination) = picker.await else {
                let _ = this.update(cx, |app, cx| {
                    if app.docx_import.complete(generation) {
                        app.status = app.tr(Msg::StatusDocxImportCanceled).into();
                        cx.notify();
                    }
                });
                return;
            };

            let may_publish = this
                .update(cx, |app, cx| {
                    if !app.docx_import.accepts(generation) {
                        return false;
                    }
                    if find_tab_with_document_path(&app.tabs, &destination).is_some() {
                        let display = destination.display().to_string();
                        app.status = app.trf(Msg::StatusDocxImportDestinationOpen, &[&display]);
                        app.docx_import.complete(generation);
                        cx.notify();
                        return false;
                    }
                    if !app.docx_import.start_saving(generation) {
                        return false;
                    }
                    app.status = app.tr(Msg::StatusDocxImportSaving).into();
                    cx.notify();
                    true
                })
                .unwrap_or(false);
            if !may_publish {
                return;
            }
            let destination_for_worker = destination.clone();
            let outcome = cx
                .background_executor()
                .spawn(
                    async move { markion::publish_docx_import(&prepared, &destination_for_worker) },
                )
                .await;

            let _ = window_handle.update(cx, |_, _window, cx| {
                let _ = this.update(cx, |app, cx| {
                    if !app.docx_import.finish_saving(generation) {
                        return;
                    }
                    match outcome {
                        Ok(published) => {
                            let display = published.markdown_path.display().to_string();
                            if let Err(error) = app.open_supported_path(
                                published.markdown_path,
                                OpenPathIntent::OpenInNewTab,
                                cx,
                            ) {
                                app.status = app
                                    .trf(Msg::StatusDocxImportSavedOpenFailed, &[&display, &error]);
                                cx.notify();
                            } else {
                                if published.retained_paths.is_empty() {
                                    app.status = app.trf(Msg::StatusSaved, &[&display]);
                                } else {
                                    let retained = published
                                        .retained_paths
                                        .iter()
                                        .map(|path| path.display().to_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    app.status = app.trf(
                                        Msg::StatusDocxImportSavedWithRemnants,
                                        &[&display, &retained],
                                    );
                                }
                                cx.notify();
                            }
                        }
                        Err(error) => {
                            app.status =
                                app.trf(Msg::StatusDocxImportFailed, &[&error.to_string()]);
                            cx.notify();
                        }
                    }
                });
            });
        })
        .detach();
    }
}

fn docx_import_report(language: Language, prepared: &PreparedImport) -> String {
    let mut info = 0usize;
    let mut formatting = 0usize;
    let mut content = 0usize;
    let mut lines = Vec::new();
    for diagnostic in &prepared.diagnostics {
        let severity = match diagnostic.severity {
            DiagnosticSeverity::Info => {
                info += 1;
                Msg::DiagnosticInfo
            }
            DiagnosticSeverity::FormattingLoss => {
                formatting += 1;
                Msg::DiagnosticFormattingLoss
            }
            DiagnosticSeverity::ContentLoss => {
                content += 1;
                Msg::DiagnosticContentLoss
            }
        };
        let mut line = format!(
            "• {}: {}",
            t(language, severity),
            t(language, diagnostic_message(diagnostic.code))
        );
        let location = match (diagnostic.part.as_deref(), diagnostic.location.as_deref()) {
            (Some(part), Some(location)) => Some(format!("{part}; {location}")),
            (Some(part), None) => Some(part.to_owned()),
            (None, Some(location)) => Some(location.to_owned()),
            (None, None) => None,
        };
        if let Some(location) = location {
            line.push_str(" [");
            line.push_str(&location);
            line.push(']');
        }
        if let Some(context) = diagnostic.context.as_deref() {
            let context = context.split_whitespace().collect::<Vec<_>>().join(" ");
            if !context.is_empty() {
                line.push_str(": ");
                line.extend(context.chars().take(120));
            }
        }
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(format!("• {}", t(language, Msg::DiagnosticInfo)));
    }

    tf(
        language,
        Msg::DialogDocxImportSummary,
        &[
            &prepared.summary.paragraphs.to_string(),
            &prepared.summary.tables.to_string(),
            &prepared.summary.images.to_string(),
            &prepared.summary.footnotes.to_string(),
            &info.to_string(),
            &formatting.to_string(),
            &content.to_string(),
            &lines.join("\n"),
        ],
    )
}

fn docx_report_buttons(language: Language, has_content_loss: bool) -> (Vec<PromptButton>, usize) {
    if has_content_loss {
        (
            vec![
                PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
                PromptButton::ok(t(language, Msg::DialogButtonContinueImport)),
            ],
            1,
        )
    } else {
        (
            vec![
                PromptButton::ok(t(language, Msg::DialogButtonSaveImport)),
                PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
            ],
            0,
        )
    }
}

const fn diagnostic_message(code: DiagnosticCode) -> Msg {
    match code {
        DiagnosticCode::LayoutNormalized => Msg::DocxDiagLayoutNormalized,
        DiagnosticCode::HeadingDepthReduced => Msg::DocxDiagHeadingDepthReduced,
        DiagnosticCode::NumberingNormalized => Msg::DocxDiagNumberingNormalized,
        DiagnosticCode::UnsafeHyperlink => Msg::DocxDiagUnsafeHyperlink,
        DiagnosticCode::MissingBookmark => Msg::DocxDiagMissingBookmark,
        DiagnosticCode::MergedTableHtml => Msg::DocxDiagMergedTableHtml,
        DiagnosticCode::ComplexTableSimplified => Msg::DocxDiagComplexTableSimplified,
        DiagnosticCode::FloatingImageNormalized => Msg::DocxDiagFloatingImageNormalized,
        DiagnosticCode::MissingImage => Msg::DocxDiagMissingImage,
        DiagnosticCode::UnsupportedImage => Msg::DocxDiagUnsupportedImage,
        DiagnosticCode::UnsupportedEquation => Msg::DocxDiagUnsupportedEquation,
        DiagnosticCode::RevisionsAccepted => Msg::DocxDiagRevisionsAccepted,
        DiagnosticCode::AmbiguousRevision => Msg::DocxDiagAmbiguousRevision,
        DiagnosticCode::UnsupportedContent => Msg::DocxDiagUnsupportedContent,
        DiagnosticCode::ExternalResourceBlocked => Msg::DocxDiagExternalResourceBlocked,
        DiagnosticCode::FieldResultPreserved => Msg::DocxDiagFieldResultPreserved,
    }
}

fn localized_import_error(language: Language, error: &DocxImportFailure) -> String {
    match error {
        DocxImportFailure::Read(detail) => tf(language, Msg::DocxErrorRead, &[detail]),
        DocxImportFailure::Convert(ImportError::SourceTooLarge) => {
            t(language, Msg::DocxErrorSourceTooLarge).to_owned()
        }
        DocxImportFailure::Convert(ImportError::MacroEnabled) => {
            t(language, Msg::DocxErrorMacroEnabled).to_owned()
        }
        DocxImportFailure::Convert(ImportError::Encrypted) => {
            t(language, Msg::DocxErrorEncrypted).to_owned()
        }
        DocxImportFailure::Convert(ImportError::LimitExceeded(_)) => {
            t(language, Msg::DocxErrorLimitExceeded).to_owned()
        }
        DocxImportFailure::Convert(ImportError::DeadlineExceeded) => {
            t(language, Msg::DocxErrorDeadline).to_owned()
        }
        DocxImportFailure::Convert(ImportError::EmptyDocument) => {
            t(language, Msg::DocxErrorEmpty).to_owned()
        }
        DocxImportFailure::Convert(ImportError::Cancelled) => {
            t(language, Msg::StatusDocxImportCanceled).to_owned()
        }
        DocxImportFailure::Convert(
            ImportError::InvalidPackage(_)
            | ImportError::MissingPart(_)
            | ImportError::InvalidRelationship(_)
            | ImportError::InvalidXml { .. }
            | ImportError::MissingAssetReference(_),
        ) => t(language, Msg::DocxErrorInvalidDocument).to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use markion_docx_import::{Diagnostic, ImportSummary, MarkdownChunk, PackageStats};

    #[test]
    fn report_counts_diagnostics_and_localizes_labels() {
        let prepared = PreparedImport {
            chunks: vec![MarkdownChunk::Text("hello".into())],
            assets: Vec::new(),
            diagnostics: vec![Diagnostic {
                code: DiagnosticCode::UnsupportedContent,
                severity: DiagnosticSeverity::ContentLoss,
                part: Some("word/document.xml".into()),
                location: None,
                context: Some("drawing canvas".into()),
            }],
            summary: ImportSummary {
                paragraphs: 1,
                tables: 0,
                images: 0,
                footnotes: 0,
                revisions_accepted: false,
            },
            package: PackageStats {
                entries: 1,
                decompressed_bytes: 1,
                limits: ImportLimits::default(),
            },
        };

        let report = docx_import_report(Language::ZhHans, &prepared);
        assert!(report.contains("可能的内容丢失"));
        assert!(report.contains("已省略不支持的文档内容"));
        assert!(report.contains("drawing canvas"));
    }

    #[test]
    fn coordinator_cancels_previous_worker_and_rejects_stale_results() {
        let mut coordinator = DocxImportCoordinator::default();
        let first_generation = coordinator.begin().unwrap();
        let first_token = CancellationToken::default();
        assert!(coordinator.install_worker(first_generation, first_token.clone()));
        let second_generation = coordinator.begin().unwrap();
        assert!(first_token.is_cancelled());
        assert!(!coordinator.finish_conversion(first_generation));
        let second_token = CancellationToken::default();
        assert!(coordinator.install_worker(second_generation, second_token));
        assert!(coordinator.finish_conversion(second_generation));
        assert!(coordinator.is_running());
        assert!(coordinator.complete(second_generation));
        assert!(!coordinator.is_running());
    }

    #[test]
    fn dropping_coordinator_cancels_worker() {
        let token = CancellationToken::default();
        {
            let mut coordinator = DocxImportCoordinator::default();
            let generation = coordinator.begin().unwrap();
            assert!(coordinator.install_worker(generation, token.clone()));
        }
        assert!(token.is_cancelled());
    }

    #[test]
    fn coordinator_cancels_review_and_picker_states_but_not_publication() {
        let mut coordinator = DocxImportCoordinator::default();
        let review_generation = coordinator.begin().unwrap();
        assert!(coordinator.is_running());
        assert!(coordinator.cancel());
        assert!(!coordinator.accepts(review_generation));

        let saving_generation = coordinator.begin().unwrap();
        assert!(coordinator.start_saving(saving_generation));
        assert!(!coordinator.is_running());
        assert!(!coordinator.cancel());
        assert!(coordinator.begin().is_none());
        assert!(coordinator.finish_saving(saving_generation));
        assert!(coordinator.begin().is_some());
    }

    #[test]
    fn fatal_import_categories_are_localized() {
        assert_eq!(
            localized_import_error(
                Language::ZhHans,
                &DocxImportFailure::Convert(ImportError::MacroEnabled)
            ),
            "不支持启用宏的 Word 文档"
        );
        assert_eq!(
            localized_import_error(
                Language::De,
                &DocxImportFailure::Convert(ImportError::Encrypted)
            ),
            "Verschlüsselte Word-Dokumente werden nicht unterstützt"
        );
    }

    #[test]
    fn content_loss_report_requires_non_default_continue_action() {
        let (buttons, accepted) = docx_report_buttons(Language::En, true);
        assert!(matches!(buttons[0], PromptButton::Cancel(_)));
        assert!(matches!(buttons[1], PromptButton::Ok(_)));
        assert_eq!(accepted, 1);

        let (buttons, accepted) = docx_report_buttons(Language::En, false);
        assert!(matches!(buttons[0], PromptButton::Ok(_)));
        assert_eq!(accepted, 0);
    }
}
