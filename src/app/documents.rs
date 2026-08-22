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
        // File → Open follows the default open-target preference. A dirty
        // guard is needed only when that resolution can replace the active
        // tab (open-in-current-tab on); the OS picker is modal, so the
        // resolution cannot drift between this guard and the open below.
        if self.default_open_intent() == OpenPathIntent::ReplaceActive {
            self.confirm_discard_then(
                window,
                cx,
                Msg::DialogDiscardTitle,
                Msg::DialogDiscardOpenDetail,
                Self::open_document_confirmed,
            );
        } else {
            Self::open_document_confirmed(self, cx);
        }
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
                        match this.update(cx, |app, cx| {
                            app.open_supported_path(path, app.default_open_intent(), cx)
                        }) {
                            Ok(Ok(())) => return,
                            Ok(Err(error)) => tf(language, Msg::StatusOpenFailed, &[&error]),
                            Err(error) => {
                                tf(language, Msg::StatusOpenFailed, &[&error.to_string()])
                            }
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
                            app.set_workspace_root(path, cx);
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
                        match this.update(cx, |app, cx| {
                            app.open_supported_path(path, OpenPathIntent::OpenInNewTab, cx)
                        }) {
                            Ok(Ok(())) => return,
                            Ok(Err(error)) => tf(language, Msg::StatusOpenFailed, &[&error]),
                            Err(error) => {
                                tf(language, Msg::StatusOpenFailed, &[&error.to_string()])
                            }
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
        if self.active_tab().is_document()
            && let Some(recovery) = self.active_tab_mut().last_recovery_file.take()
        {
            let _ = delete_recovery_file(recovery);
        }
        let active = self.active_tab;
        self.release_tab_image_claims(active, cx);
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
        if self.active_tab().is_document() {
            self.refresh_search_matches();
        } else {
            self.search_matches.clear();
            self.current_search_index = None;
        }
        self.sync_and_persist_session();
        self.sync_git_branch_context(cx);
        self.status = t(self.language, Msg::StatusNewDocument).into();
        self.reveal_active_tab_in_strip();
        cx.notify();
    }

    /// Close the tabs identity-matched by `targets` (indexes may have shifted
    /// since capture). Dirty tabs' recovery snapshots are discarded on the way
    /// out — the same discard path as app exit — and the active index follows
    /// the removals so it keeps pointing at the same tab.
    fn remove_tabs_by_identity(&mut self, targets: &[TabContextTarget], cx: &mut Context<Self>) {
        let indexes: Vec<usize> = self
            .tabs
            .iter()
            .enumerate()
            .filter_map(|(index, tab)| {
                targets
                    .iter()
                    .any(|target| target.matches(tab))
                    .then_some(index)
            })
            .collect();
        for index in indexes.into_iter().rev() {
            self.release_tab_image_claims(index, cx);
            if let Some(state) = self.tabs[index].document_tab_mut()
                && let Some(recovery) = state.last_recovery_file.take()
            {
                let _ = delete_recovery_file(recovery);
            }
            self.tabs.remove(index);
            if index < self.active_tab {
                self.active_tab -= 1;
            }
        }
    }

    /// Settle shared post-removal state for batch closes.
    fn settle_after_batch_close(&mut self, cx: &mut Context<Self>) {
        self.active_menu = None;
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        if self.active_tab().is_document() {
            self.refresh_search_matches();
        } else {
            self.search_matches.clear();
            self.current_search_index = None;
        }
        self.sync_and_persist_session();
        self.sync_git_branch_context(cx);
        cx.notify();
    }

    /// Shared body of Close Others / Close to the Right: in-scope clean tabs
    /// close immediately and silently; dirty tabs are kept open and reported
    /// by one summary dialog offering a discard-all confirmation. The clicked
    /// tab (active, never in scope) always survives.
    fn close_tabs_in_scope(
        &mut self,
        scope: Vec<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut clean_targets: Vec<TabContextTarget> = Vec::new();
        let mut dirty_targets: Vec<TabContextTarget> = Vec::new();
        for index in scope {
            let Some(tab) = self.tabs.get(index) else {
                continue;
            };
            if tab.is_dirty() {
                dirty_targets.push(TabContextTarget::capture(index, tab));
            } else {
                clean_targets.push(TabContextTarget::capture(index, tab));
            }
        }
        if !clean_targets.is_empty() {
            self.remove_tabs_by_identity(&clean_targets, cx);
            self.settle_after_batch_close(cx);
        }
        if dirty_targets.is_empty() {
            return;
        }
        let count = dirty_targets.len().to_string();
        let detail = tf(self.language, Msg::DialogCloseTabsDirtyDetail, &[&count]);
        // A single aggregate prompt (not per-tab prompts): GPUI prompts are
        // not re-entrant, and this mirrors the request_quit precedent.
        let answer = window.prompt(
            PromptLevel::Warning,
            self.tr(Msg::DialogCloseTabsDirtyTitle),
            Some(&detail),
            &[
                PromptButton::ok(self.tr(Msg::DialogButtonDiscardAndCloseTabs)),
                PromptButton::cancel(self.tr(Msg::DialogButtonKeepOpen)),
            ],
            cx,
        );
        self.status = t(self.language, Msg::StatusWaitingConfirm).into();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let discard = matches!(answer.await, Ok(0));
            let _ = this.update(cx, |app, cx| {
                if discard {
                    app.remove_tabs_by_identity(&dirty_targets, cx);
                    app.settle_after_batch_close(cx);
                } else {
                    app.status = t(app.language, Msg::StatusCanceled).into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Tab context menu: close every tab except the clicked one. The clicked
    /// tab is active when this runs (switch-then-operate).
    pub(super) fn close_other_tabs(
        &mut self,
        keep: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let scope: Vec<usize> = (0..self.tabs.len())
            .filter(|&index| index != keep)
            .collect();
        self.close_tabs_in_scope(scope, window, cx);
    }

    /// Tab context menu: close every tab right of the clicked one.
    pub(super) fn close_tabs_to_the_right(
        &mut self,
        anchor: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let scope: Vec<usize> = ((anchor + 1)..self.tabs.len()).collect();
        self.close_tabs_in_scope(scope, window, cx);
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
        if self.active_tab().is_image() {
            self.status = t(self.language, Msg::StatusImageActionUnavailable).into();
            cx.notify();
            return;
        }
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
        if save_result
            .as_ref()
            .is_err_and(|err| err.kind() == io::ErrorKind::AlreadyExists)
        {
            self.active_tab_mut().external_conflict = Some(DiskState::Modified);
            self.prompt_external_save_conflict(window, cx);
            return;
        }
        self.status = match save_result {
            Ok(()) => {
                self.active_tab_mut().external_conflict = None;
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

    fn prompt_external_save_conflict(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path = self
            .active_tab()
            .document
            .path()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let answer = window.prompt(
            PromptLevel::Warning,
            p0_t(self.language, P0Msg::ExternalDialogTitle),
            Some(&p0_tf(self.language, P0Msg::ExternalDialogDetail, &[&path])),
            &[
                PromptButton::ok(p0_t(self.language, P0Msg::Reload)),
                PromptButton::ok(p0_t(self.language, P0Msg::Overwrite)),
                PromptButton::ok(p0_t(self.language, P0Msg::SaveCopy)),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );
        let window_handle = window.window_handle();
        self.status = p0_t(self.language, P0Msg::WaitingExternalDecision).into();
        cx.notify();

        cx.spawn(async move |this, cx| match answer.await {
            Ok(0) => {
                let _ = this.update(cx, |app, cx| app.reload_active_external(cx));
            }
            Ok(1) => {
                let _ = this.update(cx, |app, cx| {
                    match app.active_tab_mut().document.force_save() {
                        Ok(()) => {
                            app.active_tab_mut().external_conflict = None;
                            app.discard_current_recovery_file();
                            app.status = p0_t(app.language, P0Msg::ExternalOverwritten).into();
                        }
                        Err(err) => {
                            app.status = app.trf(Msg::StatusSaveFailed, &[&err.to_string()]);
                        }
                    }
                    cx.notify();
                });
            }
            Ok(2) => {
                let _ = window_handle.update(cx, |_, window, cx| {
                    let _ = this.update(cx, |app, cx| {
                        app.save_document_as(&SaveDocumentAs, window, cx)
                    });
                });
            }
            _ => {
                let _ = this.update(cx, |app, cx| {
                    app.status = t(app.language, Msg::StatusCanceled).into();
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn reload_active_external(&mut self, cx: &mut Context<Self>) {
        let active = self.active_tab;
        self.release_tab_image_claims(active, cx);
        let tab = self.active_tab_mut();
        match tab.document.reload_from_disk() {
            Ok(()) => {
                tab.external_conflict = None;
                tab.selected_range = 0..0;
                tab.selection_reversed = false;
                tab.marked_range = None;
                tab.undo_stack.clear();
                tab.redo_stack.clear();
                tab.reset_preview_list();
                if let Some(recovery) = tab.last_recovery_file.take() {
                    let _ = delete_recovery_file(recovery);
                }
                self.status = p0_t(self.language, P0Msg::ExternalReloadDiscarded).into();
            }
            Err(err) => self.status = self.trf(Msg::StatusOpenFailed, &[&err.to_string()]),
        }
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
                                app.active_tab_mut().external_conflict = None;
                                app.discard_current_recovery_file();
                                app.update_workspace_root_from_document(cx);
                                app.record_recent_path(&path);
                                app.flush_pending_image_import(cx);
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
                if status == t(language, Msg::StatusSaveCanceled) {
                    app.pending_image_import = None;
                }
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

    /// DOCX export runs a small options step before the save-path prompt:
    /// page size, table of contents (offered only when the pandoc engine is
    /// available — the built-in writer has no TOC support), and the image
    /// policy. Each prompt shows the last-used choice; the chosen options are
    /// kept in `export_preferences.docx` and persisted after a successful
    /// export (see `export_with_prompt`).
    pub(super) fn export_docx(
        &mut self,
        _: &ExportDocx,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let options = self.export_preferences.docx.clone();
        let language = self.language;
        let toc_available = pandoc_available(self.export_preferences.pandoc_path.as_deref());
        let window_handle = window.window_handle();

        let current = page_size_label(options.page_size);
        let page_answer = window.prompt(
            PromptLevel::Info,
            self.tr(Msg::DialogDocxPageSizeTitle),
            Some(&tf(language, Msg::DialogDocxPageSizeDetail, &[current])),
            &[
                PromptButton::ok("A4"),
                PromptButton::ok("Letter"),
                PromptButton::ok("Legal"),
                PromptButton::cancel(self.tr(Msg::DialogButtonCancel)),
            ],
            cx,
        );

        self.active_menu = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            // Any dismissal/cancel aborts the whole flow.
            let Ok(page_index) = page_answer.await else {
                return;
            };
            if page_index > 2 {
                return;
            }
            let page_size =
                [DocxPageSize::A4, DocxPageSize::Letter, DocxPageSize::Legal][page_index];

            let toc = if toc_available {
                let current = t(
                    language,
                    if options.toc {
                        Msg::DialogDocxTocOn
                    } else {
                        Msg::DialogDocxTocOff
                    },
                );
                let answer = window_handle.update(cx, |_, window, cx| {
                    window.prompt(
                        PromptLevel::Info,
                        t(language, Msg::DialogDocxTocTitle),
                        Some(&tf(language, Msg::DialogDocxTocDetail, &[current])),
                        &[
                            PromptButton::ok(t(language, Msg::DialogDocxTocOn)),
                            PromptButton::ok(t(language, Msg::DialogDocxTocOff)),
                            PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
                        ],
                        cx,
                    )
                });
                match answer {
                    Ok(answer) => match answer.await {
                        Ok(0) => true,
                        Ok(1) => false,
                        _ => return,
                    },
                    Err(_) => return,
                }
            } else {
                // Built-in writer ignores TOC; keep the stored choice.
                options.toc
            };

            let current = t(
                language,
                match options.image_policy {
                    DocxImagePolicy::Embed => Msg::DialogDocxImagesEmbed,
                    DocxImagePolicy::TextFallback => Msg::DialogDocxImagesText,
                },
            );
            let answer = window_handle.update(cx, |_, window, cx| {
                window.prompt(
                    PromptLevel::Info,
                    t(language, Msg::DialogDocxImagesTitle),
                    Some(&tf(language, Msg::DialogDocxImagesDetail, &[current])),
                    &[
                        PromptButton::ok(t(language, Msg::DialogDocxImagesEmbed)),
                        PromptButton::ok(t(language, Msg::DialogDocxImagesText)),
                        PromptButton::cancel(t(language, Msg::DialogButtonCancel)),
                    ],
                    cx,
                )
            });
            let image_policy = match answer {
                Ok(answer) => match answer.await {
                    Ok(0) => DocxImagePolicy::Embed,
                    Ok(1) => DocxImagePolicy::TextFallback,
                    _ => return,
                },
                Err(_) => return,
            };

            let chosen = DocxExportOptions {
                page_size,
                toc,
                image_policy,
            };
            let _ = window_handle.update(cx, |_, window, cx| {
                let _ = this.update(cx, |app, cx| {
                    app.export_preferences.docx = chosen;
                    app.export_with_prompt(ExportFormat::Docx, window, cx);
                });
            });
        })
        .detach();
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
                        // The DOCX options step stashes the chosen options in
                        // `export_preferences.docx`; persist them as the
                        // last-used choices once the export succeeds.
                        if outcome.is_ok() && format == ExportFormat::Docx {
                            app.persist_preferences();
                        }
                        app.status = match outcome {
                            // Disclose the producing backend for the formats
                            // where the pandoc engine competes with the
                            // built-in writers; on fallback, name the engine
                            // failure category as well.
                            Ok(outcome)
                                if matches!(format, ExportFormat::Pdf | ExportFormat::Docx) =>
                            {
                                let msg =
                                    backend_status_msg(outcome.backend, outcome.engine_failure);
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

/// Page-size tokens are standard paper names; they stay untranslated.
fn page_size_label(size: DocxPageSize) -> &'static str {
    match size {
        DocxPageSize::A4 => "A4",
        DocxPageSize::Letter => "Letter",
        DocxPageSize::Legal => "Legal",
    }
}
