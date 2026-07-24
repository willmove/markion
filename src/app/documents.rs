use super::*;

impl MarkionApp {
    pub(super) fn new_document(
        &mut self,
        _: &NewDocument,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.confirm_discard_then(
            window,
            cx,
            Msg::DialogDiscardTitle,
            Msg::DialogDiscardNewDetail,
            Self::new_document_confirmed,
        );
    }

    pub(super) fn new_document_confirmed(&mut self, cx: &mut Context<Self>) {
        self.replace_active_tab(MarkdownDocument::new(), cx);
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusNewDocument).into();
        cx.notify();
    }

    pub(super) fn open_document(
        &mut self,
        _: &OpenDocument,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.confirm_discard_then(
            window,
            cx,
            Msg::DialogDiscardTitle,
            Msg::DialogDiscardOpenDetail,
            Self::open_document_confirmed,
        );
    }

    pub(super) fn open_document_confirmed(&mut self, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(self.tr(Msg::PromptOpenMarkdown).into()),
        });

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusOpening).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        let display_path = path.display().to_string();
                        let focused_existing = this
                            .update(cx, |app, cx| {
                                if app.focus_existing_tab_for_path(&path, cx) {
                                    app.record_recent_path(&path);
                                    app.active_menu = None;
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                    true
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);
                        if focused_existing {
                            return;
                        }

                        match MarkdownDocument::open(&path) {
                            Ok(document) => {
                                let _ = this.update(cx, |app, cx| {
                                    app.replace_active_tab(document, cx);
                                    app.active_menu = None;
                                    app.update_workspace_root_from_document(cx);
                                    app.record_recent_path(&path);
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                });
                                return;
                            }
                            Err(err) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                        }
                    } else {
                        t(language, Msg::StatusOpenCanceled).to_string()
                    }
                }
                Ok(Ok(None)) => t(language, Msg::StatusOpenCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusOpenCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn open_folder(&mut self, _: &OpenFolder, _: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(open_folder_prompt_options(self.language));

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusOpeningFolder).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        let display_path = path.display().to_string();
                        let _ = this.update(cx, |app, cx| {
                            app.set_workspace_root(path);
                            app.sidebar_visible = true;
                            app.sidebar_tab = SidebarTab::Files;
                            app.active_menu = None;
                            app.persist_preferences();
                            app.schedule_file_tree_scan(Some(display_path), cx);
                            cx.notify();
                        });
                        return;
                    }
                    t(language, Msg::StatusOpenFolderCanceled).to_string()
                }
                Ok(Ok(None)) => t(language, Msg::StatusOpenFolderCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusOpenFolderFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusOpenFolderCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn quit(&mut self, _: &Quit, window: &mut Window, cx: &mut Context<Self>) {
        self.request_quit(window, cx);
    }

    /// Action: open a fresh empty document in a brand-new tab. Unlike
    /// `NewDocument` (which replaces the active tab), this always adds a tab, so
    /// it is the only way to get a blank tab without going through a file.
    pub(super) fn new_tab(&mut self, _: &NewTab, _window: &mut Window, cx: &mut Context<Self>) {
        self.open_in_new_tab(MarkdownDocument::new(), cx);
        self.active_menu = None;
        self.status = t(self.language, Msg::StatusNewDocument).into();
        cx.notify();
    }

    /// Action: prompt for a file and open it in a brand-new tab.
    pub(super) fn open_in_new_tab_action(
        &mut self,
        _: &OpenInNewTab,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(self.tr(Msg::PromptOpenMarkdown).into()),
        });

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusOpening).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match receiver.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.into_iter().next() {
                        let display_path = path.display().to_string();
                        let focused_existing = this
                            .update(cx, |app, cx| {
                                if app.focus_existing_tab_for_path(&path, cx) {
                                    app.record_recent_path(&path);
                                    app.active_menu = None;
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                    true
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(false);
                        if focused_existing {
                            return;
                        }

                        match MarkdownDocument::open(&path) {
                            Ok(document) => {
                                let _ = this.update(cx, |app, cx| {
                                    app.open_in_new_tab(document, cx);
                                    app.active_menu = None;
                                    app.update_workspace_root_from_document(cx);
                                    app.record_recent_path(&path);
                                    app.status = app.trf(Msg::StatusOpened, &[&display_path]);
                                    cx.notify();
                                });
                                return;
                            }
                            Err(err) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                        }
                    } else {
                        t(language, Msg::StatusOpenCanceled).to_string()
                    }
                }
                Ok(Ok(None)) => t(language, Msg::StatusOpenCanceled).to_string(),
                Ok(Err(err)) => tf(language, Msg::StatusOpenFailed, &[&err.to_string()]),
                Err(_) => t(language, Msg::StatusOpenCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    /// Action: close the active tab. If it is dirty, confirm first. Closing the
    /// last tab leaves a fresh untitled document so the window stays open.
    pub(super) fn close_tab(&mut self, _: &CloseTab, window: &mut Window, cx: &mut Context<Self>) {
        self.confirm_discard_then(
            window,
            cx,
            Msg::DialogDiscardTitle,
            Msg::DialogDiscardNewDetail,
            Self::close_tab_confirmed,
        );
    }

    pub(super) fn close_tab_confirmed(&mut self, cx: &mut Context<Self>) {
        // Discard the active tab's recovery file before removing it.
        if let Some(recovery) = self.active_tab_mut().last_recovery_file.take() {
            let _ = delete_recovery_file(recovery);
        }
        if self.tabs.len() <= 1 {
            // Closing the last tab leaves a fresh untitled document.
            self.tabs[0] = self.editor_tab_for_document(MarkdownDocument::new());
            self.active_tab = 0;
        } else {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
        self.active_menu = None;
        self.refresh_search_matches();
        self.sync_and_persist_session();
        self.status = t(self.language, Msg::StatusNewDocument).into();
        cx.notify();
    }

    /// Action: cycle to the next tab (wraps). Bound to Ctrl+Tab.
    pub(super) fn next_tab(&mut self, _: &NextTab, _: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.len() > 1 {
            let index = (self.active_tab + 1) % self.tabs.len();
            self.switch_active_tab(index, cx);
        }
    }

    /// Action: cycle to the previous tab (wraps). Bound to Ctrl+Shift+Tab.
    pub(super) fn prev_tab(&mut self, _: &PrevTab, _: &mut Window, cx: &mut Context<Self>) {
        if self.tabs.len() > 1 {
            let index = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
            self.switch_active_tab(index, cx);
        }
    }

    pub(super) fn save_document(
        &mut self,
        _: &SaveDocument,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.active_tab().document.path().is_none() {
            self.save_document_as(&SaveDocumentAs, window, cx);
            return;
        }

        let display_path = self
            .active_tab()
            .document
            .path()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let saved_path = self.active_tab().document.path().map(Path::to_path_buf);
        let save_result = self.active_tab_mut().document.save();
        self.status = match save_result {
            Ok(()) => {
                self.discard_current_recovery_file();
                if let Some(path) = saved_path.as_ref() {
                    self.record_recent_path(path);
                } else {
                    self.sync_and_persist_session();
                }
                self.trf(Msg::StatusSaved, &[&display_path])
            }
            Err(err) => self.trf(Msg::StatusSaveFailed, &[&err.to_string()]),
        };
        self.active_menu = None;
        cx.notify();
    }

    pub(super) fn save_document_as(
        &mut self,
        _: &SaveDocumentAs,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = SaveTarget::Markdown;
        let directory = self.suggested_directory();
        let suggested_name = self
            .active_tab()
            .document
            .path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled.md")
            .to_string();
        let save_future =
            prompt_for_save_path(window, &directory, &suggested_name, self.language, target);

        self.active_menu = None;
        self.status = t(self.language, Msg::StatusChoosingSaveLocation).into();
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match save_future.await {
                Some(path) => {
                    let display_path = path.display().to_string();
                    let _ = this.update(cx, |app, cx| {
                        let save_result = app.active_tab_mut().document.save_as(&path);
                        app.status = match save_result {
                            Ok(()) => {
                                app.discard_current_recovery_file();
                                app.update_workspace_root_from_document(cx);
                                app.record_recent_path(&path);
                                app.trf(Msg::StatusSaved, &[&display_path])
                            }
                            Err(err) => app.trf(Msg::StatusSaveFailed, &[&err.to_string()]),
                        };
                        app.active_menu = None;
                        cx.notify();
                    });
                    return;
                }
                None => t(language, Msg::StatusSaveCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn export_html(
        &mut self,
        _: &ExportHtml,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::Html, window, cx);
    }

    pub(super) fn export_plain_html(
        &mut self,
        _: &ExportPlainHtml,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::PlainHtml, window, cx);
    }

    pub(super) fn export_pdf(
        &mut self,
        _: &ExportPdf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::Pdf, window, cx);
    }

    pub(super) fn export_latex(
        &mut self,
        _: &ExportLatex,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::Latex, window, cx);
    }

    pub(super) fn export_docx(
        &mut self,
        _: &ExportDocx,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::Docx, window, cx);
    }

    pub(super) fn export_png(
        &mut self,
        _: &ExportPng,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::Png, window, cx);
    }

    pub(super) fn export_jpeg(
        &mut self,
        _: &ExportJpeg,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.export_with_prompt(ExportFormat::Jpeg, window, cx);
    }

    pub(super) fn export_with_prompt(
        &mut self,
        format: ExportFormat,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = SaveTarget::Export(format);
        let profile = target.profile();
        let directory = self.suggested_directory();
        let suggested_name = self.suggested_export_name(target);
        let save_future =
            prompt_for_save_path(window, &directory, &suggested_name, self.language, target);

        self.active_menu = None;
        self.status = self.trf(
            Msg::StatusChoosingExportLocation,
            &[profile.suggested_suffix],
        );
        cx.notify();

        let language = self.language;
        cx.spawn(async move |this, cx| {
            let status = match save_future.await {
                Some(path) => {
                    let display_path = path.display().to_string();
                    let _ = this.update(cx, |app, cx| {
                        let export_preferences = app.export_preferences.clone();
                        let language = app.language;
                        let tab = app.active_tab_mut();
                        let outcome =
                            tab.document
                                .export_to_with(&path, format, &export_preferences);
                        app.status = match outcome {
                            // Disclose the producing backend for the formats
                            // where the pandoc engine competes with the
                            // built-in writers.
                            Ok(backend)
                                if matches!(format, ExportFormat::Pdf | ExportFormat::Docx) =>
                            {
                                let msg = match backend {
                                    ExportBackend::PandocEngine => Msg::StatusExportedEngine,
                                    ExportBackend::BuiltIn => Msg::StatusExportedBuiltin,
                                };
                                tf(language, msg, &[&display_path]).into()
                            }
                            Ok(_) => tf(language, Msg::StatusExported, &[&display_path]).into(),
                            Err(err) => {
                                tf(language, Msg::StatusExportFailed, &[&err.to_string()]).into()
                            }
                        };
                        app.active_menu = None;
                        cx.notify();
                    });
                    return;
                }
                None => t(language, Msg::StatusExportCanceled).to_string(),
            };

            let _ = this.update(cx, |app, cx| {
                app.active_menu = None;
                app.status = status.into();
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn suggested_directory(&self) -> PathBuf {
        self.active_tab()
            .document
            .path()
            .and_then(Path::parent)
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub(super) fn suggested_export_name(&self, target: SaveTarget) -> String {
        let stem = self
            .active_tab()
            .document
            .path()
            .and_then(Path::file_stem)
            .and_then(|stem| stem.to_str())
            .filter(|stem| !stem.is_empty())
            .unwrap_or("Untitled");
        target.suggested_name(stem)
    }
}
