use super::*;

impl Focusable for MarkionApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MarkionApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = self.palette();
        let typography = self.typography_metrics();
        // The preview pane is hidden in Edit mode, so skip the full-document
        // parse that produces its blocks. That parse is invalidated on every
        // keystroke and, on large documents, is the dominant per-key cost
        // (~4ms at 100 KB, ~25ms at 600 KB); paying it while nothing renders it
        // is pure waste. Split/Read still parse eagerly as before.
        let preview_blocks: std::sync::Arc<Vec<PreviewBlock>> =
            if matches!(self.view_mode, ViewMode::Edit | ViewMode::VisualEdit) {
                std::sync::Arc::new(Vec::new())
            } else {
                // Debounced: mid-typing renders reuse the previous parse and a
                // timer re-renders once typing settles (see PREVIEW_DEBOUNCE).
                // The parse itself runs on a background thread and lands via
                // spawn_preview_parse, so it never stalls a frame.
                let blocks = self.preview_blocks_debounced(cx);
                // Fold the blocks into the virtualized preview list (splices
                // only the changed range, preserving scroll; a reused Arc is a
                // pointer-compare no-op).
                self.active_tab_mut().sync_preview_list(&blocks);
                blocks
            };
        let visual_blocks: std::sync::Arc<Vec<VisualBlock>> =
            if matches!(self.view_mode, ViewMode::VisualEdit) {
                let blocks = self.active_tab().document.visual_blocks_shared();
                self.active_tab_mut().sync_visual_list(&blocks);
                if let Some(index) = self
                    .active_tab_mut()
                    .take_visual_cursor_reveal_index(&blocks)
                {
                    self.active_tab().visual_list.scroll_to_reveal_item(index);
                }
                blocks
            } else {
                std::sync::Arc::new(Vec::new())
            };
        // Diagram cache warming needs both the preview blocks (Split/Read) and
        // the visual blocks (Visual Edit) because Visual Edit no longer parses
        // preview blocks — its diagram fences live only in `visual_blocks`.
        self.ensure_diagram_renders(&preview_blocks, &visual_blocks, cx);
        self.ensure_math_renders(
            &preview_blocks,
            &visual_blocks,
            1.0,
            window.scale_factor(),
            palette.text,
            cx,
        );
        // Proportional scroll coupling for Split Preview + Sync scroll. Runs
        // each frame *after* the preview list is in sync with the current
        // blocks (so max_offset reflects the real content height) and *before*
        // the scrollbar views read offsets to draw thumbs. Detects which pane
        // drove the latest change via per-tab cached fractions and writes only
        // the non-driving pane, converging in one frame without a feedback loop.
        self.reconcile_sync_scroll();
        let title = title_from_path(self.active_tab().document.path());
        let document_dir = self
            .active_tab()
            .document
            .path()
            .and_then(Path::parent)
            .map(PathBuf::from);
        let is_dirty = self.active_tab().document.is_dirty();
        let dirty_marker = if is_dirty { " *" } else { "" };
        let save_state = t(
            self.language,
            if is_dirty {
                Msg::TitleModified
            } else {
                Msg::TitleSaved
            },
        );
        let (editor_width, preview_width) =
            view_mode_pane_widths(self.view_mode, self.editor_split_ratio);
        let constrain_read_preview =
            read_mode_preview_is_constrained(self.view_mode, self.preview_adaptive_width);
        // Captured by value into the virtualized preview `list`'s per-item
        // render closure, which must be `'static` and cannot borrow `self`.
        let preview_items = preview_blocks.clone();
        let preview_items_doc_dir = document_dir.clone();
        let preview_code_line_numbers = self.code_line_numbers;
        let preview_display_scale = window.scale_factor();
        let preview_list_state = self.active_tab().preview_list.clone();
        let visual_items = visual_blocks.clone();
        let visual_items_doc_dir = document_dir.clone();
        let visual_list_state = self.active_tab().visual_list.clone();

        div()
            .size_full()
            .relative()
            .bg(palette.app_bg)
            .text_color(palette.text)
            .font_family(".SystemUIFont")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(Self::new_document))
            .on_action(cx.listener(Self::open_document))
            .on_action(cx.listener(Self::open_folder))
            .on_action(cx.listener(Self::clear_recent_files))
            .on_action(cx.listener(Self::save_document))
            .on_action(cx.listener(Self::save_document_as))
            .on_action(cx.listener(Self::export_html))
            .on_action(cx.listener(Self::export_plain_html))
            .on_action(cx.listener(Self::export_pdf))
            .on_action(cx.listener(Self::export_latex))
            .on_action(cx.listener(Self::export_docx))
            .on_action(cx.listener(Self::export_png))
            .on_action(cx.listener(Self::export_jpeg))
            .on_action(cx.listener(Self::toggle_view_mode))
            .on_action(cx.listener(Self::set_edit_mode))
            .on_action(cx.listener(Self::set_visual_edit_mode))
            .on_action(cx.listener(Self::set_split_preview_mode))
            .on_action(cx.listener(Self::set_read_mode))
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_action(cx.listener(Self::toggle_outline))
            .on_action(cx.listener(Self::toggle_file_tree))
            .on_action(cx.listener(Self::focus_file_tree_search))
            .on_action(cx.listener(Self::clear_file_tree_search))
            .on_action(cx.listener(Self::refresh_file_tree_action))
            .on_action(cx.listener(Self::create_tree_file))
            .on_action(cx.listener(Self::create_tree_folder))
            .on_action(cx.listener(Self::rename_tree_entry))
            .on_action(cx.listener(Self::delete_tree_entry))
            .on_action(cx.listener(Self::confirm_pending_name))
            .on_action(cx.listener(Self::cycle_theme))
            .on_action(cx.listener(Self::toggle_focus_mode))
            .on_action(cx.listener(Self::toggle_typewriter_mode))
            .on_action(cx.listener(Self::toggle_code_line_numbers))
            .on_action(cx.listener(Self::show_find))
            .on_action(cx.listener(Self::show_replace))
            .on_action(cx.listener(Self::find_next))
            .on_action(cx.listener(Self::find_previous))
            .on_action(cx.listener(Self::replace_current_match))
            .on_action(cx.listener(Self::replace_all_matches))
            .on_action(cx.listener(Self::toggle_find_case_sensitive))
            .on_action(cx.listener(Self::toggle_find_regex))
            .on_action(cx.listener(Self::show_shortcuts))
            .on_action(cx.listener(Self::show_preferences))
            .on_action(cx.listener(Self::reset_preferences))
            .on_action(cx.listener(Self::about))
            .on_action(cx.listener(Self::quit))
            .on_action(cx.listener(Self::new_tab))
            .on_action(cx.listener(Self::open_in_new_tab_action))
            .on_action(cx.listener(Self::close_tab))
            .on_action(cx.listener(Self::next_tab))
            .on_action(cx.listener(Self::prev_tab))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::insert_newline))
            .on_action(cx.listener(Self::indent))
            .on_action(cx.listener(Self::outdent))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::bold))
            .on_action(cx.listener(Self::italic))
            .on_action(cx.listener(Self::inline_code))
            .on_action(cx.listener(Self::insert_link))
            .on_action(cx.listener(Self::insert_image))
            .on_action(cx.listener(Self::heading1))
            .on_action(cx.listener(Self::heading2))
            .on_action(cx.listener(Self::heading3))
            .on_action(cx.listener(Self::heading4))
            .on_action(cx.listener(Self::heading5))
            .on_action(cx.listener(Self::heading6))
            .on_action(cx.listener(Self::unordered_list))
            .on_action(cx.listener(Self::ordered_list))
            .on_action(cx.listener(Self::task_list))
            .on_action(cx.listener(Self::block_quote))
            .on_action(cx.listener(Self::code_fence))
            .on_action(cx.listener(Self::format_table))
            .on_action(cx.listener(Self::table_add_row))
            .on_action(cx.listener(Self::table_delete_row))
            .on_action(cx.listener(Self::table_move_row_up))
            .on_action(cx.listener(Self::table_move_row_down))
            .on_action(cx.listener(Self::table_add_column))
            .on_action(cx.listener(Self::table_delete_column))
            .flex()
            .flex_col()
            .child(
                div().h(px(28.)).child(
                    div()
                        .h(px(28.))
                        .px_2()
                        .border_b_1()
                        .border_color(palette.border)
                        .bg(palette.panel_bg)
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(menu_title_button(
                            self.tr(Msg::MenuFile),
                            self.active_menu == Some(AppMenu::File),
                            palette,
                            cx.listener(Self::toggle_file_menu),
                            cx.listener(|app, _: &MouseMoveEvent, _, cx| {
                                app.hover_menu(AppMenu::File, cx);
                            }),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuEdit),
                            self.active_menu == Some(AppMenu::Edit),
                            palette,
                            cx.listener(Self::toggle_edit_menu),
                            cx.listener(|app, _: &MouseMoveEvent, _, cx| {
                                app.hover_menu(AppMenu::Edit, cx);
                            }),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuView),
                            self.active_menu == Some(AppMenu::View),
                            palette,
                            cx.listener(Self::toggle_view_menu),
                            cx.listener(|app, _: &MouseMoveEvent, _, cx| {
                                app.hover_menu(AppMenu::View, cx);
                            }),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuFormat),
                            self.active_menu == Some(AppMenu::Format),
                            palette,
                            cx.listener(Self::toggle_format_menu),
                            cx.listener(|app, _: &MouseMoveEvent, _, cx| {
                                app.hover_menu(AppMenu::Format, cx);
                            }),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuExport),
                            self.active_menu == Some(AppMenu::Export),
                            palette,
                            cx.listener(Self::toggle_export_menu),
                            cx.listener(|app, _: &MouseMoveEvent, _, cx| {
                                app.hover_menu(AppMenu::Export, cx);
                            }),
                        ))
                        .child(menu_title_button(
                            self.tr(Msg::MenuHelp),
                            self.active_menu == Some(AppMenu::Help),
                            palette,
                            cx.listener(Self::toggle_help_menu),
                            cx.listener(|app, _: &MouseMoveEvent, _, cx| {
                                app.hover_menu(AppMenu::Help, cx);
                            }),
                        )),
                ),
            )
            .child(self.tab_bar_view(cx))
            .child(
                div()
                    .id("main-content-row")
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::close_menu))
                    // Drag-move/drop for the two resize dividers. DragMoveEvent
                    // gives the cursor position plus the bounds of this row, so
                    // each handler converts the x offset into a ratio / width.
                    .on_drag_move::<DraggedEditorSplitHandle>(
                        cx.listener(Self::on_editor_split_drag),
                    )
                    .on_drop::<DraggedEditorSplitHandle>(cx.listener(Self::on_editor_split_drop))
                    .on_drag_move::<DraggedSidebarHandle>(cx.listener(Self::on_sidebar_resize_drag))
                    .on_drop::<DraggedSidebarHandle>(cx.listener(|_, _, _, cx| {
                        cx.notify();
                    }))
                    .child(sidebar_view(self, cx))
                    // Sidebar/pane divider: only when the sidebar is visible.
                    .when(self.sidebar_visible, |d| {
                        d.child(sidebar_resize_handle_view(palette.border, cx))
                    })
                    .child(
                        div()
                            // Sized by the draggable split ratio instead of a flat
                            // flex_1, so dragging the divider actually resizes it.
                            .flex_basis(DefiniteLength::Fraction(editor_width))
                            .flex_shrink()
                            .min_w_0()
                            .min_h_0()
                            .p(px(PANE_OUTER_PADDING))
                            .when(matches!(self.view_mode, ViewMode::Split), |style| {
                                style.border_r_1()
                            })
                            .border_color(palette.border)
                            .flex()
                            .flex_col()
                            // Accept files dragged from the OS file manager.
                            // The preview pane registers the same handler; the
                            // sidebar/file-tree area deliberately does not.
                            .on_drop::<ExternalPaths>(cx.listener(Self::handle_external_drop))
                            .child(if matches!(self.view_mode, ViewMode::VisualEdit) {
                                visual_edit_surface_view(
                                    visual_items,
                                    visual_items_doc_dir,
                                    visual_list_state,
                                    palette,
                                    window.scale_factor(),
                                    typography,
                                    document_tab_band_visible(self.tabs.len()),
                                    constrain_read_preview,
                                    cx,
                                )
                            } else {
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h_0()
                                    .child(
                                        div()
                                            .size_full()
                                            .p(px(PANE_INNER_PADDING))
                                            .bg(palette.surface_bg)
                                            .border_1()
                                            .when(
                                                document_tab_band_visible(self.tabs.len()),
                                                |style| style.border_t_0(),
                                            )
                                            .border_color(palette.border)
                                            .line_height(px(typography.editor_line_height))
                                            .text_size(px(typography.editor_font_size))
                                            .cursor(CursorStyle::IBeam)
                                            .id("editor-scroll")
                                            .overflow_y_scroll()
                                            .scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))
                                            .track_scroll(&self.active_tab().editor_scroll)
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(Self::on_mouse_down),
                                            )
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                cx.listener(Self::on_mouse_up),
                                            )
                                            .on_mouse_up_out(
                                                MouseButton::Left,
                                                cx.listener(Self::on_mouse_up),
                                            )
                                            .on_mouse_move(cx.listener(Self::on_mouse_move))
                                            .child(EditorElement { app: cx.entity() }),
                                    )
                                    .child(pane_scrollbar_view(
                                        PaneScrollTarget::Editor,
                                        &self.active_tab().editor_scroll,
                                        palette,
                                        cx,
                                    ))
                            })
                            .when(matches!(self.view_mode, ViewMode::Read), |style| {
                                style.hidden()
                            }),
                    )
                    // Editor/preview divider: only in Split view, where both
                    // panes are visible. In Edit/Read the hidden pane is
                    // display:none and the divider would have nothing to split.
                    .when(matches!(self.view_mode, ViewMode::Split), |d| {
                        d.child(editor_split_handle_view(palette.border, cx))
                    })
                    .child(
                        div()
                            // Remaining fraction goes to the preview pane.
                            .flex_basis(DefiniteLength::Fraction(preview_width))
                            .flex_shrink()
                            .min_w_0()
                            .min_h_0()
                            .p(px(PANE_OUTER_PADDING))
                            .flex()
                            .flex_col()
                            // Same external-file drop handler as the editor
                            // pane, so a drop lands wherever the cursor is.
                            .on_drop::<ExternalPaths>(cx.listener(Self::handle_external_drop))
                            .child(
                                div()
                                    .relative()
                                    .flex_1()
                                    .min_h_0()
                                    .child(
                                        div()
                                            .size_full()
                                            .pl(px(PANE_INNER_PADDING))
                                            .pr(px(PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING))
                                            .bg(palette.surface_bg)
                                            .border_1()
                                            .when(
                                                document_tab_band_visible(self.tabs.len()),
                                                |style| style.border_t_0(),
                                            )
                                            .border_color(palette.border)
                                            .on_mouse_up(
                                                MouseButton::Right,
                                                cx.listener(|app, event: &MouseUpEvent, _, cx| {
                                                    app.show_preview_context_menu(
                                                        event.position,
                                                        None,
                                                        cx,
                                                    );
                                                }),
                                            )
                                            .child(
                                                // Virtualized preview: `list` builds
                                                // elements only for blocks near the
                                                // viewport, so preview render cost is
                                                // O(visible) rather than O(document).
                                                //
                                                // GPUI's list uses padding for vertical
                                                // scroll math, but item layout still starts
                                                // at the list's left edge. Keep horizontal
                                                // padding on the parent surface so rows are
                                                // actually inset from the overlay scrollbar.
                                                list(
                                                    preview_list_state,
                                                    cx.processor(
                                                        move |app, ix: usize, _window, cx| {
                                                            let block = &preview_items[ix];
                                                            let row = div()
                                                                .w_full()
                                                                .line_height(px(typography
                                                                    .preview_row_line_height))
                                                                .child(preview_block_view(
                                                                    app,
                                                                    block,
                                                                    ix,
                                                                    preview_items_doc_dir
                                                                        .as_deref(),
                                                                    preview_code_line_numbers,
                                                                    preview_display_scale,
                                                                    cx,
                                                                ));
                                                            if constrain_read_preview {
                                                                div()
                                                                    .w_full()
                                                                    .flex()
                                                                    .justify_center()
                                                                    .child(row.max_w(px(
                                                                        READ_MODE_PREVIEW_MAX_WIDTH,
                                                                    )))
                                                                    .into_any_element()
                                                            } else {
                                                                row.into_any_element()
                                                            }
                                                        },
                                                    ),
                                                )
                                                .size_full()
                                                .pt(px(PANE_INNER_PADDING))
                                                .pb(px(PANE_INNER_PADDING)),
                                            ),
                                    )
                                    .child(preview_list_scrollbar_view(
                                        &self.active_tab().preview_list,
                                        palette,
                                        cx,
                                    )),
                            )
                            .when(
                                matches!(self.view_mode, ViewMode::Edit | ViewMode::VisualEdit),
                                |style| style.hidden(),
                            ),
                    ),
            )
            .child(
                div()
                    .h(px(28.))
                    .px_4()
                    .border_t_1()
                    .border_color(palette.border)
                    .text_size(px(12.))
                    .text_color(palette.muted)
                    .flex()
                    .items_center()
                    .child(format!(
                        "Markion - {title}{dirty_marker} | {save_state} | {}",
                        self.status
                    )),
            )
            .child(active_menu_dropdown(
                self.active_menu,
                self.language,
                self.heading_menu_max_level,
                self.session.recent_files.clone(),
                palette,
                cx,
            ))
            .when(self.search_visible, |root| {
                root.child(search_panel_view(self, cx))
            })
            .when(self.file_tree_context_menu.is_some(), |root| {
                root.child(file_tree_context_menu_view(self, cx))
            })
            .when(self.preview_context_menu.is_some(), |root| {
                root.child(preview_context_menu_view(self, cx))
            })
            .when(self.preferences_panel_open, |root| {
                root.child(preferences_panel_view(self, cx))
            })
            .when(self.shortcut_panel_open, |root| {
                root.child(shortcut_panel_view(self, cx))
            })
    }
}

pub(super) fn visual_edit_surface_view(
    items: std::sync::Arc<Vec<VisualBlock>>,
    document_dir: Option<PathBuf>,
    list_state: ListState,
    palette: ThemePalette,
    display_scale: f32,
    typography: DocumentTypographyMetrics,
    connected_to_tab_band: bool,
    constrain_width: bool,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let is_empty = items.is_empty();
    let input_bridge = VisualInputElement { app: cx.entity() };
    div()
        .relative()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .size_full()
                .pl(px(PANE_INNER_PADDING))
                .pr(px(PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING))
                .bg(palette.surface_bg)
                .border_1()
                .when(connected_to_tab_band, |surface| surface.border_t_0())
                .border_color(palette.border)
                .cursor(CursorStyle::IBeam)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|app, _: &MouseDownEvent, _, cx| {
                        // Bubble after row/chrome handlers: collapse expanded
                        // source panes unless a collapsible block retained this click.
                        if !app.active_tab().expanded_visual_source_blocks.is_empty()
                            || app.active_tab().retain_visual_source_expand.is_some()
                        {
                            app.active_tab_mut().apply_visual_source_outside_click();
                            cx.notify();
                        }
                    }),
                )
                .when(is_empty, |surface| {
                    surface.child(
                        div()
                            .p(px(PANE_INNER_PADDING))
                            .text_size(px(typography.rendered_font_size))
                            .text_color(palette.muted)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|app, _, window, cx| {
                                    window.focus(&app.focus_handle(cx));
                                    app.move_to(0, cx);
                                }),
                            )
                            .child("Type Markdown here..."),
                    )
                })
                .when(!is_empty, move |surface| {
                    surface.child(
                        list(
                            list_state,
                            cx.processor(move |app, ix: usize, _window, cx| {
                                let row = div()
                                    .w_full()
                                    .line_height(px(typography.preview_row_line_height))
                                    .child(visual_block_view(
                                        app,
                                        &items[ix],
                                        ix,
                                        document_dir.as_deref(),
                                        display_scale,
                                        cx,
                                    ));
                                if constrain_width {
                                    div()
                                        .w_full()
                                        .flex()
                                        .justify_center()
                                        .child(row.max_w(px(READ_MODE_PREVIEW_MAX_WIDTH)))
                                        .into_any_element()
                                } else {
                                    row.into_any_element()
                                }
                            }),
                        )
                        .size_full()
                        .pt(px(PANE_INNER_PADDING))
                        .pb(px(PANE_INNER_PADDING)),
                    )
                }),
        )
        .child(
            div()
                .absolute()
                .top(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .left(px(0.))
                .child(input_bridge),
        )
}

pub(super) fn search_panel_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let current = app.current_search_index.map(|index| index + 1).unwrap_or(0);
    let total = app.search_matches.len();
    let summary = if app.search_query.is_empty() {
        "No query".to_string()
    } else {
        format!("{current}/{total}")
    };
    let top = px(36. + document_tab_band_height(app.tabs.len()));
    let max_width = if app.replace_visible {
        px(720.)
    } else {
        px(560.)
    };

    div()
        .absolute()
        .top(top)
        .left(px(16.))
        .right(px(16.))
        .flex()
        .justify_end()
        .child(
            div()
                .w_full()
                .max_w(max_width)
                .px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(palette.border)
                .bg(palette.panel_bg)
                .text_color(palette.text)
                .shadow_md()
                .occlude()
                .flex()
                .items_center()
                .flex_wrap()
                .gap_2()
                .child(search_field_view(
                    app.tr(Msg::SearchFind),
                    &app.search_query,
                    app.search_focus == Some(SearchField::Find),
                    palette,
                    cx.listener(MarkionApp::focus_find_field),
                ))
                .when(app.replace_visible, |panel| {
                    panel.child(search_field_view(
                        app.tr(Msg::SearchReplace),
                        &app.replace_text,
                        app.search_focus == Some(SearchField::Replace),
                        palette,
                        cx.listener(MarkionApp::focus_replace_field),
                    ))
                })
                .child(toolbar_button(
                    app.tr(Msg::SearchPrev),
                    palette,
                    cx.listener(MarkionApp::click_find_previous),
                ))
                .child(toolbar_button(
                    app.tr(Msg::SearchNext),
                    palette,
                    cx.listener(MarkionApp::click_find_next),
                ))
                .when(app.replace_visible, |panel| {
                    panel
                        .child(toolbar_button(
                            app.tr(Msg::SearchReplace),
                            palette,
                            cx.listener(MarkionApp::click_replace_current),
                        ))
                        .child(toolbar_button(
                            app.tr(Msg::SearchAll),
                            palette,
                            cx.listener(MarkionApp::click_replace_all),
                        ))
                })
                .child(toolbar_button(
                    if app.search_case_sensitive {
                        app.tr(Msg::SearchCaseSensitiveMark)
                    } else {
                        app.tr(Msg::SearchCaseInsensitiveMark)
                    },
                    palette,
                    cx.listener(MarkionApp::click_toggle_case),
                ))
                .child(toolbar_button(
                    if app.search_regex {
                        app.tr(Msg::SearchRegexMark)
                    } else {
                        app.tr(Msg::SearchLiteral)
                    },
                    palette,
                    cx.listener(MarkionApp::click_toggle_regex),
                ))
                .child(
                    div()
                        .ml_1()
                        .text_size(px(12.))
                        .text_color(palette.muted)
                        .child(summary),
                )
                .child(toolbar_button(
                    "×",
                    palette,
                    cx.listener(MarkionApp::click_close_search),
                )),
        )
}

pub(super) fn hide_search_overlay_state(
    search_visible: &mut bool,
    replace_visible: &mut bool,
    search_focus: &mut Option<SearchField>,
    input_marked_len: &mut usize,
) {
    *search_visible = false;
    *replace_visible = false;
    *search_focus = None;
    *input_marked_len = 0;
}

pub(super) fn search_field_view(
    label: &'static str,
    value: &str,
    active: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let border = if active {
        palette.active_bg
    } else {
        palette.border
    };
    let text = if value.is_empty() {
        format!("{label}: ")
    } else {
        format!("{label}: {value}")
    };

    div()
        .min_w(px(180.))
        .max_w(px(280.))
        .flex_1()
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(palette.surface_bg)
        .text_color(palette.text)
        .text_size(px(12.))
        .cursor_pointer()
        .hover(move |style| style.border_color(palette.active_bg))
        .child(text)
        .on_mouse_up(MouseButton::Left, listener)
}

/// Inline name prompt for a file-tree create/rename action. Reuses the
/// redirected-text-input path: clicking the field focuses the name buffer so
/// IME keystrokes route into `pending_name_input.buffer` instead of the
/// document. The label is "Name" and the buffer is shown after it.
pub(super) fn pending_name_prompt_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let app_entity = cx.entity();
    let Some(pending) = &app.pending_name_input else {
        return div().hidden();
    };
    let label = app.tr(Msg::FileTreeNamePromptLabel);
    let text = if pending.buffer.is_empty() {
        format!("{label}: ")
    } else {
        format!("{label}: {}", pending.buffer)
    };

    div()
        .mb_2()
        .min_w(px(180.))
        .max_w(px(320.))
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        // The prompt is always active when shown (it captures keystrokes),
        // so use the same accent blue as the active search field.
        .border_color(rgb(0x2563eb))
        .bg(palette.surface_bg)
        .text_size(px(12.))
        .text_color(palette.text)
        .cursor_pointer()
        .child(text)
        .on_mouse_up(MouseButton::Left, move |_, _window, cx| {
            // Re-assert focus if the user clicks the prompt while it is open.
            app_entity.update(cx, |app, cx| {
                if app.pending_name_input.is_some() {
                    app.input_marked_len = 0;
                    cx.notify();
                }
            });
        })
}

pub(super) fn file_tree_panel_body(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();

    // Empty state: until a real Markdown file is opened the tree has no chosen
    // root (the welcome document has no path), so we deliberately show a
    // placeholder instead of scanning the program's working directory. The
    // filter input and toolbar are hidden here because they operate on a tree
    // that does not exist yet.
    if app.file_tree.is_none() {
        return div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px_4()
            .py_6()
            .text_size(px(12.))
            .text_color(palette.muted)
            .text_center()
            .child(app.tr(Msg::FileTreeEmptyState).to_string());
    }

    let app_entity = cx.entity();
    let active_path = app.active_tab().document.path().map(Path::to_path_buf);
    let selected_path = app.selected_tree_path.clone();
    // Cap how many rows the panel builds per frame: this view is rebuilt on
    // every keystroke, and an uncapped workspace scan can hold thousands of
    // entries.
    const MAX_VISIBLE_TREE_ENTRIES: usize = 300;
    let (entries, total_entries) = app
        .file_tree
        .as_ref()
        .map(|tree| {
            filtered_visible_file_tree_entries(
                tree,
                &app.file_tree_query,
                &app.collapsed_tree_paths,
                MAX_VISIBLE_TREE_ENTRIES,
            )
        })
        .unwrap_or_default();
    let hidden_entries = total_entries.saturating_sub(entries.len());
    let tree_content_width = file_tree_content_width(&entries);
    let background_app_entity = app_entity.clone();
    let root_label = app
        .workspace_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| app.tr(Msg::FileTreeWorkspaceFallback))
        .to_string();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        // The tab already says "Files"; show the workspace name as a muted
        // subheading so users still know which directory they are browsing.
        .child(
            div()
                .mb_2()
                .text_size(px(12.))
                .text_color(palette.muted)
                .child(root_label),
        )
        // Inline name prompt overlay for create/rename. Rendered as a focused
        // input line above the tree (not as a tree row), so bounded-row
        // rendering is unaffected. Reuses the redirected-text-input path.
        .when(app.pending_name_input.is_some(), |panel| {
            panel.child(pending_name_prompt_view(app, cx))
        })
        .child(
            div()
                .id("file-tree-scroll")
                .flex_1()
                .min_h_0()
                .overflow_x_scroll()
                .overflow_y_scroll()
                .scrollbar_width(px(8.))
                .track_scroll(&app.file_tree_scroll)
                .on_mouse_up(MouseButton::Right, move |event, _, cx| {
                    background_app_entity.update(cx, |app, cx| {
                        app.show_file_tree_context_menu(
                            FileTreeContextTarget::Workspace,
                            event.position,
                            cx,
                        );
                    });
                })
                .children(entries.into_iter().map(move |entry| {
                    let left_app_entity = app_entity.clone();
                    let right_app_entity = app_entity.clone();
                    let path = entry.path.clone();
                    let entry_kind = entry.kind;
                    let context_target = match entry.kind {
                        FileTreeEntryKind::Directory => {
                            FileTreeContextTarget::Directory(entry.path.clone())
                        }
                        FileTreeEntryKind::File => FileTreeContextTarget::File(entry.path.clone()),
                    };
                    let is_active = active_path.as_ref() == Some(&entry.path);
                    let is_selected = selected_path.as_ref() == Some(&entry.path);
                    let is_collapsed = entry.kind == FileTreeEntryKind::Directory
                        && app.collapsed_tree_paths.contains(&entry.path);
                    // Only Markdown files are collected into the tree (see
                    // `collect_file_tree_entries`), so every File row opens a
                    // document; Directory rows toggle their descendants.
                    // `entry.is_markdown` is read defensively in case the
                    // collection filter relaxes.
                    let clickable = entry.kind == FileTreeEntryKind::File && entry.is_markdown;
                    let bg = if is_active {
                        palette.active_bg
                    } else if is_selected {
                        palette.surface_bg
                    } else {
                        palette.panel_bg
                    };
                    let text_color = if is_active || is_selected {
                        palette.active_text
                    } else if clickable {
                        palette.text
                    } else {
                        palette.muted
                    };

                    div()
                        .mb(px(0.))
                        .ml(px(entry.depth as f32 * 12.))
                        .w_full()
                        .min_w(px(tree_content_width))
                        .px_2()
                        .py(px(1.))
                        .rounded_md()
                        .bg(bg)
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(text_color)
                        .cursor_pointer()
                        .hover(move |style| {
                            if clickable || entry_kind == FileTreeEntryKind::Directory {
                                style.bg(palette.active_bg)
                            } else {
                                style
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .min_w_0()
                                .child(file_tree_entry_icon(
                                    entry.kind,
                                    !is_collapsed,
                                    palette,
                                    text_color,
                                ))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .whitespace_nowrap()
                                        .child(entry.name),
                                ),
                        )
                        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                            let focus_handle = left_app_entity.read(cx).focus_handle.clone();
                            window.focus(&focus_handle);
                            let path = path.clone();
                            left_app_entity.update(cx, |app, cx| {
                                app.selected_tree_path = Some(path.clone());
                                app.file_tree_query_focused = false;
                                app.input_marked_len = 0;
                                match entry_kind {
                                    FileTreeEntryKind::File if clickable => {
                                        app.open_tree_file(path, window, cx);
                                    }
                                    FileTreeEntryKind::Directory => {
                                        if app.collapsed_tree_paths.contains(&path) {
                                            app.collapsed_tree_paths.remove(&path);
                                        } else {
                                            app.collapsed_tree_paths.insert(path.clone());
                                        }
                                        app.status =
                                            t(app.language, Msg::StatusSelectedTreeEntry).into();
                                        cx.notify();
                                    }
                                    FileTreeEntryKind::File => {
                                        app.status =
                                            t(app.language, Msg::StatusSelectedTreeEntry).into();
                                        cx.notify();
                                    }
                                }
                            });
                            if !clickable {}
                        })
                        .on_mouse_up(MouseButton::Right, {
                            move |event, window, cx| {
                                // The scroll container also handles right-clicks to
                                // open the workspace/background menu. Without stopping
                                // propagation here, that parent handler runs next and
                                // immediately replaces this file/folder menu (which has
                                // Rename/Delete) with the workspace menu (which does not).
                                cx.stop_propagation();
                                let focus_handle = right_app_entity.read(cx).focus_handle.clone();
                                window.focus(&focus_handle);
                                right_app_entity.update(cx, |app, cx| {
                                    app.show_file_tree_context_menu(
                                        context_target.clone(),
                                        event.position,
                                        cx,
                                    );
                                });
                            }
                        })
                }))
                .children((hidden_entries > 0).then(|| {
                    div()
                        .mt_1()
                        .px_2()
                        .text_size(px(11.))
                        .text_color(palette.muted)
                        .child(app.trf(Msg::FileTreeMoreHidden, &[&hidden_entries.to_string()]))
                })),
        )
}

pub(super) fn filtered_visible_file_tree_entries(
    tree: &FileTree,
    query: &str,
    collapsed_paths: &HashSet<PathBuf>,
    limit: usize,
) -> (Vec<FileTreeEntry>, usize) {
    let query = query.trim().to_ascii_lowercase();
    let mut entries = Vec::new();
    let mut matched = 0usize;
    let mut collapsed_depth = None;

    'entries: for entry in &tree.entries {
        if let Some(depth) = collapsed_depth {
            if entry.depth > depth {
                continue 'entries;
            }
            collapsed_depth = None;
        }

        let collapsed = query.is_empty()
            && entry.kind == FileTreeEntryKind::Directory
            && collapsed_paths.contains(&entry.path);
        if file_tree_entry_matches_query(entry, &tree.root, &query) {
            if matched < limit {
                entries.push(entry.clone());
            }
            matched += 1;
        }
        if collapsed {
            collapsed_depth = Some(entry.depth);
        }
    }

    (entries, matched)
}

pub(super) fn file_tree_entry_matches_query(
    entry: &FileTreeEntry,
    root: &Path,
    query: &str,
) -> bool {
    query.is_empty()
        || entry.name.to_ascii_lowercase().contains(query)
        || entry
            .path
            .strip_prefix(root)
            .ok()
            .and_then(Path::to_str)
            .map(|path| path.to_ascii_lowercase().contains(query))
            .unwrap_or(false)
}

pub(super) fn file_tree_content_width(entries: &[FileTreeEntry]) -> f32 {
    entries
        .iter()
        .map(|entry| entry.depth as f32 * 12. + 34. + estimate_file_tree_text_width(&entry.name))
        .fold(1., f32::max)
}

pub(super) fn estimate_file_tree_text_width(text: &str) -> f32 {
    text.chars()
        .map(|ch| if ch.is_ascii() { 7. } else { 12. })
        .sum()
}

/// Whether `path` is a directory that contains at least one entry.
/// Used to decide whether deleting the folder needs a second (recursive)
/// confirmation: empty folders are safe to remove with a single confirm.
pub(super) fn dir_is_non_empty(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

pub(super) fn reveal_in_system_file_manager(path: &Path, select_file: bool) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer");
        if select_file {
            command.arg(format!("/select,{}", path.display()));
        } else {
            command.arg(path);
        }
        command.spawn().map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        if select_file {
            command.arg("-R").arg(path);
        } else {
            command.arg(path);
        }
        command.spawn().map(|_| ())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = if select_file {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        Command::new("xdg-open").arg(target).spawn().map(|_| ())
    }
}

pub(super) fn file_tree_entry_icon(
    kind: FileTreeEntryKind,
    expanded: bool,
    palette: ThemePalette,
    color: Rgba,
) -> Div {
    match kind {
        FileTreeEntryKind::Directory if expanded => div()
            .relative()
            .w(px(16.))
            .h(px(16.))
            .flex_none()
            // The rear cover and tab stay visible above the wider front flap.
            .child(
                div()
                    .absolute()
                    .top(px(1.))
                    .left(px(2.))
                    .w(px(7.))
                    .h(px(5.))
                    .rounded_t_xs()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(4.))
                    .left(px(1.))
                    .w(px(14.))
                    .h(px(9.))
                    .rounded_xs()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(4.))
                    .left(px(3.))
                    .w(px(5.))
                    .h(px(1.))
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(7.))
                    .left(px(0.))
                    .w(px(16.))
                    .h(px(8.))
                    .rounded_xs()
                    .border_1()
                    .border_color(color)
                    .bg(palette.panel_bg),
            ),
        FileTreeEntryKind::Directory => div()
            .relative()
            .w(px(16.))
            .h(px(16.))
            .flex_none()
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(px(2.))
                    .w(px(7.))
                    .h(px(5.))
                    .rounded_t_xs()
                    .border_1()
                    .border_color(color)
                    .bg(palette.panel_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(5.))
                    .left(px(1.))
                    .w(px(14.))
                    .h(px(10.))
                    .rounded_xs()
                    .border_1()
                    .border_color(color)
                    .bg(palette.panel_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(5.))
                    .left(px(3.))
                    .w(px(5.))
                    .h(px(1.))
                    .bg(palette.panel_bg),
            ),
        FileTreeEntryKind::File => div()
            .relative()
            .w(px(14.))
            .h(px(16.))
            .flex_none()
            .child(
                div()
                    .absolute()
                    .top(px(1.))
                    .left(px(1.))
                    .w(px(11.))
                    .h(px(14.))
                    .rounded_sm()
                    .border_1()
                    .border_color(color)
                    .bg(palette.surface_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(2.))
                    .left(px(8.))
                    .w(px(4.))
                    .h(px(4.))
                    .border_l_1()
                    .border_b_1()
                    .border_color(color)
                    .bg(palette.panel_bg),
            )
            .child(
                div()
                    .absolute()
                    .top(px(5.))
                    .left(px(3.))
                    .text_size(px(7.))
                    .line_height(px(7.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(color)
                    .child("M"),
            ),
    }
}

pub(super) fn outline_panel_body(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let outline = app.active_tab().document.outline();
    let current = app
        .active_tab()
        .document
        .current_heading_index(app.active_tab().cursor_offset());
    let app_entity = cx.entity();

    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .child(
            div()
                .flex_1()
                .min_h_0()
                .children(outline.iter().enumerate().map(|(index, heading)| {
                    let app_entity = app_entity.clone();
                    let offset = heading.offset;
                    let active = current == Some(index);
                    let title = heading.title.clone();
                    let background = if active {
                        palette.active_bg
                    } else {
                        palette.panel_bg
                    };

                    div()
                        .mb_1()
                        .ml(px((heading.level.saturating_sub(1) as f32) * 12.))
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(background)
                        .text_size(px(12.))
                        .text_color(if active {
                            palette.active_text
                        } else {
                            palette.text
                        })
                        .cursor_pointer()
                        .hover(move |style| style.bg(palette.active_bg))
                        .child(title)
                        .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                            let focus_handle = app_entity.read(cx).focus_handle.clone();
                            window.focus(&focus_handle);
                            app_entity.update(cx, |app, cx| {
                                app.jump_to_offset(offset, cx);
                            });
                        })
                })),
        )
}

/// Unified left sidebar: a tab bar switches between the Files panel and the
/// document Outline, and the whole column can be toggled on/off as one unit.
pub(super) fn sidebar_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let app_entity = cx.entity();
    let active_tab = app.sidebar_tab;
    // Width is driven by `app.sidebar_width` so the resize divider can change
    // it; clamped to a sane range in the drag handler.
    let sidebar_width = app.sidebar_width;

    let files_active = active_tab == SidebarTab::Files;
    let outline_active = active_tab == SidebarTab::Outline;
    let files_bg = if files_active {
        palette.active_bg
    } else {
        palette.panel_bg
    };
    let files_text = if files_active {
        palette.active_text
    } else {
        palette.text
    };
    let outline_bg = if outline_active {
        palette.active_bg
    } else {
        palette.panel_bg
    };
    let outline_text = if outline_active {
        palette.active_text
    } else {
        palette.text
    };
    let hover_bg = palette.active_bg;

    div()
        .w(px(sidebar_width))
        .min_h_0()
        .flex_shrink_0()
        .p(px(SIDEBAR_COMPACT_PADDING))
        .border_r_1()
        .border_color(palette.border)
        .bg(palette.panel_bg)
        .flex()
        .flex_col()
        // NOTE: `.hidden()` must come *after* `.flex()`/`.flex_col()`. In GPUI
        // both set the same `display` field, so a later `.flex()` would clobber
        // an earlier `.hidden()` and the sidebar would never actually hide.
        .when(!app.sidebar_visible, |style| style.hidden())
        // Tab bar: Files / Outline. The active tab uses the same active-palette
        // highlight as tree rows so the two stay visually consistent.
        .child(
            div()
                .mb(px(PANE_OUTER_PADDING))
                .flex()
                .gap_1()
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(files_bg)
                        .text_size(px(12.))
                        .text_color(files_text)
                        .cursor_pointer()
                        .hover(move |style| style.bg(hover_bg))
                        .child(app.tr(Msg::LabelFiles))
                        .on_mouse_up(MouseButton::Left, {
                            let app_entity = app_entity.clone();
                            move |_, _, cx| {
                                app_entity.update(cx, |app, cx| {
                                    app.set_sidebar_tab(SidebarTab::Files, cx);
                                });
                            }
                        }),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(outline_bg)
                        .text_size(px(12.))
                        .text_color(outline_text)
                        .cursor_pointer()
                        .hover(move |style| style.bg(hover_bg))
                        .child(app.tr(Msg::LabelOutline))
                        .on_mouse_up(MouseButton::Left, {
                            let app_entity = app_entity.clone();
                            move |_, _, cx| {
                                app_entity.update(cx, |app, cx| {
                                    app.set_sidebar_tab(SidebarTab::Outline, cx);
                                });
                            }
                        }),
                ),
        )
        // Only build the active panel body when the sidebar is actually
        // visible. The whole sidebar is `.hidden()` when collapsed, but the
        // element tree (and, for the Outline tab, the full-document heading
        // parse in `outline_panel_body`) was still constructed every frame.
        // Skipping it here means a collapsed sidebar costs nothing per keystroke.
        .when(app.sidebar_visible, |container| {
            container.child(match active_tab {
                SidebarTab::Files => file_tree_panel_body(app, cx),
                SidebarTab::Outline => outline_panel_body(app, cx),
            })
        })
}

pub(super) fn pane_scrollbar_view(
    target: PaneScrollTarget,
    scroll_handle: &ScrollHandle,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    let id = match target {
        PaneScrollTarget::Editor => "editor-pane-scrollbar",
        PaneScrollTarget::Preview => "preview-pane-scrollbar",
    };
    let viewport_height = scroll_handle.bounds().size.height;
    let max_scroll = scroll_handle.max_offset().height.max(px(0.));
    if viewport_height <= px(0.) || max_scroll <= px(1.) {
        return div().id(id).hidden();
    }

    let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
    let track_height = (viewport_height - track_inset - track_inset).max(px(0.));
    if track_height <= px(0.) {
        return div().id(id).hidden();
    }

    let content_height = viewport_height + max_scroll;
    let thumb_height = (track_height * (viewport_height / content_height))
        .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
    let thumb_travel = (track_height - thumb_height).max(px(0.));
    let offset_y = (-scroll_handle.offset().y).clamp(px(0.), max_scroll);
    let thumb_top = track_inset + thumb_travel * (offset_y / max_scroll);

    let entity = cx.entity();
    let scroll_handle = scroll_handle.clone();

    div()
        .id(id)
        .absolute()
        .top(thumb_top)
        .right(px(2.))
        .w(px(PANE_SCROLLBAR_THUMB_WIDTH))
        .h(thumb_height)
        .rounded_md()
        .bg(palette.muted)
        .hover(move |style| style.bg(palette.active_text))
        .block_mouse_except_scroll()
        .child(
            canvas(
                |_, _, _| (),
                move |thumb_bounds, _, window, _| {
                    window.on_mouse_event({
                        let entity = entity.clone();
                        move |event: &MouseDownEvent, _, _, cx| {
                            if !thumb_bounds.contains(&event.position) {
                                return;
                            }
                            entity.update(cx, |app, _| {
                                app.pane_scrollbar_drag = Some(PaneScrollbarDrag {
                                    target,
                                    thumb_grab_offset_y: event.position.y - thumb_bounds.top(),
                                });
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        move |_: &MouseUpEvent, _, _, cx| {
                            entity.update(cx, |app, _| {
                                if app
                                    .pane_scrollbar_drag
                                    .is_some_and(|drag| drag.target == target)
                                {
                                    app.pane_scrollbar_drag = None;
                                }
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let scroll_handle = scroll_handle.clone();
                        move |event: &MouseMoveEvent, _, _, cx| {
                            if !event.dragging() {
                                return;
                            }
                            let Some(drag) = entity.read(cx).pane_scrollbar_drag else {
                                return;
                            };
                            if drag.target != target {
                                return;
                            }

                            let viewport_height = scroll_handle.bounds().size.height;
                            let max_scroll = scroll_handle.max_offset().height.max(px(0.));
                            if viewport_height <= px(0.) || max_scroll <= px(1.) {
                                return;
                            }

                            let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
                            let track_height =
                                (viewport_height - track_inset - track_inset).max(px(0.));
                            if track_height <= px(0.) {
                                return;
                            }

                            let content_height = viewport_height + max_scroll;
                            let thumb_height = (track_height * (viewport_height / content_height))
                                .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
                            let thumb_travel = (track_height - thumb_height).max(px(0.));
                            if thumb_travel <= px(0.) {
                                return;
                            }

                            let local_y = event.position.y
                                - scroll_handle.bounds().top()
                                - track_inset
                                - drag.thumb_grab_offset_y;
                            let percentage = (local_y / thumb_travel).clamp(0., 1.);
                            let scroll_y = max_scroll * percentage;
                            scroll_handle.set_offset(point(scroll_handle.offset().x, -scroll_y));
                            cx.notify(entity.entity_id());
                        }
                    });
                },
            )
            .size_full(),
        )
}

/// The custom-drawn overlay scrollbar for the virtualized preview `list`.
///
/// Mirrors [`pane_scrollbar_view`] but reads geometry from [`ListState`]'s
/// scrollbar API (`viewport_bounds`, `max_offset_for_scrollbar`,
/// `scroll_px_offset_for_scrollbar`) instead of a `ScrollHandle`, and drives the
/// list during a drag via `set_offset_from_scrollbar`. The
/// `scrollbar_drag_started`/`_ended` calls freeze the reported content height
/// for the duration of a drag so the thumb does not jump as off-screen blocks
/// get measured.
pub(super) fn preview_list_scrollbar_view(
    list_state: &ListState,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    let id = "preview-pane-scrollbar";
    let target = PaneScrollTarget::Preview;
    let viewport_height = list_state.viewport_bounds().size.height;
    let max_scroll = list_state.max_offset_for_scrollbar().height.max(px(0.));
    if viewport_height <= px(0.) || max_scroll <= px(1.) {
        return div().id(id).hidden();
    }

    let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
    let track_height = (viewport_height - track_inset - track_inset).max(px(0.));
    if track_height <= px(0.) {
        return div().id(id).hidden();
    }

    let content_height = viewport_height + max_scroll;
    let thumb_height = (track_height * (viewport_height / content_height))
        .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
    let thumb_travel = (track_height - thumb_height).max(px(0.));
    let offset_y = (-list_state.scroll_px_offset_for_scrollbar().y).clamp(px(0.), max_scroll);
    let thumb_top = track_inset + thumb_travel * (offset_y / max_scroll);

    let entity = cx.entity();
    let list_state = list_state.clone();

    div()
        .id(id)
        .absolute()
        .top(thumb_top)
        .right(px(2.))
        .w(px(PANE_SCROLLBAR_THUMB_WIDTH))
        .h(thumb_height)
        .rounded_md()
        .bg(palette.muted)
        .hover(move |style| style.bg(palette.active_text))
        .block_mouse_except_scroll()
        .child(
            canvas(
                |_, _, _| (),
                move |thumb_bounds, _, window, _| {
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let list_state = list_state.clone();
                        move |event: &MouseDownEvent, _, _, cx| {
                            if !thumb_bounds.contains(&event.position) {
                                return;
                            }
                            list_state.scrollbar_drag_started();
                            entity.update(cx, |app, _| {
                                app.pane_scrollbar_drag = Some(PaneScrollbarDrag {
                                    target,
                                    thumb_grab_offset_y: event.position.y - thumb_bounds.top(),
                                });
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let list_state = list_state.clone();
                        move |_: &MouseUpEvent, _, _, cx| {
                            entity.update(cx, |app, _| {
                                if app
                                    .pane_scrollbar_drag
                                    .is_some_and(|drag| drag.target == target)
                                {
                                    app.pane_scrollbar_drag = None;
                                    list_state.scrollbar_drag_ended();
                                }
                            });
                        }
                    });
                    window.on_mouse_event({
                        let entity = entity.clone();
                        let list_state = list_state.clone();
                        move |event: &MouseMoveEvent, _, _, cx| {
                            if !event.dragging() {
                                return;
                            }
                            let Some(drag) = entity.read(cx).pane_scrollbar_drag else {
                                return;
                            };
                            if drag.target != target {
                                return;
                            }

                            let viewport_height = list_state.viewport_bounds().size.height;
                            let max_scroll =
                                list_state.max_offset_for_scrollbar().height.max(px(0.));
                            if viewport_height <= px(0.) || max_scroll <= px(1.) {
                                return;
                            }

                            let track_inset = px(PANE_SCROLLBAR_EDGE_INSET);
                            let track_height =
                                (viewport_height - track_inset - track_inset).max(px(0.));
                            if track_height <= px(0.) {
                                return;
                            }

                            let content_height = viewport_height + max_scroll;
                            let thumb_height = (track_height * (viewport_height / content_height))
                                .clamp(px(PANE_SCROLLBAR_MIN_THUMB_HEIGHT), track_height);
                            let thumb_travel = (track_height - thumb_height).max(px(0.));
                            if thumb_travel <= px(0.) {
                                return;
                            }

                            let local_y = event.position.y
                                - list_state.viewport_bounds().top()
                                - track_inset
                                - drag.thumb_grab_offset_y;
                            let percentage = (local_y / thumb_travel).clamp(0., 1.);
                            let scroll_y = max_scroll * percentage;
                            list_state.set_offset_from_scrollbar(point(px(0.), -scroll_y));
                            cx.notify(entity.entity_id());
                        }
                    });
                },
            )
            .size_full(),
        )
}

/// A vertical resize divider rendered between two panes. Visually it is a 1px
/// rule, but an 8px-wide transparent handle is layered on top so the grab target
/// is usable. Double-clicking the handle restores the default split. Pattern
/// follows Zed's `render_resize_handle`.
pub(super) fn editor_split_handle_view(
    border_color: Rgba,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    div()
        .id("editor-split-resize-container")
        .relative()
        .h_full()
        .flex_shrink_0()
        .w(px(SIDEBAR_DIVIDER_WIDTH))
        .bg(border_color)
        .child(
            div()
                .id("editor-split-resize-handle")
                .absolute()
                .left(px(-RESIZE_HANDLE_WIDTH / 2.0))
                .w(px(RESIZE_HANDLE_WIDTH))
                .h_full()
                .cursor(CursorStyle::ResizeColumn)
                .block_mouse_except_scroll()
                .on_click(cx.listener(move |app, event: &ClickEvent, _, cx| {
                    // Double-click resets the editor/preview split to 50/50.
                    if event.click_count() >= 2 {
                        app.editor_split_ratio = 0.5;
                        cx.notify();
                    }
                }))
                .on_drag(DraggedEditorSplitHandle, move |_, _, _, cx| {
                    cx.new(|_| Empty)
                }),
        )
}

/// Resize divider for the sidebar's right edge: same visual pattern as the
/// editor split handle, but keyed on `DraggedSidebarHandle` and resets to the
/// default sidebar width on double-click.
pub(super) fn sidebar_resize_handle_view(
    border_color: Rgba,
    cx: &mut Context<MarkionApp>,
) -> Stateful<Div> {
    div()
        .id("sidebar-resize-container")
        .relative()
        .h_full()
        .flex_shrink_0()
        .w(px(1.))
        .bg(border_color)
        .child(
            div()
                .id("sidebar-resize-handle")
                .absolute()
                .left(px(-RESIZE_HANDLE_WIDTH / 2.0))
                .w(px(RESIZE_HANDLE_WIDTH))
                .h_full()
                .cursor(CursorStyle::ResizeColumn)
                .block_mouse_except_scroll()
                .on_click(cx.listener(move |app, event: &ClickEvent, _, cx| {
                    if event.click_count() >= 2 {
                        app.sidebar_width = DEFAULT_SIDEBAR_WIDTH;
                        cx.notify();
                    }
                }))
                .on_drag(DraggedSidebarHandle, move |_, _, _, cx| cx.new(|_| Empty)),
        )
}

pub(super) fn toolbar_button(
    label: &'static str,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .h(px(26.))
        .min_w(px(26.))
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_bg)
        .text_color(palette.text)
        .text_size(px(12.))
        .cursor_pointer()
        .hover(move |style| style.bg(palette.active_bg).text_color(palette.active_text))
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}

pub(super) fn file_tree_context_menu_view(
    app: &MarkionApp,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let palette = app.palette();
    let menu = app
        .file_tree_context_menu
        .as_ref()
        .expect("file-tree context menu view requires open menu state");
    let position = menu.position;
    let target_kind = menu.target.kind();
    let app_entity = cx.entity();

    // `anchored` measures the menu and flips/snaps it back inside the viewport.
    // A raw absolute cursor position lets the lower actions (notably Delete)
    // fall below the window when a row near the bottom is right-clicked.
    anchored().position(position).child(
        div()
            .w(px(208.))
            // Without occlude, mouse-down on a menu item falls through to
            // #main-content-row's close_menu handler, which clears the menu state
            // before the item's on_mouse_up can dispatch its action (same bug the
            // app-menu dropdown had — see active_menu_dropdown).
            .occlude()
            .p_1()
            .rounded_md()
            .border_1()
            .border_color(palette.border)
            .bg(palette.surface_bg)
            .shadow_md()
            .children(
                file_tree_context_actions(target_kind)
                    .iter()
                    .copied()
                    .map(move |action| {
                        let app_entity = app_entity.clone();
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .text_size(px(12.))
                            .text_color(palette.text)
                            .cursor_pointer()
                            .hover(move |style| style.bg(palette.active_bg))
                            .child(t(app.language, file_tree_context_action_label(action)))
                            .on_mouse_up(MouseButton::Left, move |_, window, cx| {
                                app_entity.update(cx, |app, cx| {
                                    app.handle_file_tree_context_action(action, window, cx);
                                });
                            })
                    }),
            ),
    )
}

pub(super) fn preview_context_action_label(action: PreviewContextAction) -> Msg {
    match action {
        PreviewContextAction::CopyPlain => Msg::ItemPreviewCopyPlain,
        PreviewContextAction::CopyMarkdown => Msg::ItemPreviewCopyMarkdown,
        PreviewContextAction::CopyHtml => Msg::ItemPreviewCopyHtml,
        PreviewContextAction::SelectAll => Msg::ItemPreviewSelectAll,
        PreviewContextAction::CopyLinkAddress => Msg::ItemPreviewCopyLinkAddress,
    }
}

pub(super) fn preview_context_menu_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let Some(menu) = &app.preview_context_menu else {
        return div().hidden();
    };
    let position = menu.position;
    let link_url = menu.link_url.clone();
    let blocks = app.active_tab().preview_list_blocks.clone();
    let has_selection = preview_selection_takes_copy_precedence(
        app.active_tab().preview_selection.as_ref(),
        &blocks,
    );
    let app_entity = cx.entity();

    let items: Vec<(PreviewContextAction, bool)> = {
        let mut items = vec![
            (PreviewContextAction::CopyPlain, has_selection),
            (PreviewContextAction::CopyMarkdown, has_selection),
            (PreviewContextAction::CopyHtml, has_selection),
            (PreviewContextAction::SelectAll, true),
        ];
        if link_url.is_some() {
            items.push((PreviewContextAction::CopyLinkAddress, true));
        }
        items
    };

    div()
        .absolute()
        .top(position.y)
        .left(position.x)
        .w(px(220.))
        .occlude()
        .p_1()
        .rounded_md()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_bg)
        .shadow_md()
        .children(items.into_iter().map(move |(action, enabled)| {
            let app_entity = app_entity.clone();
            let label = t(app.language, preview_context_action_label(action));
            let text_color = if enabled { palette.text } else { palette.muted };
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .text_size(px(12.))
                .text_color(text_color)
                .when(enabled, |style| {
                    style
                        .cursor_pointer()
                        .hover(move |style| style.bg(palette.active_bg))
                        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                            app_entity.update(cx, |app, cx| {
                                app.handle_preview_context_action(action, cx);
                            });
                        })
                })
                .child(label)
        }))
}

pub(super) fn menu_title_button(
    label: &'static str,
    active: bool,
    palette: ThemePalette,
    click_listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
    hover_listener: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let background = if active {
        palette.active_bg
    } else {
        palette.panel_bg
    };
    let foreground = if active {
        palette.active_text
    } else {
        palette.text
    };
    let hover_bg = if active {
        palette.active_bg
    } else {
        palette.surface_bg
    };

    div()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(background)
        .text_size(px(13.))
        .text_color(foreground)
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg))
        .on_mouse_up(MouseButton::Left, click_listener)
        .on_mouse_move(hover_listener)
        .child(label)
}

pub(super) fn menu_action_button(
    label: impl Into<SharedString>,
    shortcut: Option<&'static str>,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let row = div()
        .w_full()
        .px_3()
        .py_1()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .text_size(px(12.))
        .text_color(palette.text)
        .cursor_pointer()
        .hover(move |style| style.bg(palette.surface_bg))
        .on_mouse_up(MouseButton::Left, listener)
        .child(div().min_w_0().child(label.into()));

    if let Some(shortcut) = shortcut {
        row.child(div().flex_none().text_color(palette.muted).child(shortcut))
    } else {
        row
    }
}

pub(super) fn menu_muted_label(label: impl Into<SharedString>, palette: ThemePalette) -> Div {
    div()
        .w_full()
        .px_3()
        .py_1()
        .text_size(px(12.))
        .text_color(palette.muted)
        .child(label.into())
}

pub(super) fn menu_separator(palette: ThemePalette) -> Div {
    div().h(px(1.)).my_1().bg(palette.border)
}

pub(super) fn heading_item_msg(level: u8) -> Msg {
    match level {
        1 => Msg::ItemH1,
        2 => Msg::ItemH2,
        3 => Msg::ItemH3,
        4 => Msg::ItemH4,
        5 => Msg::ItemH5,
        6 => Msg::ItemH6,
        _ => Msg::ItemH1,
    }
}

pub(super) fn heading_native_menu_items(language: Language, max_level: u8) -> Vec<MenuItem> {
    (1..=max_level)
        .map(|level| match level {
            1 => MenuItem::action(t(language, heading_item_msg(level)), Heading1),
            2 => MenuItem::action(t(language, heading_item_msg(level)), Heading2),
            3 => MenuItem::action(t(language, heading_item_msg(level)), Heading3),
            4 => MenuItem::action(t(language, heading_item_msg(level)), Heading4),
            5 => MenuItem::action(t(language, heading_item_msg(level)), Heading5),
            6 => MenuItem::action(t(language, heading_item_msg(level)), Heading6),
            _ => MenuItem::action(t(language, heading_item_msg(level)), Heading1),
        })
        .collect()
}

pub(super) fn active_menu_dropdown(
    menu: Option<AppMenu>,
    language: Language,
    heading_menu_max_level: u8,
    recent_files: Vec<PathBuf>,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let Some(menu) = menu else {
        return div()
            .absolute()
            .top(px(28.))
            .left(px(0.))
            .w(px(0.))
            .h(px(0.));
    };

    let panel = div()
        .absolute()
        .top(px(28.))
        .left(menu.dropdown_left(language))
        .w(menu.dropdown_width(language))
        // Without occlude the dropdown does not capture mouse hits, so clicks
        // fall through to the content underneath (which closes the menu) and
        // the item's action is never dispatched.
        .occlude()
        .py_1()
        .border_1()
        .border_color(palette.border)
        .rounded_md()
        .bg(palette.panel_bg)
        .text_color(palette.text)
        .shadow_md()
        .flex()
        .flex_col();

    // Menu items call their handler directly via cx.listener, exactly like the
    // menu-bar title buttons. Dispatching through window.dispatch_action here
    // proved unreliable (the action never reached the focused handlers), so the
    // items appeared to do nothing when clicked.
    //
    // Labels are translated at render time from the app's active language, so
    // switching language via View → Language reflows the menu immediately.
    let shortcut_platform = ShortcutPlatform::current();
    macro_rules! action_item {
        (@build $msg:expr, $method:ident, $action:expr, $shortcut:expr) => {
            menu_action_button(
                t(language, $msg),
                $shortcut,
                palette,
                cx.listener(move |app, _: &MouseUpEvent, window, cx| {
                    app.$method(&$action, window, cx);
                }),
            )
        };
        ($msg:expr, $method:ident, $action:expr, $shortcut:expr) => {
            action_item!(
                @build $msg,
                $method,
                $action,
                Some($shortcut.label(shortcut_platform))
            )
        };
        ($msg:expr, $method:ident, $action:expr) => {
            action_item!(@build $msg, $method, $action, None)
        };
    }

    match menu {
        AppMenu::File => panel
            .child(action_item!(
                Msg::ItemNew,
                new_document,
                NewDocument,
                menu_shortcuts::NEW_DOCUMENT
            ))
            .child(action_item!(
                Msg::ItemOpen,
                open_document,
                OpenDocument,
                menu_shortcuts::OPEN_DOCUMENT
            ))
            .child(action_item!(Msg::ItemOpenFolder, open_folder, OpenFolder))
            .child(menu_separator(palette))
            .child(menu_muted_label(t(language, Msg::ItemOpenRecent), palette))
            .when(recent_files.is_empty(), |panel| {
                panel.child(menu_muted_label(
                    t(language, Msg::ItemOpenRecentEmpty),
                    palette,
                ))
            })
            .children(recent_files.into_iter().map(|path| {
                let label = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.to_string())
                    .unwrap_or_else(|| path.display().to_string());
                let open_path = path.clone();
                menu_action_button(
                    label,
                    None,
                    palette,
                    cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                        app.open_recent_path(open_path.clone(), cx);
                    }),
                )
            }))
            .child(action_item!(
                Msg::ItemClearRecentFiles,
                clear_recent_files,
                ClearRecentFiles
            ))
            .child(action_item!(
                Msg::ItemSave,
                save_document,
                SaveDocument,
                menu_shortcuts::SAVE_DOCUMENT
            ))
            .child(action_item!(
                Msg::ItemSaveAs,
                save_document_as,
                SaveDocumentAs,
                menu_shortcuts::SAVE_DOCUMENT_AS
            ))
            .child(menu_separator(palette))
            .child(action_item!(Msg::ItemNewTab, new_tab, NewTab))
            .child(action_item!(
                Msg::ItemOpenInNewTab,
                open_in_new_tab_action,
                OpenInNewTab,
                menu_shortcuts::OPEN_IN_NEW_TAB
            ))
            .child(action_item!(
                Msg::ItemCloseTab,
                close_tab,
                CloseTab,
                menu_shortcuts::CLOSE_TAB
            ))
            .child(action_item!(
                Msg::ItemNextTab,
                next_tab,
                NextTab,
                menu_shortcuts::NEXT_TAB
            ))
            .child(action_item!(
                Msg::ItemPrevTab,
                prev_tab,
                PrevTab,
                menu_shortcuts::PREV_TAB
            ))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemPreferences,
                show_preferences,
                ShowPreferences,
                menu_shortcuts::SHOW_PREFERENCES
            ))
            .child(action_item!(
                Msg::ItemResetPreferences,
                reset_preferences,
                ResetPreferences
            ))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemExit,
                quit,
                Quit,
                menu_shortcuts::QUIT
            )),
        AppMenu::Edit => panel
            .child(action_item!(
                Msg::ItemUndo,
                undo,
                Undo,
                menu_shortcuts::UNDO
            ))
            .child(action_item!(
                Msg::ItemRedo,
                redo,
                Redo,
                menu_shortcuts::REDO
            ))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemCopy,
                copy,
                Copy,
                menu_shortcuts::COPY
            ))
            .child(action_item!(Msg::ItemCut, cut, Cut, menu_shortcuts::CUT))
            .child(action_item!(
                Msg::ItemPaste,
                paste,
                Paste,
                menu_shortcuts::PASTE
            ))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemSelectAll,
                select_all,
                SelectAll,
                menu_shortcuts::SELECT_ALL
            )),
        AppMenu::View => panel
            .child(action_item!(
                Msg::ItemToggleView,
                toggle_view_mode,
                ToggleViewMode,
                menu_shortcuts::TOGGLE_VIEW_MODE
            ))
            .child(action_item!(
                Msg::ItemEditMode,
                set_edit_mode,
                SetEditMode,
                menu_shortcuts::SET_EDIT_MODE
            ))
            .child(action_item!(
                Msg::ItemVisualEditMode,
                set_visual_edit_mode,
                SetVisualEditMode,
                menu_shortcuts::SET_VISUAL_EDIT_MODE
            ))
            .child(action_item!(
                Msg::ItemSplitPreviewMode,
                set_split_preview_mode,
                SetSplitPreviewMode,
                menu_shortcuts::SET_SPLIT_PREVIEW_MODE
            ))
            .child(action_item!(
                Msg::ItemReadMode,
                set_read_mode,
                SetReadMode,
                menu_shortcuts::SET_READ_MODE
            ))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemToggleSidebar,
                toggle_sidebar,
                ToggleSidebar,
                menu_shortcuts::TOGGLE_SIDEBAR
            ))
            .child(action_item!(
                Msg::ItemFiles,
                toggle_file_tree,
                ToggleFileTree,
                menu_shortcuts::TOGGLE_FILE_TREE
            ))
            .child(action_item!(
                Msg::ItemOutline,
                toggle_outline,
                ToggleOutline,
                menu_shortcuts::TOGGLE_OUTLINE
            ))
            .child(action_item!(
                Msg::ItemFocusMode,
                toggle_focus_mode,
                ToggleFocusMode,
                menu_shortcuts::TOGGLE_FOCUS_MODE
            ))
            .child(action_item!(
                Msg::ItemTypewriterMode,
                toggle_typewriter_mode,
                ToggleTypewriterMode,
                menu_shortcuts::TOGGLE_TYPEWRITER_MODE
            ))
            .child(action_item!(
                Msg::ItemCodeLineNumbers,
                toggle_code_line_numbers,
                ToggleCodeLineNumbers,
                menu_shortcuts::TOGGLE_CODE_LINE_NUMBERS
            ))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemFind,
                show_find,
                ShowFind,
                menu_shortcuts::SHOW_FIND
            ))
            .child(action_item!(
                Msg::ItemReplace,
                show_replace,
                ShowReplace,
                menu_shortcuts::SHOW_REPLACE
            ))
            .child(action_item!(
                Msg::ItemFindNext,
                find_next,
                FindNext,
                menu_shortcuts::FIND_NEXT
            ))
            .child(action_item!(
                Msg::ItemFindPrevious,
                find_previous,
                FindPrevious,
                menu_shortcuts::FIND_PREVIOUS
            ))
            .child(menu_separator(palette))
            .child(action_item!(
                Msg::ItemCycleTheme,
                cycle_theme,
                CycleTheme,
                menu_shortcuts::CYCLE_THEME
            )),
        AppMenu::Format => {
            let with_core_headings = panel
                .child(action_item!(
                    Msg::ItemBold,
                    bold,
                    Bold,
                    menu_shortcuts::BOLD
                ))
                .child(action_item!(
                    Msg::ItemItalic,
                    italic,
                    Italic,
                    menu_shortcuts::ITALIC
                ))
                .child(action_item!(
                    Msg::ItemInlineCode,
                    inline_code,
                    InlineCode,
                    menu_shortcuts::INLINE_CODE
                ))
                .child(action_item!(
                    Msg::ItemLink,
                    insert_link,
                    InsertLink,
                    menu_shortcuts::INSERT_LINK
                ))
                .child(action_item!(
                    Msg::ItemImage,
                    insert_image,
                    InsertImage,
                    menu_shortcuts::INSERT_IMAGE
                ))
                .child(menu_separator(palette))
                .child(action_item!(
                    Msg::ItemH1,
                    heading1,
                    Heading1,
                    menu_shortcuts::HEADING_1
                ))
                .child(action_item!(
                    Msg::ItemH2,
                    heading2,
                    Heading2,
                    menu_shortcuts::HEADING_2
                ))
                .child(action_item!(
                    Msg::ItemH3,
                    heading3,
                    Heading3,
                    menu_shortcuts::HEADING_3
                ));
            let with_headings = with_core_headings
                .child(action_item!(
                    Msg::ItemH4,
                    heading4,
                    Heading4,
                    menu_shortcuts::HEADING_4
                ))
                .child(action_item!(
                    Msg::ItemH5,
                    heading5,
                    Heading5,
                    menu_shortcuts::HEADING_5
                ));
            let with_headings = if heading_menu_max_level >= EXTENDED_HEADING_MENU_MAX_LEVEL {
                with_headings.child(action_item!(
                    Msg::ItemH6,
                    heading6,
                    Heading6,
                    menu_shortcuts::HEADING_6
                ))
            } else {
                with_headings
            };
            with_headings
                .child(menu_separator(palette))
                .child(action_item!(
                    Msg::ItemBullets,
                    unordered_list,
                    UnorderedList
                ))
                .child(action_item!(Msg::ItemNumbers, ordered_list, OrderedList))
                .child(action_item!(Msg::ItemTask, task_list, TaskList))
                .child(action_item!(Msg::ItemQuote, block_quote, BlockQuote))
                .child(action_item!(Msg::ItemCodeFence, code_fence, CodeFence))
                .child(menu_separator(palette))
                .child(action_item!(
                    Msg::ItemFormatTable,
                    format_table,
                    FormatTable,
                    menu_shortcuts::FORMAT_TABLE
                ))
                .child(action_item!(
                    Msg::ItemAddTableRow,
                    table_add_row,
                    TableAddRow,
                    menu_shortcuts::TABLE_ADD_ROW
                ))
                .child(action_item!(
                    Msg::ItemDeleteTableRow,
                    table_delete_row,
                    TableDeleteRow,
                    menu_shortcuts::TABLE_DELETE_ROW
                ))
                .child(action_item!(
                    Msg::ItemMoveRowUp,
                    table_move_row_up,
                    TableMoveRowUp,
                    menu_shortcuts::TABLE_MOVE_ROW_UP
                ))
                .child(action_item!(
                    Msg::ItemMoveRowDown,
                    table_move_row_down,
                    TableMoveRowDown,
                    menu_shortcuts::TABLE_MOVE_ROW_DOWN
                ))
                .child(action_item!(
                    Msg::ItemAddTableColumn,
                    table_add_column,
                    TableAddColumn,
                    menu_shortcuts::TABLE_ADD_COLUMN
                ))
                .child(action_item!(
                    Msg::ItemDeleteTableColumn,
                    table_delete_column,
                    TableDeleteColumn,
                    menu_shortcuts::TABLE_DELETE_COLUMN
                ))
        }
        AppMenu::Export => panel
            .child(action_item!(
                Msg::ItemExportHtml,
                export_html,
                ExportHtml,
                menu_shortcuts::EXPORT_HTML
            ))
            .child(action_item!(
                Msg::ItemExportPlainHtml,
                export_plain_html,
                ExportPlainHtml,
                menu_shortcuts::EXPORT_PLAIN_HTML
            ))
            .child(action_item!(
                Msg::ItemExportPdf,
                export_pdf,
                ExportPdf,
                menu_shortcuts::EXPORT_PDF
            ))
            .child(action_item!(
                Msg::ItemExportLatex,
                export_latex,
                ExportLatex,
                menu_shortcuts::EXPORT_LATEX
            ))
            .child(action_item!(
                Msg::ItemExportDocx,
                export_docx,
                ExportDocx,
                menu_shortcuts::EXPORT_DOCX
            ))
            .child(action_item!(
                Msg::ItemExportPng,
                export_png,
                ExportPng,
                menu_shortcuts::EXPORT_PNG
            ))
            .child(action_item!(
                Msg::ItemExportJpeg,
                export_jpeg,
                ExportJpeg,
                menu_shortcuts::EXPORT_JPEG
            )),
        AppMenu::Help => panel
            .child(action_item!(
                Msg::ItemKeyboardShortcuts,
                show_shortcuts,
                ShowShortcuts,
                menu_shortcuts::SHOW_SHORTCUTS
            ))
            .child(action_item!(Msg::ItemAboutMarkion, about, AboutMarkion)),
    }
}

/// Theme-aware Help -> Keyboard Shortcuts modal. The platform and category
/// selectors only change transient app state; the catalog itself stays in the
/// i18n layer so labels and displayed bindings share one source of truth.
pub(super) fn shortcut_panel_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let language = app.language;
    let selected_platform = app.shortcut_platform;
    let selected_category = app.shortcut_category;
    let catalog = shortcut_catalog(language, app.heading_menu_max_level);
    let section = catalog
        .section(selected_category)
        .expect("shortcut catalog contains every category");

    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .p_4()
        .bg(rgba(0x00000066))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .occlude()
                .track_focus(&app.shortcut_panel_focus)
                .w(px(720.))
                .max_w(px(720.))
                .flex_none()
                .min_w_0()
                .h(px(560.))
                .overflow_hidden()
                .bg(palette.panel_bg)
                .border_1()
                .border_color(palette.border)
                .rounded_lg()
                .shadow_lg()
                .text_color(palette.text)
                .flex()
                .flex_col()
                .child(
                    div()
                        .w(px(718.))
                        .h(px(52.))
                        .px_4()
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_4()
                        .child(
                            div()
                                .min_w_0()
                                .text_size(px(15.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(app.tr(Msg::DialogShortcutsTitle)),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .text_color(palette.muted)
                                .hover(move |style| style.bg(palette.surface_bg))
                                .child("✕")
                                .on_mouse_up(
                                    MouseButton::Left,
                                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                                        app.close_shortcut_panel(window, cx);
                                    }),
                                ),
                        ),
                )
                .child(
                    div()
                        .w(px(718.))
                        .px_4()
                        .pb_3()
                        .flex_none()
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .p(px(2.))
                                .rounded_md()
                                .bg(palette.surface_bg)
                                .border_1()
                                .border_color(palette.border)
                                .flex()
                                .gap_1()
                                .children(ShortcutPlatform::ALL.into_iter().map(|platform| {
                                    shortcut_platform_tab(
                                        platform.label(language),
                                        platform == selected_platform,
                                        palette,
                                        cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                                            app.select_shortcut_platform(platform, cx);
                                        }),
                                    )
                                })),
                        ),
                )
                .child(
                    div()
                        .w(px(718.))
                        .flex_1()
                        .min_h_0()
                        .border_t_1()
                        .border_color(palette.border)
                        .overflow_hidden()
                        .flex()
                        .child(
                            div()
                                .id("shortcut-panel-categories")
                                .w(px(152.))
                                .flex_none()
                                .py_2()
                                .border_r_1()
                                .border_color(palette.border)
                                .bg(palette.surface_bg)
                                .overflow_y_scroll()
                                .scrollbar_width(px(8.))
                                .children(catalog.sections.iter().map(|section| {
                                    let category = section.category;
                                    shortcut_category_button(
                                        section.label,
                                        category == selected_category,
                                        palette,
                                        cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                                            app.select_shortcut_category(category, cx);
                                        }),
                                    )
                                })),
                        )
                        .child(
                            div()
                                .w(px(566.))
                                .flex_none()
                                .min_w_0()
                                .min_h_0()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .h(px(42.))
                                        .px_4()
                                        .flex_none()
                                        .flex()
                                        .items_center()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_size(px(13.))
                                        .child(section.label),
                                )
                                .child(
                                    div()
                                        .id("shortcut-panel-actions")
                                        .flex_1()
                                        .min_h_0()
                                        .overflow_y_scroll()
                                        .scrollbar_width(px(8.))
                                        .border_t_1()
                                        .border_color(palette.border)
                                        .children(section.actions.iter().map(|action| {
                                            shortcut_action_row(
                                                action.label,
                                                action.combinations(selected_platform),
                                                palette,
                                            )
                                        })),
                                ),
                        ),
                ),
        )
}

fn shortcut_platform_tab(
    label: &'static str,
    active: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let background = if active {
        palette.active_bg
    } else {
        palette.surface_bg
    };
    let foreground = if active {
        palette.active_text
    } else {
        palette.text
    };

    div()
        .min_w(px(128.))
        .px_3()
        .py_1()
        .rounded_sm()
        .bg(background)
        .text_color(foreground)
        .text_size(px(12.))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .hover(move |style| {
            if !active {
                style.bg(palette.active_bg).text_color(palette.active_text)
            } else {
                style
            }
        })
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}

fn shortcut_category_button(
    label: &'static str,
    active: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let background = if active {
        palette.active_bg
    } else {
        palette.surface_bg
    };
    let foreground = if active {
        palette.active_text
    } else {
        palette.text
    };

    div()
        .w_full()
        .min_h(px(38.))
        .px_4()
        .py_2()
        .bg(background)
        .text_color(foreground)
        .text_size(px(12.))
        .cursor_pointer()
        .flex()
        .items_center()
        .hover(move |style| {
            if !active {
                style.bg(palette.active_bg).text_color(palette.active_text)
            } else {
                style
            }
        })
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}

fn shortcut_action_row(
    label: &'static str,
    combinations: &'static [&'static str],
    palette: ThemePalette,
) -> Div {
    div()
        .w_full()
        .min_h(px(46.))
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(palette.border)
        .flex()
        .items_start()
        .justify_between()
        .gap_3()
        .child(
            div()
                .w(px(150.))
                .flex_none()
                .pt_1()
                .text_size(px(13.))
                .child(label),
        )
        .child(
            div()
                .w(px(350.))
                .flex_none()
                .flex()
                .flex_wrap()
                .justify_end()
                .gap_1()
                .children(
                    combinations
                        .iter()
                        .copied()
                        .map(|shortcut| shortcut_key_label(shortcut, palette)),
                ),
        )
}

fn shortcut_key_label(shortcut: &'static str, palette: ThemePalette) -> Div {
    div()
        .min_h(px(26.))
        .px_2()
        .py_1()
        .rounded_sm()
        .border_1()
        .border_color(palette.border)
        .bg(palette.surface_bg)
        .text_color(palette.text)
        .text_size(px(11.))
        .flex()
        .items_center()
        .justify_center()
        .child(shortcut)
}

/// Modal overlay for the in-app Preferences panel. Clicks dispatch through
/// `cx.listener` closures so each setting updates live app state and persists
/// through the existing preferences path.
pub(super) fn preferences_panel_view(app: &MarkionApp, cx: &mut Context<MarkionApp>) -> Div {
    let palette = app.palette();
    let app_entity = cx.entity();
    let themes = app.available_themes();
    let active_name = app.selected_theme_name.clone();

    div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .bg(rgba(0x00000055))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .occlude()
                .w(px(560.))
                .max_h(px(560.))
                .py_4()
                .bg(palette.panel_bg)
                .border_1()
                .border_color(palette.border)
                .rounded_lg()
                .shadow_lg()
                .text_color(palette.text)
                .flex()
                .flex_col()
                // Title bar.
                .child(
                    div()
                        .px_4()
                        .pb_3()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(15.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(app.tr(Msg::PrefPanelTitle)),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .text_color(palette.muted)
                                .hover(|style| style.bg(palette.surface_bg))
                                .child("✕")
                                .on_mouse_up(
                                    MouseButton::Left,
                                    cx.listener(move |app, _: &MouseUpEvent, _window, cx| {
                                        app.close_preferences(cx);
                                    }),
                                ),
                        ),
                )
                // Scrollable body.
                .child(
                    div()
                        .id("preferences-panel-body")
                        .px_4()
                        .overflow_y_scroll()
                        .scrollbar_width(px(8.))
                        .flex()
                        .flex_col()
                        .gap_4()
                        // Language choices appear first so the rest of the
                        // panel immediately reflects the user's UI language.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child(app.tr(Msg::PrefPanelLanguageSection)),
                                )
                                .child(div().flex().gap_2().children(Language::all().iter().map(
                                    |&lang| {
                                        let is_active = app.language == lang;
                                        preference_option_button(
                                            format!(
                                                "{}  {}",
                                                if is_active { "✓" } else { " " },
                                                lang.native_name()
                                            ),
                                            is_active,
                                            palette,
                                            cx.listener(
                                                move |app, _: &MouseUpEvent, _window, cx| {
                                                    app.apply_language(lang, cx);
                                                },
                                            ),
                                        )
                                    },
                                ))),
                        )
                        // Theme grid.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child(app.tr(Msg::PrefPanelThemeSection)),
                                )
                                .child(div().flex().flex_wrap().gap_2().children(
                                    themes.iter().map(|theme| {
                                        let theme_name = theme.name.clone();
                                        let is_active =
                                            theme.name.eq_ignore_ascii_case(&active_name);
                                        let colors = theme.colors;
                                        let app_entity = app_entity.clone();
                                        let border = if is_active {
                                            palette.active_bg
                                        } else {
                                            palette.border
                                        };
                                        div()
                                            .w(px(120.))
                                            .p_2()
                                            .rounded_md()
                                            .border_1()
                                            .border_color(border)
                                            .bg(rgb(colors.panel_bg))
                                            .cursor_pointer()
                                            .hover(move |style| {
                                                style.border_color(palette.active_bg)
                                            })
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                                                app_entity.update(cx, |app, cx| {
                                                    app.apply_theme_by_name(&theme_name, cx);
                                                });
                                            })
                                            .child(
                                                div()
                                                    .h(px(28.))
                                                    .rounded_sm()
                                                    .flex()
                                                    .child(div().flex_1().bg(rgb(colors.app_bg)))
                                                    .child(
                                                        div().flex_1().bg(rgb(colors.surface_bg)),
                                                    )
                                                    .child(div().flex_1().bg(rgb(colors.active_bg)))
                                                    .child(
                                                        div().w(px(6.)).bg(rgb(colors.active_text)),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .gap_1()
                                                    .text_size(px(11.))
                                                    .child(
                                                        div()
                                                            .min_w_0()
                                                            .text_color(rgb(colors.text))
                                                            .child(theme.name.clone()),
                                                    )
                                                    .when(is_active, |row| {
                                                        row.child(
                                                            div()
                                                                .text_color(palette.active_bg)
                                                                .child("✓"),
                                                        )
                                                    }),
                                            )
                                    }),
                                )),
                        )
                        // Document typography.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child(app.tr(Msg::PrefPanelTypographySection)),
                                )
                                .child(preference_numeric_row(
                                    app.tr(Msg::PrefPanelEditorFontSize),
                                    app.editor_font_size,
                                    MIN_EDITOR_FONT_SIZE,
                                    MAX_EDITOR_FONT_SIZE,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        if let Some(value) = preference_step_value(
                                            app.editor_font_size,
                                            MIN_EDITOR_FONT_SIZE,
                                            MAX_EDITOR_FONT_SIZE,
                                            -1,
                                        ) {
                                            app.set_editor_font_size(value as i64, cx);
                                        }
                                    }),
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        if let Some(value) = preference_step_value(
                                            app.editor_font_size,
                                            MIN_EDITOR_FONT_SIZE,
                                            MAX_EDITOR_FONT_SIZE,
                                            1,
                                        ) {
                                            app.set_editor_font_size(value as i64, cx);
                                        }
                                    }),
                                ))
                                .child(preference_numeric_row(
                                    app.tr(Msg::PrefPanelRenderedFontSize),
                                    app.rendered_font_size,
                                    MIN_RENDERED_FONT_SIZE,
                                    MAX_RENDERED_FONT_SIZE,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        if let Some(value) = preference_step_value(
                                            app.rendered_font_size,
                                            MIN_RENDERED_FONT_SIZE,
                                            MAX_RENDERED_FONT_SIZE,
                                            -1,
                                        ) {
                                            app.set_rendered_font_size(value as i64, cx);
                                        }
                                    }),
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        if let Some(value) = preference_step_value(
                                            app.rendered_font_size,
                                            MIN_RENDERED_FONT_SIZE,
                                            MAX_RENDERED_FONT_SIZE,
                                            1,
                                        ) {
                                            app.set_rendered_font_size(value as i64, cx);
                                        }
                                    }),
                                ))
                                .child(preference_numeric_row(
                                    app.tr(Msg::PrefPanelParagraphSpacing),
                                    app.paragraph_spacing,
                                    MIN_PARAGRAPH_SPACING,
                                    MAX_PARAGRAPH_SPACING,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        if let Some(value) = preference_step_value(
                                            app.paragraph_spacing,
                                            MIN_PARAGRAPH_SPACING,
                                            MAX_PARAGRAPH_SPACING,
                                            -1,
                                        ) {
                                            app.set_paragraph_spacing(value as i64, cx);
                                        }
                                    }),
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        if let Some(value) = preference_step_value(
                                            app.paragraph_spacing,
                                            MIN_PARAGRAPH_SPACING,
                                            MAX_PARAGRAPH_SPACING,
                                            1,
                                        ) {
                                            app.set_paragraph_spacing(value as i64, cx);
                                        }
                                    }),
                                )),
                        )
                        // Other settings.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.muted)
                                        .child(app.tr(Msg::PrefPanelOtherSection)),
                                )
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelFocusMode),
                                    app.focus_mode,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                                        app.toggle_focus_mode(&ToggleFocusMode, window, cx);
                                    }),
                                ))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelTypewriterMode),
                                    app.typewriter_mode,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                                        app.toggle_typewriter_mode(
                                            &ToggleTypewriterMode,
                                            window,
                                            cx,
                                        );
                                    }),
                                ))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelCodeLineNumbers),
                                    app.code_line_numbers,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                                        app.toggle_code_line_numbers(
                                            &ToggleCodeLineNumbers,
                                            window,
                                            cx,
                                        );
                                    }),
                                ))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelPreviewAdaptiveWidth),
                                    app.preview_adaptive_width,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        app.toggle_preview_adaptive_width(cx);
                                    }),
                                ))
                                .child(preference_heading_menu_row(app, palette, cx))
                                .child(preference_boolean_row(
                                    app.tr(Msg::PrefPanelSyncScroll),
                                    app.sync_scroll,
                                    app.language,
                                    palette,
                                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                                        app.toggle_sync_scroll(cx);
                                    }),
                                ))
                                .child(preference_sidebar_row(app, palette, cx)),
                        ),
                ),
        )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn preference_numeric_row(
    label: &'static str,
    value: u16,
    min: u16,
    max: u16,
    palette: ThemePalette,
    decrement: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
    increment: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.))
        .px_1()
        .py_1()
        .gap_3()
        .child(div().text_color(palette.muted).child(label))
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(preference_numeric_button(
                    "−",
                    value > min,
                    palette,
                    decrement,
                ))
                .child(
                    div()
                        .min_w(px(54.))
                        .h(px(26.))
                        .px_2()
                        .rounded_sm()
                        .border_1()
                        .border_color(palette.border)
                        .bg(palette.surface_bg)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(palette.text)
                        .child(format!("{value} px")),
                )
                .child(preference_numeric_button(
                    "+",
                    value < max,
                    palette,
                    increment,
                )),
        )
}

pub(super) fn preference_step_value(value: u16, min: u16, max: u16, delta: i8) -> Option<u16> {
    let stepped = (value as i64 + delta as i64).clamp(min as i64, max as i64) as u16;
    (stepped != value).then_some(stepped)
}

pub(super) fn preference_numeric_button(
    label: &'static str,
    enabled: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    div()
        .w(px(28.))
        .h(px(26.))
        .rounded_sm()
        .border_1()
        .border_color(palette.border)
        .bg(if enabled {
            palette.surface_bg
        } else {
            palette.panel_bg
        })
        .text_color(if enabled { palette.text } else { palette.muted })
        .flex()
        .items_center()
        .justify_center()
        .child(label)
        .when(enabled, |button| {
            button
                .cursor_pointer()
                .hover(move |style| style.bg(palette.active_bg))
                .on_mouse_up(MouseButton::Left, listener)
        })
}

pub(super) fn preference_boolean_row(
    label: &'static str,
    enabled: bool,
    language: Language,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let value = t(language, if enabled { Msg::PrefOn } else { Msg::PrefOff }).to_string();

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.))
        .px_1()
        .py_1()
        .gap_3()
        .child(div().text_color(palette.muted).child(label))
        .child(preference_option_button(value, enabled, palette, listener))
}

pub(super) fn preference_heading_menu_row(
    app: &MarkionApp,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let extended = app.heading_menu_max_level >= EXTENDED_HEADING_MENU_MAX_LEVEL;

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.))
        .px_1()
        .py_1()
        .gap_3()
        .child(
            div()
                .text_color(palette.muted)
                .child(app.tr(Msg::PrefPanelHeadingMenu)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(preference_option_button(
                    app.tr(Msg::PrefPanelHeadingMenuThree).to_string(),
                    !extended,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.set_heading_menu_max_level(DEFAULT_HEADING_MENU_MAX_LEVEL, cx);
                    }),
                ))
                .child(preference_option_button(
                    app.tr(Msg::PrefPanelHeadingMenuSix).to_string(),
                    extended,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.set_heading_menu_max_level(EXTENDED_HEADING_MENU_MAX_LEVEL, cx);
                    }),
                )),
        )
}

pub(super) fn preference_sidebar_row(
    app: &MarkionApp,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let language = app.language;

    div()
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(12.))
        .px_1()
        .py_1()
        .gap_3()
        .child(
            div()
                .text_color(palette.muted)
                .child(app.tr(Msg::PrefPanelSidebar)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(preference_option_button(
                    t(
                        language,
                        if app.sidebar_visible {
                            Msg::PrefOn
                        } else {
                            Msg::PrefOff
                        },
                    )
                    .to_string(),
                    app.sidebar_visible,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, window, cx| {
                        app.toggle_sidebar(&ToggleSidebar, window, cx);
                    }),
                ))
                .child(preference_option_button(
                    sidebar_tab_label(language, SidebarTab::Files).to_string(),
                    app.sidebar_visible && app.sidebar_tab == SidebarTab::Files,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.select_preferences_sidebar_tab(SidebarTab::Files, cx);
                    }),
                ))
                .child(preference_option_button(
                    sidebar_tab_label(language, SidebarTab::Outline).to_string(),
                    app.sidebar_visible && app.sidebar_tab == SidebarTab::Outline,
                    palette,
                    cx.listener(|app, _: &MouseUpEvent, _window, cx| {
                        app.select_preferences_sidebar_tab(SidebarTab::Outline, cx);
                    }),
                )),
        )
}

pub(super) fn preference_option_button(
    label: String,
    active: bool,
    palette: ThemePalette,
    listener: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
) -> Div {
    let background = if active {
        palette.active_bg
    } else {
        palette.surface_bg
    };
    let foreground = if active {
        palette.active_text
    } else {
        palette.text
    };
    let border = if active {
        palette.active_bg
    } else {
        palette.border
    };

    div()
        .min_w(px(48.))
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(border)
        .bg(background)
        .text_color(foreground)
        .text_size(px(12.))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .hover(move |style| style.border_color(palette.active_bg))
        .on_mouse_up(MouseButton::Left, listener)
        .child(label)
}
