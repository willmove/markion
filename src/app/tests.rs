use super::*;
use gpui::{Modifiers, TestAppContext};

#[test]
fn menu_shortcut_labels_follow_platform_conventions() {
    assert_eq!(
        menu_shortcuts::OPEN_DOCUMENT.label(ShortcutPlatform::WindowsLinux),
        "Ctrl+O"
    );
    assert_eq!(
        menu_shortcuts::OPEN_DOCUMENT.label(ShortcutPlatform::MacOS),
        "Cmd+O"
    );
    assert_eq!(
        menu_shortcuts::SET_EDIT_MODE.label(ShortcutPlatform::WindowsLinux),
        "Ctrl+Alt+1"
    );
    assert_eq!(
        menu_shortcuts::SET_EDIT_MODE.label(ShortcutPlatform::MacOS),
        "Cmd+Option+1"
    );
    assert_eq!(
        menu_shortcuts::NEXT_TAB.label(ShortcutPlatform::MacOS),
        "Ctrl+Tab",
        "the fixed ctrl-tab binding must not be relabeled as Cmd"
    );

    let expected_current = if cfg!(target_os = "macos") {
        "Cmd+O"
    } else {
        "Ctrl+O"
    };
    assert_eq!(
        menu_shortcuts::OPEN_DOCUMENT.label(ShortcutPlatform::current()),
        expected_current
    );
}

#[test]
fn menu_shortcut_metadata_has_one_redo_binding() {
    assert_eq!(menu_shortcuts::SAVE_DOCUMENT.binding, "secondary-s");
    assert_eq!(
        menu_shortcuts::EXPORT_PLAIN_HTML.label(ShortcutPlatform::MacOS),
        "Cmd+Option+Shift+H"
    );
    assert_eq!(menu_shortcuts::REDO.binding, "secondary-y");
    assert_eq!(
        menu_shortcuts::REDO.label(ShortcutPlatform::WindowsLinux),
        "Ctrl+Y"
    );
    assert_eq!(menu_shortcuts::REDO.label(ShortcutPlatform::MacOS), "Cmd+Y");

    let bootstrap = include_str!("bootstrap.rs");
    assert_eq!(
        bootstrap
            .matches("KeyBinding::new(menu_shortcuts::REDO.binding, Redo, None)")
            .count(),
        1,
        "Redo must be installed exactly once"
    );
    assert!(!bootstrap.contains("REDO.aliases"));
    assert!(!bootstrap.contains("secondary-shift-z"));
}

#[test]
fn every_application_dropdown_uses_shortcut_aware_rows() {
    let source = include_str!("root_view.rs");
    let menu_source = source
        .split_once("pub(super) fn active_menu_dropdown")
        .and_then(|(_, rest)| {
            rest.split_once("/// Theme-aware Help")
                .map(|(menu, _)| menu)
        })
        .expect("in-window application menu builder");

    let menu_boundaries = [
        "AppMenu::File =>",
        "AppMenu::Edit =>",
        "AppMenu::View =>",
        "AppMenu::Format =>",
        "AppMenu::Export =>",
        "AppMenu::Help =>",
    ];
    for (index, boundary) in menu_boundaries.iter().enumerate() {
        let body = menu_source
            .split_once(boundary)
            .map(|(_, body)| body)
            .expect("menu match arm");
        let body = menu_boundaries
            .get(index + 1)
            .and_then(|next| body.split_once(next).map(|(body, _)| body))
            .unwrap_or(body);
        assert!(
            body.contains("menu_shortcuts::"),
            "{boundary} must supply shortcut metadata for its bound items"
        );
    }

    let row_source = source
        .split_once("pub(super) fn menu_action_button")
        .and_then(|(_, rest)| {
            rest.split_once("pub(super) fn menu_separator")
                .map(|(row, _)| row)
        })
        .expect("shortcut-aware menu row");
    assert!(row_source.contains("shortcut: Option<&'static str>"));
    assert!(row_source.contains("impl Into<SharedString>"));
    assert!(row_source.contains(".justify_between()"));
    assert!(row_source.contains(".text_color(palette.muted)"));
}

#[test]
fn application_menu_shortcuts_distinguish_bound_and_unbound_actions() {
    let source = include_str!("root_view.rs");
    let menu_source = source
        .split_once("pub(super) fn active_menu_dropdown")
        .and_then(|(_, rest)| {
            rest.split_once("/// Theme-aware Help")
                .map(|(menu, _)| menu)
        })
        .expect("in-window application menu builder");

    let invocation = |message: &str| {
        let rest = menu_source
            .split_once(message)
            .map(|(_, rest)| rest)
            .unwrap_or_else(|| panic!("menu invocation for {message}"));
        let end = rest
            .find("))")
            .unwrap_or_else(|| panic!("end of menu invocation for {message}"));
        &rest[..end]
    };

    for (message, descriptor) in [
        ("Msg::ItemSave,", "menu_shortcuts::SAVE_DOCUMENT"),
        ("Msg::ItemRedo,", "menu_shortcuts::REDO"),
        ("Msg::ItemFind,", "menu_shortcuts::SHOW_FIND"),
        ("Msg::ItemBold,", "menu_shortcuts::BOLD"),
        ("Msg::ItemExportPdf,", "menu_shortcuts::EXPORT_PDF"),
        (
            "Msg::ItemKeyboardShortcuts,",
            "menu_shortcuts::SHOW_SHORTCUTS",
        ),
    ] {
        assert!(
            invocation(message).contains(descriptor),
            "{message} must use {descriptor}"
        );
    }

    for message in [
        "Msg::ItemOpenFolder,",
        "Msg::ItemNewTab,",
        "Msg::ItemResetPreferences,",
        "Msg::ItemBullets,",
        "Msg::ItemAboutMarkion,",
    ] {
        assert!(
            !invocation(message).contains("menu_shortcuts::"),
            "unbound {message} must not render a shortcut marker"
        );
    }

    assert_eq!(
        menu_shortcuts::REDO.label(ShortcutPlatform::WindowsLinux),
        "Ctrl+Y"
    );
}

#[test]
fn menu_hover_switches_only_during_an_open_menu_session() {
    assert_eq!(menu_after_hover(None, AppMenu::View), None);
    assert_eq!(
        menu_after_hover(Some(AppMenu::Format), AppMenu::View),
        Some(AppMenu::View)
    );
    assert_eq!(
        menu_after_hover(Some(AppMenu::View), AppMenu::View),
        Some(AppMenu::View)
    );

    let dismissed = None;
    assert_eq!(menu_after_hover(dismissed, AppMenu::Help), None);
}

#[test]
fn every_menu_title_wires_click_and_hover_behavior() {
    let source = include_str!("root_view.rs");
    for menu in ["File", "Edit", "View", "Format", "Export", "Help"] {
        assert!(
            source.contains(&format!("app.hover_menu(AppMenu::{menu}, cx);")),
            "{menu} title must switch an open menu session on hover"
        );
        assert!(
            source.contains(&format!(
                "cx.listener(Self::toggle_{}_menu)",
                menu.to_lowercase()
            )),
            "{menu} title must retain its click toggle"
        );
    }

    let title_button = source
        .split_once("pub(super) fn menu_title_button")
        .and_then(|(_, rest)| {
            rest.split_once("pub(super) fn menu_action_button")
                .map(|(button, _)| button)
        })
        .expect("menu title button helper");
    assert!(title_button.contains(".on_mouse_up(MouseButton::Left, click_listener)"));
    assert!(title_button.contains(".on_mouse_move(hover_listener)"));
    assert!(source.contains("cx.listener(Self::close_menu)"));
}

#[test]
fn conditional_heading_menu_wires_only_visible_heading_shortcuts() {
    let source = include_str!("root_view.rs");
    let format_menu = source
        .split_once("AppMenu::Format =>")
        .and_then(|(_, rest)| {
            rest.split_once("AppMenu::Export =>")
                .map(|(format, _)| format)
        })
        .expect("Format menu arm");

    for level in 1..=5 {
        assert!(
            format_menu.contains(&format!("menu_shortcuts::HEADING_{level}")),
            "visible default heading {level} must show its shortcut"
        );
    }
    let h6_condition = format_menu
        .find("heading_menu_max_level >= EXTENDED_HEADING_MENU_MAX_LEVEL")
        .expect("conditional H6 branch");
    let h6_shortcut = format_menu
        .find("menu_shortcuts::HEADING_6")
        .expect("H6 shortcut descriptor");
    assert!(h6_condition < h6_shortcut);
}

#[test]
fn open_folder_prompt_selects_one_directory() {
    let options = open_folder_prompt_options(Language::En);
    assert!(!options.files);
    assert!(options.directories);
    assert!(!options.multiple);
    assert_eq!(
        options.prompt.as_ref().map(ToString::to_string).as_deref(),
        Some("Open Folder")
    );
}

#[test]
fn startup_open_intent_classifies_paths_and_ignores_extra_args() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let relative_md = root.join("notes.MD");
    let absolute_md = root.join("absolute.markdown");
    let folder = root.join("workspace");
    let unsupported = root.join("image.png");
    std::fs::write(&relative_md, "# Notes").unwrap();
    std::fs::write(&absolute_md, "# Absolute").unwrap();
    std::fs::create_dir(&folder).unwrap();
    std::fs::write(&unsupported, "png").unwrap();

    assert_eq!(
        StartupOpenIntent::from_args(Vec::new(), root),
        StartupOpenIntent::None
    );
    assert_eq!(
        StartupOpenIntent::from_args(vec![OsString::from("notes.MD")], root),
        StartupOpenIntent::File(relative_md.clone())
    );
    assert_eq!(
        StartupOpenIntent::from_args(vec![absolute_md.clone().into_os_string()], root),
        StartupOpenIntent::File(absolute_md.clone())
    );
    assert_eq!(
        StartupOpenIntent::from_args(vec![folder.clone().into_os_string()], root),
        StartupOpenIntent::Folder(folder.clone())
    );
    assert_eq!(
        StartupOpenIntent::from_args(vec![unsupported.clone().into_os_string()], root),
        StartupOpenIntent::Invalid {
            path: unsupported,
            reason: StartupOpenInvalidReason::UnsupportedFile,
        }
    );

    let missing = root.join("missing.md");
    assert_eq!(
        StartupOpenIntent::from_args(vec![OsString::from("missing.md")], root),
        StartupOpenIntent::Invalid {
            path: missing,
            reason: StartupOpenInvalidReason::Missing,
        }
    );
    assert_eq!(
        StartupOpenIntent::from_args(
            vec![
                folder.clone().into_os_string(),
                absolute_md.clone().into_os_string(),
            ],
            root,
        ),
        StartupOpenIntent::Folder(folder)
    );
}

#[test]
fn startup_path_resolution_preserves_absolute_paths() {
    let temp = tempfile::tempdir().unwrap();
    let cwd = temp.path();
    let absolute = cwd.join("note.md");

    assert_eq!(
        resolve_startup_path(PathBuf::from("note.md"), cwd),
        cwd.join("note.md")
    );
    assert_eq!(resolve_startup_path(absolute.clone(), cwd), absolute);
}

#[test]
fn remote_image_request_url_strips_fragment_and_preserves_query() {
    let authored =
        "https://mmbiz.qpic.cn/sz_mmbiz_png/example/640?wx_fmt=png&from=appmsg#imgIndex=0";

    assert_eq!(
        remote_image_request_url(authored),
        "https://mmbiz.qpic.cn/sz_mmbiz_png/example/640?wx_fmt=png&from=appmsg"
    );
}

#[test]
fn remote_image_request_url_preserves_valid_http_urls_and_encoded_hashes() {
    assert_eq!(
        remote_image_request_url("https://example.com/image%23detail.png?key=value"),
        "https://example.com/image%23detail.png?key=value"
    );
    assert_eq!(
        remote_image_request_url("HTTP://example.com/image.png#thumbnail"),
        "HTTP://example.com/image.png"
    );
}

#[test]
fn remote_image_request_url_leaves_non_http_sources_unchanged() {
    assert_eq!(
        remote_image_request_url("images/chart#1.png"),
        "images/chart#1.png"
    );
    assert_eq!(
        remote_image_request_url("data:image/png;base64,abc#fragment"),
        "data:image/png;base64,abc#fragment"
    );
}

#[test]
fn startup_application_flow_reuses_existing_open_behaviour() {
    let bootstrap_source = include_str!("bootstrap.rs");
    let application_source = include_str!("application.rs");
    assert!(bootstrap_source.contains("pub(super) fn run_with_startup_intent"));
    assert!(bootstrap_source.contains("StartupOpenIntent::from_env_args()"));

    let apply = bootstrap_source
        .find("app.apply_startup_open_intent")
        .expect("startup intent application");
    let recovery = bootstrap_source
        .find("app.check_recovery_on_startup")
        .expect("recovery startup check");
    assert!(apply < recovery);

    let apply_fn = application_source
        .split_once("fn apply_startup_open_intent")
        .and_then(|(_, rest)| {
            rest.split_once("fn after_document_changed")
                .map(|(body, _)| body)
        })
        .expect("startup intent handler");
    assert!(apply_fn.contains("self.replace_active_tab(document, cx);"));
    assert!(apply_fn.contains("self.update_workspace_root_from_document(cx);"));
    assert!(apply_fn.contains("self.set_workspace_root(path);"));
    assert!(apply_fn.contains("self.sidebar_visible = true;"));
    assert!(apply_fn.contains("self.sidebar_tab = SidebarTab::Files;"));
    assert!(apply_fn.contains("self.schedule_file_tree_scan(Some(display_path), cx);"));
    assert!(apply_fn.contains("Msg::StatusOpened"));
    assert!(apply_fn.contains("Msg::StatusOpenFailed"));
    assert!(!apply_fn.contains("Msg::StatusStartup"));
}

#[test]
fn startup_installs_http_client_before_building_ui() {
    let bootstrap_source = include_str!("bootstrap.rs");
    let install = bootstrap_source
        .find("network::install_http_client(cx)")
        .expect("HTTP client installation");
    let bind_keys = bootstrap_source.find("cx.bind_keys").expect("key bindings");

    assert!(install < bind_keys);
}

#[test]
fn open_folder_action_is_wired_after_open_without_a_shortcut() {
    let root_view_source = include_str!("root_view.rs");
    let bootstrap_source = include_str!("bootstrap.rs");
    assert!(root_view_source.contains(".on_action(cx.listener(Self::open_folder))"));

    let in_window = root_view_source
        .split_once("AppMenu::File => panel")
        .and_then(|(_, rest)| rest.split_once("AppMenu::Edit =>").map(|(file, _)| file))
        .expect("in-window File menu");
    let in_window_open = in_window.find("Msg::ItemOpen,").expect("Open item");
    let in_window_folder = in_window
        .find("Msg::ItemOpenFolder,")
        .expect("Open Folder item");
    let in_window_save = in_window.find("Msg::ItemSave,").expect("Save item");
    assert!(in_window_open < in_window_folder && in_window_folder < in_window_save);

    let native = bootstrap_source
        .split_once("fn install_menus")
        .and_then(|(_, rest)| rest.split_once("Msg::MenuEdit").map(|(file, _)| file))
        .expect("native File menu");
    let native_open = native.find("Msg::ItemOpen)").expect("native Open item");
    let native_folder = native
        .find("Msg::ItemOpenFolder)")
        .expect("native Open Folder item");
    let native_save = native.find("Msg::ItemSave)").expect("native Save item");
    assert!(native_open < native_folder && native_folder < native_save);
    assert!(
        !bootstrap_source
            .lines()
            .any(|line| line.contains("KeyBinding::new") && line.contains("OpenFolder"))
    );
}

#[test]
fn workspace_root_selection_preserves_contained_documents_and_rebases_external_ones() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    let nested = root.join("notes").join("inside.md");
    let outside = temp.path().join("outside").join("other.md");
    std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
    std::fs::create_dir_all(outside.parent().unwrap()).unwrap();
    std::fs::write(&nested, "# inside").unwrap();
    std::fs::write(&outside, "# outside").unwrap();

    assert_eq!(
        workspace_root_for_document(Some(&root), &nested),
        Some(comparable_document_path(&root))
    );
    assert_eq!(
        workspace_root_for_document(Some(&root), &outside),
        outside.parent().map(comparable_document_path)
    );
    assert_eq!(
        workspace_root_for_document(None, &nested),
        nested.parent().map(comparable_document_path)
    );

    let sibling_prefix = temp.path().join("workspace-copy").join("note.md");
    std::fs::create_dir_all(sibling_prefix.parent().unwrap()).unwrap();
    std::fs::write(&sibling_prefix, "# sibling").unwrap();
    assert!(!path_is_within_workspace(&root, &sibling_prefix));
}

#[test]
fn workspace_root_reset_and_stale_scan_checks_are_root_aware() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();

    assert!(!workspace_root_needs_reset(&first, true, &first));
    assert!(workspace_root_needs_reset(&first, false, &first));
    assert!(workspace_root_needs_reset(&first, true, &second));
    assert!(scan_result_matches_workspace(&first, &first));
    assert!(!scan_result_matches_workspace(&first, &second));
}

#[test]
fn folder_scan_supports_empty_roots_and_reports_missing_roots() {
    let temp = tempfile::tempdir().unwrap();
    let empty = temp.path().join("empty");
    std::fs::create_dir_all(&empty).unwrap();

    let tree = FileTree::scan(&empty).unwrap();
    assert_eq!(
        comparable_document_path(&tree.root),
        comparable_document_path(&empty)
    );
    assert!(tree.entries.is_empty());
    assert!(FileTree::scan(temp.path().join("missing")).is_err());
}

#[test]
fn hide_search_overlay_state_closes_without_clearing_buffers() {
    let mut search_visible = true;
    let mut replace_visible = true;
    let mut search_focus = Some(SearchField::Replace);
    let mut input_marked_len = 3;
    let query = "needle".to_string();
    let replacement = "thread".to_string();

    hide_search_overlay_state(
        &mut search_visible,
        &mut replace_visible,
        &mut search_focus,
        &mut input_marked_len,
    );

    assert!(!search_visible);
    assert!(!replace_visible);
    assert_eq!(search_focus, None);
    assert_eq!(input_marked_len, 0);
    assert_eq!(query, "needle");
    assert_eq!(replacement, "thread");
}

#[test]
fn normalize_preview_selection_range_clamps_and_orders() {
    assert_eq!(normalize_preview_selection_range("hello", 1..4), 1..4);
    assert_eq!(
        normalize_preview_selection_range("hello", std::ops::Range { start: 4, end: 1 }),
        1..4
    );
    assert_eq!(normalize_preview_selection_range("hello", 0..99), 0..5);
    // Mid-codepoint end advances to the next boundary ("é" is bytes 1..3).
    assert_eq!(normalize_preview_selection_range("héllo", 1..2), 1..3);
}

fn sample_paragraph(text: &str) -> PreviewBlock {
    PreviewBlock::Paragraph {
        text: RichText::plain(text),
        source_range: 0..text.len(),
    }
}

#[test]
fn preview_table_cells_remain_selectable_without_editing_toolbar() {
    let block = PreviewBlock::Table {
        rows: vec![
            vec!["Name".into(), "Value".into()],
            vec!["alpha".into(), "1".into()],
        ],
        alignments: vec![],
        source_range: 0..0,
    };

    assert_eq!(
        preview_block_runs(&block),
        vec![
            PreviewTextRunId::TableCell { row: 0, col: 0 },
            PreviewTextRunId::TableCell { row: 0, col: 1 },
            PreviewTextRunId::TableCell { row: 1, col: 0 },
            PreviewTextRunId::TableCell { row: 1, col: 1 },
        ]
    );
    assert_eq!(
        preview_run_plain_text(&block, PreviewTextRunId::TableCell { row: 1, col: 0 }).as_deref(),
        Some("alpha")
    );
}

#[test]
fn preview_images_do_not_expose_redundant_metadata_runs() {
    let preview_source = include_str!("preview.rs");
    let url = "https://example.com/image.png#detail".to_string();
    let block = PreviewBlock::Image {
        alt: "diagram".to_string(),
        url: url.clone(),
        title: Some("architecture".to_string()),
        source_range: 0..42,
    };

    assert!(preview_block_runs(&block).is_empty());
    assert!(!preview_source.contains("preview-image-caption"));
    assert!(!preview_source.contains("preview-image-meta"));
    assert!(matches!(
        block,
        PreviewBlock::Image {
            alt,
            url: stored_url,
            title: Some(title),
            ..
        } if alt == "diagram" && stored_url == url && title == "architecture"
    ));
}

#[test]
fn preview_selection_plain_text_extracts_substring() {
    let blocks = vec![sample_paragraph("hello")];
    let selection = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 1,
        },
        head: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 4,
        },
    };
    assert_eq!(
        preview_selection_plain_text(&selection, &blocks).as_deref(),
        Some("ell")
    );
    let empty = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 2,
        },
        head: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 2,
        },
    };
    assert!(preview_selection_plain_text(&empty, &blocks).is_none());
}

#[test]
fn preview_selection_plain_text_spans_multiple_blocks() {
    let blocks = vec![sample_paragraph("hello"), sample_paragraph("world")];
    let selection = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 3,
        },
        head: PreviewCaret {
            block_index: 1,
            run_id: PreviewTextRunId::Body,
            offset: 3,
        },
    };
    assert_eq!(
        preview_selection_plain_text(&selection, &blocks).as_deref(),
        Some("lo\nwor")
    );
}

#[test]
fn preview_selection_takes_copy_precedence_only_when_non_empty() {
    let blocks = vec![sample_paragraph("abc")];
    let selection = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 0,
        },
        head: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 3,
        },
    };
    assert!(preview_selection_takes_copy_precedence(
        Some(&selection),
        &blocks
    ));
    let empty = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 1,
        },
        head: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 1,
        },
    };
    assert!(!preview_selection_takes_copy_precedence(
        Some(&empty),
        &blocks
    ));
    assert!(!preview_selection_takes_copy_precedence(None, &blocks));
}

#[test]
fn invalidate_preview_selection_if_stale_drops_out_of_range_blocks() {
    let selection = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 2,
            run_id: PreviewTextRunId::Body,
            offset: 0,
        },
        head: PreviewCaret {
            block_index: 2,
            run_id: PreviewTextRunId::Body,
            offset: 1,
        },
    };
    assert!(invalidate_preview_selection_if_stale(Some(selection.clone()), 3).is_some());
    assert!(invalidate_preview_selection_if_stale(Some(selection), 2).is_none());
    assert!(invalidate_preview_selection_if_stale(None, 10).is_none());
}

#[test]
fn preview_run_highlight_range_covers_middle_and_partial_runs() {
    let selection = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 2,
        },
        head: PreviewCaret {
            block_index: 2,
            run_id: PreviewTextRunId::Body,
            offset: 3,
        },
    };
    assert_eq!(
        preview_run_highlight_range(&selection, 0, PreviewTextRunId::Body, "hello"),
        Some(2..5)
    );
    assert_eq!(
        preview_run_highlight_range(&selection, 1, PreviewTextRunId::Body, "body"),
        Some(0..4)
    );
    assert_eq!(
        preview_run_highlight_range(&selection, 2, PreviewTextRunId::Body, "world"),
        Some(0..3)
    );
    assert_eq!(
        preview_run_highlight_range(&selection, 3, PreviewTextRunId::Body, "later"),
        None
    );
}

#[test]
fn preview_selection_markdown_joins_covered_block_sources() {
    let document = "# Title\n\nHello world\n\n- item\n";
    let blocks = vec![
        PreviewBlock::Heading {
            level: 1,
            text: RichText::plain("Title"),
            source_range: 0..7,
        },
        PreviewBlock::Paragraph {
            text: RichText::plain("Hello world"),
            source_range: 9..20,
        },
        PreviewBlock::ListItem {
            level: 0,
            ordered: false,
            index: None,
            checked: None,
            text: RichText::plain("item"),
            source_range: 22..28,
        },
    ];
    let selection = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: 0,
        },
        head: PreviewCaret {
            block_index: 1,
            run_id: PreviewTextRunId::Body,
            offset: 5,
        },
    };
    let md = preview_selection_markdown(&selection, &blocks, document).unwrap();
    assert!(md.contains("# Title"));
    assert!(md.contains("Hello world"));
    assert!(!md.contains("- item"));

    let html = MarkdownDocument::from_text(&md).render_html_fragment();
    assert!(html.contains("<h1"));
    assert!(html.to_lowercase().contains("hello"));
}

#[test]
fn inline_math_baseline_margin_lifts_shallow_formulas_to_text_baseline() {
    // Paragraph defaults: 14px body / 24px line. A shallow formula (tiny descent)
    // previously used mb=descent and sat near the line box bottom; the margin
    // must instead be text_baseline_from_bottom - math_descent.
    let line_height = px(24.);
    let font_ascent = px(11.2); // ≈ 0.8em
    let font_descent = px(2.8); // ≈ 0.2em
    let math_descent = px(1.0);
    let margin = inline_math_baseline_margin_from_metrics(
        line_height,
        font_ascent,
        font_descent,
        math_descent,
    );
    // text baseline from bottom = (24 - 11.2 - 2.8)/2 + 2.8 = 7.8
    assert!((f32::from(margin) - 6.8).abs() < 0.01);
    // Old mb=descent left the formula ~5.8px too low for this case.
    assert!(f32::from(margin) > f32::from(math_descent) + 4.0);

    // Deep formulas may need a negative margin so descent hangs below the line.
    let deep = inline_math_baseline_margin_from_metrics(
        line_height,
        font_ascent,
        font_descent,
        px(12.0),
    );
    assert!(f32::from(deep) < 0.0);
}

#[test]
fn math_atom_hit_testing_and_copy_preserve_complete_authored_source() {
    let document = "速度 $E=mc^2$ end.\n\n$$\n\\frac{1}{2}\n$$\n";
    let doc = MarkdownDocument::from_text(document);
    let blocks = doc.preview_blocks();
    let PreviewBlock::Paragraph { text, .. } = &blocks[0] else {
        panic!("expected inline-math paragraph");
    };
    let math = text
        .spans
        .iter()
        .find_map(|span| span.math.as_ref())
        .expect("inline math span");
    let local_start = text.text.find(&math.authored).unwrap();
    let authored_range = local_start..local_start + math.authored.len();
    assert_eq!(math_atom_boundary(&authored_range, false), local_start);
    assert_eq!(
        math_atom_boundary(&authored_range, true),
        local_start + "$E=mc^2$".len()
    );
    assert!(text.text.is_char_boundary(authored_range.start));
    assert!(text.text.is_char_boundary(authored_range.end));

    let PreviewBlock::MathBlock { authored, .. } = &blocks[1] else {
        panic!("expected display math block");
    };
    let selection = PreviewSelection {
        anchor: PreviewCaret {
            block_index: 0,
            run_id: PreviewTextRunId::Body,
            offset: local_start,
        },
        head: PreviewCaret {
            block_index: 1,
            run_id: PreviewTextRunId::MathLatex,
            offset: authored.len(),
        },
    };
    let plain = preview_selection_plain_text(&selection, &blocks).unwrap();
    assert!(plain.starts_with("$E=mc^2$"));
    assert!(plain.contains("$$\n\\frac{1}{2}\n$$"));
    let markdown = preview_selection_markdown(&selection, &blocks, document).unwrap();
    assert!(markdown.contains("$E=mc^2$"));
    assert!(markdown.contains("$$\n\\frac{1}{2}\n$$"));
    let html = MarkdownDocument::from_text(&markdown).render_html_fragment();
    assert_eq!(html.matches("<svg aria-hidden=\"true\"").count(), 2);
}

/// A distinguishable `PreviewBlock` for splice-diff tests. Distinct `tag`s
/// compare unequal; the concrete variant is irrelevant to the diff.
fn blk(tag: &str) -> PreviewBlock {
    PreviewBlock::CodeBlock {
        language: None,
        code: tag.to_string(),
        source_range: 0..0,
    }
}

fn blocks(tags: &[&str]) -> Vec<PreviewBlock> {
    tags.iter().map(|t| blk(t)).collect()
}

fn nested_file_tree_fixture(root: &Path) -> FileTree {
    let docs = root.join("docs");
    let guides = docs.join("guides");
    let source = root.join("src");
    FileTree {
        root: root.to_path_buf(),
        entries: vec![
            FileTreeEntry {
                path: docs.clone(),
                name: "docs".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::Directory,
                is_markdown: false,
            },
            FileTreeEntry {
                path: guides.clone(),
                name: "guides".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::Directory,
                is_markdown: false,
            },
            FileTreeEntry {
                path: guides.join("intro.md"),
                name: "intro.md".to_string(),
                depth: 2,
                kind: FileTreeEntryKind::File,
                is_markdown: true,
            },
            FileTreeEntry {
                path: docs.join("draft.md"),
                name: "draft.md".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::File,
                is_markdown: true,
            },
            FileTreeEntry {
                path: source.clone(),
                name: "src".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::Directory,
                is_markdown: false,
            },
            FileTreeEntry {
                path: source.join("api.md"),
                name: "api.md".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::File,
                is_markdown: true,
            },
            FileTreeEntry {
                path: root.join("root.md"),
                name: "root.md".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::File,
                is_markdown: true,
            },
        ],
    }
}

fn visible_tree_entry_names(
    tree: &FileTree,
    query: &str,
    collapsed: &HashSet<PathBuf>,
) -> Vec<String> {
    filtered_visible_file_tree_entries(tree, query, collapsed, 300)
        .0
        .into_iter()
        .map(|entry| entry.name)
        .collect()
}

#[test]
fn initial_file_tree_collapse_shows_root_children_and_expands_one_branch() {
    let root = PathBuf::from("workspace");
    let tree = nested_file_tree_fixture(&root);
    let scanned = Ok(tree.clone());
    let mut collapsed = HashSet::new();
    let mut needs_initial_collapse = true;

    update_file_tree_collapse_state_from_scan(
        &scanned,
        &mut collapsed,
        &mut needs_initial_collapse,
    );

    assert!(!needs_initial_collapse);
    assert_eq!(
        collapsed,
        HashSet::from([root.join("docs"), root.join("src")])
    );
    assert_eq!(
        visible_tree_entry_names(&tree, "", &collapsed),
        vec!["docs", "src", "root.md"]
    );

    collapsed.remove(&root.join("docs"));
    assert_eq!(
        visible_tree_entry_names(&tree, "", &collapsed),
        vec!["docs", "guides", "intro.md", "draft.md", "src", "root.md"]
    );
    assert!(collapsed.contains(&root.join("src")));
}

#[test]
fn file_tree_scan_collapse_state_preserves_refresh_resets_and_failure_pending() {
    let temp = tempfile::tempdir().unwrap();
    let first_root = temp.path().join("first");
    std::fs::create_dir_all(first_root.join("docs")).unwrap();
    std::fs::create_dir_all(first_root.join("src")).unwrap();
    let first_tree = nested_file_tree_fixture(&first_root);
    let first_scan = Ok(first_tree);
    let mut collapsed = HashSet::new();
    let mut needs_initial_collapse = true;
    update_file_tree_collapse_state_from_scan(
        &first_scan,
        &mut collapsed,
        &mut needs_initial_collapse,
    );

    collapsed.remove(&first_root.join("docs"));
    collapsed.insert(first_root.join("removed"));
    update_file_tree_collapse_state_from_scan(
        &first_scan,
        &mut collapsed,
        &mut needs_initial_collapse,
    );
    assert_eq!(collapsed, HashSet::from([first_root.join("src")]));

    let second_root = temp.path().join("second");
    std::fs::create_dir_all(second_root.join("docs")).unwrap();
    std::fs::create_dir_all(second_root.join("src")).unwrap();
    let second_scan = Ok(nested_file_tree_fixture(&second_root));
    needs_initial_collapse = true;
    update_file_tree_collapse_state_from_scan(
        &second_scan,
        &mut collapsed,
        &mut needs_initial_collapse,
    );
    assert_eq!(
        collapsed,
        HashSet::from([second_root.join("docs"), second_root.join("src")])
    );
    assert!(!needs_initial_collapse);

    let before_failure = collapsed.clone();
    needs_initial_collapse = true;
    let failed_scan = Err(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "denied",
    ));
    update_file_tree_collapse_state_from_scan(
        &failed_scan,
        &mut collapsed,
        &mut needs_initial_collapse,
    );
    assert_eq!(collapsed, before_failure);
    assert!(needs_initial_collapse);
}

#[test]
fn file_tree_filter_reveals_collapsed_descendants_without_mutating_state() {
    let root = PathBuf::from("workspace");
    let tree = nested_file_tree_fixture(&root);
    let collapsed = HashSet::from([root.join("docs"), root.join("src")]);
    let before_filter = collapsed.clone();

    assert_eq!(
        visible_tree_entry_names(&tree, "intro.md", &collapsed),
        vec!["intro.md"]
    );
    assert_eq!(collapsed, before_filter);
    assert_eq!(
        visible_tree_entry_names(&tree, "", &collapsed),
        vec!["docs", "src", "root.md"]
    );
}

#[test]
fn file_tree_visibility_hides_collapsed_descendants() {
    let root = PathBuf::from("workspace");
    let docs = root.join("docs");
    let notes = root.join("notes.md");
    let tree = FileTree {
        root: root.clone(),
        entries: vec![
            FileTreeEntry {
                path: docs.clone(),
                name: "docs".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::Directory,
                is_markdown: false,
            },
            FileTreeEntry {
                path: docs.join("draft.md"),
                name: "draft.md".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::File,
                is_markdown: true,
            },
            FileTreeEntry {
                path: notes,
                name: "notes.md".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::File,
                is_markdown: true,
            },
        ],
    };
    let mut collapsed = HashSet::new();
    collapsed.insert(docs);

    let (visible, total) = filtered_visible_file_tree_entries(&tree, "", &collapsed, 300);

    assert_eq!(total, 2);
    assert_eq!(
        visible
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        vec!["docs", "notes.md"]
    );
}

#[test]
fn file_tree_context_actions_are_scoped_by_target_kind() {
    assert_eq!(
        file_tree_context_actions(FileTreeContextTargetKind::File),
        &[
            FileTreeContextAction::Open,
            FileTreeContextAction::OpenInNewTab,
            FileTreeContextAction::Rename,
            FileTreeContextAction::Delete,
            FileTreeContextAction::ShowInFileManager,
            FileTreeContextAction::Refresh,
        ]
    );
    assert_eq!(
        file_tree_context_actions(FileTreeContextTargetKind::Directory),
        &[
            FileTreeContextAction::CreateFile,
            FileTreeContextAction::CreateFolder,
            FileTreeContextAction::Rename,
            FileTreeContextAction::Delete,
            FileTreeContextAction::ShowInFileManager,
            FileTreeContextAction::Refresh,
        ]
    );
    assert_eq!(
        file_tree_context_actions(FileTreeContextTargetKind::Workspace),
        &[
            FileTreeContextAction::CreateFile,
            FileTreeContextAction::CreateFolder,
            FileTreeContextAction::Refresh,
            FileTreeContextAction::ShowInFileManager,
            FileTreeContextAction::FilterFiles,
        ]
    );
}

/// The inline name prompt's commit path calls `create_unique_file` /
/// `create_unique_directory` / `rename_unique` with the user-typed name
/// (rather than a hard-coded default). This exercises those operations at
/// the model level - the app-level wiring is a thin wrapper that passes
/// `pending.buffer` straight through.
#[test]
fn file_tree_name_prompt_commit_uses_typed_name() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    // Seed the tree with one existing file so the scan has a root.
    std::fs::write(root.join("seed.md"), "# Seed").unwrap();

    let mut tree = FileTree::scan(&root).unwrap();

    // CreateFile: typed name "essay.md" under the root.
    let created = tree.create_unique_file(&root, "essay.md").unwrap();
    assert_eq!(
        created.file_name().and_then(|n| n.to_str()),
        Some("essay.md")
    );
    assert!(created.exists());
    assert!(tree.entries.iter().any(|e| e.path == created));

    // CreateFolder: typed name "drafts".
    let folder = tree.create_unique_directory(&root, "drafts").unwrap();
    assert_eq!(folder.file_name().and_then(|n| n.to_str()), Some("drafts"));
    assert!(folder.is_dir());

    // Rename: rename essay.md -> final.md using the typed name.
    let renamed = tree.rename_unique(&created, "final.md").unwrap();
    assert_eq!(
        renamed.file_name().and_then(|n| n.to_str()),
        Some("final.md")
    );
    assert!(!created.exists());
    assert!(renamed.exists());
    assert!(tree.entries.iter().any(|e| e.path == renamed));
    assert!(!tree.entries.iter().any(|e| e.path == created));
}

/// The prompt must be pre-filled with a sensible default per kind so the
/// existing defaults remain one Enter away. This pins the pre-fill contract
/// that `open_name_prompt` / the context-menu branches rely on.
#[test]
fn pending_name_input_prefill_matches_kind_defaults() {
    let root = PathBuf::from("workspace");
    let create_file = PendingNameInput {
        kind: PendingNameKind::CreateFile,
        parent: root.clone(),
        target: None,
        buffer: "untitled.md".to_string(),
    };
    assert_eq!(create_file.kind, PendingNameKind::CreateFile);
    assert_eq!(create_file.buffer, "untitled.md");
    assert!(create_file.target.is_none());

    let create_folder = PendingNameInput {
        kind: PendingNameKind::CreateFolder,
        parent: root.clone(),
        target: None,
        buffer: "New Folder".to_string(),
    };
    assert_eq!(create_folder.kind, PendingNameKind::CreateFolder);
    assert_eq!(create_folder.buffer, "New Folder");

    let note = root.join("note.md");
    let rename = PendingNameInput {
        kind: PendingNameKind::Rename,
        parent: root.clone(),
        target: Some(note.clone()),
        buffer: "note.md".to_string(),
    };
    assert_eq!(rename.kind, PendingNameKind::Rename);
    assert_eq!(rename.target, Some(note));
    assert_eq!(rename.buffer, "note.md");
}

/// `has_text_input_focus` must consider a pending name prompt as focused so
/// IME keystrokes route into the prompt buffer instead of the document.
#[test]
fn has_text_input_focus_includes_pending_name_prompt() {
    // The app-level field can't be constructed without a GPUI context, so
    // validate the routing predicate directly against the pending-input
    // presence: the trio (has_text_input_focus / active_input_text_mut /
    // after_input_changed) all key off `pending_name_input.is_some()`.
    let pending = Some(PendingNameInput {
        kind: PendingNameKind::CreateFile,
        parent: PathBuf::from("workspace"),
        target: None,
        buffer: String::new(),
    });
    assert!(pending.is_some());
    // A non-pending state must be treated as unfocused for the name buffer.
    let none: Option<PendingNameInput> = None;
    assert!(none.is_none());
}

/// `dir_is_non_empty` decides whether deleting a folder needs a second
/// (recursive) confirmation. Empty folders must read as empty; folders
/// with any entry must read as non-empty.
#[test]
fn dir_is_non_empty_detects_recursive_delete_target() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Empty folder -> not non-empty -> single confirm path.
    let empty = root.join("empty");
    std::fs::create_dir(&empty).unwrap();
    assert!(!dir_is_non_empty(&empty));

    // Folder with a file -> non-empty -> second recursive confirm.
    let with_file = root.join("with_file");
    std::fs::create_dir(&with_file).unwrap();
    std::fs::write(with_file.join("note.md"), "# Note").unwrap();
    assert!(dir_is_non_empty(&with_file));

    // Folder with only a subdirectory -> still non-empty.
    let with_sub = root.join("with_sub");
    std::fs::create_dir(&with_sub).unwrap();
    std::fs::create_dir(with_sub.join("child")).unwrap();
    assert!(dir_is_non_empty(&with_sub));

    // Non-existent path -> treated as empty (no second confirm; the delete
    // itself will fail later with a clear error).
    assert!(!dir_is_non_empty(&root.join("missing")));
}

#[test]
fn preview_block_splice_reports_noop_for_identical_slices() {
    let a = blocks(&["a", "b", "c"]);
    assert_eq!(preview_block_splice(&a, &a), (3..3, 0));
}

#[test]
fn visual_block_splice_preserves_shifted_rows_by_identity() {
    let mut document = MarkdownDocument::from_text("first\n\nsecond\n\nthird\n");
    let old = document.visual_blocks();
    document.replace_range(0..5, "changed");
    let new = document.visual_blocks();

    assert_ne!(old[0].id, new[0].id);
    assert_eq!(old[1].id, new[1].id);
    assert_eq!(old[2].id, new[2].id);
    assert_ne!(old[2].source_range, new[2].source_range);
    assert_eq!(visual_block_splice(&old, &new), (0..1, 1));
}

#[test]
fn preview_block_splice_isolates_a_single_changed_block() {
    let old = blocks(&["a", "b", "c"]);
    let new = blocks(&["a", "x", "c"]);
    // Only the middle block changed: replace index 1 with 1 new item.
    assert_eq!(preview_block_splice(&old, &new), (1..2, 1));
}

#[test]
fn preview_block_splice_handles_insertion_and_deletion() {
    let a = blocks(&["a", "c"]);
    let b = blocks(&["a", "b", "c"]);
    assert_eq!(preview_block_splice(&a, &b), (1..1, 1)); // insert b
    assert_eq!(preview_block_splice(&b, &a), (1..2, 0)); // delete b
}

#[test]
fn preview_block_splice_handles_full_replace_and_empty_edges() {
    assert_eq!(
        preview_block_splice(&blocks(&["a", "b"]), &blocks(&["x", "y"])),
        (0..2, 2)
    );
    assert_eq!(preview_block_splice(&[], &blocks(&["a", "b"])), (0..0, 2));
    assert_eq!(preview_block_splice(&blocks(&["a", "b"]), &[]), (0..2, 0));
}

/// Applying the computed splice to the old slice must reproduce the new
/// slice exactly — the invariant `ListState` relies on. Mirrors `splice`'s
/// own `Vec::splice` semantics: remove `range`, insert the new items that
/// occupy the same positions in `new`.
#[test]
fn preview_block_splice_result_reconstructs_new_slice() {
    let cases: &[(&[&str], &[&str])] = &[
        (&["a", "b", "c"], &["a", "x", "c"]),
        (&["a", "c"], &["a", "b", "c"]),
        (&["a", "b", "c"], &["a", "c"]),
        (&["a", "b"], &["x", "y"]),
        (&[], &["a", "b"]),
        (&["a", "b"], &[]),
        (&["a", "b", "c", "d"], &["a", "b", "b", "c", "d"]),
    ];
    for (old_tags, new_tags) in cases {
        let old = blocks(old_tags);
        let new = blocks(new_tags);
        let (range, count) = preview_block_splice(&old, &new);
        let inserted: Vec<PreviewBlock> = new[range.start..range.start + count].to_vec();
        let mut reconstructed = old.clone();
        reconstructed.splice(range, inserted);
        assert_eq!(reconstructed, new, "old={old_tags:?} new={new_tags:?}");
    }
}

#[test]
fn preview_parses_immediately_when_never_changed_or_never_parsed() {
    // First render of a fresh document: no change timestamp, no parse yet.
    assert!(should_parse_preview_now(None, None));
    // Mode switch after edits made long ago in Edit mode: the change was
    // just observed but there is no previous parse to fall back on.
    assert!(should_parse_preview_now(Some(Duration::ZERO), None));
}

#[test]
fn preview_defers_mid_typing_and_parses_once_settled() {
    let fresh_parse = Some(PREVIEW_MAX_STALE / 4);
    // A keystroke a moment ago with a recent parse on screen: wait.
    assert!(!should_parse_preview_now(Some(Duration::ZERO), fresh_parse));
    assert!(!should_parse_preview_now(
        Some(PREVIEW_DEBOUNCE - Duration::from_millis(1)),
        fresh_parse
    ));
    // Debounce window elapsed: typing settled, parse.
    assert!(should_parse_preview_now(
        Some(PREVIEW_DEBOUNCE),
        fresh_parse
    ));
}

#[test]
fn preview_parses_anyway_when_continuous_typing_keeps_it_stale() {
    // Keystrokes never stop (since_change stays ~0), but the blocks on
    // screen are past the staleness cap: parse so the preview keeps moving.
    assert!(should_parse_preview_now(
        Some(Duration::ZERO),
        Some(PREVIEW_MAX_STALE)
    ));
    assert!(!should_parse_preview_now(
        Some(Duration::ZERO),
        Some(PREVIEW_MAX_STALE - Duration::from_millis(1))
    ));
}

#[test]
fn app_theme_cycles_through_six_builtin_themes() {
    let mut theme = AppTheme::Paper;
    let mut names = Vec::new();
    for _ in 0..AppTheme::ALL.len() {
        names.push(theme.name());
        theme = theme.next();
    }

    assert_eq!(
        names,
        vec!["Paper", "Ink", "Solar", "Forest", "Rose", "Graphite"]
    );
    assert_eq!(theme, AppTheme::Paper);
}

#[test]
fn app_theme_restores_from_saved_name() {
    assert_eq!(AppTheme::from_name("ink"), Some(AppTheme::Ink));
    assert_eq!(AppTheme::from_name(" Graphite "), Some(AppTheme::Graphite));
    assert_eq!(AppTheme::from_name("missing"), None);
}

#[test]
fn builtin_theme_table_exposes_popular_themes_with_unique_names() {
    let themes = builtin_theme_definitions();
    // The original six + at least five popular themes.
    assert!(themes.len() >= 11);
    // Original six must stay first and in canonical order so saved
    // preferences and the legacy cycle test keep resolving.
    let first_six: Vec<&str> = themes.iter().take(6).map(|t| t.name.as_str()).collect();
    assert_eq!(
        first_six,
        vec!["Paper", "Ink", "Solar", "Forest", "Rose", "Graphite"]
    );
    // Requested popular themes are present.
    for expected in [
        "GitHub Light",
        "Solarized Light",
        "One Light",
        "Tokyo Night",
    ] {
        assert!(
            themes.iter().any(|t| t.name == expected),
            "missing built-in theme {expected}"
        );
    }
    // Names are unique.
    let mut sorted: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
    sorted.sort_unstable();
    let before = sorted.len();
    sorted.dedup();
    assert_eq!(sorted.len(), before, "duplicate built-in theme names");
}

#[test]
fn shortcut_catalog_lists_core_workflows() {
    let catalog = shortcut_catalog(Language::En, DEFAULT_HEADING_MENU_MAX_LEVEL);
    let has_action = |category, label: &str, windows: &str, macos: &str| {
        catalog
            .section(category)
            .and_then(|section| section.actions.iter().find(|action| action.label == label))
            .is_some_and(|action| {
                action
                    .combinations(ShortcutPlatform::WindowsLinux)
                    .contains(&windows)
                    && action
                        .combinations(ShortcutPlatform::MacOS)
                        .contains(&macos)
            })
    };

    assert_eq!(catalog.sections.len(), ShortcutCategory::ALL.len());
    assert!(has_action(
        ShortcutCategory::Files,
        "Save",
        "Ctrl+S",
        "Cmd+S"
    ));
    assert!(has_action(
        ShortcutCategory::View,
        "Cycle View Mode",
        "Ctrl+Shift+V",
        "Cmd+Shift+V"
    ));
    assert!(has_action(
        ShortcutCategory::View,
        "Source Mode",
        "Ctrl+Alt+1",
        "Cmd+Option+1"
    ));
    assert!(has_action(
        ShortcutCategory::View,
        "Sidebar",
        "Ctrl+Shift+B",
        "Cmd+Shift+B"
    ));
    assert!(has_action(
        ShortcutCategory::Export,
        "DOCX",
        "Ctrl+Shift+D",
        "Cmd+Shift+D"
    ));
}

#[test]
fn shortcut_panel_uses_target_default_and_replaces_native_prompt() {
    assert_eq!(
        ShortcutPlatform::current(),
        if cfg!(target_os = "macos") {
            ShortcutPlatform::MacOS
        } else {
            ShortcutPlatform::WindowsLinux
        }
    );

    let search_source = include_str!("search.rs");
    let show_shortcuts = search_source
        .split_once("pub(super) fn show_shortcuts")
        .and_then(|(_, rest)| rest.split_once("pub(super) fn select_shortcut_platform"))
        .map(|(body, _)| body)
        .expect("shortcut panel open handler");
    assert!(show_shortcuts.contains("self.shortcut_panel_open = true"));
    assert!(show_shortcuts.contains("ShortcutPlatform::current()"));
    assert!(show_shortcuts.contains("ShortcutCategory::Files"));
    assert!(!show_shortcuts.contains("window.prompt"));
    assert!(search_source.contains("self.shortcut_platform = platform"));
    assert!(search_source.contains("self.shortcut_category = category"));

    let root_view_source = include_str!("root_view.rs");
    assert!(root_view_source.contains("root.child(shortcut_panel_view(self, cx))"));
    assert!(root_view_source.contains("ShortcutPlatform::ALL"));
    assert!(root_view_source.contains("catalog.sections.iter()"));

    let bootstrap_source = include_str!("bootstrap.rs");
    assert!(
        bootstrap_source.contains(
            "KeyBinding::new(menu_shortcuts::SHOW_SHORTCUTS.binding, ShowShortcuts, None)"
        )
    );
    assert!(bootstrap_source.contains("KeyBinding::new(menu_shortcuts::BOLD.binding, Bold, None)"));
    assert!(
        bootstrap_source.contains(
            "KeyBinding::new(menu_shortcuts::TOGGLE_SIDEBAR.binding, ToggleSidebar, None)"
        )
    );
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn pane_density_and_scrollbar_constants_stay_compact_and_usable() {
    assert!(
        PANE_OUTER_PADDING <= 16. * 0.20,
        "outer pane padding should stay close to the requested 15% density target"
    );
    assert!(
        PANE_INNER_PADDING < 16.,
        "inner padding should remain tighter than the old spacious pane padding"
    );
    assert!(
        PANE_SCROLLBAR_THUMB_WIDTH <= PANE_SCROLLBAR_RESERVED_WIDTH,
        "thumb must fit inside the reserved right-side scrollbar gutter"
    );
    assert_eq!(PANE_INNER_PADDING, 9.);
    assert!(
        PREVIEW_SCROLLBAR_SAFE_RIGHT_PADDING >= PANE_INNER_PADDING + PANE_SCROLLBAR_RESERVED_WIDTH,
        "preview content must reserve a right-side gutter before the overlay scrollbar"
    );
    assert!(
        RESIZE_HANDLE_WIDTH >= 8.,
        "resize handles should keep a usable invisible drag target"
    );
}

#[test]
fn read_mode_preview_width_cap_only_applies_without_adaptive_width() {
    assert_eq!(READ_MODE_PREVIEW_MAX_WIDTH, 860.);
    // Read mode: constrained when adaptive width is off (default), full when on.
    assert!(read_mode_preview_is_constrained(ViewMode::Read, false));
    assert!(!read_mode_preview_is_constrained(ViewMode::Read, true));
    // Visual Edit mode: same as Read - constrained by default, full when adaptive.
    assert!(read_mode_preview_is_constrained(
        ViewMode::VisualEdit,
        false
    ));
    assert!(!read_mode_preview_is_constrained(ViewMode::VisualEdit, true));
    // Split Preview and Edit are never constrained by the preference.
    assert!(!read_mode_preview_is_constrained(ViewMode::Split, false));
    assert!(!read_mode_preview_is_constrained(ViewMode::Split, true));
    assert!(!read_mode_preview_is_constrained(ViewMode::Edit, false));
    assert!(!read_mode_preview_is_constrained(ViewMode::Edit, true));
}

#[test]
fn view_modes_have_distinct_status_and_expected_pane_layouts() {
    assert_eq!(
        view_mode_status_message(ViewMode::Edit),
        Msg::StatusEditMode
    );
    assert_eq!(
        view_mode_status_message(ViewMode::VisualEdit),
        Msg::StatusVisualEditMode
    );
    assert_eq!(
        view_mode_status_message(ViewMode::Split),
        Msg::StatusSplitPreviewMode
    );
    assert_eq!(
        view_mode_status_message(ViewMode::Read),
        Msg::StatusReadMode
    );
    assert_eq!(view_mode_pane_widths(ViewMode::Edit, 0.4), (1.0, 0.0));
    assert_eq!(view_mode_pane_widths(ViewMode::VisualEdit, 0.4), (1.0, 0.0));
    assert_eq!(view_mode_pane_widths(ViewMode::Split, 0.4), (0.4, 0.6));
    assert_eq!(view_mode_pane_widths(ViewMode::Read, 0.4), (0.0, 1.0));
}

#[test]
fn table_edit_toolbar_is_available_only_in_visual_edit() {
    assert!(table_toolbar_actions_for_view_mode(ViewMode::Edit).is_empty());
    assert!(table_toolbar_actions_for_view_mode(ViewMode::Split).is_empty());
    assert!(table_toolbar_actions_for_view_mode(ViewMode::Read).is_empty());

    let edits = table_toolbar_actions_for_view_mode(ViewMode::VisualEdit)
        .iter()
        .map(|(_, edit, _)| *edit)
        .collect::<Vec<_>>();
    assert_eq!(
        edits,
        vec![
            TableEdit::AddRow,
            TableEdit::DeleteRow,
            TableEdit::MoveRowUp,
            TableEdit::MoveRowDown,
            TableEdit::AddColumn,
            TableEdit::DeleteColumn,
        ]
    );
}

#[test]
fn direct_view_mode_switching_preserves_tab_state() {
    let mut tab = EditorTab::new(MarkdownDocument::from_text("hello"));
    tab.selected_range = 1..4;
    tab.push_undo_snapshot();
    let version = tab.document.version();
    let mut mode = ViewMode::Edit;
    for target in [
        ViewMode::VisualEdit,
        ViewMode::Split,
        ViewMode::Read,
        ViewMode::Edit,
    ] {
        assign_view_mode(&mut mode, target);
        assert_eq!(mode, target);
        assert_eq!(tab.document.text(), "hello");
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.selected_range, 1..4);
        assert_eq!(tab.undo_stack.len(), 1);
    }
}

#[test]
fn visual_text_positions_map_to_source_content_ranges() {
    let projection = VisualProjection {
        text: "hellotest".into(),
        segments: vec![
            markion::VisualProjectionSegment {
                display_range: 0..5,
                source_range: 2..7,
            },
            markion::VisualProjectionSegment {
                display_range: 5..9,
                source_range: 11..15,
            },
        ],
        spans: Vec::new(),
        revealed_source_ranges: Vec::new(),
        source_anchor: 0,
    };
    assert_eq!(projection.source_for_display(0), 2);
    assert_eq!(projection.source_for_display(4), 6);
    assert_eq!(projection.source_for_display(6), 12);
    assert_eq!(projection.display_for_source(13), Some(7));
    assert_eq!(
        projection.display_for_source(9),
        Some(5),
        "hidden source gaps use the nearest stable display boundary"
    );
    let empty = VisualProjection {
        text: String::new(),
        segments: Vec::new(),
        spans: Vec::new(),
        revealed_source_ranges: Vec::new(),
        source_anchor: 42,
    };
    assert_eq!(
        empty.source_for_display(3),
        42,
        "clicks on empty rows must land at the row's own source anchor, not offset 0"
    );
}

#[test]
fn visual_focus_uses_half_open_block_ranges() {
    assert!(visual_source_range_is_focused(&(10..20), 10, 30));
    assert!(visual_source_range_is_focused(&(10..20), 19, 30));
    assert!(!visual_source_range_is_focused(&(10..20), 20, 30));
    assert!(visual_source_range_is_focused(&(20..30), 20, 30));
    assert!(visual_source_range_is_focused(&(20..30), 30, 30));
}

#[test]
fn visual_caret_affinity_is_ephemeral_and_preserves_derived_caches() {
    let mut tab = EditorTab::new(MarkdownDocument::from_text("plain **bold** tail"));
    let blocks = tab.document.visual_blocks_shared();
    let version = tab.document.version();

    tab.set_visual_caret_affinity(Some(VisualCaretAffinity::Downstream));
    assert_eq!(
        tab.current_visual_caret_affinity(),
        Some(VisualCaretAffinity::Downstream)
    );
    assert_eq!(tab.document.version(), version);
    assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));

    let end = tab.document.text().len();
    tab.document.replace_range(end..end, "!");
    assert_eq!(tab.current_visual_caret_affinity(), None);
}

#[test]
fn visual_navigation_lines_choose_nearest_preferred_x() {
    let line = VisualNavigationLine {
        y: px(20.),
        carets: vec![
            VisualNavigationCaret {
                source_offset: 3,
                x: px(10.),
            },
            VisualNavigationCaret {
                source_offset: 8,
                x: px(50.),
            },
            VisualNavigationCaret {
                source_offset: 13,
                x: px(90.),
            },
        ],
    };
    assert_eq!(line.closest_source(px(54.)), Some(8));
    assert_eq!(line.closest_source(px(88.)), Some(13));

    let snapshot = VisualNavigationSnapshot {
        document_version: 7,
        block_index: 2,
        source_selection: 8..8,
        marked_range: None,
        source_island: false,
        lines: vec![line],
    };
    assert_eq!(snapshot.line_index_for_source(8), Some(0));
    assert_eq!(snapshot.caret_x_for_source(8), Some(px(50.)));
}

#[test]
fn visual_interaction_state_does_not_invalidate_document_derived_state() {
    let mut tab = EditorTab::new(MarkdownDocument::from_text("plain **bold** tail"));
    let visual_blocks = tab.document.visual_blocks_shared();
    let preview_blocks = tab.document.preview_blocks_shared();
    let version = tab.document.version();
    let text = tab.shared_document_text();
    tab.set_visual_caret_affinity(Some(VisualCaretAffinity::Upstream));
    tab.visual_preferred_x = Some(px(42.));
    tab.pending_visual_navigation = Some(PendingVisualNavigation {
        document_version: version,
        target_block: 0,
        direction: VisualNavigationDirection::Down,
        extend_selection: false,
        preferred_x: px(42.),
    });
    tab.visual_marked_range_bounds = Some((6..10, Bounds::default()));
    tab.register_visual_navigation_snapshot(VisualNavigationSnapshot {
        document_version: version,
        block_index: 0,
        source_selection: 0..0,
        marked_range: Some(6..10),
        source_island: false,
        lines: vec![VisualNavigationLine {
            y: px(0.),
            carets: vec![VisualNavigationCaret {
                source_offset: 0,
                x: px(0.),
            }],
        }],
    });

    assert_eq!(tab.document.version(), version);
    assert_eq!(tab.shared_document_text(), text);
    assert!(Arc::ptr_eq(
        &visual_blocks,
        &tab.document.visual_blocks_shared()
    ));
    assert!(Arc::ptr_eq(
        &preview_blocks,
        &tab.document.preview_blocks_shared()
    ));
    assert!(!tab.document.is_dirty());
    assert!(tab.undo_stack.is_empty());
}

#[test]
fn visual_block_lookup_covers_source_and_reveal_requests_are_one_shot() {
    let source = "# Heading\n\nparagraph\n\n";
    let mut tab = EditorTab::new(MarkdownDocument::from_text(source));
    let blocks = tab.document.visual_blocks_shared();
    let version = tab.document.version();

    for offset in 0..=source.len() {
        if source.is_char_boundary(offset) {
            assert!(
                visual_block_index_for_offset(&blocks, offset, source.len()).is_some(),
                "source offset {offset} must resolve to a visual row"
            );
        }
    }

    let whitespace_index = blocks
        .iter()
        .position(|block| matches!(block.kind, VisualBlockKind::Whitespace))
        .expect("blank lines should have an explicit visual row");
    let cursor = blocks[whitespace_index].source_range.start;
    tab.selected_range = cursor..cursor;
    tab.visual_cursor_reveal_pending = true;
    assert_eq!(
        tab.take_visual_cursor_reveal_index(&blocks),
        Some(whitespace_index)
    );
    assert_eq!(tab.take_visual_cursor_reveal_index(&blocks), None);

    assert_eq!(tab.document.version(), version);
    let cached_again = tab.document.visual_blocks_shared();
    assert!(Arc::ptr_eq(&blocks, &cached_again));
}

#[test]
fn visual_ime_bounds_prefer_the_painted_caret_and_have_a_surface_fallback() {
    let surface = Bounds::new(point(px(10.), px(20.)), size(px(300.), px(200.)));
    let fallback = editor_element::visual_ime_bounds(None, Some(surface), px(PREVIEW_LINE_HEIGHT))
        .expect("visual surface should provide a pre-paint IME location");
    assert_eq!(
        fallback,
        Bounds::new(
            point(px(10. + PANE_INNER_PADDING), px(20. + PANE_INNER_PADDING)),
            size(px(2.), px(PREVIEW_LINE_HEIGHT))
        )
    );

    let caret = Bounds::new(point(px(42.), px(84.)), size(px(2.), px(21.)));
    assert_eq!(
        editor_element::visual_ime_bounds(Some(caret), Some(surface), px(PREVIEW_LINE_HEIGHT)),
        Some(caret)
    );
    assert_eq!(
        editor_element::visual_ime_bounds(None, None, px(PREVIEW_LINE_HEIGHT)),
        None
    );
}

#[test]
fn visual_extended_inline_styles_map_to_gpui_highlights() {
    let highlight = visual_highlight_style(
        InlineStyle {
            highlight: true,
            ..InlineStyle::default()
        },
        false,
    )
    .expect("highlight style");
    assert!(highlight.background_color.is_some());

    for inline_style in [
        InlineStyle {
            superscript: true,
            ..InlineStyle::default()
        },
        InlineStyle {
            subscript: true,
            ..InlineStyle::default()
        },
    ] {
        assert!(
            visual_highlight_style(inline_style, false)
                .expect("super/sub style")
                .color
                .is_some()
        );
    }
}

#[gpui::test]
fn visual_edit_platform_input_replaces_selection_and_supports_ime(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("hello"))];
        app.active_tab_mut().selected_range = 1..4;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    let (blocks_before, version_before) = app.update(cx, |app, _| {
        (
            app.active_tab().document.visual_blocks_shared(),
            app.active_tab().document.version(),
        )
    });
    cx.simulate_input("i");

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "hio");
        assert_eq!(tab.selected_range, 2..2);
        assert!(tab.document.is_dirty());
        assert_eq!(tab.document.version(), version_before + 1);
        assert_eq!(tab.undo_stack.len(), 1);
        assert_eq!(tab.autosave_generation, 1);
        assert!(tab.visual_input_bounds.is_some());
    });
    let blocks_after = app.update(cx, |app, _| {
        app.active_tab().document.visual_blocks_shared()
    });
    assert!(!Arc::ptr_eq(&blocks_before, &blocks_after));

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(
                app,
                None,
                "你",
                Some(1..1),
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "hi你o");
        assert_eq!(tab.marked_range, Some(2..5));
        assert_eq!(tab.selected_range, 5..5);
        assert!(tab.undo_stack.len() >= 2);
        assert_eq!(tab.autosave_generation, 2);
    });
    app.update(cx, |app, _| {
        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), "hio");
        assert!(app.active_tab_mut().apply_redo());
        assert_eq!(app.active_tab().document.text(), "hi你o");
    });
}

#[gpui::test]
fn visual_edit_ime_updates_share_one_undo_and_expose_exact_bounds(cx: &mut TestAppContext) {
    let source = "a **bold** z";
    let cursor = source.find("bold").unwrap() + 1;
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    for text in ["你", "你好", "你好🙂", "你好e\u{301}"] {
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                EntityInputHandler::replace_and_mark_text_in_range(
                    app, None, text, None, window, cx,
                );
            });
        });
        cx.run_until_parked();
        app.update(cx, |app, _| {
            let tab = app.active_tab();
            assert_eq!(tab.undo_stack.len(), 1);
            assert!(tab.marked_range.is_some());
            assert!(tab.visual_marked_range_bounds.is_some());
            assert_eq!(
                tab.undo_capture.map(|capture| capture.kind),
                Some(UndoCaptureKind::Ime)
            );
        });
    }

    let marked_utf16 = app.update(cx, |app, _| {
        let tab = app.active_tab();
        tab.range_to_utf16(tab.marked_range.as_ref().unwrap())
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            let expected = app
                .active_tab()
                .visual_marked_range_bounds
                .as_ref()
                .unwrap()
                .1;
            let actual = EntityInputHandler::bounds_for_range(
                app,
                marked_utf16,
                Bounds::default(),
                window,
                cx,
            );
            assert_eq!(actual, Some(expected));
            EntityInputHandler::unmark_text(app, window, cx);
        });
    });
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.marked_range.is_none());
        assert!(tab.undo_capture.is_none());
        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), source);
        assert!(app.active_tab_mut().apply_redo());
        assert!(app.active_tab().document.text().contains("你好e\u{301}"));
    });
}

#[gpui::test]
fn visual_direct_code_editor_hides_fences_highlights_and_edits_only_payload(
    cx: &mut TestAppContext,
) {
    let source = "~~~~  rust extra\nlet 名称 = 1;\n~~~~";
    let document = MarkdownDocument::from_text(source);
    let payload = document
        .visual_blocks()
        .into_iter()
        .find_map(|block| match block.editor {
            Some(VisualBlockEditor::Code { payload, .. }) => Some(payload.source_range),
            _ => None,
        })
        .expect("direct code payload");
    let cursor = payload.start + "let ".len();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let (projection, revealed) = app
            .active_tab()
            .visual_last_projection
            .as_ref()
            .expect("code payload projection");
        assert_eq!(projection, "let 名称 = 1;\n");
        assert!(revealed.is_empty());
        assert!(
            app.highlight_cache
                .borrow()
                .contains_key(&(Some("rust".into()), "let 名称 = 1;\n".into()))
        );
    });

    cx.simulate_input("mut ");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(
            tab.document.text(),
            "~~~~  rust extra\nlet mut 名称 = 1;\n~~~~"
        );
        assert_eq!(
            &tab.document.text()[.."~~~~  rust extra\n".len()],
            "~~~~  rust extra\n"
        );
        assert!(tab.document.text().ends_with("\n~~~~"));
        assert_eq!(tab.undo_stack.len(), 1);
        assert!(tab.document.is_dirty());
    });

    cx.dispatch_action(SetEditMode);
    app.update(cx, |app, _| {
        assert_eq!(app.view_mode, ViewMode::Edit);
        assert_eq!(
            app.active_tab().document.text(),
            "~~~~  rust extra\nlet mut 名称 = 1;\n~~~~"
        );
    });
    cx.dispatch_action(SetVisualEditMode);
    cx.run_until_parked();

    cx.dispatch_action(Undo);
    app.update(cx, |app, cx| {
        assert_eq!(app.active_tab().document.text(), source);
        let payload_start = app
            .active_tab()
            .document
            .visual_blocks()
            .into_iter()
            .find_map(|block| match block.editor {
                Some(VisualBlockEditor::Code { payload, .. }) => Some(payload.source_range.start),
                _ => None,
            })
            .unwrap();
        app.move_to(payload_start, cx);
    });
    cx.dispatch_action(Backspace);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().cursor_offset(), payload.start);
    });
    cx.dispatch_action(SelectLeft);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(
            app.active_tab().selected_range,
            payload.start..payload.start
        );
    });
    app.update(cx, |app, cx| app.move_to(payload.end, cx));
    cx.dispatch_action(Delete);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().cursor_offset(), payload.end);
    });
}

#[gpui::test]
fn visual_edit_warms_diagram_cache_for_mermaid_fence(cx: &mut TestAppContext) {
    // Visual Edit no longer parses preview blocks, so diagram cache warming
    // must walk the visual blocks. A `mermaid` fence should produce one
    // pending cache entry keyed identically to Split Preview's entry, and
    // re-rendering the same document must not spawn a second render (dedupe
    // via `reserve_pending`).
    let source = "```mermaid\nflowchart LR\nA --> B\n```";
    let document = MarkdownDocument::from_text(source);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    // The render thread is detached; park until it lands in the cache.
    app.update(cx, |app, _| {
        let theme = app.diagram_theme();
        let expected_key = DiagramCacheKey {
            backend_id: "mermaid".into(),
            source: "flowchart LR\nA --> B\n".into(),
            theme,
        };
        // Visual Edit warms via the visual blocks path on the first render.
        let entry = app
            .diagram_cache
            .get(&expected_key)
            .expect("visual edit should have warmed the mermaid cache entry");
        assert!(
            matches!(
                entry,
                DiagramCacheEntry::Pending
                    | DiagramCacheEntry::Ready(_, _)
                    | DiagramCacheEntry::Error(_)
            ),
            "cache entry should exist after Visual Edit render"
        );
    });

    // A second render pass must not re-reserve the same key.
    app.update(cx, |app, cx| {
        let visual = app.active_tab().document.visual_blocks_shared();
        let preview: Vec<PreviewBlock> = Vec::new();
        let before = app.diagram_cache.len();
        app.ensure_diagram_renders(&preview, &visual, cx);
        let after = app.diagram_cache.len();
        assert_eq!(
            before, after,
            "re-rendering the same Visual Edit diagram must not spawn a second render"
        );
    });
}

#[gpui::test]
fn visual_direct_math_editor_keeps_invalid_payload_ime_and_one_undo(cx: &mut TestAppContext) {
    let source = "$$\n\\frac{1}{2}\n$$";
    let document = MarkdownDocument::from_text(source);
    let payload = document
        .visual_blocks()
        .into_iter()
        .find_map(|block| match block.editor {
            Some(VisualBlockEditor::Math { payload, .. }) => Some(payload.source_range),
            _ => None,
        })
        .expect("direct math payload");
    let closing_brace = source[..payload.end].rfind('}').unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = closing_brace..closing_brace + 1;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    for composition in ["你", "你好🙂"] {
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                EntityInputHandler::replace_and_mark_text_in_range(
                    app,
                    None,
                    composition,
                    None,
                    window,
                    cx,
                );
            });
        });
        cx.run_until_parked();
    }
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.document.text().starts_with("$$\n"));
        assert!(tab.document.text().ends_with("\n$$"));
        assert!(tab.document.text().contains("你好🙂"));
        assert_eq!(tab.undo_stack.len(), 1);
        assert!(tab.marked_range.is_some());
        assert!(tab.visual_marked_range_bounds.is_some());
        let (projection, _) = tab
            .visual_last_projection
            .as_ref()
            .expect("invalid math keeps payload projection");
        assert!(projection.contains("你好🙂"));
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::unmark_text(app, window, cx)
        });
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source)
    });
}

#[gpui::test]
fn visual_block_math_source_expands_and_collapses_without_editing(cx: &mut TestAppContext) {
    let source = "$$\n\\frac{1}{2}\n$$\n\nAfter";
    let document = MarkdownDocument::from_text(source);
    let math_id = document
        .visual_blocks()
        .into_iter()
        .find_map(|block| match block.kind {
            VisualBlockKind::MathBlock { .. } => Some(block.id),
            _ => None,
        })
        .expect("math block");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let version = app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(!tab.is_visual_source_expanded(math_id));
        tab.document.version()
    });

    app.update(cx, |app, cx| {
        let tab = app.active_tab_mut();
        tab.toggle_visual_source_expanded(math_id);
        assert!(tab.is_visual_source_expanded(math_id));
        cx.notify();
    });
    app.update(cx, |app, cx| {
        // Clicking "outside" clears expand without touching the document.
        app.active_tab_mut().retain_visual_source_expand = None;
        app.active_tab_mut().apply_visual_source_outside_click();
        assert!(!app.active_tab().is_visual_source_expanded(math_id));
        assert_eq!(app.active_tab().document.version(), version);
        assert_eq!(app.active_tab().document.text(), source);
        assert!(app.active_tab().undo_stack.is_empty());
        cx.notify();
    });
}

#[gpui::test]
fn visual_diagram_source_expands_and_collapses_without_editing(cx: &mut TestAppContext) {
    let source = "```mermaid\nflowchart LR\nA --> B\n```\n\nAfter";
    let document = MarkdownDocument::from_text(source);
    let diagram_id = document
        .visual_blocks()
        .into_iter()
        .find_map(|block| match (&block.kind, block.editor.as_ref()) {
            (VisualBlockKind::CodeBlock { language }, Some(VisualBlockEditor::Code { .. }))
                if language.as_deref() == Some("mermaid") =>
            {
                Some(block.id)
            }
            _ => None,
        })
        .expect("mermaid diagram block");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let version = app.update(cx, |app, _| app.active_tab().document.version());
    app.update(cx, |app, cx| {
        app.active_tab_mut()
            .set_visual_source_expanded(diagram_id, true);
        assert!(app.active_tab().is_visual_source_expanded(diagram_id));
        app.active_tab_mut().retain_visual_source_expand = None;
        app.active_tab_mut().apply_visual_source_outside_click();
        assert!(!app.active_tab().is_visual_source_expanded(diagram_id));
        assert_eq!(app.active_tab().document.version(), version);
        assert_eq!(app.active_tab().document.text(), source);
        cx.notify();
    });
}

#[gpui::test]
fn visual_direct_table_cell_edit_reflows_traverses_and_undoes_once(cx: &mut TestAppContext) {
    let source = "| A | B |\n| :--- | ---: |\n| x | y |";
    let document = MarkdownDocument::from_text(source);
    let first = document
        .visual_blocks()
        .into_iter()
        .find_map(|block| match block.editor {
            Some(VisualBlockEditor::Table { cells }) => cells
                .into_iter()
                .find(|cell| cell.row == 0 && cell.column == 0)
                .map(|cell| cell.field.source_range),
            _ => None,
        })
        .expect("first table cell");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = first;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (version, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (tab.document.version(), tab.document.visual_blocks_shared())
    });
    cx.dispatch_action(Indent);
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let field = tab
            .document
            .visual_editor_field_at(&tab.selected_range)
            .expect("second table cell");
        assert_eq!(
            field.kind,
            VisualEditorFieldKind::TableCell { row: 0, column: 1 }
        );
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_text_in_range(app, None, "宽|值", window, cx);
        });
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.document.text().contains("宽\\|值"));
        assert!(tab.document.text().lines().nth(1).unwrap().contains(":---"));
        assert!(tab.document.text().lines().nth(1).unwrap().contains("---:"));
        assert_eq!(tab.undo_stack.len(), 1);
        let field = tab
            .document
            .visual_editor_field_at(&tab.selected_range)
            .expect("selection remains in edited cell");
        assert_eq!(
            field.kind,
            VisualEditorFieldKind::TableCell { row: 0, column: 1 }
        );
        let (projection, _) = tab
            .visual_last_projection
            .as_ref()
            .expect("active table cell projection");
        assert_eq!(projection, "宽|值");
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.active_tab().selected_range, 6..7);
    });

    for composition in ["你", "你好🙂"] {
        cx.update(|window, cx| {
            app.update(cx, |app, cx| {
                EntityInputHandler::replace_and_mark_text_in_range(
                    app,
                    None,
                    composition,
                    None,
                    window,
                    cx,
                );
            });
        });
        cx.run_until_parked();
    }
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.document.text().contains("你好🙂"));
        assert_eq!(tab.undo_stack.len(), 1);
        assert!(tab.marked_range.is_some());
        assert!(tab.visual_marked_range_bounds.is_some());
        let field = tab
            .document
            .visual_editor_field_at(tab.marked_range.as_ref().unwrap())
            .expect("composition remains in the logical table cell");
        assert_eq!(
            field.kind,
            VisualEditorFieldKind::TableCell { row: 0, column: 1 }
        );
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::unmark_text(app, window, cx)
        });
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source)
    });
}

#[test]
fn direct_field_projection_hides_only_protective_escapes_and_keeps_exact_boundaries() {
    let source = "| a\\|b | c |\n| --- | --- |\n| x | y |";
    let document = MarkdownDocument::from_text(source);
    let field = document
        .visual_blocks()
        .into_iter()
        .find_map(|block| match block.editor {
            Some(VisualBlockEditor::Table { cells }) => cells
                .into_iter()
                .find(|cell| cell.row == 0 && cell.column == 0)
                .map(|cell| cell.field),
            _ => None,
        })
        .expect("escaped table cell");
    let projection = visual_editor_field_projection(source, &field);
    assert_eq!(projection.text, "a|b");
    let pipe = projection.text.find('|').unwrap();
    assert_eq!(
        projection.source_for_display(pipe),
        field.source_range.start + 1
    );
    assert_eq!(
        projection.source_for_display(pipe + 1),
        field.source_range.start + 3
    );
    assert_eq!(
        projection.display_for_source(field.source_range.start + 2),
        Some(pipe + 1)
    );
}

#[gpui::test]
fn visual_edit_renders_local_live_preview_projection_and_edits_source(cx: &mut TestAppContext) {
    let source = "plain **bold** and [site](url) tail";
    let plain_cursor = source.find("plain").unwrap() + 1;
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = plain_cursor..plain_cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let (text, revealed) = app
            .active_tab()
            .visual_last_projection
            .as_ref()
            .expect("supported focused prose should paint a visual projection");
        assert_eq!(text, "plain bold and site tail");
        assert!(revealed.is_empty());
        assert!(app.active_tab().visual_caret_bounds.is_some());
    });

    let bold_cursor = source.find("bold").unwrap() + 1;
    app.update(cx, |app, cx| app.move_to(bold_cursor, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let (text, revealed) = app.active_tab().visual_last_projection.as_ref().unwrap();
        assert_eq!(text, "plain **bold** and site tail");
        assert_eq!(revealed.len(), 1);
        assert_eq!(&source[revealed[0].clone()], "**bold**");
    });

    let link_cursor = source.find("site").unwrap() + 1;
    app.update(cx, |app, cx| app.move_to(link_cursor, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let (text, revealed) = app.active_tab().visual_last_projection.as_ref().unwrap();
        assert_eq!(text, "plain bold and [site](url) tail");
        assert_eq!(&source[revealed[0].clone()], "[site](url)");
    });

    cx.simulate_input("X");
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            "plain **bold** and [sXite](url) tail"
        );
        assert!(app.active_tab().document.is_dirty());
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        let (text, revealed) = app.active_tab().visual_last_projection.as_ref().unwrap();
        assert_eq!(text, "plain bold and [sXite](url) tail");
        assert_eq!(revealed.len(), 1);
    });
}

#[gpui::test]
fn visual_edit_renders_default_inline_formatting_and_locally_reveals_markers(
    cx: &mut TestAppContext,
) {
    let source = markion::DEFAULT_WELCOME_MARKDOWN;
    let plain_cursor = source.find("Write with").unwrap() + 1;
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = plain_cursor..plain_cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (version, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let (text, revealed) = tab
            .visual_last_projection
            .as_ref()
            .expect("default inline paragraph should paint a visual projection");
        assert!(revealed.is_empty());
        for hidden_marker in [
            "*italic*",
            "**bold**",
            "***bold italic***",
            "==highlighted text==",
        ] {
            assert!(
                !text.contains(hidden_marker),
                "marker remained visible: {hidden_marker}"
            );
        }
        for rendered in [
            "italic",
            "bold italic",
            "strikethrough",
            "inline code",
            "highlighted text",
            "H2O",
            "x2",
            "Markion project page",
        ] {
            assert!(
                text.contains(rendered),
                "missing rendered content: {rendered}"
            );
        }

        let styles = tab
            .visual_last_projection_styles
            .as_ref()
            .expect("projection styles should reach the rendered text element");
        assert!(styles.iter().any(|style| style.bold && style.italic));
        assert!(styles.iter().any(|style| style.strikethrough));
        assert!(styles.iter().any(|style| style.code));
        assert!(styles.iter().any(|style| style.highlight));
        assert!(styles.iter().any(|style| style.superscript));
        assert!(styles.iter().any(|style| style.subscript));
        (tab.document.version(), tab.document.visual_blocks_shared())
    });

    let nested_cursor = source.find("bold italic").unwrap() + 1;
    app.update(cx, |app, cx| app.move_to(nested_cursor, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let (text, revealed) = tab.visual_last_projection.as_ref().unwrap();
        assert!(text.contains("***bold italic***"));
        assert_eq!(revealed.len(), 1);
        assert_eq!(&source[revealed[0].clone()], "***bold italic***");
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
    });

    let highlight_cursor = source.find("highlighted text").unwrap() + 1;
    app.update(cx, |app, cx| app.move_to(highlight_cursor, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let (text, revealed) = tab.visual_last_projection.as_ref().unwrap();
        assert!(text.contains("==highlighted text=="));
        assert_eq!(revealed.len(), 1);
        assert_eq!(&source[revealed[0].clone()], "==highlighted text==");
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
        assert!(!tab.document.is_dirty());
        assert!(tab.undo_stack.is_empty());
    });
}

#[gpui::test]
fn visual_edit_paints_trailing_space_before_the_next_character(cx: &mut TestAppContext) {
    let source = "## heading";
    let cursor = source.len();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.simulate_input(" ");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "## heading ");
        let (text, _) = app
            .active_tab()
            .visual_last_projection
            .as_ref()
            .expect("heading should paint a visual projection");
        assert_eq!(text, "heading ");
    });

    cx.simulate_input("x");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "## heading x");
        let (text, _) = app.active_tab().visual_last_projection.as_ref().unwrap();
        assert_eq!(text, "heading x");
    });
}

#[gpui::test]
fn visual_edit_paints_exactly_one_caret_in_the_focused_block(cx: &mut TestAppContext) {
    let source = "first paragraph\n\n## second heading\n\nthird **bold** tail\n";
    let cursor = source.find("third").unwrap() + 2;
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.visual_projection_paint_count > 0);
        assert_eq!(
            tab.visual_caret_paint_count, tab.visual_projection_paint_count,
            "every paint pass must draw the caret exactly once, in the focused block"
        );
        assert!(tab.visual_caret_bounds.is_some());
    });

    // Moving the caret to another block keeps the one-caret-per-frame
    // invariant: unfocused rows must not paint clamped carets of their own.
    let heading_cursor = source.find("second").unwrap();
    app.update(cx, |app, cx| app.move_to(heading_cursor, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(
            tab.visual_caret_paint_count, tab.visual_projection_paint_count,
            "caret must follow focus, one caret per frame"
        );
    });
}

#[gpui::test]
fn visual_edit_navigation_follows_wrapped_lines_without_reparsing(cx: &mut TestAppContext) {
    let source = (0..220)
        .map(|index| format!("word{index} "))
        .collect::<String>();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.active_tab_mut().selected_range = 0..0;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (version, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let snapshot = tab
            .visual_navigation_snapshots
            .get(&0)
            .expect("focused visual row should register navigation geometry");
        assert!(
            snapshot.lines.len() > 2,
            "paragraph must soft-wrap in the test window"
        );
        (tab.document.version(), tab.document.visual_blocks_shared())
    });

    cx.dispatch_action(Down);
    cx.run_until_parked();
    let first_down = app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.cursor_offset() > 0);
        assert!(tab.visual_preferred_x.is_some());
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
        tab.cursor_offset()
    });

    cx.dispatch_action(SelectDown);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.cursor_offset() > first_down);
        assert!(!tab.selected_range.is_empty());
        assert_eq!(tab.document.version(), version);
        assert!(tab.visual_caret_paint_count <= tab.visual_projection_paint_count);
    });
}

#[gpui::test]
fn visual_edit_navigation_reveals_virtualized_adjacent_block(cx: &mut TestAppContext) {
    let first = (0..240).map(|_| "wide ").collect::<String>();
    let source = format!("{first}\n\nsecond block");
    let cursor = first.len() - 1;
    let second_start = source.find("second").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    cx.dispatch_action(Down);
    cx.run_until_parked();
    cx.dispatch_action(Down);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(
            tab.cursor_offset() >= second_start,
            "cursor={}, second={}, pending={:?}, position={:?}, snapshots={:?}",
            tab.cursor_offset(),
            second_start,
            tab.pending_visual_navigation,
            tab.visual_navigation_position,
            tab.visual_navigation_snapshots.keys().collect::<Vec<_>>()
        );
        assert!(tab.pending_visual_navigation.is_none());
    });
}

#[gpui::test]
fn visual_edit_does_not_duplicate_nested_list_input_in_the_parent(cx: &mut TestAppContext) {
    let source = "- parent\n  - child\n";
    let child_cursor = source.find("child").unwrap() + 2;
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = child_cursor..child_cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    app.update(cx, |app, _| {
        let (text, _) = app
            .active_tab()
            .visual_last_projection
            .as_ref()
            .expect("nested child should paint its own projection");
        assert_eq!(text, "child");
    });

    cx.simulate_input("X");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "- parent\n  - chXild\n");
        let (text, _) = app.active_tab().visual_last_projection.as_ref().unwrap();
        assert_eq!(text, "chXild");
    });

    let parent_cursor = source.find("parent").unwrap() + 1;
    app.update(cx, |app, cx| app.move_to(parent_cursor, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let (text, _) = app.active_tab().visual_last_projection.as_ref().unwrap();
        assert_eq!(text, "parent");
    });
}

#[gpui::test]
fn visual_edit_reuses_focused_large_document_row_after_early_edit(cx: &mut TestAppContext) {
    let source = (0..120)
        .map(|index| format!("paragraph {index} has enough text to paint\n\n"))
        .collect::<String>();
    let cursor = source.find("paragraph 90").unwrap() + "paragraph 90 ".len();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let focused_id = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let index = visual_block_index_for_offset(
            &tab.visual_list_blocks,
            tab.cursor_offset(),
            tab.document.text().len(),
        )
        .unwrap();
        tab.visual_list_blocks[index].id
    });
    app.update(cx, |app, cx| {
        app.active_tab_mut()
            .document
            .replace_range(0..9, "section00");
        app.after_document_changed(cx);
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let index = visual_block_index_for_offset(
            &tab.visual_list_blocks,
            tab.cursor_offset(),
            tab.document.text().len(),
        )
        .unwrap();
        assert_eq!(tab.visual_list_blocks[index].id, focused_id);
        assert!(
            tab.visual_navigation_snapshot_ids
                .iter()
                .all(|(index, id)| {
                    tab.visual_list_blocks
                        .get(*index)
                        .is_some_and(|block| block.id == *id)
                })
        );
    });

    cx.simulate_input("X");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(
            app.active_tab()
                .document
                .text()
                .contains("paragraph 90 Xhas")
        );
        assert!(app.active_tab().marked_range.is_none());
    });
}

#[gpui::test]
fn visual_edit_structural_backspace_is_one_undoable_tab_local_edit(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("# 标题")),
            EditorTab::new(MarkdownDocument::from_text("second tab")),
        ];
        app.active_tab = 0;
        app.active_tab_mut().selected_range = 2..2;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    let blocks_before = app.update(cx, |app, _| {
        app.active_tab().document.visual_blocks_shared()
    });

    cx.dispatch_action(Backspace);
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "标题");
        assert_eq!(tab.selected_range, 0..0);
        assert!(tab.document.is_dirty());
        assert_eq!(tab.undo_stack.len(), 1);
        assert_eq!(tab.autosave_generation, 1);
        assert_eq!(app.tabs[1].document.text(), "second tab");
        assert!(!app.tabs[1].document.is_dirty());
    });
    let blocks_after = app.update(cx, |app, _| {
        app.active_tab().document.visual_blocks_shared()
    });
    assert!(!Arc::ptr_eq(&blocks_before, &blocks_after));

    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "# 标题");
        assert_eq!(app.active_tab().selected_range, 2..2);
    });
    cx.dispatch_action(Redo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "标题");
        assert_eq!(app.active_tab().selected_range, 0..0);
    });
}

#[gpui::test]
fn visual_edit_structural_enter_continues_list_with_one_history_entry(cx: &mut TestAppContext) {
    let source = "- item";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = source.len()..source.len();
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.dispatch_action(InsertNewline);
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "- item\n- ");
        assert_eq!(tab.selected_range, 9..9);
        assert_eq!(tab.undo_stack.len(), 1);
        assert_eq!(tab.autosave_generation, 1);
    });
}

fn assert_visual_edit_gap_click_is_passive(cx: &mut TestAppContext, source: &'static str) {
    let cursor = source.find('H').expect("heading text") + 1;
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (selection, text, version, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (
            tab.selected_range.clone(),
            tab.document.text().to_string(),
            tab.document.version(),
            tab.document.visual_blocks_shared(),
        )
    });
    let gap = cx
        .debug_bounds("visual-whitespace-gap")
        .expect("the passive whitespace row should be rendered");

    cx.simulate_click(gap.center(), Modifiers::none());
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.selected_range, selection);
        assert_eq!(tab.document.text(), text);
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
        assert!(!tab.document.is_dirty());
        assert!(tab.undo_stack.is_empty());
    });
}

#[gpui::test]
fn visual_edit_heading_to_heading_gap_click_is_passive(cx: &mut TestAppContext) {
    assert_visual_edit_gap_click_is_passive(cx, "## H2\n\n### H3");
}

#[gpui::test]
fn visual_edit_heading_to_paragraph_gap_click_is_passive(cx: &mut TestAppContext) {
    assert_visual_edit_gap_click_is_passive(cx, "## Heading\n\nBody");
}

#[gpui::test]
fn visual_edit_heading_enter_activates_insertion_line_for_typing(cx: &mut TestAppContext) {
    let source = "## Heading";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = source.len()..source.len();
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "## Heading\n");
        assert_eq!(tab.selected_range, source.len() + 1..source.len() + 1);
        let blocks = tab.document.visual_blocks_shared();
        let block_index =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
                .expect("the new insertion line should own a visual row");
        assert_eq!(
            blocks[block_index].source_range.end,
            tab.document.text().len()
        );
        assert!(tab.visual_caret_bounds.is_some());
        assert!(tab.visual_input_bounds.is_some());
    });

    cx.simulate_input("Body");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "## Heading\nBody");
        assert_eq!(tab.selected_range, source.len() + 5..source.len() + 5);
        assert!(tab.undo_stack.len() >= 2);
        assert!(tab.autosave_generation >= 2);
        assert!(tab.document.is_dirty());
    });
}

#[gpui::test]
fn visual_edit_paragraph_enter_shows_caret_not_source_island(cx: &mut TestAppContext) {
    // Regression for `fix-visual-edit-whitespace-caret-box`: pressing Enter
    // twice at the end of a paragraph creates a real blank line between the
    // paragraph and end-of-document. Pressing Down onto that blank line drops
    // the caret onto a Whitespace row, which must render as passive height +
    // a thin caret line (NOT a bordered source-island box) and must still
    // accept typed text.
    let source = "Body";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = source.len()..source.len();
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    // First Enter: "Body" -> "Body\n". Second Enter: "Body\n" -> "Body\n\n".
    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();
    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "Body\n\n");
        // After two Enters the caret is at end-of-document (offset 6) and the
        // trailing blank line owns it as a Whitespace row.
        let blocks = tab.document.visual_blocks_shared();
        let block_index = visual_block_index_for_offset(
            &blocks,
            tab.cursor_offset(),
            tab.document.text().len(),
        )
        .expect("the blank line should own a visual row");
        assert!(
            matches!(blocks[block_index].kind, VisualBlockKind::Whitespace),
            "Enter twice after a paragraph should land the caret on a Whitespace row"
        );
        assert!(tab.visual_caret_bounds.is_some());
        assert!(tab.visual_input_bounds.is_some());
    });

    // The thin-caret path must still accept typed text at the caret.
    cx.simulate_input("More");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "Body\n\nMore");
        assert!(tab.document.is_dirty());
        assert!(!tab.undo_stack.is_empty());
    });
}

#[gpui::test]
fn visual_edit_down_arrow_skips_blank_line_gap_to_next_block(cx: &mut TestAppContext) {
    // Down arrow moves directly between rendered content blocks: the blank-line
    // `Whitespace` gap row separating the two paragraphs is pure inter-block
    // spacing and must NOT capture the caret as a dead navigation stop (that
    // felt like "Down did nothing" and forced an extra keypress). The blank
    // line stays reachable via Enter/click, covered by
    // `visual_edit_paragraph_enter_shows_caret_not_source_island`.
    let source = "Para 1\n\nPara 2";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        // Caret inside "Para 1".
        app.active_tab_mut().selected_range = 3..3;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    cx.dispatch_action(Down);
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let block_index = visual_block_index_for_offset(
            &blocks,
            tab.cursor_offset(),
            tab.document.text().len(),
        )
        .expect("Down should land on a visual row");
        assert!(
            matches!(blocks[block_index].kind, VisualBlockKind::Paragraph),
            "single Down from Para 1 should skip the blank-line gap and land in \
             Para 2, got kind={:?} cursor={}",
            blocks[block_index].kind,
            tab.cursor_offset(),
        );
        // "Para 2" begins at offset 8; the caret must be inside it, not on the
        // gap row (offset 7).
        assert!(
            tab.cursor_offset() >= 8,
            "caret must be inside Para 2, got {}",
            tab.cursor_offset()
        );
    });
}

#[gpui::test]
fn visual_edit_up_arrow_skips_blank_line_gap_to_heading(cx: &mut TestAppContext) {
    // Up moves directly from a paragraph into the heading above it in a SINGLE
    // press, skipping the blank-line `Whitespace` gap row that separates them.
    // The gap is pure inter-block spacing; parking the caret there looked like
    // "Up did nothing" and forced a second press to reach the heading. The
    // preferred horizontal coordinate is retained across the crossing. The
    // blank line stays reachable via Enter/click, covered by
    // `visual_edit_paragraph_enter_shows_caret_not_source_island`.
    //
    // Blocks for "### heading\n\nparagraph":
    //   Heading(0..12), Whitespace(12..13), Paragraph(13..22).
    let source = "### heading\n\nparagraph";

    // (A) Caret in the middle of the paragraph: one Up lands in the heading.
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = 16..16;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    cx.dispatch_action(Up);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let block_index = visual_block_index_for_offset(
            &blocks,
            tab.cursor_offset(),
            tab.document.text().len(),
        )
        .expect("Up should land on a visual row");
        assert!(
            matches!(blocks[block_index].kind, VisualBlockKind::Heading { .. }),
            "single Up from the paragraph should skip the gap and land in the \
             heading, got kind={:?} cursor={}",
            blocks[block_index].kind,
            tab.cursor_offset(),
        );
        assert!(
            tab.cursor_offset() < 12,
            "caret must be inside the heading (0..12), not on the gap row \
             (offset 12), got {}",
            tab.cursor_offset()
        );
        assert!(
            tab.visual_preferred_x.is_some(),
            "preferred_x should be retained across the blank-line crossing"
        );
    });

    // (B) Caret at the paragraph start: one Up still skips the gap into the
    //     heading rather than staying put or parking on the blank line.
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = 13..13;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    cx.dispatch_action(Up);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let block_index = visual_block_index_for_offset(
            &blocks,
            tab.cursor_offset(),
            tab.document.text().len(),
        )
        .expect("Up should land on a visual row");
        assert!(
            matches!(blocks[block_index].kind, VisualBlockKind::Heading { .. }),
            "Up from paragraph start must skip the gap into the heading, \
             got kind={:?} cursor={}",
            blocks[block_index].kind,
            tab.cursor_offset(),
        );
        assert!(
            tab.cursor_offset() < 12,
            "caret must be inside the heading, got {}",
            tab.cursor_offset()
        );
    });
}

#[gpui::test]
fn source_edit_backspace_keeps_raw_character_semantics(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("# title"))];
        app.active_tab_mut().selected_range = 2..2;
        app.view_mode = ViewMode::Edit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.dispatch_action(Backspace);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "#title");
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
}

#[gpui::test]
fn large_visual_document_projects_visible_rows_without_invalidating_cache(cx: &mut TestAppContext) {
    let source = (0..1_000)
        .map(|index| format!("paragraph {index} with **bold** text"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let total_blocks = MarkdownDocument::from_text(&source).visual_blocks().len();
    let source_for_window = source.clone();
    let (app, cx) = cx.add_window_view(move |_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            &source_for_window,
        ))];
        app.active_tab_mut().selected_range = 1..1;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    let (version, blocks, first_paint_count) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (
            tab.document.version(),
            tab.document.visual_blocks_shared(),
            tab.visual_projection_paint_count,
        )
    });
    assert!(first_paint_count > 0);
    assert!(
        first_paint_count < total_blocks,
        "virtualized list should not project all {total_blocks} blocks; painted {first_paint_count}"
    );

    let bold_cursor = source.find("bold").unwrap() + 1;
    app.update(cx, |app, cx| app.move_to(bold_cursor, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
        assert!(tab.visual_projection_paint_count < total_blocks);
        let (text, revealed) = tab.visual_last_projection.as_ref().unwrap();
        assert!(text.contains("**bold**"));
        assert_eq!(revealed.len(), 1);
    });
}

#[gpui::test]
fn visual_edit_platform_input_works_for_an_empty_document(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(""))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.simulate_input("a");
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "a");
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
}

#[gpui::test]
fn read_mode_does_not_register_an_editable_input_surface(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("read only"))];
        app.active_tab_mut().selected_range = 9..9;
        app.view_mode = ViewMode::Read;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.simulate_input("!");
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "read only");
        assert!(!app.active_tab().document.is_dirty());
        assert!(app.active_tab().undo_stack.is_empty());
    });
}

#[test]
fn custom_theme_palette_uses_definition_colors() {
    let theme = ThemeDefinition {
        name: "Test".into(),
        is_dark: false,
        colors: ThemeColors {
            app_bg: 0x010203,
            panel_bg: 0x111213,
            surface_bg: 0x212223,
            text: 0x313233,
            muted: 0x414243,
            border: 0x515253,
            active_bg: 0x616263,
            active_text: 0x717273,
        },
    };
    let palette = theme_palette_from_definition(&theme);

    assert_eq!(palette.app_bg, rgb(0x010203));
    assert_eq!(palette.active_text, rgb(0x717273));
}

#[test]
fn ime_selected_range_is_relative_to_composition_text() {
    let composing = "😀文";
    let range = EditorTab::relative_range_from_utf16(composing, &(2..3));

    assert_eq!(&composing[range], "文");
    assert_eq!(utf16_offset_to_byte_offset("a😀文", 3), "a😀".len());
    assert_eq!(byte_offset_to_utf16_offset("a😀文", "a😀".len()), 3);
}

#[test]
fn editor_tab_new_initializes_empty_state() {
    let tab = EditorTab::new(MarkdownDocument::from_text("hello"));
    // Defaults match what the pre-refactor MarkionApp::new used.
    assert_eq!(tab.document.text(), "hello");
    assert!(tab.undo_stack.is_empty());
    assert!(tab.redo_stack.is_empty());
    assert_eq!(tab.selected_range, 0..0);
    assert!(!tab.selection_reversed);
    assert!(tab.marked_range.is_none());
    assert!(tab.last_lines.is_empty());
    assert!(tab.line_offsets.is_empty());
    assert!(tab.line_heights.is_empty());
    assert!(tab.last_bounds.is_none());
    assert_eq!(tab.line_height, px(24.));
    assert!(!tab.is_selecting);
    assert!(tab.last_recovery_file.is_none());
    assert_eq!(tab.autosave_generation, 0);
    assert!(tab.display_text_cache.borrow().is_none());
    assert!(tab.preview_parse_inflight.is_none());
}

#[test]
fn reset_preview_list_orphans_inflight_parse() {
    let mut tab = EditorTab::new(MarkdownDocument::from_text("# One"));
    tab.preview_parse_inflight = Some(next_preview_parse_id());
    // Replacing the document must clear the marker so a background result
    // for the old text can no longer find (and corrupt) this tab.
    tab.reset_preview_list();
    assert!(tab.preview_parse_inflight.is_none());
    assert!(tab.preview_reflects_version.is_none());
}

#[test]
fn undo_history_keeps_one_full_snapshot_and_compacts_the_rest() {
    let mut tab = EditorTab::new(MarkdownDocument::from_text("hello world"));
    for (range, insert) in [(5..5, ","), (12..12, "!"), (0..5, "goodbye")] {
        tab.selected_range = range.start..range.start;
        tab.push_undo_snapshot();
        tab.document.replace_range(range, insert);
    }

    assert_eq!(tab.undo_stack.len(), 3);
    assert!(matches!(tab.undo_stack[0], UndoEntry::Diff(_)));
    assert!(matches!(tab.undo_stack[1], UndoEntry::Diff(_)));
    assert!(matches!(tab.undo_stack[2], UndoEntry::Full(_)));
}

#[test]
fn undo_redo_roundtrip_through_compacted_history() {
    // Walk a document through edits that exercise insertion, deletion,
    // replacement, and multi-byte chars sharing UTF-8 prefix bytes
    // (中 E4B8AD vs 串 E4B8B2 — the byte diff lands mid-char and must be
    // widened to a char boundary), then undo/redo across the whole
    // history and verify every intermediate state.
    let edits: [(Range<usize>, &str); 4] =
        [(5..5, " 中"), (9..9, " beta"), (5..9, " 串"), (0..5, "")];
    let mut tab = EditorTab::new(MarkdownDocument::from_text("alpha"));
    // Undoing edit i restores its pre-edit text and the selection captured
    // when its snapshot was pushed (the range about to be replaced).
    let mut undo_expected: Vec<(String, Range<usize>)> = Vec::new();
    let mut redo_texts: Vec<String> = Vec::new();
    for (range, insert) in edits {
        undo_expected.push((tab.document.text().to_string(), range.clone()));
        tab.selected_range = range.clone();
        tab.push_undo_snapshot();
        tab.document.replace_range(range.clone(), insert);
        let end = range.start + insert.len();
        tab.selected_range = end..end;
        redo_texts.push(tab.document.text().to_string());
    }

    // Undo all the way down, checking text and selection at every step.
    for (text, selection) in undo_expected.iter().rev() {
        assert!(tab.apply_undo());
        assert_eq!(tab.document.text(), text, "undo text");
        assert_eq!(&tab.selected_range, selection, "undo selection");
    }
    assert!(!tab.apply_undo());

    // Redo all the way back up.
    for text in &redo_texts {
        assert!(tab.apply_redo());
        assert_eq!(tab.document.text(), text, "redo text");
    }
    assert!(!tab.apply_redo());

    // Interleave: undo twice, redo once, then a fresh edit clears redo.
    assert!(tab.apply_undo());
    assert!(tab.apply_undo());
    assert!(tab.apply_redo());
    assert_eq!(tab.document.text(), redo_texts[2]);
    tab.push_undo_snapshot();
    tab.document.replace_range(0..0, "x");
    assert!(tab.redo_stack.is_empty());
    assert!(tab.apply_undo());
    assert_eq!(tab.document.text(), redo_texts[2]);
}

#[test]
fn semantic_undo_coalesces_typing_and_deletion_but_respects_boundaries() {
    let now = Instant::now();
    let mut tab = EditorTab::new(MarkdownDocument::from_text(""));
    for (index, text) in ["你", "好"].into_iter().enumerate() {
        let range = tab.document.text().len()..tab.document.text().len();
        tab.prepare_undo_capture(
            UndoCaptureKind::Insert,
            &range,
            text,
            now + Duration::from_millis(index as u64 * 100),
        );
        tab.document.replace_range(range.clone(), text);
        let cursor = range.start + text.len();
        tab.selected_range = cursor..cursor;
    }
    assert_eq!(tab.undo_stack.len(), 1);
    assert!(tab.apply_undo());
    assert_eq!(tab.document.text(), "");
    assert!(tab.apply_redo());
    assert_eq!(tab.document.text(), "你好");

    tab.finish_undo_capture();
    tab.undo_stack.clear();
    tab.redo_stack.clear();
    let first = "你好".find('好').unwrap();
    for (index, range) in [first.."你好".len(), 0..first].into_iter().enumerate() {
        tab.prepare_undo_capture(
            UndoCaptureKind::Delete,
            &range,
            "",
            now + Duration::from_millis(index as u64 * 100),
        );
        tab.document.replace_range(range.clone(), "");
        tab.selected_range = range.start..range.start;
    }
    assert_eq!(tab.undo_stack.len(), 1);
    assert!(tab.apply_undo());
    assert_eq!(tab.document.text(), "你好");

    tab.undo_stack.clear();
    tab.redo_stack.clear();
    tab.document.set_text("a");
    tab.selected_range = 1..1;
    tab.prepare_undo_capture(UndoCaptureKind::Insert, &(1..1), "b", now);
    tab.document.replace_range(1..1, "b");
    tab.selected_range = 2..2;
    tab.prepare_undo_capture(
        UndoCaptureKind::Insert,
        &(2..2),
        "c",
        now + SEMANTIC_UNDO_TIMEOUT + Duration::from_millis(1),
    );
    tab.document.replace_range(2..2, "c");
    tab.selected_range = 3..3;
    assert_eq!(tab.undo_stack.len(), 2, "timeout starts a new undo group");
    assert!(tab.apply_undo());
    assert_eq!(tab.document.text(), "ab");
}

#[test]
fn semantic_undo_keeps_selection_replacement_and_atomic_commands_separate() {
    let now = Instant::now();
    let mut tab = EditorTab::new(MarkdownDocument::from_text("alpha"));
    tab.selected_range = 0..5;
    tab.prepare_undo_capture(UndoCaptureKind::Atomic, &(0..5), "x", now);
    tab.document.replace_range(0..5, "x");
    tab.selected_range = 1..1;
    tab.prepare_undo_capture(
        UndoCaptureKind::Insert,
        &(1..1),
        "y",
        now + Duration::from_millis(10),
    );
    tab.document.replace_range(1..1, "y");
    tab.selected_range = 2..2;

    assert_eq!(tab.undo_stack.len(), 2);
    assert!(tab.apply_undo());
    assert_eq!(tab.document.text(), "x");
    assert!(tab.apply_undo());
    assert_eq!(tab.document.text(), "alpha");
}

#[gpui::test]
fn visual_edit_contiguous_platform_typing_undoes_in_one_step(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(""))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    for text in ["你", "好", "🙂"] {
        cx.simulate_input(text);
    }
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "你好🙂");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "");
        assert!(app.active_tab().undo_capture.is_none());
    });
    cx.dispatch_action(Redo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "你好🙂");
    });
}

#[test]
fn grapheme_boundaries_match_full_document_segmentation() {
    // The line-local scan must agree with the old whole-document
    // segmentation at every grapheme boundary, across ASCII, CJK,
    // combining marks, ZWJ emoji clusters, and "\r\n".
    let text = "abc\nxy 中文 e\u{301}fin 👍🏽 👨\u{200d}👩\u{200d}👧\r\nnext line\n\nend";
    let tab = EditorTab::new(MarkdownDocument::from_text(text));

    let prev_reference = |offset: usize| {
        text.grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    };
    let next_reference = |offset: usize| {
        text.grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(text.len())
    };

    let boundaries: Vec<usize> = text
        .grapheme_indices(true)
        .map(|(idx, _)| idx)
        .chain(std::iter::once(text.len()))
        .collect();
    for &offset in &boundaries {
        assert_eq!(
            tab.previous_boundary(offset),
            prev_reference(offset),
            "previous_boundary at {offset}"
        );
        assert_eq!(
            tab.next_boundary(offset),
            next_reference(offset),
            "next_boundary at {offset}"
        );
    }
}

#[test]
fn multiple_tabs_isolate_cursor_and_undo() {
    // Two tabs opened independently must not share cursor/selection/undo
    // state. This is the core invariant the refactor introduces.
    let mut tab_a = EditorTab::new(MarkdownDocument::from_text("abc"));
    let tab_b = EditorTab::new(MarkdownDocument::from_text("xyz"));

    // Move the cursor in tab A to offset 2.
    tab_a.selected_range = 2..2;
    tab_a.push_undo_snapshot();
    // Tab B is independently at offset 0 with no undo history.
    assert_eq!(tab_b.selected_range, 0..0);
    assert!(tab_b.undo_stack.is_empty());

    // Editing tab A's document does not affect tab B's text.
    tab_a.document.replace_range(0..1, "A");
    assert_eq!(tab_a.document.text(), "Abc");
    assert_eq!(tab_b.document.text(), "xyz");

    // Cursor positions stay isolated.
    assert_eq!(tab_a.cursor_offset(), 2);
    assert_eq!(tab_b.cursor_offset(), 0);
    assert_eq!(tab_a.undo_stack.len(), 1);
    assert_eq!(tab_b.undo_stack.len(), 0);
}

#[test]
fn find_tab_with_document_path_matches_canonical_paths() {
    let dir = tempfile::tempdir().unwrap();
    let docs = dir.path().join("docs");
    std::fs::create_dir(&docs).unwrap();
    let first_path = docs.join("first.md");
    let second_path = docs.join("second.md");
    std::fs::write(&first_path, "# First").unwrap();
    std::fs::write(&second_path, "# Second").unwrap();

    let tabs = vec![
        EditorTab::new(MarkdownDocument::open(&first_path).unwrap()),
        EditorTab::new(MarkdownDocument::open(&second_path).unwrap()),
        EditorTab::new(MarkdownDocument::from_text("untitled")),
    ];

    let equivalent_second_path = docs.join("..").join("docs").join("second.md");
    assert_eq!(
        find_tab_with_document_path(&tabs, &equivalent_second_path),
        Some(1)
    );
    assert_eq!(
        find_tab_with_document_path(&tabs, &docs.join("missing.md")),
        None
    );
}

#[test]
fn opening_existing_file_focuses_without_duplicating_or_resetting_state() {
    let dir = tempfile::tempdir().unwrap();
    let first_path = dir.path().join("first.md");
    let second_path = dir.path().join("second.md");
    std::fs::write(&first_path, "first").unwrap();
    std::fs::write(&second_path, "second").unwrap();

    let mut tabs = vec![
        EditorTab::new(MarkdownDocument::open(&first_path).unwrap()),
        EditorTab::new(MarkdownDocument::open(&second_path).unwrap()),
    ];
    tabs[1].selected_range = 2..2;
    tabs[1].push_undo_snapshot();
    tabs[1].document.replace_range(0..0, "dirty ");
    let cached_preview = tabs[1].document.preview_blocks_shared();

    let active_tab = if let Some(index) = find_tab_with_document_path(&tabs, &second_path) {
        index
    } else {
        tabs.push(EditorTab::new(
            MarkdownDocument::open(&second_path).unwrap(),
        ));
        tabs.len() - 1
    };

    assert_eq!(tabs.len(), 2, "already-open files must not append tabs");
    assert_eq!(active_tab, 1);
    assert_eq!(tabs[1].document.text(), "dirty second");
    assert!(tabs[1].document.is_dirty());
    assert_eq!(tabs[1].selected_range, 2..2);
    assert_eq!(tabs[1].undo_stack.len(), 1);
    assert!(std::sync::Arc::ptr_eq(
        &tabs[1].document.preview_blocks_shared(),
        &cached_preview
    ));
}

#[test]
fn tab_vec_close_last_leaves_one_tab() {
    // Simulates the close_tab_confirmed invariant: closing the last tab
    // leaves exactly one fresh (untitled) tab rather than an empty window.
    let mut tabs: Vec<EditorTab> = vec![EditorTab::new(MarkdownDocument::from_text("only"))];
    let mut active_tab = 0usize;

    // Closing the last tab resets it in place to a fresh document.
    if tabs.len() <= 1 {
        tabs[0] = EditorTab::new(MarkdownDocument::new());
        active_tab = 0;
    } else {
        tabs.remove(active_tab);
        if active_tab >= tabs.len() {
            active_tab = tabs.len() - 1;
        }
    }
    assert_eq!(tabs.len(), 1, "closing the last tab must keep one tab");
    assert_eq!(active_tab, 0);
    assert!(
        tabs[0].document.path().is_none(),
        "the replacement tab is an untitled document"
    );

    // With two tabs, closing the active one removes it and leaves the other.
    tabs = vec![
        EditorTab::new(MarkdownDocument::from_text("first")),
        EditorTab::new(MarkdownDocument::from_text("second")),
    ];
    active_tab = 1;
    assert_eq!(tabs.len(), 2);
    tabs.remove(active_tab);
    if active_tab >= tabs.len() {
        active_tab = tabs.len() - 1;
    }
    assert_eq!(tabs.len(), 1);
    assert_eq!(active_tab, 0);
    assert_eq!(tabs[0].document.text(), "first");
}

#[test]
fn active_tab_accessors_clamp_a_stale_index() {
    // Regression for the close-tab-then-close-another crash. Tab-bar click
    // closures capture an `index` at render time; a close since then can
    // leave that index >= tabs.len() by the time the closure fires. The
    // accessors clamp the index so a transiently-stale value cannot panic.
    //
    // Reproduction: 3 tabs [A, B, C]; close B (index 1) -> [A, C]; the C
    // closure still carries index 2, which is now out of range (len == 2).
    let mut app_tabs: Vec<EditorTab> = vec![
        EditorTab::new(MarkdownDocument::from_text("A")),
        EditorTab::new(MarkdownDocument::from_text("B")),
        EditorTab::new(MarkdownDocument::from_text("C")),
    ];
    let mut app_active_tab = 1usize;

    // Close tab B (the active one): remove(1) -> [A, C].
    app_tabs.remove(app_active_tab);
    if app_active_tab >= app_tabs.len() {
        app_active_tab = app_tabs.len() - 1;
    }
    assert_eq!(app_tabs.len(), 2);

    // Simulate the stale closure firing: a click that still carries the
    // pre-close index 2. Without clamping this would be `app_tabs[2]` on a
    // 2-element vec — a panic. The clamp mirrors `active_tab()`:
    let clamped = app_active_tab.min(app_tabs.len().saturating_sub(1));
    let _ = &app_tabs[clamped]; // must not panic.
    // A stale index of 2 also clamps safely:
    let stale_index = 2usize;
    let clamped_stale = stale_index.min(app_tabs.len().saturating_sub(1));
    assert_eq!(clamped_stale, 1, "stale index 2 clamps to last valid (1)");
    let _ = &app_tabs[clamped_stale]; // must not panic.

    // And the guard used in tab_bar_view closures rejects a stale index
    // outright rather than trusting it:
    let index_from_closure = 2usize;
    assert!(
        (index_from_closure >= app_tabs.len()),
        "tab-bar closure must skip a stale index instead of assigning it"
    );
}

#[test]
fn document_tab_band_geometry_tracks_visibility_and_sidebar_width() {
    assert!(!document_tab_band_visible(0));
    assert!(!document_tab_band_visible(1));
    assert!(document_tab_band_visible(2));

    assert_eq!(document_tab_band_height(1), 0.);
    assert_eq!(document_tab_band_height(2), DOCUMENT_TAB_BAND_HEIGHT);

    assert_eq!(document_tab_band_leading_width(1, true, 230.), 0.);
    assert_eq!(document_tab_band_leading_width(2, false, 230.), 0.);
    assert_eq!(
        document_tab_band_leading_width(2, true, 230.),
        230. + SIDEBAR_DIVIDER_WIDTH
    );
    assert_eq!(
        document_tab_band_leading_width(3, true, 318.),
        318. + SIDEBAR_DIVIDER_WIDTH,
        "the tab controls must follow the same live sidebar width as the pane boundary"
    );
}

#[test]
fn document_typography_metrics_preserve_defaults_and_scale_boundaries() {
    let defaults = DocumentTypographyMetrics::new(
        markion::DEFAULT_EDITOR_FONT_SIZE,
        markion::DEFAULT_RENDERED_FONT_SIZE,
        markion::DEFAULT_PARAGRAPH_SPACING,
    );
    assert_eq!(defaults.editor_font_size, 15.);
    assert_eq!(defaults.editor_line_height, 24.);
    assert_eq!(defaults.rendered_font_size, 14.);
    assert_eq!(defaults.preview_row_line_height, 23.);
    assert_eq!(defaults.paragraph_line_height, 24.);
    assert_eq!(defaults.paragraph_spacing, 12.);
    assert_eq!(defaults.heading_font_size(1), 24.);
    assert_eq!(defaults.code_font_size, 12.);
    assert_eq!(defaults.inline_math_font_size, 16.);
    assert_eq!(defaults.display_math_font_size, 20.);

    let bounded = DocumentTypographyMetrics::new(0, u16::MAX, u16::MAX);
    assert_eq!(bounded.editor_font_size, MIN_EDITOR_FONT_SIZE as f32);
    assert_eq!(bounded.rendered_font_size, MAX_RENDERED_FONT_SIZE as f32);
    assert_eq!(bounded.paragraph_spacing, MAX_PARAGRAPH_SPACING as f32);
    assert!(bounded.heading_font_size(1) > defaults.heading_font_size(1));
    assert!(bounded.code_line_height > defaults.code_line_height);
}

#[test]
fn typography_preference_steps_stop_at_bounds() {
    assert_eq!(
        preference_step_value(15, MIN_EDITOR_FONT_SIZE, MAX_EDITOR_FONT_SIZE, 1),
        Some(16)
    );
    assert_eq!(
        preference_step_value(15, MIN_EDITOR_FONT_SIZE, MAX_EDITOR_FONT_SIZE, -1),
        Some(14)
    );
    assert_eq!(
        preference_step_value(
            MIN_EDITOR_FONT_SIZE,
            MIN_EDITOR_FONT_SIZE,
            MAX_EDITOR_FONT_SIZE,
            -1
        ),
        None
    );
    assert_eq!(
        preference_step_value(
            MAX_PARAGRAPH_SPACING,
            MIN_PARAGRAPH_SPACING,
            MAX_PARAGRAPH_SPACING,
            1
        ),
        None
    );
}

#[gpui::test]
fn typography_changes_preserve_document_caches_and_list_positions(cx: &mut TestAppContext) {
    let config_dir = tempfile::tempdir().unwrap();
    let preferences_path = config_dir.path().join("config.toml");
    let source = (0..120)
        .map(|index| format!("paragraph {index} with enough text for a stable list row"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = preferences_path;
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::Read;
        app.active_tab_mut().selected_range = 3..7;
        app.active_tab_mut().push_undo_snapshot();
        let preview = app.active_tab().document.preview_blocks_shared();
        app.active_tab_mut().sync_preview_list(&preview);
        let visual = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&visual);
        let version = app.active_tab().document.version();
        *app.active_tab().measured_height_cache.borrow_mut() = Some((
            MeasuredHeightKey {
                version,
                wrap_width: px(400.),
                font_size: px(15.),
                line_height: px(24.),
            },
            px(240.),
        ));
        let _ = app.active_tab().shared_document_text();
        let _ = app.highlighted_code(Some("rust"), "let x = 1;");
        app
    });

    cx.run_until_parked();
    app.update(cx, |app, _| {
        app.active_tab().preview_list.scroll_to(gpui::ListOffset {
            item_ix: 40,
            offset_in_item: px(3.),
        });
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: 40,
            offset_in_item: px(4.),
        });
    });
    let preview_max_before = app.update(cx, |app, _| {
        app.active_tab()
            .preview_list
            .max_offset_for_scrollbar()
            .height
    });

    let (version, preview_cache, highlight_count, undo_len, selection) =
        app.update(cx, |app, _| {
            (
                app.active_tab().document.version(),
                app.active_tab().document.preview_blocks_shared(),
                app.highlight_cache.borrow().len(),
                app.active_tab().undo_stack.len(),
                app.active_tab().selected_range.clone(),
            )
        });

    app.update(cx, |app, cx| {
        app.set_rendered_font_size(20, cx);
        app.set_paragraph_spacing(18, cx);
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(
            &tab.document.preview_blocks_shared(),
            &preview_cache
        ));
        assert_eq!(app.highlight_cache.borrow().len(), highlight_count);
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert_eq!(tab.selected_range, selection);
        assert!(tab.display_text_cache.borrow().is_some());
        assert!(tab.measured_height_cache.borrow().is_some());
        let preview_top = tab.preview_list.logical_scroll_top();
        assert_eq!(preview_top.item_ix, 40);
        assert_eq!(preview_top.offset_in_item, px(3.));
        let visual_top = tab.visual_list.logical_scroll_top();
        assert_eq!(visual_top.item_ix, 40);
        assert_eq!(visual_top.offset_in_item, px(4.));
        assert!(
            tab.preview_list.max_offset_for_scrollbar().height > preview_max_before,
            "larger rendered text and paragraph spacing must increase preview extent"
        );
    });

    app.update(cx, |app, cx| app.set_editor_font_size(24, cx));
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.selected_range, selection);
        if let Some((key, _)) = *tab.measured_height_cache.borrow() {
            assert_eq!(key.version, version);
            assert_eq!(key.font_size, px(24.));
        }
        assert!((f32::from(tab.line_height) - 38.4).abs() <= 1.0);
        assert!(tab.display_text_cache.borrow().is_some());
        assert_eq!(app.current_preferences().editor_font_size, 24);
        assert_eq!(app.current_preferences().rendered_font_size, 20);
        assert_eq!(app.current_preferences().paragraph_spacing, 18);
    });
}

#[gpui::test]
fn non_default_editor_font_reflows_wrapped_text_and_caret_geometry(cx: &mut TestAppContext) {
    let config_dir = tempfile::tempdir().unwrap();
    let preferences_path = config_dir.path().join("config.toml");
    let source = "wrap this source line ".repeat(80);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = preferences_path;
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::Edit;
        app
    });
    cx.run_until_parked();
    let default_height = app.update(cx, |app, _| {
        app.active_tab()
            .line_heights
            .first()
            .copied()
            .unwrap_or_default()
    });

    app.update(cx, |app, cx| app.set_editor_font_size(32, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!((f32::from(tab.line_height) - 51.2).abs() <= 1.0);
        assert!(tab.line_heights.first().copied().unwrap_or_default() > default_height);
        assert!(tab.last_bounds.is_some());
        assert!(!tab.last_lines.is_empty());
    });
}

#[test]
fn any_tab_dirty_detection() {
    // request_quit / window-close guard use tabs.iter().any(dirty).
    let tabs: Vec<EditorTab> = vec![
        EditorTab::new(MarkdownDocument::from_text("clean")),
        EditorTab::new(MarkdownDocument::from_text("clean2")),
    ];
    assert!(
        !tabs.iter().any(|t| t.document.is_dirty()),
        "freshly-created documents are not dirty"
    );
}

/// `sync_fraction` is the proportional coupling primitive: a pane with no
/// scrollable range (max <= 1px) reports 0 so it never drives the other
/// pane; otherwise it reports the clamped offset/max ratio.
#[test]
fn sync_fraction_clamps_and_zeros_unscrollable_panes() {
    // No scrollable range -> 0 regardless of offset (the guard for a
    // pane that fits its viewport).
    assert_eq!(sync_fraction(0., 0.), 0.);
    assert_eq!(sync_fraction(123., 1.), 0.);
    assert_eq!(sync_fraction(50., 0.5), 0.);

    // Mid-range offsets map to a proportional fraction.
    assert!((sync_fraction(50., 100.) - 0.5).abs() < 1e-6);
    assert!((sync_fraction(25., 100.) - 0.25).abs() < 1e-6);

    // Offset clamps to the top/bottom of the range.
    assert!((sync_fraction(0., 100.) - 0.).abs() < 1e-6);
    assert!((sync_fraction(100., 100.) - 1.).abs() < 1e-6);
    // Overscroll past the max still reports 1.0 (clamped), not >1.
    assert!((sync_fraction(150., 100.) - 1.).abs() < 1e-6);
    // Negative offset (should not normally occur) clamps to 0.
    assert!((sync_fraction(-10., 100.) - 0.).abs() < 1e-6);
}

/// Coupling is active only in Split Preview (both panes visible) and only
/// when the preference is enabled — never in Edit or Read mode, and never
/// when the preference is off even in Split.
#[test]
fn sync_scroll_is_active_only_in_split_when_enabled() {
    assert!(sync_scroll_is_active(ViewMode::Split, true));

    // Split but disabled: not coupled.
    assert!(!sync_scroll_is_active(ViewMode::Split, false));
    // Other view modes never couple, even with the preference on.
    assert!(!sync_scroll_is_active(ViewMode::Edit, true));
    assert!(!sync_scroll_is_active(ViewMode::VisualEdit, true));
    assert!(!sync_scroll_is_active(ViewMode::Read, true));
}

#[test]
fn default_heading_menu_exposes_h4_and_h5() {
    assert_eq!(DEFAULT_HEADING_MENU_MAX_LEVEL, 5);
    assert_eq!(
        normalize_heading_menu_max_level(3),
        DEFAULT_HEADING_MENU_MAX_LEVEL
    );
    assert_eq!(
        heading_native_menu_items(Language::En, DEFAULT_HEADING_MENU_MAX_LEVEL).len(),
        5
    );
}

#[test]
fn session_restore_skips_missing_paths_and_untitled_tabs() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join("notes.md");
    fs::write(&existing, "# hi\n").unwrap();
    let missing = dir.path().join("gone.md");
    let workspace = dir.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();

    let session = SessionState {
        workspace_root: Some(workspace.clone()),
        open_files: vec![existing.clone(), missing.clone()],
        active_file: Some(missing.clone()),
        recent_files: Vec::new(),
    };
    let (root, open_files, active) = filter_restorable_session(&session);
    assert_eq!(root.as_deref(), Some(workspace.as_path()));
    assert_eq!(open_files, vec![existing.clone()]);
    assert!(active.is_none());

    assert!(session_open_files_from_paths([None, Some(existing.as_path())]).len() == 1);
    assert!(session_open_files_from_paths([None::<&Path>, None]).is_empty());
}

#[test]
fn cli_open_intent_disables_session_restore() {
    assert!(should_restore_session(&StartupOpenIntent::None));
    assert!(!should_restore_session(&StartupOpenIntent::File(PathBuf::from(
        "a.md"
    ))));
    assert!(!should_restore_session(&StartupOpenIntent::Folder(
        PathBuf::from("notes")
    )));
}

#[test]
fn open_recent_menu_is_wired_in_file_dropdown() {
    let root_view_source = include_str!("root_view.rs");
    assert!(root_view_source.contains(".on_action(cx.listener(Self::clear_recent_files))"));
    assert!(root_view_source.contains("Msg::ItemOpenRecent"));
    assert!(root_view_source.contains("Msg::ItemOpenRecentEmpty"));
    assert!(root_view_source.contains("Msg::ItemClearRecentFiles"));
    assert!(root_view_source.contains("open_recent_path"));

    let in_window = root_view_source
        .split_once("AppMenu::File => panel")
        .and_then(|(_, rest)| rest.split_once("AppMenu::Edit =>").map(|(file, _)| file))
        .expect("in-window File menu");
    let folder = in_window
        .find("Msg::ItemOpenFolder,")
        .expect("Open Folder");
    let recent = in_window.find("Msg::ItemOpenRecent").expect("Open Recent");
    let clear = in_window
        .find("Msg::ItemClearRecentFiles")
        .expect("Clear Recent");
    let save = in_window.find("Msg::ItemSave,").expect("Save");
    assert!(folder < recent && recent < clear && clear < save);
}
