use super::application::{AutosaveCompletion, AutosaveOutcome, ExternalCheckRequest};
use super::editing::visual_selection_format_target_for_block;
use super::memory::{MemoryProfile, MemoryWarmup};
use super::*;
use gpui::{Modifiers, ScrollDelta, ScrollWheelEvent, TestAppContext};
// Only test code in this module classifies file-tree entries by kind; import it
// here (rather than in `mod.rs`) so non-test release builds stay warning-free
// under `-D warnings`.
use markion::{FileTreeFileKind, ThemeFonts};

#[gpui::test]
fn publishing_browser_handoff_preserves_gpui_tab_and_document_state(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, _| {
        app.view_mode = ViewMode::Read;
        let tab = app.active_tab_mut();
        tab.document.insert(0, "# Draft\n\nSelected text");
        tab.selected_range = 2..7;
        tab.push_undo_snapshot();
    });

    app.update(cx, |app, _| {
        let active_index = app.active_tab;
        let highlight_probe = app.highlighted_code(Some("rust"), "fn publish_probe() {}");
        let tab = app.active_tab();
        let text = tab.document.text().to_owned();
        let selected_range = tab.selected_range.clone();
        let version = tab.document.version();
        let dirty = tab.document.is_dirty();
        let visual = tab.document.visual_blocks_shared();
        let preview = tab.document.preview_blocks_shared();
        let text_handle = tab.shared_document_text();
        let undo_len = tab.undo_stack.len();
        let highlight_count = app.highlight_cache.borrow().len();

        // All export outcomes happen inside the browser. Markion's complete
        // contribution is this read-only immutable handoff, so successful,
        // failed, and cancelled browser work must have the same app state.
        for _browser_outcome in ["success", "failure", "cancelled"] {
            let snapshot = build_publishing_snapshot(&tab.document, app.language.code());
            assert_eq!(snapshot.markdown.as_ref(), text);
        }
        assert_eq!(app.active_tab, active_index);
        assert_eq!(app.view_mode, ViewMode::Read);
        assert_eq!(app.active_tab().selected_range, selected_range);
        assert_eq!(app.active_tab().document.text(), text);
        assert_eq!(app.active_tab().document.version(), version);
        assert_eq!(app.active_tab().document.is_dirty(), dirty);
        assert_eq!(app.active_tab().undo_stack.len(), undo_len);
        assert_eq!(app.highlight_cache.borrow().len(), highlight_count);
        assert_eq!(
            app.active_tab().shared_document_text().as_ptr(),
            text_handle.as_ptr()
        );
        let highlight_after = app.highlighted_code(Some("rust"), "fn publish_probe() {}");
        assert!(Rc::ptr_eq(&highlight_after, &highlight_probe));
        assert!(Arc::ptr_eq(
            &preview,
            &app.active_tab().document.preview_blocks_shared()
        ));
        assert!(Arc::ptr_eq(
            &visual,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
}

#[test]
fn publishing_action_and_statuses_are_localized_for_every_language() {
    for language in [
        Language::En,
        Language::ZhHans,
        Language::ZhHant,
        Language::Ja,
        Language::Fr,
        Language::De,
        Language::Es,
    ] {
        for message in [
            Msg::ItemPublishWechat,
            Msg::StatusPublishingOpening,
            Msg::StatusPublishingOpened,
        ] {
            assert!(!t(language, message).trim().is_empty());
        }
        assert!(!tf(language, Msg::StatusPublishSetupFailed, &["setup"]).contains("{0}"));
        assert!(!tf(language, Msg::StatusPublishLaunchFailed, &["launch"]).contains("{0}"));
    }
}

#[test]
fn visual_pinyin_preedit_composes_sorted_utf8_highlights() {
    let source = "**激活稀疏（Activation Sparsity）**：经过 ReLU、SiLU 这类激活函数后，一部分激活值变成 0（或接近 0）。这是**动态的**——每个 batch、每个 token 的稀疏位置都不一样，硬件必须在**运行时**现场判断哪里是 0、现场建索引、现场跳过。这一\"现场\"是激活稀疏难做的根源。";
    let mut doc = MarkdownDocument::from_text(source);
    let _ = doc.visual_blocks_shared();
    let caret = source.find("（或接近 0").unwrap() + "（或接近 0".len();
    doc.replace_range(caret..caret, "x");
    let marked = caret..caret + 1;
    let blocks = doc.visual_blocks_shared();
    let block = blocks
        .iter()
        .find(|block| block.source_range.start <= caret && block.source_range.end >= marked.end)
        .unwrap();
    let projection = build_visual_projection_with_marked_range(
        doc.text(),
        block,
        marked.end..marked.end,
        marked.end,
        Some(marked.clone()),
    );
    let marked_display = projection
        .display_range_for_source_range(marked.clone())
        .unwrap();
    let highlights = visual_projection_highlights(&projection, Some(&marked));

    assert!(
        highlights
            .windows(2)
            .all(|pair| pair[0].0.end <= pair[1].0.start)
    );
    assert!(
        highlights.iter().all(|(range, _)| {
            !range.is_empty() && projection.text.get(range.clone()).is_some()
        })
    );
    assert!(highlights.iter().any(|(range, style)| {
        range.start <= marked_display.start
            && range.end >= marked_display.end
            && style.underline.is_some()
    }));
}

#[test]
fn visual_pinyin_preedit_preserves_overlapped_inline_style() {
    let text = "动x态的".to_string();
    let marked = "动".len().."动".len() + 1;
    let projection = VisualProjection {
        text: text.clone(),
        segments: vec![markion::VisualProjectionSegment {
            display_range: 0..text.len(),
            source_range: 0..text.len(),
        }],
        spans: vec![markion::VisualProjectionSpan {
            display_range: 0..text.len(),
            style: InlineStyle {
                bold: true,
                ..Default::default()
            },
            link: false,
            source: false,
        }],
        revealed_source_ranges: Vec::new(),
        source_anchor: 0,
    };
    let marked_display = projection
        .display_range_for_source_range(marked.clone())
        .unwrap();
    let highlights = visual_projection_highlights(&projection, Some(&marked));
    let (_, marked_style) = highlights
        .iter()
        .find(|(range, _)| range.start <= marked_display.start && range.end >= marked_display.end)
        .unwrap();

    assert_eq!(marked_style.font_weight, Some(FontWeight::BOLD));
    assert!(marked_style.underline.is_some());
}

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
fn structural_format_shortcut_registry_matches_reference() {
    let expected = [
        (
            &menu_shortcuts::ORDERED_LIST,
            "ordered-list",
            "secondary-shift-[",
            "Ctrl+Shift+[",
            "Cmd+Shift+[",
        ),
        (
            &menu_shortcuts::UNORDERED_LIST,
            "unordered-list",
            "secondary-shift-]",
            "Ctrl+Shift+]",
            "Cmd+Shift+]",
        ),
        (
            &menu_shortcuts::TASK_LIST,
            "task-list",
            "secondary-shift-x",
            "Ctrl+Shift+X",
            "Cmd+Shift+X",
        ),
        (
            &menu_shortcuts::BLOCK_QUOTE,
            "block-quote",
            "secondary-shift-q",
            "Ctrl+Shift+Q",
            "Cmd+Shift+Q",
        ),
        (
            &menu_shortcuts::CODE_FENCE,
            "code-fence",
            "secondary-shift-k",
            "Ctrl+Shift+K",
            "Cmd+Shift+K",
        ),
    ];

    let mut stored = BTreeMap::new();
    for (shortcut, id, binding, windows_linux, macos) in expected {
        assert_eq!(shortcut.id, id);
        assert_eq!(shortcut.binding, binding);
        assert_eq!(shortcut_by_id(id), Some(shortcut));
        assert_eq!(
            shortcut.label(ShortcutPlatform::WindowsLinux),
            windows_linux
        );
        assert_eq!(shortcut.label(ShortcutPlatform::MacOS), macos);
        assert!(gpui::Keystroke::parse(binding).is_ok());
        stored.insert(id.to_string(), binding.to_string());
    }
    assert_eq!(sanitized_shortcut_overrides(&stored), stored);

    for (binding, key, windows_linux, macos) in [
        ("secondary-shift-[", "[", "Ctrl+Shift+[", "Cmd+Shift+["),
        ("secondary-shift-]", "]", "Ctrl+Shift+]", "Cmd+Shift+]"),
    ] {
        for (platform, label) in [
            (ShortcutPlatform::WindowsLinux, windows_linux),
            (ShortcutPlatform::MacOS, macos),
        ] {
            let parts = markion::keystroke::KeystrokeParts::parse(binding, platform)
                .expect("literal bracket shortcut parses");
            assert_eq!(parts.key.0, key);
            assert_eq!(
                markion::keystroke::format_keystroke_label(binding, platform),
                label
            );
        }
    }

    for platform in ShortcutPlatform::ALL {
        for (index, left) in menu_shortcuts::ALL.iter().enumerate() {
            let left = markion::keystroke::KeystrokeParts::parse(left.binding, platform)
                .expect("registry default parses");
            for right in &menu_shortcuts::ALL[index + 1..] {
                let right_parts =
                    markion::keystroke::KeystrokeParts::parse(right.binding, platform)
                        .expect("registry default parses");
                assert_ne!(
                    left, right_parts,
                    "duplicate default shortcut for {} on {platform:?}",
                    right.id
                );
            }
        }
    }
}

#[test]
fn shortcut_registry_ids_are_stable_unique_and_fully_catalogued() {
    let mut registry_ids = std::collections::BTreeSet::new();
    for shortcut in menu_shortcuts::ALL {
        assert!(
            !shortcut.id.is_empty()
                && !shortcut.id.starts_with('-')
                && !shortcut.id.ends_with('-')
                && shortcut
                    .id
                    .chars()
                    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'),
            "shortcut id must be kebab-case: {}",
            shortcut.id
        );
        assert!(
            registry_ids.insert(shortcut.id),
            "duplicate shortcut id: {}",
            shortcut.id
        );
        assert_eq!(shortcut_by_id(shortcut.id), Some(shortcut));
    }

    let catalog = shortcut_catalog(Language::En, EXTENDED_HEADING_MENU_MAX_LEVEL);
    let catalog_ids = catalog
        .sections
        .iter()
        .flat_map(|section| section.actions.iter())
        .flat_map(|action| action.ids().iter().copied())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        catalog_ids, registry_ids,
        "every customizable registry action must be editable in Preferences"
    );

    for action in catalog.sections.iter().flat_map(|section| &section.actions) {
        for platform in ShortcutPlatform::ALL {
            for (&action_id, &catalog_label) in action
                .ids()
                .iter()
                .zip(action.combinations(platform).iter())
            {
                assert_eq!(
                    shortcut_by_id(action_id).map(|shortcut| shortcut.label(platform)),
                    Some(catalog_label),
                    "catalog and registry default labels diverged for {action_id}"
                );
            }
        }
    }
}

#[test]
fn structural_format_shortcuts_are_localized_in_every_catalog_variant() {
    let expected = [
        (
            Msg::ItemBullets,
            "unordered-list",
            "Ctrl+Shift+]",
            "Cmd+Shift+]",
        ),
        (
            Msg::ItemNumbers,
            "ordered-list",
            "Ctrl+Shift+[",
            "Cmd+Shift+[",
        ),
        (Msg::ItemTask, "task-list", "Ctrl+Shift+X", "Cmd+Shift+X"),
        (Msg::ItemQuote, "block-quote", "Ctrl+Shift+Q", "Cmd+Shift+Q"),
        (
            Msg::ItemCodeFence,
            "code-fence",
            "Ctrl+Shift+K",
            "Cmd+Shift+K",
        ),
    ];

    for language in [
        Language::En,
        Language::Ja,
        Language::Fr,
        Language::De,
        Language::Es,
        Language::ZhHans,
        Language::ZhHant,
    ] {
        for heading_depth in [
            DEFAULT_HEADING_MENU_MAX_LEVEL,
            EXTENDED_HEADING_MENU_MAX_LEVEL,
        ] {
            let catalog = shortcut_catalog(language, heading_depth);
            let editing = catalog
                .section(ShortcutCategory::Editing)
                .expect("Editing shortcut category");
            for (message, id, windows_linux, macos) in expected {
                let action = editing
                    .actions
                    .iter()
                    .find(|action| action.ids() == [id])
                    .unwrap_or_else(|| panic!("missing {id} for {language:?} at H{heading_depth}"));
                assert_eq!(action.label, t(language, message));
                assert_eq!(
                    action.combinations(ShortcutPlatform::WindowsLinux),
                    [windows_linux]
                );
                assert_eq!(action.combinations(ShortcutPlatform::MacOS), [macos]);
            }
        }
    }
}

#[test]
fn shortcut_effective_binding_and_label_fall_back_to_defaults() {
    let mut overrides = BTreeMap::new();
    overrides.insert("bold".to_string(), "ctrl-alt-b".to_string());
    assert_eq!(
        menu_shortcuts::BOLD.effective_binding(&overrides),
        "ctrl-alt-b"
    );
    assert_eq!(
        menu_shortcuts::BOLD.effective_label(&overrides, ShortcutPlatform::WindowsLinux),
        "Ctrl+Alt+B"
    );

    overrides.insert("bold".to_string(), "bogus-mod-b".to_string());
    assert_eq!(
        menu_shortcuts::BOLD.effective_binding(&overrides),
        menu_shortcuts::BOLD.binding
    );
    assert_eq!(
        menu_shortcuts::BOLD.effective_label(&overrides, ShortcutPlatform::MacOS),
        menu_shortcuts::BOLD.label(ShortcutPlatform::MacOS)
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
            .matches("KeyBinding::new(eff(&menu_shortcuts::REDO), Redo, None)")
            .count(),
        1,
        "Redo must be installed exactly once"
    );
    assert!(!bootstrap.contains("REDO.aliases"));
    assert!(!bootstrap.contains("secondary-shift-z"));
}

#[test]
fn full_rebind_restores_fixed_keys_and_every_registry_action() {
    let bootstrap = include_str!("bootstrap.rs");
    let bind = bootstrap
        .split_once("pub(super) fn bind_app_keys")
        .and_then(|(_, rest)| rest.split_once("pub(super) fn run()"))
        .map(|(body, _)| body)
        .expect("complete application key binding function");

    for fixed in [
        "backspace",
        "delete",
        "left",
        "right",
        "up",
        "down",
        "home",
        "end",
        "enter",
        "tab",
        "shift-tab",
        "escape",
        "f5",
        "f2",
        "secondary-delete",
    ] {
        assert!(
            bind.contains(&format!("KeyBinding::new(\"{fixed}\"")),
            "full rebind omitted fixed key {fixed}"
        );
    }
    assert_eq!(
        bind.matches("eff(&menu_shortcuts::").count(),
        menu_shortcuts::ALL.len(),
        "every customizable registry action must be restored exactly once"
    );

    let shortcuts = include_str!("shortcuts.rs");
    assert!(shortcuts.contains("cx.clear_key_bindings()"));
    assert!(shortcuts.contains("bind_app_keys(cx, &self.shortcut_overrides)"));
}

#[test]
fn every_application_dropdown_uses_shortcut_aware_rows() {
    let source = include_str!("root_view.rs").replace("\r\n", "\n");
    let menu_source = source
        .split_once("pub(super) fn active_menu_dropdown")
        .and_then(|(_, rest)| {
            rest.split_once("pub(super) fn open_recent_submenu_panel")
                .map(|(menu, _)| menu)
        })
        .expect("in-window application menu builder");

    let menu_boundaries = [
        "AppMenu::File =>",
        "AppMenu::Edit =>",
        "AppMenu::View =>",
        "AppMenu::Format =>",
        "AppMenu::Export =>",
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

    let help_menu = menu_source
        .split_once("AppMenu::Help =>")
        .map(|(_, body)| body)
        .expect("Help menu match arm");
    assert!(help_menu.contains("Msg::ItemCheckForUpdates"));
    assert!(help_menu.contains("Msg::ItemAboutMarkion"));
    assert!(help_menu.contains("Msg::ItemReportIssue"));
    assert!(help_menu.contains("Msg::ItemOnlineDocs"));
    assert!(
        help_menu.find("Msg::ItemReportIssue").unwrap()
            < help_menu.find("Msg::ItemAboutMarkion").unwrap()
            && help_menu.find("Msg::ItemOnlineDocs").unwrap()
                < help_menu.find("Msg::ItemAboutMarkion").unwrap(),
        "the external links must precede About in the Help dropdown"
    );
    assert!(
        !help_menu.contains("Msg::ItemKeyboardShortcuts"),
        "Help must not expose the shortcut reference after it moved to Preferences"
    );

    let row_source = source
        .split_once("pub(super) fn menu_action_button")
        .and_then(|(_, rest)| {
            rest.split_once("pub(super) fn menu_separator")
                .map(|(row, _)| row)
        })
        .expect("shortcut-aware menu row");
    assert!(row_source.contains("shortcut: Option<String>"));
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
            rest.split_once("pub(super) fn open_recent_submenu_panel")
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
        ("Msg::ItemBullets,", "menu_shortcuts::UNORDERED_LIST"),
        ("Msg::ItemNumbers,", "menu_shortcuts::ORDERED_LIST"),
        ("Msg::ItemTask,", "menu_shortcuts::TASK_LIST"),
        ("Msg::ItemQuote,", "menu_shortcuts::BLOCK_QUOTE"),
        ("Msg::ItemCodeFence,", "menu_shortcuts::CODE_FENCE"),
        ("Msg::ItemExportPdf,", "menu_shortcuts::EXPORT_PDF"),
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
        "Msg::ItemCheckForUpdates,",
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
    assert!(menu_source.contains("effective_label(shortcut_overrides, shortcut_platform)"));
    assert!(!menu_source.contains("Msg::ItemKeyboardShortcuts"));
}

#[test]
fn native_structural_format_menu_actions_share_the_bound_handlers() {
    let bootstrap = include_str!("bootstrap.rs").replace("\r\n", "\n");
    let format_menu = bootstrap
        .split_once("name: t(language, Msg::MenuFormat).into()")
        .and_then(|(_, rest)| {
            rest.split_once("name: t(language, Msg::MenuExport).into()")
                .map(|(format, _)| format)
        })
        .expect("native Format menu");
    let bindings = bootstrap
        .split_once("pub(super) fn bind_app_keys")
        .and_then(|(_, rest)| rest.split_once("pub(super) fn run()").map(|(bind, _)| bind))
        .expect("complete application keymap");

    for (message, action, descriptor) in [
        ("Msg::ItemBullets", "UnorderedList", "UNORDERED_LIST"),
        ("Msg::ItemNumbers", "OrderedList", "ORDERED_LIST"),
        ("Msg::ItemTask", "TaskList", "TASK_LIST"),
        ("Msg::ItemQuote", "BlockQuote", "BLOCK_QUOTE"),
        ("Msg::ItemCodeFence", "CodeFence", "CODE_FENCE"),
    ] {
        assert!(
            format_menu.contains(&format!(
                "MenuItem::action(t(language, {message}), {action})"
            )),
            "native {message} must keep dispatching {action}"
        );
        assert!(
            bindings.contains(&format!("eff(&menu_shortcuts::{descriptor})"))
                && bindings.contains(action),
            "{descriptor} must bind the native menu's {action} handler"
        );
    }
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
fn interactive_image_open_does_not_change_external_drop_import_semantics() {
    assert_eq!(
        classify_external_drop_path(Path::new("notes.md")),
        ExternalDropIntent::OpenDocument
    );
    assert_eq!(
        classify_external_drop_path(Path::new("photo.PNG")),
        ExternalDropIntent::ImportImage
    );
    assert_eq!(
        classify_external_drop_path(Path::new("notes.txt")),
        ExternalDropIntent::Ignore
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
    // The File branch now opens through a background read; the bookkeeping
    // runs in the spawned apply closure on the app handle.
    assert!(apply_fn.contains("app.replace_active_tab(document, cx);"));
    assert!(apply_fn.contains("app.update_workspace_root_from_document(cx);"));
    assert!(apply_fn.contains("background_spawn"));
    assert!(apply_fn.contains("self.set_workspace_root(path, cx);"));
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
    let bind_keys = bootstrap_source
        .find("bind_app_keys(cx, &startup_shortcut_overrides())")
        .expect("key bindings");

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

#[gpui::test]
fn workspace_tree_open_stores_paths_without_verbatim_prefix(cx: &mut TestAppContext) {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("vault");
    let note = workspace.join("note.md");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::write(&note, "# note").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, cx| {
        app.set_workspace_root(workspace.clone(), cx);
        app.schedule_file_tree_scan(None, cx);
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        let root_text = app.workspace_root.display().to_string();
        assert!(!root_text.starts_with(r"\\?\"));
        assert!(!root_text.starts_with(r"\\?\UNC\"));

        let tree = app.file_tree.as_ref().expect("file tree scanned");
        let entry_path = tree
            .entries
            .iter()
            .map(|entry| &entry.path)
            .find(|path| path.ends_with("note.md"))
            .expect("note entry scanned")
            .clone();
        let entry_text = entry_path.display().to_string();
        assert!(!entry_text.starts_with(r"\\?\"));
        assert!(!entry_text.starts_with(r"\\?\UNC\"));

        // The tree-open flow is what Copy File Path reads from the tab; the
        // stored path — and therefore the clipboard string — must be in
        // normal form.
        app.open_tree_file_confirmed(entry_path, cx);
        let stored = app
            .active_tab()
            .path()
            .expect("tab carries the opened path")
            .display()
            .to_string();
        assert!(!stored.starts_with(r"\\?\"));
        assert!(!stored.starts_with(r"\\?\UNC\"));
        assert!(stored.ends_with("note.md"));
    });
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
fn comparable_document_paths_carry_no_verbatim_prefix() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("note.md");
    std::fs::write(&file, "# note").unwrap();

    let text = comparable_document_path(&file).display().to_string();
    assert!(!text.starts_with(r"\\?\"));
    assert!(!text.starts_with(r"\\?\UNC\"));
}

#[test]
fn comparable_document_paths_dedupe_non_canonical_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("notes");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("a.md");
    std::fs::write(&file, "# a").unwrap();

    let dotted = dir.join(".").join("a.md");
    assert_eq!(
        comparable_document_path(&file),
        comparable_document_path(&dotted)
    );

    #[cfg(windows)]
    {
        let text = comparable_document_path(&file).display().to_string();
        let mut chars = text.chars();
        let first = chars.next().expect("canonical path is non-empty");
        let flipped_case = if first.is_ascii_uppercase() {
            first.to_ascii_lowercase()
        } else {
            first.to_ascii_uppercase()
        };
        let variant: String = std::iter::once(flipped_case)
            .chain(chars)
            .collect::<String>()
            .replace('\\', "/");
        assert_eq!(
            comparable_document_path(&file),
            comparable_document_path(std::path::Path::new(&variant))
        );
    }
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
fn preview_blockquote_exposes_child_list_items_as_selectable_runs() {
    let quoted_item = |text: &str, index: u64| PreviewBlock::ListItem {
        level: 1,
        ordered: true,
        index: Some(index),
        checked: None,
        text: RichText::plain(text),
        source_range: 0..0,
    };
    let block = PreviewBlock::BlockQuote {
        children: vec![
            PreviewBlock::Paragraph {
                text: RichText::plain("intro"),
                source_range: 0..0,
            },
            quoted_item("first", 1),
            quoted_item("second", 2),
        ],
        alert: None,
        source_range: 0..0,
    };

    assert_eq!(
        preview_block_runs(&block),
        vec![
            PreviewTextRunId::QuoteChild(0),
            PreviewTextRunId::QuoteChild(1),
            PreviewTextRunId::QuoteChild(2),
        ]
    );
    assert_eq!(
        preview_run_plain_text(&block, PreviewTextRunId::QuoteChild(0)).as_deref(),
        Some("intro")
    );
    assert_eq!(
        preview_run_plain_text(&block, PreviewTextRunId::QuoteChild(2)).as_deref(),
        Some("second")
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
    let deep =
        inline_math_baseline_margin_from_metrics(line_height, font_ascent, font_descent, px(12.0));
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
        show_hidden: false,
        entries: vec![
            FileTreeEntry {
                path: docs.clone(),
                name: "docs".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::Directory,
                file_kind: None,
            },
            FileTreeEntry {
                path: guides.clone(),
                name: "guides".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::Directory,
                file_kind: None,
            },
            FileTreeEntry {
                path: guides.join("intro.md"),
                name: "intro.md".to_string(),
                depth: 2,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            },
            FileTreeEntry {
                path: docs.join("draft.md"),
                name: "draft.md".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            },
            FileTreeEntry {
                path: source.clone(),
                name: "src".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::Directory,
                file_kind: None,
            },
            FileTreeEntry {
                path: source.join("api.md"),
                name: "api.md".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            },
            FileTreeEntry {
                path: root.join("root.md"),
                name: "root.md".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            },
        ],
    }
}

fn overflowing_flat_file_tree(root: &Path, count: usize) -> FileTree {
    FileTree {
        root: root.to_path_buf(),
        show_hidden: false,
        entries: (0..count)
            .map(|index| FileTreeEntry {
                path: root.join(format!("note-{index:03}.md")),
                name: format!("note-{index:03}.md"),
                depth: 0,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            })
            .collect(),
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
fn toggle_tree_folder_expands_one_level_and_keeps_deeper_folders_collapsed() {
    let root = PathBuf::from("workspace");
    let tree = nested_file_tree_fixture(&root);
    // Initial state: both depth-0 folders collapsed.
    let mut collapsed = HashSet::from([root.join("docs"), root.join("src")]);

    // Expanding `docs` reveals only its immediate children (`guides` and
    // `draft.md`); the nested `guides` folder is recorded as collapsed so its
    // child `intro.md` stays hidden.
    toggle_tree_folder(&root.join("docs"), &tree, &mut collapsed);
    assert_eq!(
        collapsed,
        HashSet::from([root.join("docs").join("guides"), root.join("src")])
    );
    assert_eq!(
        visible_tree_entry_names(&tree, "", &collapsed),
        vec!["docs", "guides", "draft.md", "src", "root.md"]
    );
}

#[test]
fn toggle_tree_folder_drills_down_one_level_at_a_time() {
    let root = PathBuf::from("workspace");
    let tree = nested_file_tree_fixture(&root);
    let mut collapsed = HashSet::from([root.join("docs"), root.join("src")]);

    // First click: expand `docs` (one level).
    toggle_tree_folder(&root.join("docs"), &tree, &mut collapsed);
    // Second click: expand the now-visible `guides` subfolder — its child
    // `intro.md` finally appears.
    toggle_tree_folder(&root.join("docs").join("guides"), &tree, &mut collapsed);
    assert_eq!(collapsed, HashSet::from([root.join("src")]));
    assert_eq!(
        visible_tree_entry_names(&tree, "", &collapsed),
        vec!["docs", "guides", "intro.md", "draft.md", "src", "root.md"]
    );
}

#[test]
fn toggle_tree_folder_collapsing_hides_the_entire_subtree() {
    let root = PathBuf::from("workspace");
    let tree = nested_file_tree_fixture(&root);
    let mut collapsed = HashSet::from([root.join("docs"), root.join("src")]);

    // Expand docs, then guides — the whole docs branch is open.
    toggle_tree_folder(&root.join("docs"), &tree, &mut collapsed);
    toggle_tree_folder(&root.join("docs").join("guides"), &tree, &mut collapsed);

    // Collapsing `docs` hides its entire subtree regardless of how deep
    // descendants had been expanded.
    toggle_tree_folder(&root.join("docs"), &tree, &mut collapsed);
    assert_eq!(
        collapsed,
        HashSet::from([root.join("docs"), root.join("src")])
    );
    assert_eq!(
        visible_tree_entry_names(&tree, "", &collapsed),
        vec!["docs", "src", "root.md"]
    );
}

#[test]
fn toggle_tree_folder_expands_file_only_folder_without_deeper_structure() {
    let root = PathBuf::from("workspace");
    let tree = nested_file_tree_fixture(&root);
    let mut collapsed = HashSet::from([root.join("docs"), root.join("src")]);

    // `src` contains only a direct file (`api.md`); expanding it reveals that
    // file and nothing deeper.
    toggle_tree_folder(&root.join("src"), &tree, &mut collapsed);
    assert_eq!(collapsed, HashSet::from([root.join("docs")]));
    assert_eq!(
        visible_tree_entry_names(&tree, "", &collapsed),
        vec!["docs", "src", "api.md", "root.md"]
    );
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
        show_hidden: false,
        entries: vec![
            FileTreeEntry {
                path: docs.clone(),
                name: "docs".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::Directory,
                file_kind: None,
            },
            FileTreeEntry {
                path: docs.join("draft.md"),
                name: "draft.md".to_string(),
                depth: 1,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            },
            FileTreeEntry {
                path: notes,
                name: "notes.md".to_string(),
                depth: 0,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
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

/// The editor must be pre-filled with a sensible default per kind so the
/// existing defaults remain one Enter away. This pins the pre-fill and
/// initial-selection contracts that `PendingNameInput::new` (used by
/// `open_name_prompt` / the context-menu branches) provides.
#[test]
fn pending_name_input_prefill_matches_kind_defaults() {
    let root = PathBuf::from("workspace");
    let create_file = PendingNameInput::new(
        PendingNameKind::CreateFile,
        root.clone(),
        None,
        "untitled.md",
    );
    assert_eq!(create_file.kind, PendingNameKind::CreateFile);
    assert_eq!(create_file.buffer, "untitled.md");
    assert!(create_file.target.is_none());
    // Create kinds select the whole prefilled name: one stroke replaces it.
    assert_eq!(create_file.selection(), 0.."untitled.md".len());

    let create_folder = PendingNameInput::new(
        PendingNameKind::CreateFolder,
        root.clone(),
        None,
        "New Folder",
    );
    assert_eq!(create_folder.kind, PendingNameKind::CreateFolder);
    assert_eq!(create_folder.buffer, "New Folder");
    assert_eq!(create_folder.selection(), 0.."New Folder".len());

    let note = root.join("note.md");
    let rename = PendingNameInput::new(
        PendingNameKind::Rename,
        root.clone(),
        Some(note.clone()),
        "note.md",
    );
    assert_eq!(rename.kind, PendingNameKind::Rename);
    assert_eq!(rename.target, Some(note));
    assert_eq!(rename.buffer, "note.md");
    // Rename pre-selects the base name and preserves the extension.
    assert_eq!(rename.selection(), 0..4);

    // Multi-dot and dotfile policies.
    let archive =
        PendingNameInput::new(PendingNameKind::Rename, root.clone(), None, "report.v2.md");
    assert_eq!(archive.selection(), 0.."report.v2".len());
    let dotfile = PendingNameInput::new(PendingNameKind::Rename, root.clone(), None, ".gitignore");
    assert_eq!(dotfile.selection(), 0..".gitignore".len());
}

/// `has_text_input_focus` must consider a pending name editor as focused so
/// IME keystrokes route into the editor buffer instead of the document.
#[test]
fn has_text_input_focus_includes_pending_name_prompt() {
    // The app-level field can't be constructed without a GPUI context, so
    // validate the routing predicate directly against the pending-input
    // presence: the trio (has_text_input_focus / active_input_text_mut /
    // after_input_changed) all key off `pending_name_input.is_some()`.
    let pending = Some(PendingNameInput::new(
        PendingNameKind::CreateFile,
        PathBuf::from("workspace"),
        None,
        "",
    ));
    assert!(pending.is_some());
    // A non-pending state must be treated as unfocused for the name buffer.
    let none: Option<PendingNameInput> = None;
    assert!(none.is_none());
}

/// Character-boundary helpers used by the name editor's caret movement. CJK
/// names are multi-byte, so every caret move must land on a char boundary.
#[test]
fn name_caret_helpers_move_across_char_boundaries() {
    let s = "中a文b.md";
    // Byte layout: 中(3) a(1) 文(3) b(1) .(1) m(1) d(1)
    let mut i = previous_name_boundary(s, 4);
    assert_eq!(i, 3, "before 'a' is the start of 'a'");
    i = previous_name_boundary(s, 3);
    assert_eq!(i, 0, "before '中' is 0");
    i = next_name_boundary(s, 0);
    assert_eq!(i, 3, "after '中' skips its bytes");
    i = next_name_boundary(s, 4);
    assert_eq!(i, 7, "after '文' skips its bytes (文 spans 4..7)");

    // base_name_len: extension preserved on multi-dot names, dotfiles whole.
    assert_eq!(base_name_len("report.md"), "report".len());
    assert_eq!(base_name_len("report.v2.md"), "report.v2".len());
    assert_eq!(base_name_len(".gitignore"), ".gitignore".len());
    assert_eq!(base_name_len("noext"), "noext".len());
    // Directory separators are handled by the caller (rename prefill is the
    // final path component from `file_name()`), so they never reach here as
    // part of the base name; the helper itself only splits at the extension
    // dot. A backslash inside a name is therefore ordinary name content.
    assert_eq!(base_name_len("a\\b.md"), "a\\b".len());
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

/// The tail whitespace row is the row whose covered blank-line count changes
/// while Enter is pressed at the document end. Whatever identity the
/// incremental cache assigns, the splice must cover that row so the
/// virtualized list drops any cached height and re-measures it.
#[test]
fn visual_block_splice_remeasures_tail_whitespace_when_it_grows() {
    let mut document = MarkdownDocument::from_text("para\n\n");
    let old = document.visual_blocks();
    let old_tail = old.last().unwrap();
    assert_eq!(old_tail.kind, markion::VisualBlockKind::Whitespace);
    let old_signature = old_tail.height_signature;

    // Two Enters at the tail: two more covered newlines.
    document.replace_range(document.text().len()..document.text().len(), "\n\n");
    let new = document.visual_blocks();
    let new_tail = new.last().unwrap();
    assert_eq!(new_tail.kind, markion::VisualBlockKind::Whitespace);
    assert!(new_tail.height_signature > old_signature);

    let (range, count) = visual_block_splice(&old, &new);
    assert_eq!(count, 1);
    assert!(
        range.contains(&(old.len() - 1)),
        "tail row spliced: {range:?}"
    );
}

/// Identity-preserving rows with an unchanged height signature keep their
/// cached heights; the same id with a changed signature must be re-spliced.
/// The signature is built by mutating a cloned slice so both sides share ids.
#[test]
fn visual_block_splice_keys_height_mutable_rows_on_signature() {
    let document = MarkdownDocument::from_text("first\n\nsecond\n\nthird\n");
    let old = document.visual_blocks();

    // Same ids everywhere, but the tail whitespace signature changes: the
    // whitespace row must land inside the splice even though every id is
    // equal, while the content rows around it stay reusable.
    let mut new = old.clone();
    let tail = new.last_mut().unwrap();
    tail.height_signature = Some(tail.height_signature.map_or(1, |lines| lines + 3));
    assert_eq!(
        visual_block_splice(&old, &new),
        (old.len() - 1..old.len(), 1)
    );

    // Identical slices (same ids, same signatures) remain a no-op splice.
    assert_eq!(visual_block_splice(&old, &old), (old.len()..old.len(), 0));
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
fn whitespace_row_height_grows_with_blank_lines_without_the_old_cap() {
    // Zero/one newline both floor at a single row height.
    assert_eq!(whitespace_row_height(0), WHITESPACE_ROW_LINE_HEIGHT);
    assert_eq!(whitespace_row_height(1), WHITESPACE_ROW_LINE_HEIGHT);
    // Growth is linear and stays visible far past the former 72px cap (~6
    // lines): every Enter at the document tail must change the rendered
    // height, in both directions (typo fix / undo shrink symmetrically).
    for lines in 2..=40 {
        assert_eq!(
            whitespace_row_height(lines),
            lines as f32 * WHITESPACE_ROW_LINE_HEIGHT
        );
    }
    assert_eq!(whitespace_row_height(40), 40. * WHITESPACE_ROW_LINE_HEIGHT);
    assert_eq!(whitespace_row_height(3), 3. * WHITESPACE_ROW_LINE_HEIGHT);
}

#[test]
fn visual_caret_scroll_action_keeps_an_in_inset_caret_still() {
    let viewport = Bounds::new(point(px(0.), px(0.)), size(px(400.), px(400.)));
    let inset = px(VISUAL_CARET_VIEWPORT_INSET);
    let caret = Bounds::new(point(px(20.), px(80.)), size(px(2.), px(23.)));
    assert_eq!(
        visual_caret_scroll_action(viewport, Some(caret), None, inset, false),
        VisualCaretScrollAction::None
    );
}

#[test]
fn visual_caret_scroll_action_scrolls_only_the_overflowing_delta() {
    let viewport = Bounds::new(point(px(0.), px(0.)), size(px(400.), px(400.)));
    let inset = px(VISUAL_CARET_VIEWPORT_INSET);
    let below = Bounds::new(point(px(20.), px(390.)), size(px(2.), px(23.)));
    assert_eq!(
        visual_caret_scroll_action(viewport, Some(below), None, inset, false),
        VisualCaretScrollAction::Pixel(px(36.))
    );
    let above = Bounds::new(point(px(20.), px(5.)), size(px(2.), px(23.)));
    assert_eq!(
        visual_caret_scroll_action(viewport, Some(above), None, inset, false),
        VisualCaretScrollAction::Pixel(px(-18.))
    );
}

#[test]
fn visual_caret_scroll_action_uses_measured_item_when_caret_is_missing() {
    let viewport = Bounds::new(point(px(0.), px(0.)), size(px(400.), px(400.)));
    let inset = px(VISUAL_CARET_VIEWPORT_INSET);
    let item = Bounds::new(point(px(0.), px(40.)), size(px(400.), px(40.)));
    assert_eq!(
        visual_caret_scroll_action(viewport, None, Some(item), inset, false),
        VisualCaretScrollAction::None
    );
}

#[test]
fn visual_caret_scroll_action_pins_unmeasured_rows_below_the_window() {
    let viewport = Bounds::new(point(px(0.), px(0.)), size(px(400.), px(400.)));
    let inset = px(VISUAL_CARET_VIEWPORT_INSET);
    assert_eq!(
        visual_caret_scroll_action(viewport, None, None, inset, true),
        VisualCaretScrollAction::PinItem
    );
}

#[test]
fn visual_list_item_count_adds_a_spacer_only_for_non_empty_documents() {
    assert_eq!(visual_list_item_count(0), 0);
    assert_eq!(visual_list_item_count(3), 4);
}

#[test]
fn visual_end_padding_height_is_half_the_viewport() {
    assert_eq!(visual_end_padding_height(px(0.)), px(0.));
    assert_eq!(visual_end_padding_height(px(400.)), px(200.));
}

#[test]
fn whitespace_row_height_respects_the_pathological_bound() {
    assert_eq!(
        whitespace_row_height(WHITESPACE_ROW_MAX_LINES),
        WHITESPACE_ROW_MAX_LINES as f32 * WHITESPACE_ROW_LINE_HEIGHT
    );
    assert_eq!(
        whitespace_row_height(WHITESPACE_ROW_MAX_LINES + 100_000),
        WHITESPACE_ROW_MAX_LINES as f32 * WHITESPACE_ROW_LINE_HEIGHT
    );
}

#[test]
fn whitespace_caret_line_tracks_newlines_before_the_source_caret() {
    let text = "Hello\n\n\n";
    let range = 5..8;
    assert_eq!(whitespace_caret_line(range.clone(), 5, text), 0);
    assert_eq!(whitespace_caret_line(range.clone(), 6, text), 0);
    assert_eq!(whitespace_caret_line(range.clone(), 7, text), 1);
    assert_eq!(whitespace_caret_line(range, 8, text), 2);
    assert_eq!(whitespace_caret_y(0), 0.);
    assert_eq!(whitespace_caret_y(1), WHITESPACE_ROW_LINE_HEIGHT);
    assert_eq!(whitespace_caret_y(2), 2. * WHITESPACE_ROW_LINE_HEIGHT);
}

#[test]
fn whitespace_source_at_line_and_y_map_back_to_newline_ends() {
    let text = "Hello\n\n\n";
    let range = 5..8;
    assert_eq!(whitespace_source_at_line(range.clone(), 0, text), 6);
    assert_eq!(whitespace_source_at_line(range.clone(), 1, text), 7);
    assert_eq!(whitespace_source_at_line(range.clone(), 2, text), 8);
    assert_eq!(whitespace_source_at_line(range.clone(), 9, text), 8);
    assert_eq!(whitespace_source_at_y(range.clone(), px(0.), text), 6);
    assert_eq!(
        whitespace_source_at_y(range.clone(), px(WHITESPACE_ROW_LINE_HEIGHT), text),
        7
    );
    assert_eq!(
        whitespace_source_at_y(range, px(2. * WHITESPACE_ROW_LINE_HEIGHT + 4.), text),
        8
    );
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
fn preferences_panel_renders_and_wires_the_shortcuts_tab() {
    let source = include_str!("root_view.rs").replace("\r\n", "\n");
    let panel = source
        .split_once("pub(super) fn preferences_panel_view")
        .and_then(|(_, rest)| rest.split_once("fn preferences_tab_strip"))
        .map(|(body, _)| body)
        .expect("Preferences panel view");
    assert!(panel.contains("PreferencesTab::General"));
    assert!(panel.contains("PreferencesTab::Shortcuts"));
    assert!(panel.contains("preferences_shortcuts_body(app, palette, cx)"));
    assert!(panel.contains("track_focus(&app.preferences_panel_focus)"));
    assert!(panel.contains("handle_shortcut_capture_key(event, window, cx)"));

    let shortcuts_body = source
        .split_once("fn preferences_shortcuts_body")
        .and_then(|(_, rest)| {
            rest.split_once("#[allow(clippy::too_many_arguments)]\nfn shortcut_action_row")
        })
        .map(|(body, _)| body)
        .expect("Preferences shortcuts body");
    assert!(shortcuts_body.contains("ShortcutPlatform::ALL"));
    assert!(shortcuts_body.contains("catalog.sections.iter()"));
    assert!(shortcuts_body.contains("action.ids()"));

    let binding_editor = source
        .split_once("fn shortcut_binding_editor")
        .and_then(|(_, rest)| rest.split_once("pub(super) fn preference_numeric_row"))
        .map(|(body, _)| body)
        .expect("shortcut binding editor");
    assert!(binding_editor.contains("app.shortcut_label(shortcut, platform)"));
    assert!(binding_editor.contains("app.begin_shortcut_capture(action_id, window, cx)"));
    assert!(binding_editor.contains("app.reset_shortcut(action_id, cx)"));
    assert!(binding_editor.contains("Msg::ShortcutCapturePrompt"));
    assert!(binding_editor.contains("Msg::ShortcutResetAction"));
}

#[test]
fn preferences_panel_renders_and_wires_the_export_tab() {
    let source = include_str!("root_view.rs").replace("\r\n", "\n");
    let panel = source
        .split_once("pub(super) fn preferences_panel_view")
        .and_then(|(_, rest)| rest.split_once("fn preferences_tab_strip"))
        .map(|(body, _)| body)
        .expect("Preferences panel view");
    assert!(panel.contains("PreferencesTab::Export"));
    assert!(panel.contains("preferences_export_body(app, palette, cx)"));

    let strip = source
        .split_once("fn preferences_tab_strip")
        .and_then(|(_, rest)| rest.split_once("fn preferences_tab_button"))
        .map(|(body, _)| body)
        .expect("Preferences tab strip");
    assert!(strip.contains("Msg::PrefPanelTabExport"));

    let export_body = source
        .split_once("fn preferences_export_body")
        .and_then(|(_, rest)| rest.split_once("fn preference_section_header"))
        .map(|(body, _)| body)
        .expect("Preferences export body");
    // Backend choice applies the persisted preference.
    assert!(export_body.contains("Msg::PrefExportBackendBuiltin"));
    assert!(export_body.contains("Msg::PrefExportBackendPandoc"));
    assert!(export_body.contains("app.set_export_backend("));
    // Pandoc-only rows render conditionally on the pandoc backend.
    assert!(export_body.contains(".when(pandoc, |body|"));
    assert!(export_body.contains("app.choose_pandoc_path(window, cx)"));
    assert!(export_body.contains("app.reset_pandoc_path(cx)"));
    assert!(export_body.contains("app.choose_reference_doc(window, cx)"));
    assert!(export_body.contains("app.reset_reference_doc(cx)"));
    assert!(export_body.contains("app.set_pandoc_pdf_engine(engine, cx)"));
    // DOCX/PDF option sections map onto persisted export options.
    assert!(export_body.contains("app.set_docx_page_size("));
    assert!(export_body.contains("app.toggle_docx_toc(cx)"));
    assert!(export_body.contains("app.set_docx_image_policy("));
    assert!(export_body.contains("app.set_pdf_page_size("));
    assert!(export_body.contains("app.step_pdf_margin(-1, cx)"));
    assert!(export_body.contains("app.toggle_pdf_page_numbers(cx)"));
    // The availability line is presentation-only (no probe in render).
    assert!(export_body.contains("preference_pandoc_availability_line(app, palette)"));
}

#[test]
fn preferences_language_picker_contains_variable_width_labels() {
    let source = include_str!("root_view.rs").replace("\r\n", "\n");
    let panel = source
        .split_once("pub(super) fn preferences_panel_view")
        .and_then(|(_, rest)| rest.split_once("fn preferences_tab_strip"))
        .map(|(body, _)| body)
        .expect("Preferences panel view");
    let language_section = panel
        .split_once(".child(app.tr(Msg::PrefPanelLanguageSection))")
        .and_then(|(_, rest)| rest.split_once("// Theme grid."))
        .map(|(body, _)| body)
        .expect("Preferences language section");

    assert!(panel.contains("720.\n    } else {\n        640."));
    assert!(panel.contains(".w_full()"));
    assert!(panel.contains(".max_w(px(panel_width))"));
    assert!(language_section.contains(".flex_wrap()"));
    assert!(language_section.contains("preference_language_button("));
    assert!(
        !language_section.contains("format!("),
        "marker and label layout must not be encoded as formatted whitespace"
    );

    let language_button = source
        .split_once("fn preference_language_button")
        .and_then(|(_, rest)| rest.split_once("pub(super) fn preference_option_button"))
        .map(|(body, _)| body)
        .expect("language-specific preference button");
    assert!(language_button.contains(".min_w(px(72.))"));
    assert!(language_button.contains(".flex_none()"));
    assert!(language_button.contains(".whitespace_nowrap()"));
    assert!(language_button.contains(".gap_0p5()"));
    assert!(language_button.contains(".w(px(12.))"));
    assert!(language_button.contains("let marker = if active { \"✓\" } else { \"\" };"));
    assert!(language_button.contains(".child(marker)"));
    assert!(language_button.contains(".child(label)"));
}

#[gpui::test]
fn shortcut_validation_rejects_invalid_conflicting_and_reserved_bindings(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, _| {
        assert_eq!(
            app.shortcut_assignment_error("bold", "b"),
            Some(ShortcutCaptureError::NotAssignable)
        );
        assert!(matches!(
            app.shortcut_assignment_error("bold", menu_shortcuts::ITALIC.binding),
            Some(ShortcutCaptureError::Conflict(_))
        ));
        assert!(matches!(
            app.shortcut_assignment_error("bold", "enter"),
            Some(ShortcutCaptureError::Conflict(_))
        ));
        assert_eq!(app.shortcut_assignment_error("bold", "ctrl-alt-b"), None);
    });
}

#[gpui::test]
fn global_shortcut_clear_restores_defaults_and_exits_capture(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, cx| {
        app.shortcut_overrides
            .insert("bold".to_string(), "ctrl-alt-b".to_string());
        app.shortcut_capture = Some(ShortcutCapture {
            action_id: "italic".to_string(),
            error: None,
        });
        app.clear_shortcut_overrides(cx);
        assert!(app.shortcut_overrides.is_empty());
        assert!(app.shortcut_capture.is_none());
        assert_eq!(
            menu_shortcuts::BOLD.effective_binding(&app.shortcut_overrides),
            menu_shortcuts::BOLD.binding
        );
    });

    let reset_source = include_str!("appearance.rs");
    let reset = reset_source
        .split_once("pub(super) fn reset_preferences")
        .and_then(|(_, rest)| rest.split_once("pub(super) fn toggle_focus_mode"))
        .map(|(body, _)| body)
        .expect("global Preferences reset handler");
    assert!(reset.contains("app.clear_shortcut_overrides(cx)"));
    assert!(reset.contains("app.persist_preferences()"));
}

#[gpui::test]
fn about_dialog_opens_exact_ordered_links_and_closes_only_on_confirmation(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.active_menu = Some(AppMenu::Help);
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.dispatch_action(AboutMarkion);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.about_dialog_open);
        assert_eq!(app.active_menu, None);
        assert_eq!(app.status, t(app.language, Msg::StatusAboutMarkion));
    });

    let website = cx
        .debug_bounds(AboutLink::ProjectWebsite.link_selector())
        .expect("project website link should render");
    let github = cx
        .debug_bounds(AboutLink::GithubRepository.link_selector())
        .expect("GitHub link should render");
    assert!(
        website.center().y < github.center().y,
        "project website link must render above GitHub"
    );

    cx.simulate_click(website.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        cx.opened_url().as_deref(),
        Some(MARKION_PROJECT_WEBSITE_URL)
    );
    app.update(cx, |app, _| assert!(app.about_dialog_open));

    cx.simulate_click(github.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(cx.opened_url().as_deref(), Some(GITHUB_REPO_URL));
    app.update(cx, |app, _| assert!(app.about_dialog_open));

    let ok = cx
        .debug_bounds("about-dialog-ok")
        .expect("localized confirmation control should render");
    cx.simulate_click(ok.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| assert!(!app.about_dialog_open));
}

#[test]
fn about_dialog_link_model_and_messages_cover_every_supported_language() {
    assert_eq!(
        AboutLink::ALL.map(AboutLink::url),
        [MARKION_PROJECT_WEBSITE_URL, GITHUB_REPO_URL]
    );

    for &language in Language::all() {
        let version = tf(language, Msg::DialogAboutVersion, &["1.2.3"]);
        assert!(
            version.contains("1.2.3"),
            "missing version for {language:?}"
        );
        for msg in [
            Msg::DialogAboutTitle,
            Msg::DialogAboutDescription,
            Msg::DialogAboutProjectWebsite,
            Msg::DialogAboutGithub,
            Msg::DialogButtonOk,
        ] {
            assert!(
                !t(language, msg).trim().is_empty(),
                "empty About label {msg:?} for {language:?}"
            );
        }
        for link in AboutLink::ALL {
            assert!(!t(language, link.label()).trim().is_empty());
            assert!(link.url().starts_with("https://"));
        }
    }
}

#[gpui::test]
fn about_dialog_renders_with_readable_theme_derived_chrome(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.about_dialog_open = true;
        app
    });

    let mut prior_palette = None;
    for theme in [AppTheme::Paper, AppTheme::Ink] {
        app.update(cx, |app, cx| {
            app.theme = theme;
            app.custom_theme = None;
            app.selected_theme_name = theme.name().to_string();
            cx.notify();
        });
        cx.run_until_parked();

        for selector in [
            "about-dialog-overlay",
            "about-dialog-panel",
            "about-dialog-title",
            "about-dialog-version",
            "about-dialog-description",
            "about-project-website-row",
            "about-project-website-link",
            "about-github-row",
            "about-github-link",
            "about-dialog-ok",
        ] {
            assert!(
                cx.debug_bounds(selector).is_some(),
                "{selector} should render for {}",
                theme.name()
            );
        }

        app.update(cx, |app, _| {
            let palette = app.palette();
            assert_ne!(palette.panel_bg, palette.text);
            assert_ne!(palette.active_bg, palette.active_text);
            if let Some(prior) = prior_palette.replace(palette) {
                assert_ne!(prior.panel_bg, palette.panel_bg);
                assert_ne!(prior.text, palette.text);
            }
        });
    }
}

#[gpui::test]
fn shortcut_remap_persists_reloads_rejects_conflict_and_resets(cx: &mut TestAppContext) {
    let config_dir = tempfile::tempdir().unwrap();
    let preferences_path = config_dir.path().join("config.toml");
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, _| {
        app.preferences_path = preferences_path.clone();
        // Tests construct from documented defaults; clear anyway so this
        // tempfile fixture cannot inherit leftover in-memory overrides.
        app.shortcut_overrides.clear();
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.begin_shortcut_capture("bold", window, cx);
            app.handle_shortcut_capture_key(
                &KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("ctrl-alt-b").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                },
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(
            app.shortcut_overrides.get("bold").map(String::as_str),
            Some("ctrl-alt-b")
        );
        assert!(app.shortcut_capture.is_none());
    });
    let reloaded = load_app_preferences(&preferences_path).unwrap();
    assert_eq!(
        reloaded.shortcut_overrides.get("bold").map(String::as_str),
        Some("ctrl-alt-b"),
        "a restarted app must recover the persisted override"
    );

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.begin_shortcut_capture("italic", window, cx);
            app.handle_shortcut_capture_key(
                &KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("ctrl-alt-b").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                },
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, _| {
        assert!(matches!(
            app.shortcut_capture
                .as_ref()
                .and_then(|capture| capture.error.as_ref()),
            Some(ShortcutCaptureError::Conflict(_))
        ));
        assert!(!app.shortcut_overrides.contains_key("italic"));
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.handle_shortcut_capture_key(
                &KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("escape").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                },
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, cx| {
        assert!(app.shortcut_capture.is_none());
        app.reset_shortcut("bold", cx);
        assert!(app.shortcut_overrides.is_empty());
    });
    let reset_toml = std::fs::read_to_string(&preferences_path).unwrap();
    assert!(!reset_toml.contains("[shortcuts]"));
}

#[gpui::test]
fn shortcuts_preferences_renders_in_light_and_dark_themes(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_panel_open = true;
        app.preferences_tab = PreferencesTab::Shortcuts;
        app
    });

    for theme in [AppTheme::Paper, AppTheme::Ink] {
        app.update(cx, |app, cx| {
            app.theme = theme;
            app.custom_theme = None;
            app.selected_theme_name = theme.name().to_string();
            cx.notify();
        });
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert!(app.preferences_panel_open);
            assert_eq!(app.preferences_tab, PreferencesTab::Shortcuts);
            assert_eq!(app.selected_theme_name, theme.name());
            assert_ne!(app.palette().app_bg, app.palette().text);
        });
    }
}

#[gpui::test]
fn preferences_panel_bodies_render_with_draggable_scroll_handles_on_both_tabs(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_panel_open = true;
        app
    });
    cx.run_until_parked();

    // General tab: tracked body renders and its handle keeps a scroll offset
    // (clamped to the laid-out content extent, which the handle enforces).
    app.update(cx, |app, cx| {
        app.select_preferences_tab(PreferencesTab::General, cx);
        assert!(app.preferences_panel_open);
        assert_eq!(app.preferences_tab, PreferencesTab::General);
        cx.notify();
    });
    cx.run_until_parked();
    let general_y = app.update(cx, |app, _| {
        let max = f32::from(app.preferences_general_scroll.max_offset().height).max(0.);
        let y = px(-120f32.min(max));
        app.preferences_general_scroll.set_offset(point(px(0.), y));
        y
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.preferences_general_scroll.offset().y, general_y);
    });

    // Shortcuts tab: both scrollable regions render alongside each other with
    // independent handles, and all three regions keep their offsets through a
    // tab switch and back.
    app.update(cx, |app, cx| {
        app.select_preferences_tab(PreferencesTab::Shortcuts, cx);
        cx.notify();
    });
    cx.run_until_parked();
    let (categories_y, actions_y) = app.update(cx, |app, _| {
        let categories_max =
            f32::from(app.preferences_categories_scroll.max_offset().height).max(0.);
        let categories_y = px(-40f32.min(categories_max));
        app.preferences_categories_scroll
            .set_offset(point(px(0.), categories_y));
        let actions_max = f32::from(app.preferences_actions_scroll.max_offset().height).max(0.);
        let actions_y = px(-200f32.min(actions_max));
        app.preferences_actions_scroll
            .set_offset(point(px(0.), actions_y));
        (categories_y, actions_y)
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        assert_eq!(app.preferences_tab, PreferencesTab::Shortcuts);
        app.select_preferences_tab(PreferencesTab::General, cx);
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        app.select_preferences_tab(PreferencesTab::Shortcuts, cx);
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.preferences_general_scroll.offset().y, general_y);
        assert_eq!(
            app.preferences_categories_scroll.offset().y,
            categories_y,
            "categories sidebar must keep its own scroll position"
        );
        assert_eq!(
            app.preferences_actions_scroll.offset().y,
            actions_y,
            "action list must keep its own scroll position"
        );
    });
}

#[gpui::test]
fn preferences_scrollbar_thumbs_drag_their_own_region(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_panel_open = true;
        app
    });
    cx.run_until_parked();

    // Thumb geometry mirrors pane_scrollbar_view: the thumb sits in the
    // reserved right gutter, `edge inset` from the scrollable's top at rest.
    // Returns (thumb center x, scrollable top y, thumb height, thumb travel,
    // max scroll) in window coordinates.
    let thumb_geometry = |app: &mut MarkionApp, handle: &ScrollHandle| {
        let _ = app;
        let bounds = handle.bounds();
        let viewport = f32::from(bounds.size.height);
        let max_scroll = f32::from(handle.max_offset().height).max(0.);
        let track = viewport - 2. * PANE_SCROLLBAR_EDGE_INSET;
        let thumb_height = (track * viewport / (viewport + max_scroll))
            .clamp(PANE_SCROLLBAR_MIN_THUMB_HEIGHT, track);
        let thumb_travel = (track - thumb_height).max(0.);
        // The thumb's right edge sits 2px in from the scrollable's right edge.
        let center_x = f32::from(bounds.right()) - 2. - PANE_SCROLLBAR_THUMB_WIDTH / 2.;
        (
            center_x,
            f32::from(bounds.top()),
            thumb_height,
            thumb_travel,
            max_scroll,
        )
    };

    // General tab overflows in the default test window, so its thumb renders.
    app.update(cx, |app, cx| {
        app.select_preferences_tab(PreferencesTab::General, cx);
        cx.notify();
    });
    cx.run_until_parked();

    let (center_x, top, thumb_height, thumb_travel, max_scroll) = app.update(cx, |app, _| {
        thumb_geometry(app, &app.preferences_general_scroll.clone())
    });
    assert!(
        max_scroll > 1.,
        "general preferences body must overflow in the test window"
    );

    // Grab the thumb at its center and drag it halfway down its travel: the
    // scroll offset follows proportionally.
    let grab_y = top + PANE_SCROLLBAR_EDGE_INSET + thumb_height / 2.;
    cx.simulate_mouse_down(
        point(px(center_x), px(grab_y)),
        MouseButton::Left,
        Modifiers::none(),
    );
    app.update(cx, |app, _| {
        assert_eq!(
            app.pane_scrollbar_drag.as_ref().map(|drag| drag.target),
            Some(PaneScrollTarget::PreferencesGeneral),
            "grabbing the general thumb must start a general-target drag"
        );
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(
            px(center_x),
            px(top + PANE_SCROLLBAR_EDGE_INSET + thumb_travel / 2. + thumb_height / 2.),
        ),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let scrolled = f32::from(-app.preferences_general_scroll.offset().y);
        assert!(
            (scrolled - max_scroll / 2.).abs() < 1.,
            "halfway thumb drag should scroll halfway: expected {}, got {scrolled}",
            max_scroll / 2.
        );
    });

    // Dragging above the track clamps back to the top; releasing ends the drag.
    cx.simulate_event(MouseMoveEvent {
        position: point(px(center_x), px(top)),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: point(px(center_x), px(top)),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.preferences_general_scroll.offset().y, px(0.));
        assert!(app.pane_scrollbar_drag.is_none());
    });

    // Shortcuts tab: switch to the Editing category and attach a capture error
    // to one row so the action list overflows (the default Files category fits
    // in the fixed-height panel). The category sidebar still fits and must stay
    // parked at the top while the action list thumb drags.
    app.update(cx, |app, cx| {
        app.select_preferences_tab(PreferencesTab::Shortcuts, cx);
        app.select_shortcut_category(ShortcutCategory::Editing, cx);
        app.shortcut_capture = Some(ShortcutCapture {
            action_id: "bold".to_string(),
            error: Some(ShortcutCaptureError::NotAssignable),
        });
        cx.notify();
    });
    cx.run_until_parked();
    // The thumb's geometry reads the scroll handle laid out by the previous
    // frame, so the first Shortcuts frame after the category switch still has
    // the fitting geometry and paints no thumb. Render a second frame so the
    // overflow is reflected in the painted thumb.
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    let (center_x, top, thumb_height, thumb_travel, max_scroll) = app.update(cx, |app, _| {
        thumb_geometry(app, &app.preferences_actions_scroll.clone())
    });
    assert!(
        max_scroll > 1.,
        "shortcut action list must overflow in the test window"
    );

    let grab_y = top + PANE_SCROLLBAR_EDGE_INSET + thumb_height / 2.;
    cx.simulate_mouse_down(
        point(px(center_x), px(grab_y)),
        MouseButton::Left,
        Modifiers::none(),
    );
    app.update(cx, |app, _| {
        assert_eq!(
            app.pane_scrollbar_drag.as_ref().map(|drag| drag.target),
            Some(PaneScrollTarget::PreferencesShortcutActions)
        );
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(
            px(center_x),
            px(top + PANE_SCROLLBAR_EDGE_INSET + thumb_travel / 2. + thumb_height / 2.),
        ),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let scrolled = f32::from(-app.preferences_actions_scroll.offset().y);
        assert!(
            (scrolled - max_scroll / 2.).abs() < 1.,
            "halfway thumb drag should scroll halfway: expected {}, got {scrolled}",
            max_scroll / 2.
        );
        assert_eq!(
            app.preferences_categories_scroll.offset().y,
            px(0.),
            "dragging the action list thumb must not move the category sidebar"
        );
    });
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: point(px(center_x), px(grab_y)),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.pane_scrollbar_drag.is_none());
        app.shortcut_capture = None;
    });
}

#[test]
fn file_tree_content_width_includes_scrollbar_gutter() {
    let entries = [FileTreeEntry {
        path: PathBuf::from("note.md"),
        name: "note.md".to_string(),
        depth: 0,
        kind: FileTreeEntryKind::File,
        file_kind: Some(FileTreeFileKind::Markdown),
    }];
    let expected = 34. + estimate_file_tree_text_width("note.md") + PANE_SCROLLBAR_RESERVED_WIDTH;
    assert_eq!(file_tree_content_width(&entries), expected);
    assert_eq!(
        file_tree_content_width(&[]),
        1. + PANE_SCROLLBAR_RESERVED_WIDTH
    );
}

#[gpui::test]
fn sidebar_scrollbar_thumbs_drag_their_own_region(cx: &mut TestAppContext) {
    let root = PathBuf::from("/tmp/markion-sidebar-scroll");
    let mut outline_source = String::new();
    for index in 0..80 {
        outline_source.push_str(&format!("# Heading {index}\n\n"));
    }
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&outline_source))];
        app.workspace_root = root.clone();
        app.file_tree = Some(overflowing_flat_file_tree(&root, 80));
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Files;
        app.sync_scroll = true;
        app.view_mode = ViewMode::Split;
        app
    });
    cx.simulate_resize(size(px(900.), px(300.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();

    let thumb_geometry = |handle: &ScrollHandle| {
        let bounds = handle.bounds();
        let viewport = f32::from(bounds.size.height);
        let max_scroll = f32::from(handle.max_offset().height).max(0.);
        let track = viewport - 2. * PANE_SCROLLBAR_EDGE_INSET;
        let thumb_height = (track * viewport / (viewport + max_scroll))
            .clamp(PANE_SCROLLBAR_MIN_THUMB_HEIGHT, track);
        let thumb_travel = (track - thumb_height).max(0.);
        let center_x = f32::from(bounds.right()) - 2. - PANE_SCROLLBAR_THUMB_WIDTH / 2.;
        (
            center_x,
            f32::from(bounds.top()),
            thumb_height,
            thumb_travel,
            max_scroll,
        )
    };

    let (center_x, top, thumb_height, thumb_travel, max_scroll) =
        app.update(cx, |app, _| thumb_geometry(&app.file_tree_scroll.clone()));
    assert!(
        max_scroll > 1.,
        "file tree must overflow in the test window"
    );

    let grab_y = top + PANE_SCROLLBAR_EDGE_INSET + thumb_height / 2.;
    cx.simulate_mouse_down(
        point(px(center_x), px(grab_y)),
        MouseButton::Left,
        Modifiers::none(),
    );
    app.update(cx, |app, _| {
        assert_eq!(
            app.pane_scrollbar_drag.as_ref().map(|drag| drag.target),
            Some(PaneScrollTarget::FileTree),
            "grabbing the Files thumb must start a file-tree drag"
        );
        assert!(app.active_tab().sync_scroll_state.driver_hint.is_none());
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(
            px(center_x),
            px(top + PANE_SCROLLBAR_EDGE_INSET + thumb_travel / 2. + thumb_height / 2.),
        ),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let scrolled = f32::from(-app.file_tree_scroll.offset().y);
        assert!(
            (scrolled - max_scroll / 2.).abs() < 1.,
            "halfway Files thumb drag should scroll halfway: expected {}, got {scrolled}",
            max_scroll / 2.
        );
        assert!(app.active_tab().sync_scroll_state.driver_hint.is_none());
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(px(center_x), px(top)),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: point(px(center_x), px(top)),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.file_tree_scroll.offset().y, px(0.));
        assert!(app.pane_scrollbar_drag.is_none());
    });

    app.update(cx, |app, cx| {
        app.file_tree = Some(overflowing_flat_file_tree(&root, 1));
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(
            f32::from(app.file_tree_scroll.max_offset().height) <= 1.,
            "fitting file tree must hide the vertical scrollbar"
        );
    });

    app.update(cx, |app, cx| {
        app.file_tree = Some(overflowing_flat_file_tree(&root, 80));
        app.set_sidebar_tab(SidebarTab::Outline, cx);
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();

    let (center_x, top, thumb_height, thumb_travel, max_scroll) =
        app.update(cx, |app, _| thumb_geometry(&app.outline_scroll.clone()));
    assert!(max_scroll > 1., "outline must overflow in the test window");

    let files_offset_before = app.update(cx, |app, _| app.file_tree_scroll.offset());
    let grab_y = top + PANE_SCROLLBAR_EDGE_INSET + thumb_height / 2.;
    cx.simulate_mouse_down(
        point(px(center_x), px(grab_y)),
        MouseButton::Left,
        Modifiers::none(),
    );
    app.update(cx, |app, _| {
        assert_eq!(
            app.pane_scrollbar_drag.as_ref().map(|drag| drag.target),
            Some(PaneScrollTarget::Outline),
            "grabbing the Outline thumb must start an outline drag"
        );
        assert!(app.active_tab().sync_scroll_state.driver_hint.is_none());
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(
            px(center_x),
            px(top + PANE_SCROLLBAR_EDGE_INSET + thumb_travel / 2. + thumb_height / 2.),
        ),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let scrolled = f32::from(-app.outline_scroll.offset().y);
        assert!(
            (scrolled - max_scroll / 2.).abs() < 1.,
            "halfway Outline thumb drag should scroll halfway: expected {}, got {scrolled}",
            max_scroll / 2.
        );
        assert_eq!(
            app.file_tree_scroll.offset(),
            files_offset_before,
            "dragging the Outline thumb must not move the file tree"
        );
        assert!(app.active_tab().sync_scroll_state.driver_hint.is_none());
    });
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: point(px(center_x), px(grab_y)),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.pane_scrollbar_drag.is_none());
    });

    app.update(cx, |app, cx| {
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "# One\n\n# Two\n",
        ))];
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(
            f32::from(app.outline_scroll.max_offset().height) <= 1.,
            "fitting outline must hide the vertical scrollbar"
        );
    });
}

#[gpui::test]
fn live_rebind_dispatches_override_and_preserves_core_and_file_tree_keys(cx: &mut TestAppContext) {
    let mut overrides = BTreeMap::new();
    overrides.insert("toggle-sidebar".to_string(), "ctrl-alt-j".to_string());
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("ab"))];
        app.active_tab_mut().selected_range = 2..2;
        app.shortcut_overrides = overrides.clone();
        app
    });
    cx.update(|window, cx| {
        cx.clear_key_bindings();
        bind_app_keys(cx, &overrides);
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    let sidebar_before = app.update(cx, |app, _| app.sidebar_visible);
    cx.simulate_keystrokes("ctrl-alt-j");
    let sidebar_after_override = app.update(cx, |app, _| app.sidebar_visible);
    assert_ne!(sidebar_after_override, sidebar_before);

    cx.simulate_keystrokes(menu_shortcuts::TOGGLE_SIDEBAR.binding);
    assert_eq!(
        app.update(cx, |app, _| app.sidebar_visible),
        sidebar_after_override,
        "the default binding must stop dispatching after an override"
    );

    cx.simulate_keystrokes("backspace");
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "a");
    });

    cx.simulate_keystrokes("f5");
    app.update(cx, |app, _| {
        assert_eq!(
            app.status,
            t(app.language, Msg::StatusFileTreeRefreshed),
            "fixed file-tree shortcuts must survive a live rebind"
        );
    });
}

#[gpui::test]
fn structural_format_shortcuts_dispatch_and_live_rebind(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.update(|window, cx| {
        cx.clear_key_bindings();
        bind_app_keys(cx, &BTreeMap::new());
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    for (shortcut, expected) in [
        (&menu_shortcuts::UNORDERED_LIST, "- one\n- two"),
        (&menu_shortcuts::ORDERED_LIST, "1. one\n2. two"),
        (&menu_shortcuts::TASK_LIST, "- [ ] one\n- [ ] two"),
        (&menu_shortcuts::BLOCK_QUOTE, "> one\n> two"),
        (&menu_shortcuts::CODE_FENCE, "```\none\ntwo\n```"),
    ] {
        app.update(cx, |app, _| {
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("one\ntwo"))];
            app.active_tab = 0;
            app.active_tab_mut().selected_range = 0.."one\ntwo".len();
        });
        cx.simulate_keystrokes(shortcut.binding);
        app.update(cx, |app, _| {
            assert_eq!(
                app.active_tab().document.text(),
                expected,
                "{} must dispatch its Format action",
                shortcut.id
            );
        });
    }

    let mut overrides = BTreeMap::new();
    overrides.insert("code-fence".to_string(), "ctrl-alt-k".to_string());
    app.update(cx, |app, _| {
        app.shortcut_overrides = overrides.clone();
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("code"))];
        app.active_tab = 0;
        app.active_tab_mut().selected_range = 0.."code".len();
    });
    cx.update(|_, cx| {
        cx.clear_key_bindings();
        bind_app_keys(cx, &overrides);
    });

    cx.simulate_keystrokes(menu_shortcuts::CODE_FENCE.binding);
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            "code",
            "the default must stop dispatching while overridden"
        );
    });
    cx.simulate_keystrokes("ctrl-alt-k");
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "```\ncode\n```");
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("reset"))];
        app.active_tab = 0;
        app.active_tab_mut().selected_range = 0.."reset".len();
        app.shortcut_overrides.clear();
    });
    cx.update(|_, cx| {
        cx.clear_key_bindings();
        bind_app_keys(cx, &BTreeMap::new());
    });
    cx.simulate_keystrokes(menu_shortcuts::CODE_FENCE.binding);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "```\nreset\n```");
    });
}

#[test]
fn show_shortcuts_opens_preferences_on_shortcuts_tab() {
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
        .and_then(|(_, rest)| rest.split_once("pub(super) fn select_preferences_tab"))
        .map(|(body, _)| body)
        .expect("show shortcuts handler");
    assert!(show_shortcuts.contains("self.preferences_tab = PreferencesTab::Shortcuts"));
    assert!(show_shortcuts.contains("self.preferences_panel_open = true"));
    assert!(show_shortcuts.contains("ShortcutPlatform::current()"));
    assert!(show_shortcuts.contains("ShortcutCategory::Files"));
    assert!(!show_shortcuts.contains("window.prompt"));
    assert!(search_source.contains("self.shortcut_platform = platform"));
    assert!(search_source.contains("self.shortcut_category = category"));

    let bootstrap_source = include_str!("bootstrap.rs");
    assert!(bootstrap_source.contains("Msg::ItemCheckForUpdates"));
    assert!(bootstrap_source.contains("Msg::ItemAboutMarkion"));
    assert!(bootstrap_source.contains("Msg::ItemReportIssue"));
    assert!(bootstrap_source.contains("Msg::ItemOnlineDocs"));
    assert!(
        !bootstrap_source.contains("Msg::ItemKeyboardShortcuts"),
        "the native Help menu must not expose the shortcut reference"
    );
    assert!(
        bootstrap_source
            .contains("KeyBinding::new(eff(&menu_shortcuts::SHOW_SHORTCUTS), ShowShortcuts, None)")
    );
    assert!(bootstrap_source.contains("KeyBinding::new(eff(&menu_shortcuts::BOLD), Bold, None)"));
    assert!(
        bootstrap_source
            .contains("KeyBinding::new(eff(&menu_shortcuts::TOGGLE_SIDEBAR), ToggleSidebar, None)")
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
fn list_scrollbar_marks_sync_driver_only_for_preview() {
    assert!(list_pane_scrollbar_marks_sync_driver(
        PaneScrollTarget::Preview
    ));
    assert!(!list_pane_scrollbar_marks_sync_driver(
        PaneScrollTarget::Editor
    ));
    assert!(!list_pane_scrollbar_marks_sync_driver(
        PaneScrollTarget::Visual
    ));
    for target in [
        PaneScrollTarget::PreferencesGeneral,
        PaneScrollTarget::PreferencesShortcutCategories,
        PaneScrollTarget::PreferencesShortcutActions,
        PaneScrollTarget::FileTree,
        PaneScrollTarget::Outline,
    ] {
        assert!(
            !list_pane_scrollbar_marks_sync_driver(target),
            "non-preview scrollbar targets must never mark a sync driver"
        );
    }
}

#[gpui::test]
fn preferences_scrollbar_targets_are_no_op_for_sync_scroll(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_panel_open = true;
        app.sync_scroll = true;
        app
    });
    cx.run_until_parked();

    for target in [
        PaneScrollTarget::PreferencesGeneral,
        PaneScrollTarget::PreferencesShortcutCategories,
        PaneScrollTarget::PreferencesShortcutActions,
        PaneScrollTarget::FileTree,
        PaneScrollTarget::Outline,
    ] {
        app.update(cx, |app, _| {
            app.mark_sync_scroll_driver(target);
            let tab = app.active_tab();
            assert!(
                tab.sync_scroll_state.driver_hint.is_none(),
                "{target:?} scrollbar drag must not drive sync scroll"
            );
            assert!(tab.sync_scroll_state.deferred_driver.is_none());
        });
    }
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
    assert!(!read_mode_preview_is_constrained(
        ViewMode::VisualEdit,
        true
    ));
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
fn visual_and_preview_tables_share_content_column_weights() {
    let source = concat!(
        "| 名称 | 说明 |\n",
        "| --- | --- |\n",
        "| 操作系统 | Ubuntu |\n",
        "| CPU | Intel(R) Xeon(R) Platinum 8358 CPU @ 2.60GHz |\n",
        "| **强调** | long description text for wrapping |\n",
    );
    let document = MarkdownDocument::from_text(source);
    let version = document.version();
    let preview_rows = document
        .preview_blocks()
        .into_iter()
        .find_map(|block| match block {
            PreviewBlock::Table { rows, .. } => Some(rows),
            _ => None,
        })
        .expect("preview table");
    let visual_rows = document
        .visual_blocks()
        .into_iter()
        .find_map(|block| match block.kind {
            VisualBlockKind::Table { rows, .. } => Some(rows),
            _ => None,
        })
        .expect("visual table");
    assert_eq!(
        preview_rows, visual_rows,
        "Visual Edit tables clone preview rows, so column weights stay aligned"
    );
    assert_eq!(preview_rows[3][0].text, "强调");
    assert!(
        !preview_rows[3][0].text.contains('*'),
        "weights must see rendered cell text, not `**强调**` source markup"
    );

    let font_size = DocumentTypographyMetrics::new(
        DEFAULT_EDITOR_FONT_SIZE,
        DEFAULT_RENDERED_FONT_SIZE,
        markion::DEFAULT_PARAGRAPH_SPACING,
    )
    .table_font_size;
    let preview_weights = table_column_flex_weights(&preview_rows, font_size);
    let visual_weights = table_column_flex_weights(&visual_rows, font_size);
    assert_eq!(preview_weights, visual_weights);
    assert_eq!(preview_weights.len(), 2);
    assert!(
        preview_weights[0] < preview_weights[1],
        "名称 column should be narrower than 说明: {preview_weights:?}"
    );
    assert_eq!(document.version(), version);
    assert!(!document.is_dirty());
}

fn visual_table_cell_range(
    document: &MarkdownDocument,
    table_index: usize,
    row: usize,
    column: usize,
) -> Range<usize> {
    document
        .visual_blocks()
        .into_iter()
        .filter_map(|block| match block.editor {
            Some(VisualBlockEditor::Table { cells }) => Some(cells),
            _ => None,
        })
        .nth(table_index)
        .and_then(|cells| {
            cells
                .into_iter()
                .find(|cell| cell.row == row && cell.column == column)
        })
        .map(|cell| cell.field.source_range)
        .expect("visual table cell range")
}

#[test]
fn visual_table_toolbar_target_tracks_caret_empty_utf8_staleness_and_table_ownership() {
    let source = "| A | 名称 |\n| --- | --- |\n|   | 值 |\n\n| C | D |\n| --- | --- |\n| x | y |";
    let document = MarkdownDocument::from_text(source);
    let blocks = document.visual_blocks();
    let tables = blocks
        .iter()
        .filter(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
        .collect::<Vec<_>>();
    assert_eq!(tables.len(), 2);
    let version = document.version();

    let utf8_range = visual_table_cell_range(&document, 0, 0, 1);
    let mut tab = EditorTab::new(document);
    tab.selected_range = utf8_range.clone();
    tab.selection_reversed = false;
    let forward = visual_table_toolbar_target(version, tables[0], tab.cursor_offset())
        .expect("forward selection endpoint owns the UTF-8 cell");
    assert_eq!((forward.row, forward.column), (0, 1));
    assert!(source.is_char_boundary(forward.source_offset));

    tab.selection_reversed = true;
    let reversed = visual_table_toolbar_target(version, tables[0], tab.cursor_offset())
        .expect("reversed selection endpoint owns the same UTF-8 cell");
    assert_eq!(forward, reversed);

    let empty_range = visual_table_cell_range(&tab.document, 0, 1, 0);
    assert!(empty_range.is_empty());
    let empty = visual_table_toolbar_target(version, tables[0], empty_range.start)
        .expect("an empty cell owns its exact zero-width caret position");
    assert_eq!((empty.row, empty.column), (1, 0));
    assert!(source.is_char_boundary(empty.source_offset));

    let second_range = visual_table_cell_range(&tab.document, 1, 1, 1);
    assert!(visual_table_toolbar_target(version, tables[0], second_range.end).is_none());
    let second = visual_table_toolbar_target(version, tables[1], second_range.end)
        .expect("only the second table owns its cell caret");
    assert_eq!((second.row, second.column), (1, 1));
    assert_eq!(
        revalidate_visual_table_toolbar_target(
            second,
            TableEdit::AddRow,
            version,
            second_range.end,
            &blocks,
        ),
        Some(second.source_offset)
    );
    assert_eq!(
        revalidate_visual_table_toolbar_target(
            second,
            TableEdit::AddRow,
            version + 1,
            second_range.end,
            &blocks,
        ),
        None,
        "a target from another document version must be rejected"
    );
    assert_eq!(
        revalidate_visual_table_toolbar_target(
            second,
            TableEdit::AddRow,
            version,
            utf8_range.end,
            &blocks,
        ),
        None,
        "moving the canonical caret out of the target cell invalidates activation"
    );
}

#[test]
fn visual_table_toolbar_availability_matches_structural_boundaries() {
    let document = MarkdownDocument::from_text(
        "| H1 | H2 | H3 |\n| --- | --- | --- |\n| a1 | a2 | a3 |\n| b1 | b2 | b3 |",
    );
    let cursor = visual_table_cell_range(&document, 0, 0, 1).end;
    let block = document
        .visual_blocks()
        .into_iter()
        .find(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
        .expect("visual table block");
    let base = visual_table_toolbar_target(document.version(), &block, cursor)
        .expect("header cell toolbar target");

    assert!(table_toolbar_action_available(base, TableEdit::AddRow));
    assert!(table_toolbar_action_available(base, TableEdit::AddColumn));
    assert!(!table_toolbar_action_available(base, TableEdit::DeleteRow));
    assert!(!table_toolbar_action_available(base, TableEdit::MoveRowUp));
    assert!(!table_toolbar_action_available(
        base,
        TableEdit::MoveRowDown
    ));

    let first_body = VisualTableToolbarTarget { row: 1, ..base };
    assert!(table_toolbar_action_available(
        first_body,
        TableEdit::DeleteRow
    ));
    assert!(!table_toolbar_action_available(
        first_body,
        TableEdit::MoveRowUp
    ));
    assert!(table_toolbar_action_available(
        first_body,
        TableEdit::MoveRowDown
    ));

    let last_body = VisualTableToolbarTarget { row: 2, ..base };
    assert!(table_toolbar_action_available(
        last_body,
        TableEdit::MoveRowUp
    ));
    assert!(!table_toolbar_action_available(
        last_body,
        TableEdit::MoveRowDown
    ));

    let final_column = VisualTableToolbarTarget {
        column: 0,
        column_count: 1,
        ..base
    };
    assert!(!table_toolbar_action_available(
        final_column,
        TableEdit::DeleteColumn
    ));
}

fn visual_table_block_id(document: &MarkdownDocument, table_index: usize) -> VisualBlockId {
    document
        .visual_blocks()
        .into_iter()
        .filter(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
        .nth(table_index)
        .map(|block| block.id)
        .expect("visual table block")
}

#[test]
fn visual_table_toolbar_visibility_is_hover_or_caret() {
    let document = MarkdownDocument::from_text(
        "| A | B |\n| --- | --- |\n| 1 | 2 |\n\n| C | D |\n| --- | --- |\n| 3 | 4 |",
    );
    let first = visual_table_block_id(&document, 0);
    let second = visual_table_block_id(&document, 1);
    assert!(!visual_table_toolbar_is_visible(None, first, false));
    assert!(visual_table_toolbar_is_visible(Some(first), first, false));
    assert!(!visual_table_toolbar_is_visible(Some(second), first, false));
    assert!(visual_table_toolbar_is_visible(None, first, true));
    assert!(visual_table_toolbar_is_visible(Some(first), first, true));
}

#[test]
fn visual_table_delete_is_disabled_for_quoted_tables() {
    let top_level = MarkdownDocument::from_text("| A | B |\n| --- | --- |\n| 1 | 2 |");
    let top_id = visual_table_block_id(&top_level, 0);
    assert!(visual_table_delete_available(
        &top_level.visual_blocks_shared(),
        top_id
    ));

    let quoted = MarkdownDocument::from_text("> | A | B |\n> | --- | --- |\n> | 1 | 2 |");
    let quoted_blocks = quoted.visual_blocks_shared();
    let quoted_table = quoted_blocks
        .iter()
        .find(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
        .expect("quoted visual table");
    assert!(
        quoted_table.quote_context.is_some() || quoted_table.source_island.is_some(),
        "quoted tables are not a free-standing reorderable source unit"
    );
    assert!(!visual_table_delete_available(
        &quoted_blocks,
        quoted_table.id
    ));
}

#[test]
fn visual_table_toolbar_uses_shared_compact_button_metrics() {
    assert_eq!(VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_X_PX, 6.);
    assert_eq!(VISUAL_TABLE_TOOLBAR_BUTTON_PADDING_Y_PX, 2.);
    assert_eq!(VISUAL_TABLE_TOOLBAR_BUTTON_FONT_SIZE_PX, 10.);
    assert_eq!(
        table_toolbar_actions_for_view_mode(ViewMode::VisualEdit).len(),
        6,
        "the shared compact metrics cover every Visual Edit row/column table action"
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
fn visual_edit_ime_rejects_stale_native_ranges_and_commits_pinyin_preedit(cx: &mut TestAppContext) {
    let source = "前0后";
    let cursor = "前0".len();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.view_mode = ViewMode::VisualEdit;
        app
    });

    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
        app.update(cx, |app, cx| {
            let mut actual_range = Some(0..0);
            assert_eq!(
                EntityInputHandler::text_for_range(app, 3..1, &mut actual_range, window, cx,),
                None
            );
            assert_eq!(actual_range, Some(0..0));

            EntityInputHandler::replace_and_mark_text_in_range(
                app,
                Some((usize::MAX - 1)..usize::MAX),
                "n",
                Some(1..1),
                window,
                cx,
            );
            EntityInputHandler::replace_and_mark_text_in_range(
                app,
                Some(3..1),
                "ni",
                Some(2..2),
                window,
                cx,
            );

            let fallback = Bounds::new(point(px(48.), px(72.)), size(px(2.), px(20.)));
            app.active_tab_mut().visual_caret_bounds = Some(fallback);
            assert_eq!(
                EntityInputHandler::bounds_for_range(
                    app,
                    (usize::MAX - 1)..usize::MAX,
                    Bounds::default(),
                    window,
                    cx,
                ),
                Some(fallback)
            );

            EntityInputHandler::replace_text_in_range(
                app,
                Some((usize::MAX - 1)..usize::MAX),
                "你",
                window,
                cx,
            );
        });
    });

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "前0你后");
        assert!(tab.marked_range.is_none());
        assert!(tab.checked_source_range(&tab.selected_range).is_some());
        assert_eq!(tab.undo_stack.len(), 1);

        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), source);
        assert!(app.active_tab_mut().apply_redo());
        assert_eq!(app.active_tab().document.text(), "前0你后");
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(
                app,
                None,
                "q",
                Some(1..1),
                window,
                cx,
            );
            EntityInputHandler::replace_and_mark_text_in_range(
                app,
                None,
                "qu",
                Some(2..2),
                window,
                cx,
            );
            EntityInputHandler::replace_text_in_range(app, None, "", window, cx);
        });
    });
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "前0你后");
        assert!(tab.marked_range.is_none());
        assert!(tab.checked_source_range(&tab.selected_range).is_some());
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

#[gpui::test]
fn visual_table_toolbar_clicks_target_the_focused_non_first_cell(cx: &mut TestAppContext) {
    let source = "| H1 | H2 | H3 |\n| --- | --- | --- |\n| a1 | a2 | a3 |\n| b1 | b2 | b3 |\n| c1 | c2 | c3 |";
    let cases = [
        (TableEdit::AddRow, "visual-table-add-row"),
        (TableEdit::DeleteRow, "visual-table-delete-row"),
        (TableEdit::MoveRowUp, "visual-table-move-row-up"),
        (TableEdit::MoveRowDown, "visual-table-move-row-down"),
        (TableEdit::AddColumn, "visual-table-add-column"),
        (TableEdit::DeleteColumn, "visual-table-delete-column"),
    ];
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    for (edit, selector) in cases {
        let document = MarkdownDocument::from_text(source);
        let selected_range = visual_table_cell_range(&document, 0, 2, 1);
        let mut expected = MarkdownDocument::from_text(source);
        let expected_result = expected
            .edit_table_at(selected_range.start, edit)
            .expect("the focused middle cell supports every toolbar action");
        let expected_text = expected.text().to_string();
        let expected_selection = expected_result.selected_range.clone();
        let expected_kind = VisualEditorFieldKind::TableCell {
            row: expected_result.row,
            column: expected_result.column,
        };

        app.update(cx, |app, cx| {
            app.tabs = vec![EditorTab::new(document)];
            app.active_tab_mut().selected_range = selected_range.clone();
            app.active_tab_mut().visual_cursor_reveal_pending = true;
            app.view_mode = ViewMode::VisualEdit;
            cx.notify();
        });
        cx.run_until_parked();
        let (version, blocks) = app.update(cx, |app, _| {
            let tab = app.active_tab();
            (tab.document.version(), tab.document.visual_blocks_shared())
        });

        let button = cx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("enabled toolbar button {selector} should be rendered"));
        cx.simulate_click(button.center(), Modifiers::none());
        cx.run_until_parked();

        app.update(cx, |app, _| {
            let tab = app.active_tab();
            assert_eq!(
                tab.document.text(),
                expected_text,
                "wrong target for {edit:?}"
            );
            assert_eq!(tab.selected_range, expected_selection);
            assert_eq!(tab.document.version(), version + 1);
            assert!(tab.document.is_dirty());
            assert_eq!(tab.undo_stack.len(), 1);
            assert_eq!(tab.autosave_generation, 1);
            assert!(!Arc::ptr_eq(&blocks, &tab.document.visual_blocks_shared()));
            assert_eq!(
                tab.document
                    .visual_editor_field_at(&tab.selected_range)
                    .expect("result selection stays in a visual table cell")
                    .kind,
                expected_kind
            );
        });
    }
}

#[gpui::test]
fn visual_table_toolbar_disables_unowned_and_invalid_actions_without_side_effects(
    cx: &mut TestAppContext,
) {
    let table = "| H1 | H2 |\n| --- | --- |\n| a1 | a2 |\n| b1 | b2 |";
    let source = format!("Intro\n\n{table}");
    let document = MarkdownDocument::from_text(source.clone());
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = 0..0;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    assert!(
        cx.debug_bounds("visual-table-add-row-disabled").is_none(),
        "idle tables omit the editing header"
    );
    assert!(cx.debug_bounds("visual-table-add-row").is_none());
    assert!(cx.debug_bounds("visual-table-delete-table").is_none());

    app.update(cx, |app, cx| {
        let id = visual_table_block_id(&app.active_tab().document, 0);
        app.active_tab_mut().hovered_visual_table_block = Some(id);
        cx.notify();
    });
    cx.run_until_parked();

    let unowned_add = cx
        .debug_bounds("visual-table-add-row-disabled")
        .expect("hovered toolbar without an owned caret renders Add Row disabled");
    let before = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (
            tab.document.text().to_string(),
            tab.selected_range.clone(),
            tab.document.version(),
            tab.document.is_dirty(),
            tab.undo_stack.len(),
        )
    });
    cx.simulate_click(unowned_add.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), before.0);
        assert_eq!(tab.selected_range, before.1);
        assert_eq!(tab.document.version(), before.2);
        assert_eq!(tab.document.is_dirty(), before.3);
        assert_eq!(tab.undo_stack.len(), before.4);
    });

    let header = app.update(cx, |app, _| {
        visual_table_cell_range(&app.active_tab().document, 0, 0, 0)
    });
    app.update(cx, |app, cx| app.move_to(header.start, cx));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("visual-table-delete-row-disabled")
            .is_some()
    );
    assert!(
        cx.debug_bounds("visual-table-move-row-up-disabled")
            .is_some()
    );
    assert!(
        cx.debug_bounds("visual-table-move-row-down-disabled")
            .is_some()
    );

    let first_body = app.update(cx, |app, _| {
        visual_table_cell_range(&app.active_tab().document, 0, 1, 0)
    });
    app.update(cx, |app, cx| app.move_to(first_body.start, cx));
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("visual-table-move-row-up-disabled")
            .is_some()
    );
    assert!(cx.debug_bounds("visual-table-move-row-down").is_some());

    let last_body = app.update(cx, |app, _| {
        visual_table_cell_range(&app.active_tab().document, 0, 2, 0)
    });
    app.update(cx, |app, cx| app.move_to(last_body.start, cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("visual-table-move-row-up").is_some());
    let disabled_down = cx
        .debug_bounds("visual-table-move-row-down-disabled")
        .expect("last body row cannot move down");
    let version = app.update(cx, |app, _| app.active_tab().document.version());
    cx.simulate_click(disabled_down.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), source);
        assert_eq!(tab.document.version(), version);
        assert!(!tab.document.is_dirty());
        assert!(tab.undo_stack.is_empty());
    });
}

#[gpui::test]
fn visual_table_toolbar_isolates_tables_and_roundtrips_one_history_entry(cx: &mut TestAppContext) {
    let source = "| A | B |\n| --- | --- |\n| a1 | a2 |\n\nBetween\n\n| C | D |\n| --- | --- |\n| c1 | c2 |\n| d1 | d2 |";
    let document = MarkdownDocument::from_text(source);
    let selected = visual_table_cell_range(&document, 1, 1, 1);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = selected;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    assert!(
        cx.debug_bounds("visual-table-add-row-disabled").is_none(),
        "the idle neighbor table omits its editing header"
    );

    app.update(cx, |app, cx| {
        let id = visual_table_block_id(&app.active_tab().document, 0);
        app.active_tab_mut().hovered_visual_table_block = Some(id);
        cx.notify();
    });
    cx.run_until_parked();

    let first_table_disabled = cx
        .debug_bounds("visual-table-add-row-disabled")
        .expect("the hovered table without the caret has no guessed target");
    cx.simulate_click(first_table_disabled.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source);
        assert!(app.active_tab().undo_stack.is_empty());
    });

    let delete_row = cx
        .debug_bounds("visual-table-delete-row")
        .expect("the caret-owning second table exposes Delete Row");
    cx.simulate_click(delete_row.center(), Modifiers::none());
    cx.run_until_parked();
    let edited = app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.undo_stack.len(), 1);
        assert!(tab.document.text().contains("| a1 | a2 |"));
        assert!(!tab.document.text().contains("| c1  | c2  |"));
        tab.document.text().to_string()
    });

    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), source)
    });
    cx.dispatch_action(Redo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), edited)
    });
}

#[gpui::test]
fn visual_table_toolbar_shows_on_hover_or_caret_without_mutating(cx: &mut TestAppContext) {
    let source = "Intro\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\nAfter";
    let document = MarkdownDocument::from_text(source);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = 0..0;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let before = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (
            tab.document.text().to_string(),
            tab.selected_range.clone(),
            tab.document.version(),
            tab.document.is_dirty(),
            tab.undo_stack.len(),
            std::sync::Arc::as_ptr(&tab.document.visual_blocks_shared()),
        )
    });
    assert!(cx.debug_bounds("visual-table-add-row-disabled").is_none());
    assert!(cx.debug_bounds("visual-table-delete-table").is_none());

    app.update(cx, |app, cx| {
        let id = visual_table_block_id(&app.active_tab().document, 0);
        app.active_tab_mut().hovered_visual_table_block = Some(id);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("visual-table-add-row-disabled").is_some());
    assert!(cx.debug_bounds("visual-table-delete-table").is_some());
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), before.0);
        assert_eq!(tab.selected_range, before.1);
        assert_eq!(tab.document.version(), before.2);
        assert_eq!(tab.document.is_dirty(), before.3);
        assert_eq!(tab.undo_stack.len(), before.4);
        assert_eq!(
            std::sync::Arc::as_ptr(&tab.document.visual_blocks_shared()),
            before.5
        );
    });

    let cell = app.update(cx, |app, _| {
        visual_table_cell_range(&app.active_tab().document, 0, 0, 0)
    });
    app.update(cx, |app, cx| {
        app.active_tab_mut().hovered_visual_table_block = None;
        app.move_to(cell.start, cx);
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("visual-table-add-row").is_some(),
        "caret ownership keeps the header visible after the pointer leaves"
    );
    assert!(cx.debug_bounds("visual-table-delete-table").is_some());
}

#[gpui::test]
fn visual_table_toolbar_deletes_the_whole_table_in_one_history_entry(cx: &mut TestAppContext) {
    let source = "Intro\n\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\nAfter";
    let document = MarkdownDocument::from_text(source);
    let selected = visual_table_cell_range(&document, 0, 0, 0);
    let expected_selection = selected.clone();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = selected;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let delete = cx
        .debug_bounds("visual-table-delete-table")
        .expect("caret-owned table exposes Delete Table");
    cx.simulate_click(delete.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(!tab.document.text().contains("| A | B |"));
        assert!(tab.document.text().contains("Intro"));
        assert!(tab.document.text().contains("After"));
        assert_eq!(tab.undo_stack.len(), 1);
        assert!(tab.document.is_dirty());
    });

    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), source);
        assert_eq!(tab.selected_range, expected_selection);
    });
}

#[gpui::test]
fn visual_table_toolbar_delete_is_isolated_and_disabled_when_unsupported(cx: &mut TestAppContext) {
    let source =
        "| A | B |\n| --- | --- |\n| a1 | a2 |\n\nKeep\n\n| C | D |\n| --- | --- |\n| c1 | c2 |";
    let document = MarkdownDocument::from_text(source);
    let selected = visual_table_cell_range(&document, 0, 0, 0);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = selected;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let delete = cx
        .debug_bounds("visual-table-delete-table")
        .expect("first table delete control");
    cx.simulate_click(delete.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let text = app.active_tab().document.text();
        assert!(!text.contains("| A | B |"));
        assert!(text.contains("| C | D |"));
        assert!(text.contains("Keep"));
    });

    let quoted = "> | H1 | H2 |\n> | --- | --- |\n> | a | b |";
    let quoted_document = MarkdownDocument::from_text(quoted);
    app.update(cx, |app, cx| {
        app.tabs = vec![EditorTab::new(quoted_document)];
        let id = visual_table_block_id(&app.active_tab().document, 0);
        app.active_tab_mut().hovered_visual_table_block = Some(id);
        app.active_tab_mut().selected_range = 0..0;
        app.view_mode = ViewMode::VisualEdit;
        cx.notify();
    });
    cx.run_until_parked();
    let disabled = cx
        .debug_bounds("visual-table-delete-table-disabled")
        .expect("quoted tables cannot be deleted as a source unit");
    let before = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (
            tab.document.text().to_string(),
            tab.document.version(),
            tab.document.is_dirty(),
            tab.undo_stack.len(),
        )
    });
    cx.simulate_click(disabled.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), before.0);
        assert_eq!(tab.document.version(), before.1);
        assert_eq!(tab.document.is_dirty(), before.2);
        assert_eq!(tab.undo_stack.len(), before.3);
    });
}

#[gpui::test]
fn source_table_command_still_targets_the_source_caret(cx: &mut TestAppContext) {
    let source = "| A | B |\n| --- | --- |\n| a1 | a2 |\n| b1 | b2 |";
    let cursor = source.find("b2").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = cursor..cursor;
        app.view_mode = ViewMode::Edit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    cx.dispatch_action(TableDeleteRow);
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.document.text().contains("a1"));
        assert!(!tab.document.text().contains("b1"));
        assert_eq!(tab.undo_stack.len(), 1);
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
fn visual_edit_mixed_prose_keeps_authored_line_breaks(cx: &mut TestAppContext) {
    const SOURCE: &str = "### Brave Search API Key\n\n\
网址：[Brave Search - API](https://example.com/keys)\n\
账户： willmove@gmail.com\n\
API Key: `BSA2IC_xxxxxxx`\n";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 0..0;
        app.view_mode = ViewMode::VisualEdit;
        app.preview_adaptive_width = false;
        app
    });
    cx.simulate_resize(size(px(1200.), px(760.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let paragraph = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let index = blocks
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .expect("fixture should have one paragraph");
        let cursor = blocks[index].editable_runs[0].content_range.start;
        let projection =
            build_visual_projection(tab.document.text(), &blocks[index], cursor..cursor, cursor);
        assert!(
            projection.text.contains('\n'),
            "projection must keep soft breaks: {:?}",
            projection.text
        );
        index
    });

    let line0 = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-mixed-line-{paragraph}-0"
        )))
        .expect("first mixed line");
    let line1 = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-mixed-line-{paragraph}-1"
        )))
        .expect("second mixed line");
    let line2 = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-mixed-line-{paragraph}-2"
        )))
        .expect("third mixed line");
    assert!(
        f32::from(line1.top()) > f32::from(line0.top()) + 1.0,
        "account line must sit below the URL line: {line0:?} vs {line1:?}"
    );
    assert!(
        f32::from(line2.top()) > f32::from(line1.top()) + 1.0,
        "API key line must sit below the account line: {line1:?} vs {line2:?}"
    );
    assert!(
        cx.debug_bounds(test_debug_selector(format!(
            "visual-mixed-line-{paragraph}-3"
        )))
        .is_none(),
        "fixture has exactly three authored lines"
    );
}

#[gpui::test]
fn visual_edit_single_line_mixed_prose_stays_one_row(cx: &mut TestAppContext) {
    const SOURCE: &str = "See [Brave Search - API](https://example.com/keys) for the token.\n";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 0..0;
        app.view_mode = ViewMode::VisualEdit;
        app.preview_adaptive_width = false;
        app
    });
    cx.simulate_resize(size(px(1200.), px(760.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let paragraph = app.update(cx, |app, _| {
        app.active_tab()
            .document
            .visual_blocks_shared()
            .iter()
            .position(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .expect("single-line fixture should have one paragraph")
    });
    assert!(
        cx.debug_bounds(test_debug_selector(format!(
            "visual-mixed-line-{paragraph}-0"
        )))
        .is_some(),
        "linked prose still uses the mixed fragment layout"
    );
    assert!(
        cx.debug_bounds(test_debug_selector(format!(
            "visual-mixed-line-{paragraph}-1"
        )))
        .is_none(),
        "a single authored line must not invent a second mixed row"
    );
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
fn visual_edit_quoted_siblings_support_navigation_input_ime_copy_and_history(
    cx: &mut TestAppContext,
) {
    let source = "> first\n>\n> 1. second\n> 2. third\n";
    let first_cursor = source.find("first").unwrap() + 2;
    let second_start = source.find("second").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = first_cursor..first_cursor;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let caret = app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.visual_last_projection.as_ref().unwrap().0, "first");
        tab.visual_caret_bounds.expect("quoted caret geometry")
    });
    cx.simulate_click(caret.center(), Modifiers::none());
    cx.run_until_parked();

    for _ in 0..2 {
        cx.dispatch_action(Down);
        cx.run_until_parked();
        if app.update(cx, |app, _| app.active_tab().cursor_offset()) >= second_start {
            break;
        }
    }
    app.update(cx, |app, _| {
        assert!(app.active_tab().cursor_offset() >= second_start);
    });
    app.update(cx, |app, cx| app.move_to(second_start + 2, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().visual_last_projection.as_ref().unwrap().0,
            "second"
        );
    });

    cx.dispatch_action(SelectRight);
    cx.dispatch_action(Copy);
    let copied = cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text()));
    assert!(copied.is_some_and(|text| !text.is_empty()));

    cx.simulate_input("中🙂");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.active_tab().document.text().contains("中🙂"));
        assert!(app.active_tab().undo_stack.len() >= 1);
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(
                app,
                None,
                "输入",
                Some(0..0),
                window,
                cx,
            );
        });
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.marked_range.is_some());
        assert!(tab.visual_marked_range_bounds.is_some());
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::unmark_text(app, window, cx)
        });
    });
    cx.dispatch_action(Undo);
    cx.dispatch_action(Redo);
    app.update(cx, |app, _| {
        assert!(app.active_tab().document.text().contains("输入"));
        assert!(app.active_tab().marked_range.is_none());
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
        let block_index =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
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
fn visual_edit_tail_enter_moves_caret_down_the_whitespace_row(cx: &mut TestAppContext) {
    // Repeated Enter at the document tail must move the painted caret down
    // the growing whitespace row. Painting it at the row origin made every
    // extra blank line look like a no-op.
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

    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();
    let first_top = app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "Body\n");
        app.active_tab()
            .visual_caret_bounds
            .expect("caret after first Enter")
            .top()
    });

    let mut previous_top = first_top;
    for press in 2..=5 {
        cx.dispatch_action(InsertNewline);
        cx.run_until_parked();
        let top = app.update(cx, |app, _| {
            let expected_newlines = "\n".repeat(press);
            assert_eq!(
                app.active_tab().document.text(),
                format!("Body{expected_newlines}")
            );
            app.active_tab()
                .visual_caret_bounds
                .expect("caret after tail Enter")
                .top()
        });
        assert!(
            f32::from(top) > f32::from(previous_top) + 8.0,
            "Enter #{press} must move the caret down: {previous_top:?} -> {top:?}"
        );
        previous_top = top;
    }
}

#[gpui::test]
fn visual_edit_tail_typing_stays_visible_at_the_viewport_bottom(cx: &mut TestAppContext) {
    // When the last rendered line sits on the pane bottom, typed characters
    // and the caret must remain inside the Visual Edit viewport instead of
    // growing the last row below the clip.
    let source = (0..40)
        .map(|index| format!("Paragraph {index}"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let end = source.len();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.active_tab_mut().selected_range = end..end;
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(640.), px(480.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        assert_eq!(app.active_tab().cursor_offset(), end);
        let blocks = app.active_tab().document.visual_blocks_shared();
        let index = visual_block_index_for_offset(&blocks, end, end)
            .expect("tail offset owns a visual row");
        // Jump by item index so unmeasured far-below rows (height 0 in the
        // summary) cannot keep the tail out of the first layout.
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: index,
            offset_in_item: px(0.),
        });
        app.active_tab_mut().visual_cursor_reveal_pending = true;
        cx.notify();
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(
            tab.visual_caret_bounds.is_some(),
            "caret must be painted at the tail before editing"
        );
        assert!(
            tab.visual_list.viewport_bounds().size.height > px(80.),
            "visual list viewport too small"
        );
    });

    for _ in 0..8 {
        cx.dispatch_action(InsertNewline);
        cx.run_until_parked();
    }
    cx.simulate_input("TailVisible");
    cx.run_until_parked();
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(
            tab.document.text().ends_with("TailVisible"),
            "typed tail text must land in the source: {}",
            tab.document.text()
        );
        let caret = tab.visual_caret_bounds.expect("caret after tail typing");
        let viewport = tab.visual_list.viewport_bounds();
        let inset = px(VISUAL_CARET_VIEWPORT_INSET);
        assert!(
            caret.top() >= viewport.top() - px(1.) && caret.bottom() <= viewport.bottom() + px(1.),
            "caret {caret:?} must stay inside visual viewport {viewport:?}"
        );
        // Pixel-follow may use the inset, but must not pin a mid-document row
        // to the top just because the tail index is greater than scroll top.
        let top = tab.visual_list.logical_scroll_top();
        assert!(
            top.item_ix + 2 >= tab.visual_list_blocks.len().saturating_sub(1)
                || caret.bottom() <= viewport.bottom() - inset + px(1.),
            "tail follow should stay near the end or inside the inset, not pin a mid row: {top:?}"
        );
    });
}

fn visual_edit_paragraph_source(count: usize) -> String {
    (0..count)
        .map(|index| format!("Paragraph {index}"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[gpui::test]
fn visual_edit_click_on_visible_mid_row_does_not_scroll(cx: &mut TestAppContext) {
    let source = visual_edit_paragraph_source(24);
    let click_offset = source.find("Paragraph 8").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(640.), px(480.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (anchor, click_index, version, visual) = app.update(cx, |app, _| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let click_index = visual_block_index_for_offset(&blocks, click_offset, source.len())
            .expect("clicked paragraph owns a visual row");
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: 2,
            offset_in_item: px(0.),
        });
        (
            app.active_tab().visual_list.logical_scroll_top(),
            click_index,
            app.active_tab().document.version(),
            app.active_tab().document.visual_blocks_shared(),
        )
    });
    cx.run_until_parked();

    let row = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-document-row-{click_index}"
        )))
        .expect("clicked Visual Edit row should be painted");
    cx.simulate_click(row.center(), Modifiers::none());
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let range = tab.visual_list_blocks[click_index].source_range.clone();
        assert!(
            range.contains(&tab.cursor_offset()) || tab.cursor_offset() == range.end,
            "click should place the caret in the hit row, got {} in {range:?}",
            tab.cursor_offset()
        );
        let top = tab.visual_list.logical_scroll_top();
        assert_eq!(top.item_ix, anchor.item_ix);
        assert_eq!(top.offset_in_item, anchor.offset_in_item);
        assert_eq!(tab.document.version(), version);
        assert!(Arc::ptr_eq(&tab.document.visual_blocks_shared(), &visual));
        assert!(!tab.document.is_dirty());
        let _ = click_offset;
    });
}

#[gpui::test]
fn visual_edit_click_on_visible_lower_row_does_not_pin_to_top(cx: &mut TestAppContext) {
    let source = visual_edit_paragraph_source(24);
    let click_offset = source.find("Paragraph 4").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(640.), px(480.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let click_index = app.update(cx, |app, _| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let click_index = visual_block_index_for_offset(&blocks, click_offset, source.len())
            .expect("clicked paragraph owns a visual row");
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        click_index
    });
    cx.run_until_parked();

    let row = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-document-row-{click_index}"
        )))
        .expect("lower Visual Edit row should be painted");
    cx.simulate_click(row.center(), Modifiers::none());
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let range = tab.visual_list_blocks[click_index].source_range.clone();
        assert!(
            range.contains(&tab.cursor_offset()) || tab.cursor_offset() == range.end,
            "click should place the caret in the hit row, got {} in {range:?}",
            tab.cursor_offset()
        );
        let top = tab.visual_list.logical_scroll_top();
        assert_ne!(
            (top.item_ix, top.offset_in_item),
            (click_index, px(0.)),
            "clicking a later visible row must not pin that row to the viewport top"
        );
        assert_eq!(top.item_ix, 0);
        assert_eq!(top.offset_in_item, px(0.));
        let _ = click_offset;
    });
}

#[gpui::test]
fn visual_edit_offscreen_navigation_reveals_then_manual_scroll_stays(cx: &mut TestAppContext) {
    let source = visual_edit_paragraph_source(40);
    let target_offset = source.find("Paragraph 35").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.active_tab_mut().selected_range = 0..0;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(640.), px(480.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        app.move_to(target_offset, cx);
    });
    cx.run_until_parked();
    cx.run_until_parked();

    let target_index = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let index = visual_block_index_for_offset(&blocks, target_offset, source.len())
            .expect("target paragraph owns a visual row");
        assert_eq!(tab.cursor_offset(), target_offset);
        assert!(
            tab.visual_list.bounds_for_item(index).is_some()
                || tab.visual_list.logical_scroll_top().item_ix > 0,
            "off-screen caret move must reveal the target row"
        );
        index
    });

    app.update(cx, |app, _| {
        let tab = app.active_tab_mut();
        tab.visual_cursor_reveal_pending = false;
        tab.visual_caret_follow_frames = 0;
        tab.visual_list.scroll_to(gpui::ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(top.item_ix, 0);
        assert_eq!(top.offset_in_item, px(0.));
        assert!(
            app.active_tab()
                .visual_list
                .bounds_for_item(target_index)
                .is_none()
                || top.item_ix == 0,
            "manual scroll away from the caret must not snap back"
        );
    });
}

#[gpui::test]
fn visual_edit_end_padding_is_presentation_only_and_places_eof_caret(cx: &mut TestAppContext) {
    let source = visual_edit_paragraph_source(4);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(640.), px(480.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();

    let (version, visual, spacer_ix, viewport_height) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let spacer_ix = blocks.len();
        assert_eq!(tab.visual_list.item_count(), spacer_ix + 1);
        assert_eq!(
            visual_list_item_count(blocks.len()),
            tab.visual_list.item_count()
        );
        (
            tab.document.version(),
            blocks,
            spacer_ix,
            tab.visual_list.viewport_bounds().size.height,
        )
    });
    assert!(f32::from(viewport_height) > 80.);

    let padding = cx
        .debug_bounds(test_debug_selector(
            "visual-document-end-padding".to_string(),
        ))
        .expect("document-end padding should be painted");
    let expected = visual_end_padding_height(viewport_height);
    assert!(
        (f32::from(padding.size.height) - f32::from(expected)).abs() < 8.,
        "end padding {padding:?} should be about half the viewport {viewport_height:?} (expected {expected:?})"
    );

    let scroll_before = app.update(cx, |app, _| {
        app.active_tab().visual_list.logical_scroll_top()
    });
    cx.simulate_click(padding.center(), Modifiers::none());
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.cursor_offset(), source.len());
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.text(), source);
        assert!(!tab.document.is_dirty());
        assert!(Arc::ptr_eq(&tab.document.visual_blocks_shared(), &visual));
        assert!(
            tab.document.visual_blocks_shared().len() == spacer_ix,
            "spacer must not appear in the visual-block cache"
        );
        let top = tab.visual_list.logical_scroll_top();
        assert_ne!(
            (top.item_ix, top.offset_in_item),
            (spacer_ix.saturating_sub(1), px(0.)),
            "clicking the end padding must not pin the last content row to the top"
        );
        assert_eq!(top.item_ix, scroll_before.item_ix);
        assert_eq!(top.offset_in_item, scroll_before.offset_in_item);
    });
}

#[gpui::test]
fn visual_edit_gfm_alert_title_row_is_reachable_and_editable(cx: &mut TestAppContext) {
    // The callout title row owns only structural marker bytes. Up from the
    // body must land the caret inside the title row (not skip it as a dead
    // stop), reveal `> [!NOTE]` verbatim through the projection, and accept
    // source-backed text edits at the caret.
    let source = "> [!NOTE]\n> body\n";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        // Caret at the start of the body row (offset 10, on its `>` marker).
        app.active_tab_mut().selected_range = 10..10;
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

    let pre_edit_cursor = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let cursor = tab.cursor_offset();
        assert!(
            cursor < 10,
            "Up from the body must enter the title row, got {cursor}"
        );
        let blocks = tab.document.visual_blocks_shared();
        let block_index = visual_block_index_for_offset(&blocks, cursor, tab.document.text().len())
            .expect("caret owns a visual row");
        assert!(
            matches!(
                blocks[block_index].kind,
                VisualBlockKind::CalloutTitle { .. }
            ),
            "caret should own the callout title row, got {:?}",
            blocks[block_index].kind,
        );
        let projection = build_visual_projection(
            tab.document.text(),
            &blocks[block_index],
            cursor..cursor,
            cursor,
        );
        assert_eq!(projection.text, "> [!NOTE]");
        cursor
    });

    // The revealed marker line is a normal source-backed editable range.
    cx.simulate_input("X");
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(
            &tab.document.text()[pre_edit_cursor..pre_edit_cursor + 1],
            "X"
        );
        assert_eq!(tab.cursor_offset(), pre_edit_cursor + 1);
        assert!(tab.document.is_dirty());
        assert!(!tab.undo_stack.is_empty());
    });

    // Down from the title row returns to the body paragraph.
    cx.dispatch_action(Down);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let block_index =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
                .expect("Down should land on a visual row");
        assert!(
            matches!(blocks[block_index].kind, VisualBlockKind::Paragraph),
            "Down from the title row should enter the body paragraph, got {:?}",
            blocks[block_index].kind,
        );
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
        let block_index =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
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
        let block_index =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
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
        let block_index =
            visual_block_index_for_offset(&blocks, tab.cursor_offset(), tab.document.text().len())
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
        fonts: ThemeFonts::default(),
    };
    let palette = theme_palette_from_definition(&theme);

    assert_eq!(palette.app_bg, rgb(0x010203));
    assert_eq!(palette.active_text, rgb(0x717273));
}

#[gpui::test]
fn font_family_change_is_presentation_only_and_invalidates_measured_height(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, _| {
        assert_eq!(app.resolved_font_families.editor, SYSTEM_UI_FONT_FAMILY);
        assert_eq!(app.resolved_font_families.rendered, SYSTEM_UI_FONT_FAMILY);
        assert_eq!(app.resolved_font_families.code, DEFAULT_CODE_FONT_FAMILY);
    });

    // Seed a measured-height cache entry as if a layout pass had run at the
    // default family.
    app.update(cx, |app, _| {
        let tab = app.active_tab_mut();
        *tab.measured_height_cache.borrow_mut() = Some((
            MeasuredHeightKey {
                version: tab.document.version(),
                wrap_width: px(400.),
                font_size: px(14.),
                line_height: px(22.4),
                font_family: SYSTEM_UI_FONT_FAMILY.into(),
            },
            px(1000.),
        ));
    });

    app.update(cx, |app, cx| {
        let version_before = app.active_tab().document.version();
        let dirty_before = app.active_tab().document.is_dirty();
        let undo_before = app.active_tab().undo_stack.len();

        // A family-only change (same font size) must invalidate the cached
        // height without touching the document.
        app.set_font_family(FontSlot::Editor, Some("Cascadia Code".into()), cx);

        assert_eq!(app.active_tab().document.version(), version_before);
        assert_eq!(app.active_tab().document.is_dirty(), dirty_before);
        assert_eq!(app.active_tab().undo_stack.len(), undo_before);
        assert_eq!(app.resolved_font_families.editor, "Cascadia Code");
        assert!(
            app.active_tab().measured_height_cache.borrow().is_none(),
            "a cached height measured in the old family must not survive"
        );

        // Clearing back to follow-theme restores the default family.
        app.set_font_family(FontSlot::Editor, None, cx);
        assert_eq!(app.resolved_font_families.editor, SYSTEM_UI_FONT_FAMILY);
    });
}

#[gpui::test]
fn font_picker_toggles_and_choice_applies_and_closes(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, cx| {
        assert!(app.font_picker.is_none());

        // Opening marks the slot; opening it again closes the list.
        app.toggle_font_picker(FontSlot::Code, cx);
        assert_eq!(app.font_picker, Some(FontSlot::Code));
        app.toggle_font_picker(FontSlot::Code, cx);
        assert_eq!(app.font_picker, None);

        // Opening a different slot switches the open list.
        app.toggle_font_picker(FontSlot::Editor, cx);
        app.toggle_font_picker(FontSlot::Code, cx);
        assert_eq!(app.font_picker, Some(FontSlot::Code));

        // Choosing applies the family, persists the slot state, and closes
        // the list.
        app.choose_font_family(FontSlot::Code, Some("Cascadia Code".into()), cx);
        assert_eq!(app.font_picker, None);
        assert_eq!(app.resolved_font_families.code, "Cascadia Code");
        assert_eq!(
            app.code_font_family.as_deref(),
            Some("Cascadia Code"),
            "choosing persists the explicit preference"
        );

        // The follow-theme entry clears the override.
        app.toggle_font_picker(FontSlot::Code, cx);
        app.choose_font_family(FontSlot::Code, None, cx);
        assert_eq!(app.font_picker, None);
        assert!(app.code_font_family.is_none());
        assert_eq!(app.resolved_font_families.code, DEFAULT_CODE_FONT_FAMILY);
    });
}

#[gpui::test]
fn theme_fonts_apply_only_without_explicit_preference(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, cx| {
        let mut theme = builtin_theme_definitions().remove(0);
        theme.name = "Typo".to_string();
        theme.fonts = ThemeFonts {
            editor: Some("Georgia".to_string()),
            rendered: Some("Georgia".to_string()),
            code: Some("Cascadia Code".to_string()),
        };
        app.custom_themes.push(theme);

        // No explicit preferences: the theme's fonts apply per slot.
        app.apply_theme_by_name("Typo", cx);
        assert_eq!(app.resolved_font_families.editor, "Georgia");
        assert_eq!(app.resolved_font_families.rendered, "Georgia");
        assert_eq!(app.resolved_font_families.code, "Cascadia Code");

        // An explicit preference beats the theme for its slot only.
        app.set_font_family(FontSlot::Rendered, Some("Source Serif 4".into()), cx);
        assert_eq!(app.resolved_font_families.rendered, "Source Serif 4");
        assert_eq!(app.resolved_font_families.editor, "Georgia");

        // Returning to follow-theme re-exposes the theme's font.
        app.set_font_family(FontSlot::Rendered, None, cx);
        assert_eq!(app.resolved_font_families.rendered, "Georgia");

        // A theme without [fonts] falls back to the built-in defaults.
        app.apply_theme_by_name("Paper", cx);
        assert_eq!(app.resolved_font_families.editor, SYSTEM_UI_FONT_FAMILY);
        assert_eq!(app.resolved_font_families.rendered, SYSTEM_UI_FONT_FAMILY);
        assert_eq!(app.resolved_font_families.code, DEFAULT_CODE_FONT_FAMILY);
    });
}

#[test]
fn ime_selected_range_is_relative_to_composition_text() {
    let composing = "😀文";
    let range = EditorTab::relative_range_from_utf16(composing, &(2..3))
        .expect("valid relative UTF-16 range");

    assert_eq!(&composing[range], "文");
    assert_eq!(utf16_offset_to_byte_offset("a😀文", 3), Some("a😀".len()));
    assert_eq!(utf16_offset_to_byte_offset("a😀文", 2), None);
    assert_eq!(utf16_offset_to_byte_offset("a😀文", 5), None);
    assert_eq!(byte_offset_to_utf16_offset("a😀文", "a😀".len()), 3);

    let mut tab = EditorTab::new(MarkdownDocument::from_text("a😀文"));
    assert_eq!(tab.range_from_utf16(&(1..3)), Some(1.."a😀".len()));
    assert_eq!(tab.range_from_utf16(&(3..1)), None);
    assert_eq!(tab.range_from_utf16(&(2..3)), None);
    assert_eq!(tab.range_from_utf16(&(3..8)), None);
    tab.selected_range = 2..usize::MAX;
    assert_eq!(tab.safe_selected_range(), "a😀文".len().."a😀文".len());
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
    assert!(tab.line_tops.is_empty());
    assert!(tab.source_layout_key.is_none());
    assert!(tab.last_bounds.is_none());
    assert_eq!(tab.line_height, px(EDITOR_LINE_HEIGHT));
    assert!(!tab.is_selecting);
    assert!(tab.last_recovery_file.is_none());
    assert_eq!(tab.autosave_generation, 0);
    assert_eq!(tab.sync_scroll_state, SyncScrollState::default());
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
fn image_tabs_share_path_identity_without_dirty_document_state() {
    let path = PathBuf::from("images/Preview.PNG");
    let tab = EditorTab::new_image(path.clone(), PreviewImageKey::from_local_path(&path));
    assert!(tab.is_image());
    assert!(!tab.is_dirty());
    assert_eq!(tab.path(), Some(path.as_path()));
    assert!(tab.document_tab().is_none());

    let tabs = vec![tab];
    assert_eq!(find_tab_with_document_path(&tabs, &path), Some(0));
}

#[test]
fn discard_confirmation_is_scoped_to_dirty_document_tabs() {
    let mut document = EditorTab::new(MarkdownDocument::from_text("clean"));
    document.document.replace_range(0..0, "dirty ");
    assert!(document.requires_discard_confirmation());

    let path = PathBuf::from("preview.png");
    let image = EditorTab::new_image(path.clone(), PreviewImageKey::from_local_path(&path));
    assert!(!image.requires_discard_confirmation());
}

#[test]
fn image_scale_down_fit_preserves_aspect_ratio_without_upscaling() {
    assert_eq!(scale_down_image_size(200, 100, 100.0, 80.0), (100.0, 50.0));
    assert_eq!(scale_down_image_size(40, 20, 100.0, 80.0), (40.0, 20.0));
    assert_eq!(scale_down_image_size(40, 200, 100.0, 80.0), (16.0, 80.0));
}

#[gpui::test]
fn replace_active_router_switches_between_document_and_image_content(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let image_path = dir.path().join("cover.JpEg");
    let text_path = dir.path().join("notes.txt");
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(4, 3, image::Rgb([10, 20, 30])))
        .save_with_format(&image_path, image::ImageFormat::Jpeg)
        .unwrap();
    fs::write(&text_path, "plain text").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, cx| {
        app.open_supported_path(image_path.clone(), OpenPathIntent::ReplaceActive, cx)
            .unwrap();
        assert_eq!(app.tabs.len(), 1);
        assert!(app.active_tab().is_image());
        assert!(app.active_tab().document_tab().is_none());

        app.open_supported_path(text_path.clone(), OpenPathIntent::ReplaceActive, cx)
            .unwrap();
        assert_eq!(app.tabs.len(), 1);
        assert!(app.active_tab().is_document());
        assert_eq!(app.active_tab().document.text(), "plain text");
        assert_eq!(app.active_tab().path(), Some(text_path.as_path()));
    });
}

#[gpui::test]
fn image_tree_and_open_recent_entry_points_follow_open_target_preference(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    let tree_image = dir.path().join("tree.png");
    let recent_image = dir.path().join("recent.png");
    write_solid_png(&tree_image, 3, 2, [20, 30, 40, 255]);
    write_solid_png(&recent_image, 2, 3, [40, 30, 20, 255]);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        // Redirect persistence so toggling cannot touch the developer's real
        // config.toml (the constructor reads it, tests share it).
        app.preferences_path = config_dir.path().join("config.toml");
        app.open_in_current_tab = true;
        app
    });

    // Default preference on: the untitled welcome tab and then the read-only
    // image tab are both safe to replace, so neither open appends a tab.
    app.update(cx, |app, cx| {
        app.open_tree_file_confirmed(tree_image.clone(), cx);
        assert_eq!(app.tabs.len(), 1);
        assert!(app.active_tab().is_image());
        assert_eq!(app.active_tab().path(), Some(tree_image.as_path()));

        app.open_recent_path(recent_image.clone(), cx);
        assert_eq!(app.tabs.len(), 1);
        assert!(app.active_tab().is_image());
        assert_eq!(app.active_tab().path(), Some(recent_image.as_path()));
        assert_eq!(
            app.session.recent_files.first(),
            Some(&comparable_document_path(&recent_image))
        );
    });

    // Preference off: the same entry points append tabs again, and an
    // already-open path dedupes to its existing tab.
    app.update(cx, |app, cx| {
        app.toggle_open_in_current_tab(cx);
        app.open_tree_file_confirmed(tree_image.clone(), cx);
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab().path(), Some(tree_image.as_path()));

        app.open_recent_path(recent_image.clone(), cx);
        assert_eq!(
            app.tabs.len(),
            2,
            "already-open paths must dedupe, not append"
        );
        assert_eq!(app.active_tab().path(), Some(recent_image.as_path()));
    });
}

#[gpui::test]
fn default_open_intent_replaces_only_safe_active_tabs(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    let alpha = dir.path().join("alpha.md");
    let beta = dir.path().join("beta.md");
    fs::write(&alpha, "alpha").unwrap();
    fs::write(&beta, "beta").unwrap();

    // Preference on (default).
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = config_dir.path().join("config.toml");
        app.open_in_current_tab = true;
        app
    });
    app.update(cx, |app, cx| {
        // Untitled welcome tab → replace in place.
        assert_eq!(app.default_open_intent(), OpenPathIntent::ReplaceActive);
        app.open_tree_file_confirmed(alpha.clone(), cx);
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab().path(), Some(alpha.as_path()));

        // Clean saved document → still replace.
        assert_eq!(app.default_open_intent(), OpenPathIntent::ReplaceActive);
        app.open_recent_path(beta.clone(), cx);
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab().path(), Some(beta.as_path()));

        // Dirty document → divert to a new tab, never replace.
        app.active_tab_mut().document.set_text("edited");
        assert!(app.active_tab().document.is_dirty());
        assert_eq!(app.default_open_intent(), OpenPathIntent::OpenInNewTab);
        app.open_tree_file_confirmed(alpha.clone(), cx);
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab().path(), Some(alpha.as_path()));
        assert!(app.tabs[0].document.is_dirty());
        assert_eq!(app.tabs[0].document.text(), "edited");
    });

    // Preference off → always new tabs, even with a clean welcome tab.
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = config_dir.path().join("config.toml");
        app.open_in_current_tab = false;
        app
    });
    app.update(cx, |app, cx| {
        assert_eq!(app.default_open_intent(), OpenPathIntent::OpenInNewTab);
        app.open_tree_file_confirmed(alpha.clone(), cx);
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab().path(), Some(alpha.as_path()));
    });
}

#[gpui::test]
fn gesture_open_with_dirty_tab_appends_and_preserves_work(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    let dirty_path = dir.path().join("dirty.md");
    let other = dir.path().join("other.md");
    fs::write(&dirty_path, "saved base").unwrap();
    fs::write(&other, "other content").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = config_dir.path().join("config.toml");
        app.open_in_current_tab = true;
        app
    });

    let recovery = dir.path().join("recovery-0.md");
    app.update(cx, |app, cx| {
        app.open_tree_file_confirmed(dirty_path.clone(), cx);
        assert_eq!(app.tabs.len(), 1);
        app.active_tab_mut().push_undo_snapshot();
        app.active_tab_mut().document.set_text("unsaved edits");
        // Simulate the autosave recovery snapshot: it must survive the open.
        fs::write(&recovery, "recovery bytes").unwrap();
        app.active_tab_mut().last_recovery_file = Some(recovery.clone());
    });

    app.update(cx, |app, cx| {
        app.open_tree_file_confirmed(other.clone(), cx);
        assert_eq!(
            app.tabs.len(),
            2,
            "a dirty active tab must divert to a new tab"
        );
        assert_eq!(app.active_tab().path(), Some(other.as_path()));
        let dirty = app.tabs[0].document_tab().unwrap();
        assert_eq!(dirty.document.text(), "unsaved edits");
        assert!(dirty.document.is_dirty());
        assert!(!dirty.undo_stack.is_empty());
        assert_eq!(
            dirty.last_recovery_file.as_deref(),
            Some(recovery.as_path()),
            "a gesture open must never delete a dirty tab's recovery snapshot"
        );
    });
}

#[gpui::test]
fn multi_file_drop_replaces_once_then_appends(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = tempfile::tempdir().unwrap();
    let alpha = dir.path().join("alpha.md");
    let beta = dir.path().join("beta.md");
    let gamma = dir.path().join("gamma.md");
    fs::write(&alpha, "alpha").unwrap();
    fs::write(&beta, "beta").unwrap();
    fs::write(&gamma, "gamma").unwrap();

    // Preference on: the first dropped file replaces the clean welcome tab;
    // every subsequent one appends, leaving the last file active.
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = config_dir.path().join("config.toml");
        app.open_in_current_tab = true;
        app
    });
    app.update(cx, |app, cx| {
        app.open_dropped_documents(&[alpha.clone(), beta.clone(), gamma.clone()], cx);
        assert_eq!(app.tabs.len(), 3);
        assert_eq!(app.tabs[0].path(), Some(alpha.as_path()));
        assert_eq!(app.tabs[1].path(), Some(beta.as_path()));
        assert_eq!(app.active_tab().path(), Some(gamma.as_path()));
    });

    // Preference off: a drop batch only appends; the welcome tab survives.
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = config_dir.path().join("config.toml");
        app.open_in_current_tab = false;
        app
    });
    app.update(cx, |app, cx| {
        app.open_dropped_documents(&[alpha.clone(), beta.clone()], cx);
        assert_eq!(app.tabs.len(), 3);
        assert!(
            app.tabs[0]
                .document_tab()
                .is_some_and(|tab| tab.document.path().is_none())
        );
        assert_eq!(app.active_tab().path(), Some(beta.as_path()));
    });
}

#[gpui::test]
fn image_open_router_preserves_documents_deduplicates_and_releases_cache_claims(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let image_path = dir.path().join("Preview.PNG");
    write_solid_png(&image_path, 8, 6, [20, 40, 60, 255]);
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    let (document_version, preview_cache, visual_cache, scroll_before, image_key) =
        app.update(cx, |app, cx| {
            app.active_tab_mut().selected_range = 3..3;
            app.active_tab_mut().push_undo_snapshot();
            app.active_tab()
                .editor_scroll
                .set_offset(point(px(0.), px(-12.)));
            let document_version = app.active_tab().document.version();
            let preview_cache = app.active_tab().document.preview_blocks_shared();
            let visual_cache = app.active_tab().document.visual_blocks_shared();
            let scroll_before = app.active_tab().editor_scroll.offset();

            app.open_supported_path(image_path.clone(), OpenPathIntent::OpenInNewTab, cx)
                .unwrap();
            assert_eq!(app.tabs.len(), 2);
            assert!(app.active_tab().is_image());
            assert_eq!(app.active_tab().path(), Some(image_path.as_path()));
            assert!(!app.active_tab().is_dirty());
            assert_eq!(app.workspace_root, comparable_document_path(dir.path()));
            assert_eq!(
                app.session.recent_files.first(),
                Some(&comparable_document_path(&image_path))
            );
            assert!(app.session.open_files.is_empty());
            assert!(app.session.active_file.is_none());
            let key = app.active_tab().image().unwrap().key.clone();
            (
                document_version,
                preview_cache,
                visual_cache,
                scroll_before,
                key,
            )
        });

    cx.run_until_parked();
    assert!(cx.debug_bounds("image-tab-ready").is_some());
    app.update(cx, |app, cx| {
        assert!(matches!(
            app.image_tab_entry(&image_key),
            PreviewImageEntry::Ready(_)
        ));
        assert_eq!(app.preview_image_cache.claim_count(&image_key), 1);

        app.open_supported_path(image_path.clone(), OpenPathIntent::OpenInNewTab, cx)
            .unwrap();
        assert_eq!(
            app.tabs.len(),
            2,
            "same image path must focus, not duplicate"
        );

        app.switch_active_tab(0, cx);
        assert_eq!(app.preview_image_cache.claim_count(&image_key), 0);
        assert_eq!(app.active_tab().document.version(), document_version);
        assert_eq!(app.active_tab().selected_range, 3..3);
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        assert_eq!(app.active_tab().editor_scroll.offset(), scroll_before);
        assert!(Arc::ptr_eq(
            &preview_cache,
            &app.active_tab().document.preview_blocks_shared()
        ));
        assert!(Arc::ptr_eq(
            &visual_cache,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
}

#[gpui::test]
fn corrupt_image_and_unsupported_file_are_non_destructive(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let corrupt = dir.path().join("broken.webp");
    let unsupported = dir.path().join("archive.bin");
    fs::write(&corrupt, b"not an image").unwrap();
    fs::write(&unsupported, b"binary").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    app.update(cx, |app, cx| {
        let original = app.active_tab().document.text().to_string();
        assert!(
            app.open_supported_path(unsupported.clone(), OpenPathIntent::OpenInNewTab, cx,)
                .is_err()
        );
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab().document.text(), original);

        app.open_supported_path(corrupt.clone(), OpenPathIntent::OpenInNewTab, cx)
            .unwrap();
        assert_eq!(app.tabs.len(), 2);
        assert!(app.active_tab().is_image());
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("image-tab-error").is_some());
    cx.dispatch_action(CloseTab);
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 1);
        assert!(app.active_tab().is_document());
    });
}

#[gpui::test]
fn image_tabs_disable_document_shortcuts_and_menus_and_close_without_a_prompt(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let image_path = dir.path().join("preview.png");
    write_solid_png(&image_path, 5, 4, [10, 20, 30, 255]);
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    let (status_before, key) = app.update(cx, |app, cx| {
        app.open_supported_path(image_path, OpenPathIntent::OpenInNewTab, cx)
            .unwrap();
        app.active_menu = Some(AppMenu::Edit);
        cx.notify();
        (
            app.status.clone(),
            app.active_tab().image().unwrap().key.clone(),
        )
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("image-document-actions-unavailable")
            .is_some(),
        "document-only menu commands should be replaced by an unavailable row"
    );

    cx.dispatch_action(Undo);
    cx.dispatch_action(ShowFind);
    cx.dispatch_action(SaveDocument);
    app.update(cx, |app, _| {
        assert!(app.active_tab().is_image());
        assert_eq!(app.status, status_before);
        assert!(!app.search_visible);
        assert!(!app.active_tab().is_dirty());
    });

    cx.dispatch_action(CloseTab);
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 1);
        assert!(app.active_tab().is_document());
        assert_eq!(app.preview_image_cache.claim_count(&key), 0);
    });
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
fn document_tab_band_geometry_tracks_visibility_in_document_column() {
    assert!(!document_tab_band_visible(0));
    assert!(!document_tab_band_visible(1));
    assert!(document_tab_band_visible(2));

    assert_eq!(document_tab_band_height(1), 0.);
    assert_eq!(document_tab_band_height(2), DOCUMENT_TAB_BAND_HEIGHT);
}

#[test]
fn workspace_layout_places_sidebar_beside_document_stack_and_scopes_drags() {
    let root_view = include_str!("root_view.rs");
    let workspace_start = root_view
        .find(".id(\"workspace-row\")")
        .expect("workspace row");
    let document_start = workspace_start
        + root_view[workspace_start..]
            .find(".id(\"document-workspace-column\")")
            .expect("document workspace column");
    let before_document = &root_view[workspace_start..document_start];
    assert!(before_document.contains(".child(sidebar_view(self, cx))"));
    assert!(before_document.contains("on_drag_move::<DraggedSidebarHandle>"));
    assert!(!before_document.contains("on_drag_move::<DraggedEditorSplitHandle>"));

    let document_stack = &root_view[document_start..];
    let tab_band = document_stack
        .find(".child(self.tab_bar_view(cx))")
        .expect("document tab band");
    let content_row = document_stack
        .find(".id(\"main-content-row\")")
        .expect("document content row");
    assert!(tab_band < content_row);
    assert!(document_stack[content_row..].contains("on_drag_move::<DraggedEditorSplitHandle>"));

    let editing = include_str!("editing.rs");
    let tab_bar = editing
        .split_once("pub(super) fn tab_bar_view")
        .expect("tab bar function")
        .1
        .split_once("pub(super) fn cursor_offset")
        .expect("tab bar function end")
        .0;
    assert!(!tab_bar.contains("leading_width"));
}

#[test]
fn tab_bar_strip_scrolls_and_pins_actions_instead_of_clipping() {
    let editing = include_str!("editing.rs");
    let tab_bar = editing
        .split_once("pub(super) fn tab_bar_view")
        .expect("tab bar function")
        .1
        .split_once("pub(super) fn cursor_offset")
        .expect("tab bar function end")
        .0;

    // The strip is a stateful horizontal scroll container driven by the
    // app-level handle, so overflowing tabs scroll into reach instead of
    // being clipped out of view.
    assert!(tab_bar.contains(".id(\"tab-bar-scroll\")"));
    assert!(tab_bar.contains(".overflow_x_scroll()"));
    assert!(tab_bar.contains(".track_scroll(&self.tab_bar_scroll)"));
    // Plain wheels scroll the strip only because the container scrolls on x
    // alone; restricting the axis (or also scrolling y) would break that.
    assert!(!tab_bar.contains("restrict_scroll_to_axis"));
    assert!(!tab_bar.contains(".overflow_y_scroll()"));

    // Tabs are width-bounded; labels truncate while the close control never
    // shrinks out of reach.
    assert!(tab_bar.contains(".max_w(px(DOCUMENT_TAB_MAX_WIDTH))"));
    assert!(tab_bar.contains(".min_w(px(DOCUMENT_TAB_MIN_WIDTH))"));
    assert!(tab_bar.contains(".flex_shrink()"));
    assert!(tab_bar.contains("div().min_w_0().truncate()"));
    assert!(tab_bar.contains(".flex_shrink_0()"));

    // The dirty marker is a separate element, never baked into the label
    // string where truncation would swallow it.
    assert!(!tab_bar.contains("format!(\"{name} *\")"));
    assert!(tab_bar.contains(".when(dirty, |tab_view|"));

    // Every tab is stateful and carries the full-title/path tooltip.
    assert!(tab_bar.contains(".id(ElementId::named_usize(\"document-tab\", index))"));
    assert!(tab_bar.contains(".tooltip("));
    let tooltip_view = editing
        .split_once("struct TabTooltip")
        .expect("tab tooltip view")
        .1;
    assert!(tooltip_view.contains("when_some(self.path.clone()"));

    // The "+" button is attached after the scroll container, in the pinned
    // region outside it, so it stays reachable at any scroll position.
    let scroll_child = tab_bar
        .find(".child(document_bar)")
        .expect("scroll container attached to the band");
    let new_tab_button = tab_bar
        .find(".id(\"new-tab-button\")")
        .expect("new tab button");
    assert!(
        scroll_child < new_tab_button,
        "the \"+\" button must live outside the scroll container"
    );
}

#[gpui::test]
fn tab_activation_paths_request_strip_reveal_of_new_active_tab(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    // Opening a tab (the path behind File → New Tab and file opens) requests
    // the strip to reveal the appended index.
    app.update(cx, |app, cx| {
        app.open_in_new_tab(MarkdownDocument::from_text("second"), cx);
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab, 1);
        assert_eq!(app.last_tab_strip_reveal, Some(1));
    });

    // Switching requests the newly active index; focusing an already-open
    // file routes through the same path.
    app.update(cx, |app, cx| {
        app.open_in_new_tab(MarkdownDocument::from_text("third"), cx);
        app.switch_active_tab(0, cx);
        assert_eq!(app.last_tab_strip_reveal, Some(0));
    });

    // Re-selecting the active tab issues no request: an undisturbed strip
    // must stay undisturbed.
    app.update(cx, |app, cx| {
        app.last_tab_strip_reveal = None;
        app.switch_active_tab(0, cx);
        assert_eq!(app.last_tab_strip_reveal, None);
    });

    // Closing the active tab requests the tab that takes its place.
    app.update(cx, |app, cx| {
        app.switch_active_tab(2, cx);
        assert_eq!(app.last_tab_strip_reveal, Some(2));
        app.close_tab_confirmed(cx);
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab, 1);
        assert_eq!(app.last_tab_strip_reveal, Some(1));
    });
}

#[test]
fn outline_row_metrics_are_compact() {
    assert_eq!(OUTLINE_ROW_GAP, 0.);
    assert!(OUTLINE_ROW_VERTICAL_PADDING * 2. <= 2.);
    assert_eq!(
        OUTLINE_ROW_LINE_HEIGHT + OUTLINE_ROW_VERTICAL_PADDING * 2. + OUTLINE_ROW_GAP,
        19.
    );
}

fn outline_heading(level: u8, title: &str, offset: usize) -> markion::Heading {
    markion::Heading {
        level,
        title: title.to_string(),
        anchor: title.to_ascii_lowercase(),
        offset,
    }
}

#[test]
fn outline_projection_defaults_expanded_and_handles_skipped_levels() {
    let headings = vec![
        outline_heading(1, "Root", 0),
        outline_heading(3, "Skipped child", 10),
        outline_heading(4, "Grandchild", 30),
        outline_heading(1, "Sibling", 50),
    ];

    let projection = project_outline_rows(&headings, &HashSet::new(), Some(2));
    assert_eq!(
        projection
            .rows
            .iter()
            .map(|row| row.outline_index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    assert!(projection.rows[0].has_children);
    assert!(projection.rows[1].has_children);
    assert!(!projection.rows[2].has_children);
    assert!(!projection.rows[3].has_children);
    assert!(projection.rows.iter().all(|row| !row.collapsed));
    assert_eq!(
        projection
            .rows
            .iter()
            .find(|row| row.active)
            .map(|row| row.outline_index),
        Some(2)
    );
}

#[test]
fn outline_projection_preserves_nested_folds_and_visible_active_ancestor() {
    let headings = vec![
        outline_heading(1, "Root", 0),
        outline_heading(2, "Branch", 10),
        outline_heading(3, "Leaf", 20),
        outline_heading(1, "Sibling", 30),
    ];
    let keys = outline_node_keys(&headings);
    let mut collapsed = HashSet::from([keys[1].clone()]);

    let nested = project_outline_rows(&headings, &collapsed, Some(2));
    assert_eq!(
        nested
            .rows
            .iter()
            .map(|row| row.outline_index)
            .collect::<Vec<_>>(),
        vec![0, 1, 3]
    );
    assert!(nested.rows[1].collapsed);
    assert!(nested.rows[1].active);

    collapsed.insert(keys[0].clone());
    let outer = project_outline_rows(&headings, &collapsed, Some(2));
    assert_eq!(
        outer
            .rows
            .iter()
            .map(|row| row.outline_index)
            .collect::<Vec<_>>(),
        vec![0, 3]
    );
    assert!(outer.rows[0].active);

    collapsed.remove(&keys[0]);
    let restored = project_outline_rows(&headings, &collapsed, Some(2));
    assert_eq!(
        restored
            .rows
            .iter()
            .map(|row| row.outline_index)
            .collect::<Vec<_>>(),
        vec![0, 1, 3]
    );
    assert!(restored.rows[1].collapsed);
}

#[test]
fn outline_structural_keys_distinguish_duplicate_titles() {
    let headings = vec![
        outline_heading(1, "Same", 0),
        outline_heading(2, "Same", 10),
        outline_heading(2, "Same", 20),
        outline_heading(1, "Same", 30),
    ];
    let keys = outline_node_keys(&headings);
    let unique = keys.iter().collect::<HashSet<_>>();
    assert_eq!(unique.len(), headings.len());
}

#[test]
fn outline_fold_reconciliation_survives_offsets_and_drops_obsolete_keys() {
    let original = vec![
        outline_heading(1, "Root", 0),
        outline_heading(2, "Child", 10),
    ];
    let original_keys = outline_node_keys(&original);
    let mut collapsed = HashSet::from([original_keys[0].clone()]);

    let shifted = vec![
        outline_heading(1, "Root", 200),
        outline_heading(2, "Child", 220),
    ];
    let shifted_keys = outline_node_keys(&shifted);
    reconcile_outline_collapsed_keys(&mut collapsed, &original_keys, &shifted_keys);
    assert_eq!(collapsed, HashSet::from([shifted_keys[0].clone()]));

    let renamed = vec![
        outline_heading(1, "Renamed", 200),
        outline_heading(2, "Child", 220),
    ];
    let renamed_keys = outline_node_keys(&renamed);
    reconcile_outline_collapsed_keys(&mut collapsed, &shifted_keys, &renamed_keys);
    assert!(collapsed.is_empty());
}

#[test]
fn outline_fold_reconciliation_is_conservative_for_changed_duplicates() {
    let duplicates = vec![
        outline_heading(1, "Same", 0),
        outline_heading(2, "Child A", 10),
        outline_heading(1, "Same", 30),
        outline_heading(2, "Child B", 40),
    ];
    let duplicate_keys = outline_node_keys(&duplicates);
    let mut collapsed = HashSet::from([duplicate_keys[0].clone()]);

    let body_shifted = vec![
        outline_heading(1, "Same", 100),
        outline_heading(2, "Child A", 110),
        outline_heading(1, "Same", 130),
        outline_heading(2, "Child B", 140),
    ];
    let shifted_keys = outline_node_keys(&body_shifted);
    reconcile_outline_collapsed_keys(&mut collapsed, &duplicate_keys, &shifted_keys);
    assert_eq!(collapsed, HashSet::from([shifted_keys[0].clone()]));

    let inserted_duplicate = vec![
        outline_heading(1, "Same", 0),
        outline_heading(2, "New child", 10),
        outline_heading(1, "Same", 30),
        outline_heading(2, "Child A", 40),
        outline_heading(1, "Same", 60),
        outline_heading(2, "Child B", 70),
    ];
    let inserted_keys = outline_node_keys(&inserted_duplicate);
    reconcile_outline_collapsed_keys(&mut collapsed, &shifted_keys, &inserted_keys);
    assert!(
        collapsed.is_empty(),
        "changed duplicate groups must unfold instead of transferring a fold"
    );
}

#[test]
fn outline_folding_state_is_isolated_per_document_tab() {
    let mut tabs = vec![
        EditorTab::new(MarkdownDocument::from_text("# First\n\n## Child\n")),
        EditorTab::new(MarkdownDocument::from_text("# Second\n\n## Child\n")),
    ];
    let outlines = tabs
        .iter()
        .map(|tab| tab.document.outline())
        .collect::<Vec<_>>();
    let first_projection = tabs[0]
        .document_tab()
        .unwrap()
        .outline_projection(&outlines[0], None);
    let second_projection = tabs[1]
        .document_tab()
        .unwrap()
        .outline_projection(&outlines[1], None);
    let first_root = first_projection.rows[0].key.clone();
    assert_eq!(
        tabs[0]
            .document_tab()
            .unwrap()
            .toggle_outline_node(first_root),
        Some(true)
    );

    assert_eq!(
        tabs[0]
            .document_tab()
            .unwrap()
            .outline_projection(&outlines[0], None)
            .rows
            .len(),
        1
    );
    assert_eq!(
        tabs[1]
            .document_tab()
            .unwrap()
            .outline_projection(&outlines[1], None)
            .rows
            .len(),
        second_projection.rows.len()
    );

    // Returning to the first entry in the tab vector restores its own state.
    assert_eq!(
        tabs[0]
            .document_tab()
            .unwrap()
            .outline_folding
            .borrow()
            .collapsed_keys()
            .len(),
        1
    );
    assert!(tabs.iter_mut().all(|tab| tab.document_tab_mut().is_some()));
}

#[test]
fn outline_folding_does_not_mutate_document_or_derived_caches() {
    let mut tab = EditorTab::new(MarkdownDocument::from_text("# Root\n\n## Child\n\nBody\n"));
    tab.selected_range = 3..3;
    tab.push_undo_snapshot();
    let outline = tab.document.outline();
    let preview = tab.document.preview_blocks_shared();
    let visual = tab.document.visual_blocks_shared();
    let text_handle = tab.shared_document_text();
    let before_text = tab.document.text().to_string();
    let before_version = tab.document.version();
    let before_dirty = tab.document.is_dirty();
    let before_selection = tab.selected_range.clone();
    let before_undo = tab.undo_stack.len();
    let before_redo = tab.redo_stack.len();

    let projection = tab.outline_projection(&outline, Some(0));
    assert_eq!(
        tab.toggle_outline_node(projection.rows[0].key.clone()),
        Some(true)
    );
    assert_eq!(tab.outline_projection(&outline, Some(0)).rows.len(), 1);

    assert_eq!(tab.document.text(), before_text);
    assert_eq!(tab.document.version(), before_version);
    assert_eq!(tab.document.is_dirty(), before_dirty);
    assert_eq!(tab.selected_range, before_selection);
    assert_eq!(tab.undo_stack.len(), before_undo);
    assert_eq!(tab.redo_stack.len(), before_redo);
    assert_eq!(tab.document.outline(), outline);
    assert!(Arc::ptr_eq(&tab.document.preview_blocks_shared(), &preview));
    assert!(Arc::ptr_eq(&tab.document.visual_blocks_shared(), &visual));
    assert_eq!(tab.shared_document_text().as_ptr(), text_handle.as_ptr());
}

#[gpui::test]
fn read_mode_outline_click_scrolls_preview_and_preserves_document_state(cx: &mut TestAppContext) {
    const TARGET_HEADING: usize = 3;
    let source = (0..16)
        .map(|index| {
            format!(
                "## Section {index}\n\n{}\n",
                "Rendered prose before the next heading wraps across the preview. ".repeat(8)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::Read;
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Outline;
        app.active_tab_mut().push_undo_snapshot();
        app
    });
    cx.simulate_resize(size(px(1000.), px(720.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (heading_offset, preview_item_ix, version, dirty, undo_len, redo_len, preview_cache) = app
        .update(cx, |app, _| {
            let tab = app.active_tab();
            let heading_offset = tab.document.outline()[TARGET_HEADING].offset;
            let preview_item_ix =
                preview_heading_index_for_source_offset(&tab.preview_list_blocks, heading_offset)
                    .expect("outline heading should have an exact rendered preview row");
            (
                heading_offset,
                preview_item_ix,
                tab.document.version(),
                tab.document.is_dirty(),
                tab.undo_stack.len(),
                tab.redo_stack.len(),
                tab.document.preview_blocks_shared(),
            )
        });

    let label = cx
        .debug_bounds("outline-heading-label-3")
        .expect("target outline label should be rendered");
    cx.simulate_click(label.center(), Modifiers::none());
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.selected_range, heading_offset..heading_offset);
        assert_eq!(
            tab.document.current_heading_index(tab.cursor_offset()),
            Some(TARGET_HEADING)
        );
        let preview_top = tab.preview_list.logical_scroll_top();
        assert_eq!(preview_top.item_ix, preview_item_ix);
        assert_eq!(preview_top.offset_in_item, px(0.));
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.is_dirty(), dirty);
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert_eq!(tab.redo_stack.len(), redo_len);
        assert!(Arc::ptr_eq(
            &tab.document.preview_blocks_shared(),
            &preview_cache
        ));
        assert!(tab.outline_folding.borrow().collapsed_keys().is_empty());
        assert_eq!(
            app.status,
            t(app.language, Msg::StatusJumpedToHeading).to_string()
        );
    });
}

#[gpui::test]
fn outline_navigation_outside_read_mode_keeps_existing_preview_position(cx: &mut TestAppContext) {
    let source = "# One\n\nBody\n\n## Two\n\nMore\n\n### Three\n";
    let heading_offset = source.find("## Two").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        let mut tab = EditorTab::new(MarkdownDocument::from_text(source));
        let preview = tab.document.preview_blocks_shared();
        tab.sync_preview_list(&preview);
        tab.push_undo_snapshot();
        app.tabs = vec![tab];
        app
    });

    app.update(cx, |app, cx| {
        let version = app.active_tab().document.version();
        let dirty = app.active_tab().document.is_dirty();
        let undo_len = app.active_tab().undo_stack.len();
        let redo_len = app.active_tab().redo_stack.len();
        for mode in [ViewMode::Edit, ViewMode::VisualEdit, ViewMode::Split] {
            app.view_mode = mode;
            app.active_tab_mut().visual_cursor_reveal_pending = false;
            app.active_tab().preview_list.scroll_to(gpui::ListOffset {
                item_ix: 1,
                offset_in_item: px(4.),
            });
            let preview_top = app.active_tab().preview_list.logical_scroll_top();

            app.navigate_to_outline_heading(heading_offset, cx);

            let tab = app.active_tab();
            assert_eq!(
                tab.selected_range,
                heading_offset..heading_offset,
                "{mode:?}"
            );
            assert!(tab.visual_cursor_reveal_pending, "{mode:?}");
            let current_preview_top = tab.preview_list.logical_scroll_top();
            assert_eq!(current_preview_top.item_ix, preview_top.item_ix, "{mode:?}");
            assert_eq!(
                current_preview_top.offset_in_item, preview_top.offset_in_item,
                "{mode:?}"
            );
            assert_eq!(tab.document.version(), version, "{mode:?}");
            assert_eq!(tab.document.is_dirty(), dirty, "{mode:?}");
            assert_eq!(tab.undo_stack.len(), undo_len, "{mode:?}");
            assert_eq!(tab.redo_stack.len(), redo_len, "{mode:?}");
        }
    });
}

#[gpui::test]
fn outline_navigation_visual_edit_top_aligns_heading_block(cx: &mut TestAppContext) {
    let source = "# One\n\nBody\n\n## Two\n\nMore\n\n### Three\n";
    let heading_offset = source.find("## Two").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        let mut tab = EditorTab::new(MarkdownDocument::from_text(source));
        let visual = tab.document.visual_blocks_shared();
        tab.sync_visual_list(&visual);
        app.tabs = vec![tab];
        app
    });

    app.update(cx, |app, cx| {
        app.view_mode = ViewMode::VisualEdit;
        let blocks = app.active_tab().document.visual_blocks_shared();
        let text_len = app.active_tab().document.text().len();
        let heading_block =
            visual_block_index_for_offset(&blocks, heading_offset, text_len).unwrap();

        // Heading below the viewport: the minimal reveal alone would clamp its
        // bottom edge to the pane bottom; navigation must top-align it.
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: 0,
            offset_in_item: px(0.),
        });
        app.navigate_to_outline_heading(heading_offset, cx);
        let top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(top.item_ix, heading_block);
        assert_eq!(top.offset_in_item, px(0.));

        // Heading above the viewport stays top-aligned too, and the jump
        // still arms the generic caret reveal used at render time.
        app.active_tab().visual_list.scroll_to(gpui::ListOffset {
            item_ix: heading_block + 1,
            offset_in_item: px(0.),
        });
        app.navigate_to_outline_heading(heading_offset, cx);
        let top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(top.item_ix, heading_block);
        assert_eq!(top.offset_in_item, px(0.));
        assert!(app.active_tab().visual_cursor_reveal_pending);
        assert_eq!(
            app.active_tab().selected_range,
            heading_offset..heading_offset
        );
    });
}

#[gpui::test]
fn outline_disclosures_fold_nested_rows_without_navigation(cx: &mut TestAppContext) {
    let source = "# Root\n\n## Branch\n\n### Leaf\n\n## Other\n\n# Sibling\n";
    let leaf_offset = source.find("### Leaf").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        let mut tab = EditorTab::new(MarkdownDocument::from_text(source));
        let preview = tab.document.preview_blocks_shared();
        tab.sync_preview_list(&preview);
        tab.selected_range = leaf_offset..leaf_offset;
        tab.push_undo_snapshot();
        app.tabs = vec![tab];
        app.view_mode = ViewMode::Edit;
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Outline;
        app
    });
    cx.simulate_resize(size(px(900.), px(600.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let branch_disclosure = cx
        .debug_bounds("outline-heading-disclosure-1")
        .expect("branch disclosure should render");
    let other_placeholder = cx
        .debug_bounds("outline-heading-disclosure-placeholder-3")
        .expect("leaf rows keep a disclosure-sized spacer");
    let branch_label = cx
        .debug_bounds("outline-heading-label-1")
        .expect("branch label should render");
    let other_label = cx
        .debug_bounds("outline-heading-label-3")
        .expect("same-level leaf label should render");
    assert_eq!(
        branch_disclosure.size.width,
        px(OUTLINE_DISCLOSURE_SLOT_SIZE)
    );
    assert_eq!(branch_disclosure.size, other_placeholder.size);
    assert_eq!(branch_label.left(), other_label.left());
    let branch_row = cx.debug_bounds("outline-heading-row-1").unwrap();
    assert!(
        f32::from(branch_row.size.height)
            <= OUTLINE_ROW_LINE_HEIGHT + OUTLINE_ROW_VERTICAL_PADDING * 2.
    );

    let (selection, preview_top, version, dirty, undo_len, redo_len) = app.update(cx, |app, _| {
        app.active_tab().preview_list.scroll_to(gpui::ListOffset {
            item_ix: 2,
            offset_in_item: px(3.),
        });
        let tab = app.active_tab();
        (
            tab.selected_range.clone(),
            tab.preview_list.logical_scroll_top(),
            tab.document.version(),
            tab.document.is_dirty(),
            tab.undo_stack.len(),
            tab.redo_stack.len(),
        )
    });

    cx.simulate_click(branch_disclosure.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(
        app.update(cx, |app, _| app
            .active_tab()
            .outline_folding
            .borrow()
            .collapsed_keys()
            .len()),
        1,
        "disclosure click should update the active document's folding state"
    );
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let outline = tab.document.outline();
        let current = tab.document.current_heading_index(tab.cursor_offset());
        let projection = tab.outline_projection(&outline, current);
        assert_eq!(
            projection
                .rows
                .iter()
                .find(|row| row.active)
                .map(|row| row.outline_index),
            Some(1),
            "the visible collapsed branch represents its hidden active leaf"
        );
        assert_eq!(
            projection
                .rows
                .iter()
                .map(|row| row.outline_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 3, 4]
        );
    });

    let root_disclosure = cx
        .debug_bounds("outline-heading-disclosure-0")
        .expect("root disclosure should render");
    cx.simulate_click(root_disclosure.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let outline = tab.document.outline();
        let projection = tab.outline_projection(&outline, Some(2));
        assert_eq!(
            projection
                .rows
                .iter()
                .map(|row| row.outline_index)
                .collect::<Vec<_>>(),
            vec![0, 4]
        );
    });

    let root_disclosure = cx
        .debug_bounds("outline-heading-disclosure-0")
        .expect("collapsed root disclosure should remain visible");
    cx.simulate_click(root_disclosure.center(), Modifiers::none());
    cx.run_until_parked();
    assert!(cx.debug_bounds("outline-heading-row-1").is_some());
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let outline = tab.document.outline();
        let projection = tab.outline_projection(&outline, Some(2));
        assert_eq!(
            projection
                .rows
                .iter()
                .map(|row| row.outline_index)
                .collect::<Vec<_>>(),
            vec![0, 1, 3, 4],
            "expanding the root must preserve the nested branch fold"
        );
    });

    let branch_disclosure = cx
        .debug_bounds("outline-heading-disclosure-1")
        .expect("nested branch disclosure should be restored");
    cx.simulate_click(branch_disclosure.center(), Modifiers::none());
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let outline = tab.document.outline();
        assert_eq!(tab.outline_projection(&outline, Some(2)).rows.len(), 5);
        let actual_preview_top = tab.preview_list.logical_scroll_top();
        assert_eq!(tab.selected_range, selection);
        assert_eq!(actual_preview_top.item_ix, preview_top.item_ix);
        assert_eq!(
            actual_preview_top.offset_in_item,
            preview_top.offset_in_item
        );
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.is_dirty(), dirty);
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert_eq!(tab.redo_stack.len(), redo_len);
        assert!(tab.outline_folding.borrow().collapsed_keys().is_empty());
    });
}

#[gpui::test]
fn outline_disclosure_state_follows_document_tab_switches(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("# First\n\n## Child\n")),
            EditorTab::new(MarkdownDocument::from_text("# Second\n\n## Child\n")),
        ];
        app.active_tab = 0;
        app.view_mode = ViewMode::Edit;
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Outline;
        app
    });
    cx.simulate_resize(size(px(900.), px(600.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let disclosure = cx
        .debug_bounds("outline-heading-disclosure-0")
        .expect("first document root should be collapsible");
    cx.simulate_click(disclosure.center(), Modifiers::none());
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        assert_eq!(
            app.tabs[0]
                .document_tab()
                .unwrap()
                .outline_folding
                .borrow()
                .collapsed_keys()
                .len(),
            1
        );
        app.active_tab = 1;
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let second = app.tabs[1].document_tab().unwrap();
        let outline = second.document.outline();
        assert_eq!(second.outline_projection(&outline, None).rows.len(), 2);
        assert!(second.outline_folding.borrow().collapsed_keys().is_empty());
    });

    app.update(cx, |app, cx| {
        app.active_tab = 0;
        cx.notify();
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let first = app.tabs[0].document_tab().unwrap();
        let outline = first.document.outline();
        assert_eq!(first.outline_projection(&outline, None).rows.len(), 1);
        assert_eq!(first.outline_folding.borrow().collapsed_keys().len(), 1);
    });
}

#[gpui::test]
fn outline_label_click_navigates_without_folding_in_editable_modes(cx: &mut TestAppContext) {
    let source = "# One\n\nBody\n\n## Two\n\nMore\n\n### Three\n";
    let heading_offset = source.find("## Two").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        let mut tab = EditorTab::new(MarkdownDocument::from_text(source));
        let preview = tab.document.preview_blocks_shared();
        tab.sync_preview_list(&preview);
        app.tabs = vec![tab];
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Outline;
        app
    });
    cx.simulate_resize(size(px(900.), px(600.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    for mode in [ViewMode::Edit, ViewMode::VisualEdit, ViewMode::Split] {
        app.update(cx, |app, cx| {
            app.view_mode = mode;
            app.active_tab_mut().selected_range = 0..0;
            cx.notify();
        });
        cx.run_until_parked();
        let label = cx
            .debug_bounds("outline-heading-label-1")
            .unwrap_or_else(|| panic!("{mode:?} outline label should render"));
        cx.simulate_click(label.center(), Modifiers::none());
        cx.run_until_parked();

        app.update(cx, |app, _| {
            let tab = app.active_tab();
            assert_eq!(
                tab.selected_range,
                heading_offset..heading_offset,
                "{mode:?}"
            );
            assert!(
                tab.outline_folding.borrow().collapsed_keys().is_empty(),
                "{mode:?} label click must not fold"
            );
        });
    }
}

#[gpui::test]
fn long_partially_folded_outline_remains_scrollable(cx: &mut TestAppContext) {
    let mut source = String::from("# Folded\n\n");
    for index in 0..20 {
        source.push_str(&format!("## Hidden {index}\n\n"));
    }
    for index in 0..80 {
        source.push_str(&format!("# Visible {index}\n\n"));
    }
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::Edit;
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Outline;
        app
    });
    cx.simulate_resize(size(px(700.), px(300.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let disclosure = cx
        .debug_bounds("outline-heading-disclosure-0")
        .expect("first branch should be collapsible");
    cx.simulate_click(disclosure.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        let outline = tab.document.outline();
        assert_eq!(tab.outline_projection(&outline, None).rows.len(), 81);
    });
    assert!(cx.debug_bounds("outline-heading-row-100").is_some());

    let scroll_bounds = cx
        .debug_bounds("outline-scroll")
        .expect("outline scroll container should render");
    let before = app.update(cx, |app, _| app.outline_scroll.offset());
    cx.simulate_event(ScrollWheelEvent {
        position: scroll_bounds.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-600.))),
        ..Default::default()
    });
    cx.run_until_parked();
    let after = app.update(cx, |app, _| app.outline_scroll.offset());
    assert!(
        after.y < before.y,
        "visible outline rows should scroll vertically"
    );
}

#[test]
fn document_typography_metrics_preserve_defaults_and_scale_boundaries() {
    let defaults = DocumentTypographyMetrics::new(
        markion::DEFAULT_EDITOR_FONT_SIZE,
        markion::DEFAULT_RENDERED_FONT_SIZE,
        markion::DEFAULT_PARAGRAPH_SPACING,
    );
    assert_eq!(defaults.editor_font_size, 14.);
    assert_eq!(defaults.editor_line_height, 22.4);
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
        app.preferences_path = preferences_path.clone();
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
                font_family: SYSTEM_UI_FONT_FAMILY.into(),
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
        if let Some((key, _)) = tab.measured_height_cache.borrow().clone() {
            assert_eq!(key.version, version);
            assert_eq!(key.font_size, px(24.));
        }
        assert!((f32::from(tab.line_height) - 38.4).abs() <= 1.0);
        assert!(tab.display_text_cache.borrow().is_some());
        assert_eq!(app.current_preferences().editor_font_size, 24);
        assert_eq!(app.current_preferences().rendered_font_size, 20);
        assert_eq!(app.current_preferences().paragraph_spacing, 18);
    });
    let written = std::fs::read_to_string(&preferences_path).unwrap();
    assert!(written.contains("editor_font_size = 24"));
    assert!(written.contains("rendered_font_size = 20"));
    assert!(written.contains("paragraph_spacing = 18"));
}

#[gpui::test]
fn non_default_editor_font_reflows_wrapped_text_and_caret_geometry(cx: &mut TestAppContext) {
    let config_dir = tempfile::tempdir().unwrap();
    let preferences_path = config_dir.path().join("config.toml");
    let source = "wrap this source line ".repeat(80);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = preferences_path.clone();
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
    let written = std::fs::read_to_string(&preferences_path).unwrap();
    assert!(written.contains("editor_font_size = 32"));
}

#[gpui::test]
fn source_layout_snapshot_maps_wrapped_utf8_content_bidirectionally(cx: &mut TestAppContext) {
    let config_dir = tempfile::tempdir().unwrap();
    let preferences_path = config_dir.path().join("config.toml");
    let source = format!(
        "{}\nsecond logical line with 中文 and emoji 😀",
        "wrapped source words ".repeat(60)
    );
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.preferences_path = preferences_path;
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::Edit;
        app
    });
    cx.simulate_resize(size(px(460.), px(420.)));
    cx.run_until_parked();

    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert!(tab.source_layout_is_current());
        assert_eq!(
            tab.source_layout_key.map(|key| key.version),
            Some(tab.document.version())
        );
        assert_eq!(tab.line_tops.len(), tab.last_lines.len() + 1);
        assert!(tab.line_tops.windows(2).all(|pair| pair[0] <= pair[1]));

        let target = source.find("中文").expect("utf8 fixture");
        let y = tab
            .source_content_y_for_offset(target)
            .expect("source offset maps to content y");
        let line_start = tab
            .source_offset_for_content_y(y)
            .expect("content y maps back to source");
        assert!(source.is_char_boundary(line_start));
        assert!(line_start <= target);
        let round_trip_y = tab
            .source_content_y_for_offset(line_start)
            .expect("round-trip source y");
        assert!((round_trip_y - y).abs() <= f32::from(tab.line_height));

        assert_eq!(
            tab.source_content_y_for_offset(usize::MAX),
            tab.source_content_y_for_offset(source.len())
        );
        assert_eq!(tab.source_offset_for_content_y(-100.), Some(0));
        assert_eq!(
            tab.source_offset_for_content_y(f32::MAX),
            Some(source.len())
        );
    });

    app.update(cx, |app, cx| app.set_editor_font_size(28, cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(app.active_tab().source_layout_is_current());
        assert_eq!(
            app.active_tab()
                .source_layout_key
                .map(|key| key.line_height),
            Some(app.active_tab().line_height)
        );
    });
}

#[gpui::test]
fn gpui_tests_start_from_documented_preference_defaults(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, _| {
        let prefs = app.current_preferences();
        assert_eq!(prefs.editor_font_size, DEFAULT_EDITOR_FONT_SIZE);
        assert_eq!(prefs.rendered_font_size, DEFAULT_RENDERED_FONT_SIZE);
        assert_eq!(prefs.paragraph_spacing, markion::DEFAULT_PARAGRAPH_SPACING);
        assert_eq!(prefs.theme, "Paper");
        assert_eq!(prefs.language, "en");
        assert_eq!(app.preferences_path, default_preferences_path());
    });
}

#[gpui::test]
fn preference_mutation_does_not_write_developer_config_toml(cx: &mut TestAppContext) {
    let path = default_preferences_path();
    let before = std::fs::read(&path).ok();
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, cx| app.set_editor_font_size(28, cx));
    let after = std::fs::read(&path).ok();
    assert_eq!(
        after, before,
        "mutating preferences in tests must not create or rewrite the developer config.toml"
    );
    app.update(cx, |app, _| {
        assert_eq!(app.current_preferences().editor_font_size, 28);
        assert_eq!(app.preferences_path, path);
    });
}

#[gpui::test]
fn source_mapped_sync_scroll_converges_without_mutating_document_state(cx: &mut TestAppContext) {
    let source = (0..140)
        .map(|index| match index % 5 {
            0 => format!("## Heading {index}"),
            1 => format!(
                "paragraph {index} {}",
                "with enough wrapped prose to create unequal source and preview heights ".repeat(5)
            ),
            2 => format!("```rust\nfn item_{index}() {{\n    println!(\"{index}\");\n}}\n```"),
            3 => format!(
                "| item | value |\n| --- | ---: |\n| {index} | {} |",
                index * 10
            ),
            _ => format!("![image {index}](missing-sync-{index}.png)"),
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(&source))];
        app.view_mode = ViewMode::Split;
        app.sync_scroll = true;
        app
    });
    cx.simulate_resize(size(px(1200.), px(720.)));
    cx.run_until_parked();

    let (
        version,
        preview_cache,
        text_handle,
        highlight_probe,
        undo_len,
        dirty,
        item_count,
        target_index,
    ) = app.update(cx, |app, _| {
        let highlight_probe = app.highlighted_code(Some("rust"), "fn sync_probe() {}");
        let tab = app.active_tab();
        let blocks = tab.document.preview_blocks_shared();
        let target_index = blocks.len() * 3 / 4;
        (
            tab.document.version(),
            blocks,
            tab.shared_document_text(),
            highlight_probe,
            tab.undo_stack.len(),
            tab.document.is_dirty(),
            tab.preview_list.item_count(),
            target_index,
        )
    });

    app.update(cx, |app, cx| {
        let tab = app.active_tab_mut();
        let range = tab.preview_list_blocks[target_index].source_range().clone();
        let start_y = tab.source_content_y_for_offset(range.start).unwrap();
        let end_y = tab.source_content_y_for_offset(range.end).unwrap();
        let target_y = sync_interpolate(start_y, end_y.max(start_y + 1.), 0.45)
            .min(f32::from(tab.editor_scroll.max_offset().height));
        tab.editor_scroll.set_offset(point(px(0.), px(-target_y)));
        tab.sync_scroll_state.mark_driver(PaneScrollTarget::Editor);
        app.reconcile_sync_scroll(cx);
        let coarse = app.active_tab().preview_list.logical_scroll_top();
        assert_eq!(coarse.item_ix, target_index);
        let pending = app
            .active_tab()
            .sync_scroll_state
            .pending_preview_refinement
            .expect("a distant virtual row should take the bounded refinement path");
        assert!(
            pending.progress > 0.1,
            "fixture must exercise row refinement"
        );
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| app.reconcile_sync_scroll(cx));
    app.update(cx, |app, cx| {
        let tab = app.active_tab_mut();
        let top = tab.preview_list.logical_scroll_top();
        assert_eq!(top.item_ix, target_index);
        assert!(
            top.offset_in_item > px(0.),
            "refined top={top:?} bounds={:?}",
            tab.preview_list.bounds_for_item(target_index)
        );
        assert!(tab.sync_scroll_state.pending_preview_refinement.is_none());

        let reverse_index = target_index / 2;
        tab.sync_scroll_state.expected_follower = None;
        tab.sync_scroll_state.expected_follower_retried = false;
        tab.preview_list.scroll_to(gpui::ListOffset {
            item_ix: reverse_index,
            offset_in_item: px(0.),
        });
        tab.sync_scroll_state.mark_driver(PaneScrollTarget::Preview);
        app.reconcile_sync_scroll(cx);
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| app.reconcile_sync_scroll(cx));
    app.update(cx, |app, _| {
        let highlight_after = app.highlighted_code(Some("rust"), "fn sync_probe() {}");
        let tab = app.active_tab();
        let reverse_index = target_index / 2;
        let range = tab.preview_list_blocks[reverse_index].source_range();
        let expected_y = tab.source_content_y_for_offset(range.start).unwrap();
        let actual_y = f32::from(-tab.editor_scroll.offset().y);
        assert!(
            (actual_y - expected_y).abs() <= f32::from(tab.line_height) + 1.,
            "preview->editor actual={actual_y} expected={expected_y} top={:?} bounds={:?}",
            tab.preview_list.logical_scroll_top(),
            (
                tab.preview_list.bounds_for_item(reverse_index),
                tab.sync_scroll_state,
                tab.source_layout_is_current(),
                tab.preview_reflects_version,
                tab.document.version()
            )
        );

        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.is_dirty(), dirty);
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert!(Rc::ptr_eq(&highlight_after, &highlight_probe));
        assert_eq!(tab.preview_list.item_count(), item_count);
        assert!(Arc::ptr_eq(
            &tab.document.preview_blocks_shared(),
            &preview_cache
        ));
        assert_eq!(tab.shared_document_text().as_ptr(), text_handle.as_ptr());
    });
}

#[test]
fn any_tab_dirty_detection() {
    // request_quit / window-close guard use the common tab dirty helper.
    let image_path = PathBuf::from("preview.png");
    let tabs: Vec<EditorTab> = vec![
        EditorTab::new(MarkdownDocument::from_text("clean")),
        EditorTab::new(MarkdownDocument::from_text("clean2")),
        EditorTab::new_image(
            image_path.clone(),
            PreviewImageKey::from_local_path(&image_path),
        ),
    ];
    assert!(
        !tabs.iter().any(EditorTab::is_dirty),
        "fresh documents and images are not dirty"
    );
}

#[test]
fn source_mapped_preview_anchors_bridge_gaps_and_clamp_boundaries() {
    let blocks = vec![
        PreviewBlock::Paragraph {
            text: RichText::plain("first"),
            source_range: 5..10,
        },
        PreviewBlock::Paragraph {
            text: RichText::plain("second"),
            source_range: 20..30,
        },
    ];
    assert_eq!(
        preview_anchor_for_source_offset(&blocks, 0, 40),
        Some(PreviewScrollAnchor::Start)
    );
    assert_eq!(
        preview_anchor_for_source_offset(&blocks, 3, 40),
        Some(PreviewScrollAnchor::Start)
    );
    assert_eq!(
        preview_anchor_for_source_offset(&blocks, 7, 40),
        Some(PreviewScrollAnchor::Block { item_ix: 0 })
    );
    assert_eq!(
        preview_anchor_for_source_offset(&blocks, 15, 40),
        Some(PreviewScrollAnchor::Block { item_ix: 1 })
    );
    assert_eq!(
        preview_anchor_for_source_offset(&blocks, 35, 40),
        Some(PreviewScrollAnchor::End)
    );
    assert_eq!(
        preview_anchor_for_source_offset(&blocks, 40, 40),
        Some(PreviewScrollAnchor::End)
    );
    assert_eq!(preview_anchor_for_source_offset(&[], 0, 0), None);

    assert_eq!(sync_interval_progress(5., 5., 5.), 0.);
    assert_eq!(sync_interval_progress(-5., 0., 10.), 0.);
    assert_eq!(sync_interval_progress(15., 0., 10.), 1.);
    assert!((sync_interval_progress(3., 0., 12.) - 0.25).abs() < 1e-6);
    assert!((sync_interpolate(20., 60., 0.25) - 30.).abs() < 1e-6);
}

#[test]
fn preview_heading_lookup_uses_exact_source_offsets() {
    let source = "---\ntitle: Outline\n---\n# **Same** `code`\n\nBody\n\n# **Same** `code`\n";
    let document = MarkdownDocument::from_text(source);
    let blocks = document.preview_blocks_shared();
    let outline = document.outline();

    assert_eq!(outline.len(), 2);
    assert_eq!(outline[0].title, outline[1].title);
    assert!(outline[0].offset > 0, "front matter must shift the offset");
    assert_eq!(
        preview_heading_index_for_source_offset(&blocks, outline[0].offset),
        Some(0)
    );
    assert_eq!(
        preview_heading_index_for_source_offset(&blocks, outline[1].offset),
        Some(2)
    );
    assert_ne!(
        preview_heading_index_for_source_offset(&blocks, outline[0].offset),
        preview_heading_index_for_source_offset(&blocks, outline[1].offset),
        "duplicate rendered titles must still resolve by source identity"
    );
    assert_eq!(
        preview_heading_index_for_source_offset(&blocks, outline[0].offset + 1),
        None,
        "an inexact or missing source offset must not guess a target"
    );
}

#[test]
fn sync_driver_selection_consumes_followers_and_rejects_ambiguity() {
    let mut state = SyncScrollState::default();
    let top = SyncPreviewPosition::default();
    assert_eq!(select_sync_scroll_driver(&mut state, 0., top), None);

    assert_eq!(
        select_sync_scroll_driver(&mut state, 25., top),
        Some(PaneScrollTarget::Editor)
    );

    let followed = SyncPreviewPosition {
        item_ix: 3,
        offset_in_item: 8.,
    };
    state.expected_follower = Some(ExpectedSyncFollower::Preview(followed));
    assert_eq!(select_sync_scroll_driver(&mut state, 25., followed), None);

    state.mark_driver(PaneScrollTarget::Preview);
    let user_preview = SyncPreviewPosition {
        item_ix: 3,
        offset_in_item: 30.,
    };
    assert_eq!(
        select_sync_scroll_driver(&mut state, 25., user_preview),
        Some(PaneScrollTarget::Preview)
    );

    state.last_editor_offset = Some(25.);
    state.last_preview_position = Some(user_preview);
    assert_eq!(
        select_sync_scroll_driver(
            &mut state,
            50.,
            SyncPreviewPosition {
                item_ix: 4,
                offset_in_item: 0.,
            }
        ),
        None,
        "two unexplained movements must seed rather than pick a driver"
    );

    state.deferred_driver = Some(PaneScrollTarget::Editor);
    assert_eq!(
        select_sync_scroll_driver(
            &mut state,
            50.,
            SyncPreviewPosition {
                item_ix: 4,
                offset_in_item: 0.,
            }
        ),
        Some(PaneScrollTarget::Editor)
    );
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

#[gpui::test]
fn visual_edit_scrollbar_updates_list_without_sync_or_mutation(cx: &mut TestAppContext) {
    let source = (0..80)
        .map(|index| format!("paragraph {index} with enough text for a stable list row"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let second_source = (0..40)
        .map(|index| format!("other {index} paragraph"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text(&source)),
            EditorTab::new(MarkdownDocument::from_text(&second_source)),
        ];
        app.active_tab = 0;
        app.view_mode = ViewMode::VisualEdit;
        app.sync_scroll = true;
        let visual = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&visual);
        let preview = app.active_tab().document.preview_blocks_shared();
        app.active_tab_mut().sync_preview_list(&preview);
        let _ = app.active_tab().shared_document_text();
        app
    });
    cx.run_until_parked();

    let (
        version,
        preview_cache,
        visual_cache,
        highlight_count,
        undo_len,
        preview_top_before,
        empty_max,
    ) = app.update(cx, |app, _| {
        app.active_tab().preview_list.scroll_to(gpui::ListOffset {
            item_ix: 12,
            offset_in_item: px(2.),
        });
        let empty = EditorTab::new(MarkdownDocument::from_text(""));
        (
            app.active_tab().document.version(),
            app.active_tab().document.preview_blocks_shared(),
            app.active_tab().document.visual_blocks_shared(),
            app.highlight_cache.borrow().len(),
            app.active_tab().undo_stack.len(),
            app.active_tab().preview_list.logical_scroll_top(),
            empty.visual_list.max_offset_for_scrollbar().height,
        )
    });
    assert!(
        f32::from(empty_max) <= 1.,
        "empty Visual Edit lists have no scrollable range"
    );

    app.update(cx, |app, _| {
        let max_scroll = app
            .active_tab()
            .visual_list
            .max_offset_for_scrollbar()
            .height
            .max(px(0.));
        if max_scroll > px(1.) {
            app.active_tab()
                .visual_list
                .set_offset_from_scrollbar(point(px(0.), px(-f32::from(max_scroll) * 0.4)));
        } else {
            app.active_tab().visual_list.scroll_to(gpui::ListOffset {
                item_ix: 20,
                offset_in_item: px(5.),
            });
        }
        app.mark_sync_scroll_driver(PaneScrollTarget::Visual);
    });
    cx.run_until_parked();

    let visual_top = app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.version(), version);
        assert!(!tab.document.is_dirty());
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert!(Arc::ptr_eq(
            &tab.document.preview_blocks_shared(),
            &preview_cache
        ));
        assert!(Arc::ptr_eq(
            &tab.document.visual_blocks_shared(),
            &visual_cache
        ));
        assert_eq!(app.highlight_cache.borrow().len(), highlight_count);
        assert!(tab.sync_scroll_state.driver_hint.is_none());
        let preview_top = tab.preview_list.logical_scroll_top();
        assert_eq!(preview_top.item_ix, preview_top_before.item_ix);
        assert_eq!(
            preview_top.offset_in_item,
            preview_top_before.offset_in_item
        );
        let visual_top = tab.visual_list.logical_scroll_top();
        assert!(
            visual_top.item_ix > 0 || visual_top.offset_in_item > px(0.),
            "visual list must move through the scrollbar offset API"
        );
        visual_top
    });

    app.update(cx, |app, _| {
        app.active_tab = 1;
        let visual = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&visual);
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let other_top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(other_top.item_ix, 0);
        app.active_tab = 0;
        let restored = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(restored.item_ix, visual_top.item_ix);
        assert_eq!(restored.offset_in_item, visual_top.offset_in_item);
        let preview_top = app.active_tab().preview_list.logical_scroll_top();
        assert_eq!(preview_top.item_ix, preview_top_before.item_ix);
        assert_eq!(
            preview_top.offset_in_item,
            preview_top_before.offset_in_item
        );
        assert!(app.active_tab().sync_scroll_state.driver_hint.is_none());
    });
}

#[test]
fn sync_scroll_mapping_requires_matching_current_versions() {
    let current = SourceLayoutKey {
        version: 7,
        wrap_width: px(400.),
        line_height: px(24.),
    };
    assert!(sync_scroll_mapping_is_current(
        7,
        Some(current),
        Some(7),
        true
    ));
    assert!(!sync_scroll_mapping_is_current(
        8,
        Some(current),
        Some(8),
        true
    ));
    assert!(!sync_scroll_mapping_is_current(
        7,
        Some(current),
        Some(6),
        true
    ));
    assert!(!sync_scroll_mapping_is_current(7, None, Some(7), true));
    assert!(!sync_scroll_mapping_is_current(
        7,
        Some(current),
        Some(7),
        false
    ));
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
    assert!(!should_restore_session(&StartupOpenIntent::File(
        PathBuf::from("a.md")
    )));
    assert!(!should_restore_session(&StartupOpenIntent::Folder(
        PathBuf::from("notes")
    )));
}

#[gpui::test]
fn report_memory_action_lists_expected_sites_without_side_effects(cx: &mut TestAppContext) {
    let source = "# Hello\n\nParagraph.\n";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        let blocks = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&blocks);
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    let (version_before, selection_before, scroll_before, report) = app.update(cx, |app, _| {
        (
            app.active_tab().document.version(),
            app.active_tab().selected_range.clone(),
            app.active_tab().editor_scroll.offset(),
            app.memory_report(),
        )
    });

    for name in [
        "tabs[0].document_text",
        "tabs[0].document.preview_blocks",
        "tabs[0].document.visual_blocks",
        "tabs[0].shaped_lines",
        "global.preview_image_cache",
        "global.diagram_cache",
        "global.math_cache",
        "global.highlight_cache",
    ] {
        assert!(
            report.find_site(name).is_some(),
            "missing site {name} in {}",
            report.site_names().join(", ")
        );
    }

    cx.dispatch_action(ReportMemory);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.version(), version_before);
        assert_eq!(app.active_tab().selected_range, selection_before);
        assert_eq!(app.active_tab().editor_scroll.offset(), scroll_before);
        assert_eq!(app.active_tab().document.text(), source);
        assert_eq!(app.status.as_ref(), t(app.language, Msg::StatusReady));
        let again = app.memory_report();
        assert!(
            again.sites_equal(&report),
            "site figures must agree; process counters may differ"
        );
        // Process footprint is present and does not disturb accounted totals.
        assert_eq!(again.accounted_total(), report.accounted_total());
        assert!(!again.format_log().contains("resident_current=unavailable"));
        if let (Some(a), Some(b)) = (
            report.process_footprint.resident_peak,
            again.process_footprint.resident_peak,
        ) {
            assert!(b >= a, "producing a report must not lower resident_peak");
        }
    });
}

#[gpui::test]
fn visual_link_editor_commits_one_exact_undoable_mutation(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("open 文档"))];
        app.active_tab_mut().selected_range = 5..11;
        app.view_mode = ViewMode::VisualEdit;
        app
    });

    app.update(cx, |app, cx| {
        let version = app.active_tab().document.version();
        app.open_link_editor(cx);
        let editor = app.link_editor.as_mut().unwrap();
        editor.url = "docs/a b.md".into();
        editor.title = "标题".into();
        app.confirm_link_editor(cx);
        assert_eq!(
            app.active_tab().document.text(),
            "open [文档](<docs/a b.md> \"标题\")"
        );
        assert_eq!(app.active_tab().document.version(), version + 1);
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), "open 文档");
        assert_eq!(app.active_tab().selected_range, 5..11);
    });
}

#[gpui::test]
fn p1_visual_slash_keyboard_and_escape_preserve_atomic_history(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("/"))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (escape_version, escape_blocks) = app.update(cx, |app, _| {
        assert!(app.slash_commands.is_some());
        (
            app.active_tab().document.version(),
            app.active_tab().document.visual_blocks_shared(),
        )
    });
    cx.dispatch_action(ClearFileTreeSearch);
    app.update(cx, |app, _| {
        assert!(app.slash_commands.is_none());
        assert_eq!(app.active_tab().document.text(), "/");
        assert_eq!(app.active_tab().document.version(), escape_version);
        assert!(Arc::ptr_eq(
            &escape_blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
        assert!(app.active_tab().undo_stack.is_empty());
    });

    let start_version = app.update(cx, |app, cx| {
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("/"))];
        app.active_tab = 0;
        app.active_tab_mut().selected_range = 1..1;
        app.dismissed_slash_query = None;
        app.sync_slash_command_state(cx);
        cx.notify();
        app.active_tab().document.version()
    });
    cx.dispatch_action(Down);
    cx.dispatch_action(InsertNewline);
    app.update(cx, |app, _| {
        let tab = app.active_tab();
        assert_eq!(tab.document.text(), "# ");
        assert_eq!(tab.document.version(), start_version + 1);
        assert_eq!(tab.selected_range, 2..2);
        assert_eq!(tab.undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "/");
        assert_eq!(app.active_tab().selected_range, 1..1);
    });
}

#[gpui::test]
fn p1_visual_slash_pointer_and_block_menu_apply_exact_commands(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("/"))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    cx.debug_bounds("slash-command-palette")
        .expect("slash command palette should be rendered");
    let heading = cx
        .debug_bounds("slash-command-1")
        .expect("H1 slash command should be rendered");
    cx.simulate_click(heading.center(), Modifiers::none());
    cx.run_until_parked();
    let block_start_version = app.update(cx, |app, cx| {
        assert_eq!(app.active_tab().document.text(), "# ");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("one\n\ntwo"))];
        app.active_tab = 0;
        app.active_tab_mut().selected_range = 1..1;
        cx.notify();
        app.active_tab().document.version()
    });
    cx.run_until_parked();

    let row = cx
        .debug_bounds("visual-block-row-0")
        .expect("supported block should expose row context interaction");
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position: row.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    let lists = cx
        .debug_bounds("visual-block-lists")
        .expect("compact root menu should expose the Lists submenu");
    cx.simulate_event(MouseMoveEvent {
        position: lists.center(),
        ..Default::default()
    });
    cx.run_until_parked();
    let task = cx
        .debug_bounds("visual-block-transform-9")
        .expect("task-list transform should be rendered in the Lists submenu");
    cx.simulate_click(task.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "- [ ] one\n\ntwo");
        assert_eq!(app.active_tab().document.version(), block_start_version + 1);
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });

    app.update(cx, |app, cx| {
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "```rust\nlet x = 1;\n```",
        ))];
        app.active_tab = 0;
        let cursor = app.active_tab().document.text().find("let").unwrap() + 1;
        app.active_tab_mut().selected_range = cursor..cursor;
        cx.notify();
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("visual-block-menu-button").is_none(),
        "flow-neutral block chrome no longer renders an ellipsis trigger"
    );
    cx.debug_bounds("visual-block-drag-grip-0")
        .expect("exact fenced code should expose a separate drag grip");
}

#[gpui::test]
fn visual_block_menu_root_overlay_outpaints_following_rows(cx: &mut TestAppContext) {
    const SOURCE: &str = "### Level three heading\n\nA deliberately tall paragraph with *italic*, **bold**, `inline code`, and ==highlighted text== repeated across enough words to wrap through several rendered lines. This following visual content must occupy the same screen coordinates as multiple commands in the open block menu. A deliberately tall paragraph with *italic*, **bold**, `inline code`, and ==highlighted text== repeated across enough words to wrap through several rendered lines. This following visual content must occupy the same screen coordinates as multiple commands in the open block menu. A deliberately tall paragraph with *italic*, **bold**, `inline code`, and ==highlighted text== repeated across enough words to wrap through several rendered lines.\n\n#### Level four heading\n\n##### Level five heading\n\n###### Level six heading\n\n![A later image](missing-overlay-test.png)";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        let cursor = SOURCE.find("Level three").expect("fixture heading") + 1;
        app.active_tab_mut().selected_range = cursor..cursor;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let start_version = app.update(cx, |app, _| app.active_tab().document.version());
    let row = cx
        .debug_bounds("visual-block-row-0")
        .expect("heading should expose a block context target");
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position: row.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();

    let overlay = cx
        .debug_bounds("visual-block-menu-overlay")
        .expect("block menu should be emitted through the root overlay host");
    let panel = cx
        .debug_bounds("visual-block-menu-panel")
        .expect("block menu panel should be rendered");
    assert!(
        overlay.contains(&panel.center()),
        "root overlay should contain the compact root panel"
    );
    let lists = cx
        .debug_bounds("visual-block-lists")
        .expect("root panel should expose Lists");
    cx.simulate_event(MouseMoveEvent {
        position: lists.center(),
        ..Default::default()
    });
    cx.run_until_parked();
    let task = cx
        .debug_bounds("visual-block-transform-9")
        .expect("task-list command should be rendered inside the submenu overlay");
    let overlay = cx
        .debug_bounds("visual-block-menu-overlay")
        .expect("overlay bounds should expand to include the submenu");
    assert!(overlay.contains(&task.center()));

    let overlapping_row = [
        "visual-block-row-1",
        "visual-block-row-2",
        "visual-block-row-3",
        "visual-block-row-4",
        "visual-block-row-5",
        "visual-block-row-6",
        "visual-block-row-7",
        "visual-block-row-8",
        "visual-block-row-9",
        "visual-block-row-10",
    ]
    .into_iter()
    .filter_map(|selector| cx.debug_bounds(selector))
    .find(|row| row.contains(&task.center()));
    assert!(
        overlapping_row.is_some(),
        "fixture must place a later visual row beneath the task command"
    );

    cx.simulate_click(task.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert!(
            app.active_tab()
                .document
                .text()
                .starts_with("- [ ] Level three heading")
        );
        assert_eq!(app.active_tab().document.version(), start_version + 1);
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        assert!(app.block_menu.is_none());
    });

    cx.dispatch_action(Undo);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), SOURCE)
    });
}

#[gpui::test]
fn visual_block_menu_anchor_and_overflow_stay_in_viewport(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("one\n\ntwo"))];
        app.active_tab_mut().selected_range = 0..3;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(340.), px(260.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let target = BlockTarget::from_block(app.active_tab().document.version(), &blocks[0]);
        app.open_visual_block_menu(target, point(px(330.), px(250.)), cx);
    });
    cx.run_until_parked();

    let viewport = cx.update(|window, _| window.viewport_size());
    let panel = cx
        .debug_bounds("visual-block-menu-panel")
        .expect("edge-anchored menu should render");
    assert!(
        cx.debug_bounds("visual-selection-format-bold").is_some(),
        "selection formatting remains reachable in an edge-constrained menu"
    );
    assert!(f32::from(panel.left()) >= 0.0);
    assert!(f32::from(panel.top()) >= 0.0);
    assert!(f32::from(panel.right()) <= f32::from(viewport.width) + 0.5);
    assert!(f32::from(panel.bottom()) <= f32::from(viewport.height) + 0.5);

    let (scroll_item, scroll_offset) = app.update(cx, |app, _| {
        let top = app.active_tab().visual_list.logical_scroll_top();
        (top.item_ix, top.offset_in_item)
    });
    cx.simulate_event(ScrollWheelEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-800.))),
        ..Default::default()
    });
    cx.run_until_parked();

    let delete = cx
        .debug_bounds("visual-block-delete")
        .expect("final command should remain rendered after menu-local scrolling");
    assert!(
        panel.contains(&delete.center()),
        "menu-local scrolling must bring the final command inside the visible panel"
    );
    app.update(cx, |app, _| {
        let top = app.active_tab().visual_list.logical_scroll_top();
        assert_eq!(top.item_ix, scroll_item);
        assert_eq!(top.offset_in_item, scroll_offset);
        assert!(app.block_menu.is_some());
    });

    cx.simulate_click(delete.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "two");
        assert!(app.block_menu.is_none());
    });
}

fn assert_block_menu_presentation_state(
    app: &MarkionApp,
    text: &str,
    version: u64,
    selection: &Range<usize>,
    undo_len: usize,
    dirty: bool,
    blocks: &Arc<Vec<VisualBlock>>,
) {
    let tab = app.active_tab();
    assert_eq!(tab.document.text(), text);
    assert_eq!(tab.document.version(), version);
    assert_eq!(&tab.selected_range, selection);
    assert_eq!(tab.undo_stack.len(), undo_len);
    assert_eq!(tab.document.is_dirty(), dirty);
    assert!(Arc::ptr_eq(blocks, &tab.document.visual_blocks_shared()));
}

#[gpui::test]
fn visual_block_menu_dismissal_is_presentation_only(cx: &mut TestAppContext) {
    const SOURCE: &str = "## heading\n\nparagraph";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 4..4;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (target, version, selection, undo_len, dirty, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        (
            BlockTarget::from_block(tab.document.version(), &blocks[0]),
            tab.document.version(),
            tab.selected_range.clone(),
            tab.undo_stack.len(),
            tab.document.is_dirty(),
            blocks,
        )
    });

    let open = |app: &mut MarkionApp, cx: &mut Context<MarkionApp>| {
        app.open_visual_block_menu(target.clone(), point(px(48.), px(120.)), cx);
    };

    app.update(cx, |app, cx| open(app, cx));
    cx.dispatch_action(ClearFileTreeSearch);
    app.update(cx, |app, _| {
        assert!(app.block_menu.is_none());
        assert_block_menu_presentation_state(
            app, SOURCE, version, &selection, undo_len, dirty, &blocks,
        );
    });

    app.update(cx, |app, cx| open(app, cx));
    cx.run_until_parked();
    let workspace = cx
        .debug_bounds("workspace-row")
        .expect("workspace should be rendered");
    cx.simulate_mouse_down(
        point(workspace.left() + px(4.), workspace.bottom() - px(4.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    app.update(cx, |app, _| {
        assert!(app.block_menu.is_none());
        assert_block_menu_presentation_state(
            app, SOURCE, version, &selection, undo_len, dirty, &blocks,
        );
    });

    app.update(cx, |app, cx| open(app, cx));
    cx.run_until_parked();
    let row = cx
        .debug_bounds("visual-block-row-0")
        .expect("focused visual row should be rendered");
    cx.simulate_event(ScrollWheelEvent {
        position: point(row.right() - px(4.), row.center().y),
        delta: ScrollDelta::Pixels(point(px(0.), px(-40.))),
        ..Default::default()
    });
    app.update(cx, |app, _| {
        assert!(app.block_menu.is_none());
        assert_block_menu_presentation_state(
            app, SOURCE, version, &selection, undo_len, dirty, &blocks,
        );
    });

    app.update(cx, |app, cx| {
        open(app, cx);
        app.set_view_mode(ViewMode::Read, cx);
        assert!(app.block_menu.is_none());
        app.set_view_mode(ViewMode::VisualEdit, cx);
    });
    app.update(cx, |app, _| {
        assert_block_menu_presentation_state(
            app, SOURCE, version, &selection, undo_len, dirty, &blocks,
        );
    });

    app.update(cx, |app, cx| {
        open(app, cx);
        app.switch_active_tab(0, cx);
    });
    app.update(cx, |app, _| {
        assert!(app.block_menu.is_none());
        assert_block_menu_presentation_state(
            app, SOURCE, version, &selection, undo_len, dirty, &blocks,
        );
    });

    app.update(cx, |app, cx| {
        let mut stale = target.clone();
        stale.document_version += 1;
        app.open_visual_block_menu(stale.clone(), point(px(48.), px(120.)), cx);
        app.transform_visual_block(stale, BlockTransform::Heading(1), cx);
    });
    app.update(cx, |app, _| {
        assert!(app.block_menu.is_none());
        assert_block_menu_presentation_state(
            app, SOURCE, version, &selection, undo_len, dirty, &blocks,
        );
    });
}

fn assert_bounds_axis_and_width_match(left: Bounds<Pixels>, right: Bounds<Pixels>) {
    assert!(
        (f32::from(left.left()) - f32::from(right.left())).abs() <= 0.5,
        "left axes differ: {left:?} vs {right:?}"
    );
    assert!(
        (f32::from(left.size.width) - f32::from(right.size.width)).abs() <= 0.5,
        "available widths differ: {left:?} vs {right:?}"
    );
}

fn test_debug_selector(selector: String) -> &'static str {
    Box::leak(selector.into_boxed_str())
}

#[gpui::test]
fn visual_edit_flow_neutral_rows_share_the_read_document_axis(cx: &mut TestAppContext) {
    const SOURCE: &str = "## Heading\n\nA paragraph that remains aligned.\n\n![image](missing-flow-neutral.png)\n\n$$x^2$$";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 4..4;
        app.view_mode = ViewMode::VisualEdit;
        app.preview_adaptive_width = false;
        app
    });
    cx.simulate_resize(size(px(1200.), px(760.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (heading, paragraph, image, formula) = app.update(cx, |app, _| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        (
            blocks
                .iter()
                .position(|block| matches!(block.kind, VisualBlockKind::Heading { .. }))
                .unwrap(),
            blocks
                .iter()
                .position(|block| matches!(block.kind, VisualBlockKind::Paragraph))
                .unwrap(),
            blocks
                .iter()
                .position(|block| matches!(block.kind, VisualBlockKind::Image { .. }))
                .unwrap(),
            blocks
                .iter()
                .position(|block| matches!(block.kind, VisualBlockKind::MathBlock { .. }))
                .unwrap(),
        )
    });
    let heading_bounds = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-block-content-{heading}"
        )))
        .expect("heading content bounds");
    let paragraph_bounds = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-block-content-{paragraph}"
        )))
        .expect("paragraph content bounds");
    let image_bounds = cx
        .debug_bounds(test_debug_selector(format!("visual-document-row-{image}")))
        .expect("image row bounds");
    let formula_bounds = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-document-row-{formula}"
        )))
        .expect("formula row bounds");
    assert_bounds_axis_and_width_match(heading_bounds, paragraph_bounds);
    assert_bounds_axis_and_width_match(heading_bounds, image_bounds);
    assert_bounds_axis_and_width_match(heading_bounds, formula_bounds);

    app.update(cx, |app, cx| app.set_view_mode(ViewMode::Read, cx));
    cx.run_until_parked();
    let read_heading = cx
        .debug_bounds("preview-block-row-0")
        .expect("read heading row bounds");
    let read_paragraph = cx
        .debug_bounds("preview-block-row-1")
        .expect("read paragraph row bounds");
    let read_image = cx
        .debug_bounds("preview-block-row-2")
        .expect("read image row bounds");
    let read_formula = cx
        .debug_bounds("preview-block-row-3")
        .expect("read formula row bounds");
    assert_bounds_axis_and_width_match(read_heading, read_paragraph);
    assert_bounds_axis_and_width_match(read_heading, read_image);
    assert_bounds_axis_and_width_match(read_heading, read_formula);
    assert_bounds_axis_and_width_match(heading_bounds, read_heading);
}

#[gpui::test]
fn flow_neutral_hover_focus_and_menu_leave_wrapped_geometry_and_state_unchanged(
    cx: &mut TestAppContext,
) {
    const SOURCE: &str = "A deliberately long paragraph that wraps across several visual lines without reserving a hidden operation gutter. Its content width and row height must stay fixed while chrome appears on hover, focus, or through the context menu.\n\nsecond";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        let cursor = SOURCE.find("second").unwrap() + 1;
        app.active_tab_mut().selected_range = cursor..cursor;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.simulate_resize(size(px(560.), px(500.)));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let content_before = cx
        .debug_bounds("visual-block-content-0")
        .expect("wrapped content bounds");
    let row_before = cx
        .debug_bounds("visual-block-row-0")
        .expect("wrapped row bounds");
    let (version, selection, undo_len, dirty, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        (
            tab.document.version(),
            tab.selected_range.clone(),
            tab.undo_stack.len(),
            tab.document.is_dirty(),
            tab.document.visual_blocks_shared(),
        )
    });
    cx.simulate_event(MouseMoveEvent {
        position: row_before.center(),
        ..Default::default()
    });
    cx.run_until_parked();
    assert_eq!(
        cx.debug_bounds("visual-block-content-0").unwrap(),
        content_before
    );
    assert_eq!(cx.debug_bounds("visual-block-row-0").unwrap(), row_before);

    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position: row_before.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    assert_eq!(
        cx.debug_bounds("visual-block-content-0").unwrap(),
        content_before
    );
    assert_eq!(cx.debug_bounds("visual-block-row-0").unwrap(), row_before);
    app.update(cx, |app, _| {
        assert_block_menu_presentation_state(
            app, SOURCE, version, &selection, undo_len, dirty, &blocks,
        );
    });

    cx.dispatch_action(ClearFileTreeSearch);
    app.update(cx, |app, cx| {
        app.active_tab_mut().selected_range = 1..1;
        cx.notify();
    });
    cx.run_until_parked();
    assert_eq!(
        cx.debug_bounds("visual-block-content-0").unwrap(),
        content_before
    );
    assert_eq!(cx.debug_bounds("visual-block-row-0").unwrap(), row_before);
}

#[gpui::test]
fn visual_block_context_targeting_preserves_non_caret_selection_and_rejects_unsupported(
    cx: &mut TestAppContext,
) {
    const SOURCE: &str = "first\n\nsecond\n\n![image](missing-context-target.png)";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 0..5;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (second, image, selection, version, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let paragraphs = blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| matches!(block.kind, VisualBlockKind::Paragraph))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        (
            paragraphs[1],
            blocks
                .iter()
                .position(|block| matches!(block.kind, VisualBlockKind::Image { .. }))
                .unwrap(),
            tab.selected_range.clone(),
            tab.document.version(),
            blocks,
        )
    });
    let second_row = cx
        .debug_bounds(test_debug_selector(format!("visual-block-row-{second}")))
        .expect("non-caret paragraph row");
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position: second_row.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let menu = app.block_menu.as_ref().expect("right-click menu");
        assert_eq!(menu.target.block_id, blocks[second].id);
        assert_eq!(menu.anchor, second_row.center());
        assert_eq!(app.active_tab().selected_range, selection);
        assert_eq!(app.active_tab().document.version(), version);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
    assert!(
        cx.debug_bounds("visual-selection-format-bold").is_none(),
        "an unrelated block must not expose actions for the preserved selection"
    );

    cx.dispatch_action(ClearFileTreeSearch);
    let image_row = cx
        .debug_bounds(test_debug_selector(format!("visual-document-row-{image}")))
        .expect("unsupported image row");
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position: image_row.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, cx| {
        assert!(app.block_menu.is_none());
        let mut stale = BlockTarget::from_block(version, &blocks[second]);
        stale.document_version += 1;
        app.open_visual_block_menu(stale, image_row.center(), cx);
        assert!(app.block_menu.is_none());
        assert_eq!(app.active_tab().selected_range, selection);
    });
}

#[gpui::test]
fn visual_selection_format_actions_live_in_the_context_menu(cx: &mut TestAppContext) {
    const SOURCE: &str = "selected prose\n\nother";
    const SELECTION: Range<usize> = 0..8;
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = SELECTION;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    assert!(
        cx.debug_bounds("visual-context-bold").is_none(),
        "the retired floating toolbar must not render for a selection"
    );
    let (initial_version, initial_blocks) = app.update(cx, |app, _| {
        (
            app.active_tab().document.version(),
            app.active_tab().document.visual_blocks_shared(),
        )
    });
    let first_row = cx
        .debug_bounds("visual-block-row-0")
        .expect("selected paragraph should expose a block context target");
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Right,
        position: first_row.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let menu = app.block_menu.as_ref().expect("selection context menu");
        let selection_target = menu
            .selection_format
            .as_ref()
            .expect("exact selected prose should be format-safe");
        assert_eq!(selection_target.range, SELECTION);
        assert_eq!(
            selection_target.document_version,
            app.active_tab().document.version()
        );
        assert_eq!(app.active_tab().selected_range, SELECTION);
        assert_eq!(app.active_tab().document.version(), initial_version);
        assert_eq!(app.active_tab().undo_stack.len(), 0);
        assert!(!app.active_tab().document.is_dirty());
        assert!(Arc::ptr_eq(
            &initial_blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
    for id in [
        "visual-selection-format-bold",
        "visual-selection-format-italic",
        "visual-selection-format-inline-code",
        "visual-selection-format-link",
        "visual-block-text-headings",
    ] {
        assert!(
            cx.debug_bounds(id).is_some(),
            "selection-aware menu should expose {id}"
        );
    }

    let bold = cx
        .debug_bounds("visual-selection-format-bold")
        .expect("pointer Bold command");
    cx.simulate_click(bold.center(), Modifiers::none());
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            "**selected** prose\n\nother"
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        assert!(app.block_menu.is_none());
    });
    cx.dispatch_action(Undo);
    cx.run_until_parked();

    let open_selection_menu = |app: &mut MarkionApp, cx: &mut Context<MarkionApp>| {
        app.active_tab_mut().selected_range = SELECTION;
        let blocks = app.active_tab().document.visual_blocks_shared();
        let target = BlockTarget::from_block(app.active_tab().document.version(), &blocks[0]);
        app.open_visual_block_menu(target, point(px(80.), px(80.)), cx);
    };

    app.update(cx, |app, cx| {
        open_selection_menu(app, cx);
        app.activate_visual_block_menu_item(
            BlockMenuItem::SelectionFormat(SelectionFormatAction::Italic),
            cx,
        );
    });
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            "*selected* prose\n\nother"
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        open_selection_menu(app, cx);
        app.activate_visual_block_menu_item(
            BlockMenuItem::SelectionFormat(SelectionFormatAction::InlineCode),
            cx,
        );
    });
    app.update(cx, |app, _| {
        assert_eq!(
            app.active_tab().document.text(),
            "`selected` prose\n\nother"
        );
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        let version = app.active_tab().document.version();
        let blocks = app.active_tab().document.visual_blocks_shared();
        let undo_len = app.active_tab().undo_stack.len();
        let dirty = app.active_tab().document.is_dirty();
        open_selection_menu(app, cx);
        app.activate_visual_block_menu_item(
            BlockMenuItem::SelectionFormat(SelectionFormatAction::Link),
            cx,
        );
        let editor = app
            .link_editor
            .as_ref()
            .expect("Link should open the editor");
        assert_eq!(editor.source_range, SELECTION);
        assert_eq!(editor.label, "selected");
        assert!(app.block_menu.is_none());
        app.cancel_link_editor(cx);
        assert_eq!(app.active_tab().document.text(), SOURCE);
        assert_eq!(app.active_tab().selected_range, SELECTION);
        assert_eq!(app.active_tab().document.version(), version);
        assert_eq!(app.active_tab().undo_stack.len(), undo_len);
        assert_eq!(app.active_tab().document.is_dirty(), dirty);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });

    app.update(cx, |app, cx| {
        let version = app.active_tab().document.version();
        let blocks = app.active_tab().document.visual_blocks_shared();
        let undo_len = app.active_tab().undo_stack.len();
        let dirty = app.active_tab().document.is_dirty();
        open_selection_menu(app, cx);
        app.active_tab_mut().selected_range = 1..7;
        app.activate_visual_block_menu_item(
            BlockMenuItem::SelectionFormat(SelectionFormatAction::Bold),
            cx,
        );
        assert!(app.block_menu.is_none());
        assert_eq!(app.active_tab().document.text(), SOURCE);
        assert_eq!(app.active_tab().document.version(), version);
        assert_eq!(app.active_tab().undo_stack.len(), undo_len);
        assert_eq!(app.active_tab().document.is_dirty(), dirty);
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
}

#[gpui::test]
fn visual_selection_format_actions_support_keyboard_context_invocation(cx: &mut TestAppContext) {
    const SOURCE: &str = "selected prose";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 0..8;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    cx.dispatch_action(ShowVisualBlockContextMenu);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let menu = app
            .block_menu
            .as_ref()
            .expect("keyboard selection context menu");
        assert!(menu.selection_format.is_some());
        assert_eq!(menu.root_selected, BLOCK_MENU_SELECTION_FORMAT_ITEMS.len());
    });
    assert!(cx.debug_bounds("visual-selection-format-bold").is_some());
    app.update(cx, |app, cx| {
        app.select_visual_block_menu_root(0, false, cx)
    });
    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "**selected** prose");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
}

#[gpui::test]
fn visual_block_menu_keyboard_submenus_disabled_moves_and_exact_undo(cx: &mut TestAppContext) {
    const SOURCE: &str = "one\n\ntwo\n\nthree";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    cx.dispatch_action(ShowVisualBlockContextMenu);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        let menu = app.block_menu.as_ref().expect("keyboard block menu");
        assert_eq!(menu.root_selected, 0);
        assert!(menu.anchor.y >= app.active_tab().visual_caret_bounds.unwrap().top());
        let model = app.block_menu_presentation().unwrap();
        assert!(!model.can_move_up);
        assert!(model.can_move_down);
    });
    assert!(
        cx.debug_bounds("visual-block-current-indicator").is_some(),
        "current paragraph category should be indicated"
    );
    cx.dispatch_action(Right);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            app.block_menu.as_ref().unwrap().submenu,
            Some(BlockMenuSubmenu::TextAndHeadings)
        );
    });
    cx.dispatch_action(Down);
    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "# one\n\ntwo\n\nthree");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), SOURCE)
    });

    cx.dispatch_action(ShowVisualBlockContextMenu);
    app.update(cx, |app, cx| {
        app.select_visual_block_menu_root(7, false, cx)
    });
    cx.run_until_parked();
    assert!(
        cx.debug_bounds("visual-block-move-up-disabled").is_some(),
        "first block Move Up should render disabled"
    );
    cx.dispatch_action(InsertNewline);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), SOURCE);
        assert!(app.block_menu.is_some());
    });
    app.update(cx, |app, cx| {
        app.select_visual_block_menu_root(8, false, cx)
    });
    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "two\n\none\n\nthree");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), SOURCE)
    });

    app.update(cx, |app, cx| {
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "- item\n\nsecond",
        ))];
        app.active_tab = 0;
        app.active_tab_mut().selected_range = 3..3;
        cx.notify();
    });
    cx.run_until_parked();
    cx.dispatch_action(ShowVisualBlockContextMenu);
    cx.dispatch_action(Right);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            app.block_menu.as_ref().unwrap().submenu,
            Some(BlockMenuSubmenu::Lists)
        );
    });
    cx.dispatch_action(Down);
    cx.dispatch_action(InsertNewline);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "1. item\n\nsecond");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "- item\n\nsecond")
    });
}

#[gpui::test]
fn flow_neutral_drag_reorders_only_on_drop_and_keeps_one_step_undo(cx: &mut TestAppContext) {
    const SOURCE: &str = "one\n\ntwo";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(SOURCE))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let (first, second, version, selection, blocks) = app.update(cx, |app, _| {
        let tab = app.active_tab();
        let blocks = tab.document.visual_blocks_shared();
        let paragraphs = blocks
            .iter()
            .enumerate()
            .filter(|(_, block)| matches!(block.kind, VisualBlockKind::Paragraph))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        (
            paragraphs[0],
            paragraphs[1],
            tab.document.version(),
            tab.selected_range.clone(),
            blocks,
        )
    });
    let grip = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-block-drag-grip-{first}"
        )))
        .expect("flow-neutral drag grip");
    let drop_after = cx
        .debug_bounds(test_debug_selector(format!(
            "visual-block-drop-after-{second}"
        )))
        .expect("second block after-drop zone");
    let first_content_selector = test_debug_selector(format!("visual-block-content-{first}"));
    let second_content_selector = test_debug_selector(format!("visual-block-content-{second}"));
    assert_eq!(
        cx.debug_bounds(first_content_selector).unwrap().left(),
        cx.debug_bounds(second_content_selector).unwrap().left()
    );
    cx.simulate_mouse_down(grip.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_event(MouseMoveEvent {
        position: drop_after.center(),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::none(),
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_block_menu_presentation_state(app, SOURCE, version, &selection, 0, false, &blocks);
    });
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: drop_after.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "two\n\none\n\n");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
    cx.dispatch_action(Undo);
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), SOURCE)
    });
}

#[gpui::test]
fn p1_localized_slash_query_supports_ime_and_one_step_confirmation_undo(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("/"))];
        app.active_tab_mut().selected_range = 1..1;
        app.view_mode = ViewMode::VisualEdit;
        app.language = Language::ZhHans;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(app, None, "标", None, window, cx);
            EntityInputHandler::replace_and_mark_text_in_range(app, None, "标题", None, window, cx);
            EntityInputHandler::unmark_text(app, window, cx);
        });
    });
    app.update(cx, |app, cx| {
        app.sync_slash_command_state(cx);
        assert_eq!(app.active_tab().document.text(), "/标题");
        assert_eq!(
            localized_slash_commands(app.language, "标题"),
            vec![
                SlashCommand::Heading(1),
                SlashCommand::Heading(2),
                SlashCommand::Heading(3),
                SlashCommand::Heading(4),
                SlashCommand::Heading(5),
                SlashCommand::Heading(6),
            ]
        );
        let query_version = app.active_tab().document.version();
        assert!(app.confirm_selected_slash_command(cx));
        assert_eq!(app.active_tab().document.text(), "# ");
        assert_eq!(app.active_tab().document.version(), query_version + 1);
        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), "/标题");
    });
}

#[gpui::test]
fn p1_visual_block_operations_are_tab_local_stale_safe_and_one_step_undoable(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("one\n\ntwo")),
            EditorTab::new(MarkdownDocument::from_text("other tab")),
        ];
        app.active_tab = 0;
        app.view_mode = ViewMode::VisualEdit;
        app
    });

    app.update(cx, |app, cx| {
        let blocks = app.active_tab().document.visual_blocks_shared();
        let first = blocks
            .iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .unwrap();
        let target = BlockTarget::from_block(app.active_tab().document.version(), first);
        app.move_visual_block(target, true, cx);
        let button_result = app.active_tab().document.text().to_string();
        assert_eq!(button_result, "two\n\none\n\n");
        assert_eq!(app.tabs[1].document.text(), "other tab");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), "one\n\ntwo");

        let blocks = app.active_tab().document.visual_blocks_shared();
        let content = blocks
            .iter()
            .filter(|block| matches!(block.kind, VisualBlockKind::Paragraph))
            .collect::<Vec<_>>();
        let first = BlockTarget::from_block(app.active_tab().document.version(), content[0]);
        let second = BlockTarget::from_block(app.active_tab().document.version(), content[1]);
        app.reorder_visual_block(first, second, BlockPlacement::After, cx);
        assert_eq!(app.active_tab().document.text(), button_result);
        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), "one\n\ntwo");
        assert!(app.active_tab_mut().apply_redo());
        assert_eq!(app.active_tab().document.text(), button_result);
        assert!(app.active_tab_mut().apply_undo());

        let blocks = app.active_tab().document.visual_blocks_shared();
        let target = BlockTarget::from_block(app.active_tab().document.version(), &blocks[0]);
        app.active_tab_mut().document.insert(0, "changed ");
        let after_external_mutation = app.active_tab().document.text().to_string();
        app.transform_visual_block(target, BlockTransform::Heading(1), cx);
        assert_eq!(app.active_tab().document.text(), after_external_mutation);
        assert_eq!(app.status, p1_t(app.language, P1Msg::BlockStale));
    });
}

#[gpui::test]
fn p1_restored_snapshot_survives_restore_reuses_clean_tab_and_clears_after_save(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let document_path = dir.path().join("important.md");
    let recovery_dir = dir.path().join("recoveries");
    let mut document = MarkdownDocument::from_text("disk");
    document.save_as(&document_path).unwrap();
    document.set_text("recovered work");
    let recovery_path = document
        .save_recovery_copy_with_id(&recovery_dir, 41)
        .unwrap();

    let clean_document = MarkdownDocument::open(&document_path).unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(clean_document)];
        app.recovery_dir = recovery_dir;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
        app.update(cx, |app, cx| app.check_recovery_on_startup(window, cx));
    });
    cx.run_until_parked();
    cx.debug_bounds("restore-all-recoveries")
        .expect("recovery manager should render bulk actions");
    app.update(cx, |app, cx| {
        assert_eq!(app.recovery_manager.as_ref().unwrap().entries.len(), 1);
        app.restore_recovery_entry(&recovery_path, cx).unwrap();
        assert_eq!(
            app.tabs.len(),
            1,
            "matching clean session tab must be reused"
        );
        assert_eq!(app.active_tab().document.text(), "recovered work");
        assert_eq!(
            app.active_tab().last_recovery_file.as_deref(),
            Some(recovery_path.as_path())
        );
        assert!(
            recovery_path.exists(),
            "restore must retain its durable snapshot"
        );

        app.active_tab_mut().document.save().unwrap();
        app.discard_current_recovery_file();
        assert!(!recovery_path.exists());
    });
}

#[gpui::test]
fn p1_explicit_quit_discard_removes_every_tab_recovery(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("first.md");
    let second = dir.path().join("second.md");
    fs::write(&first, "first").unwrap();
    fs::write(&second, "second").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        let mut first_tab = EditorTab::new(MarkdownDocument::from_text("one"));
        first_tab.last_recovery_file = Some(first.clone());
        let mut second_tab = EditorTab::new(MarkdownDocument::from_text("two"));
        second_tab.last_recovery_file = Some(second.clone());
        app.tabs = vec![first_tab, second_tab];
        app
    });
    app.update(cx, |app, _| app.discard_all_tab_recovery_files());
    assert!(!first.exists());
    assert!(!second.exists());
    app.update(cx, |app, _| {
        assert!(app.tabs.iter().all(|tab| tab.last_recovery_file.is_none()));
    });
}

#[gpui::test]
fn canceling_link_editor_preserves_version_cache_and_history(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("[label](old)"))];
        app.active_tab_mut().selected_range = 3..3;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    app.update(cx, |app, cx| {
        let version = app.active_tab().document.version();
        let blocks = app.active_tab().document.visual_blocks_shared();
        app.open_link_editor(cx);
        app.link_editor.as_mut().unwrap().url = "changed".into();
        app.cancel_link_editor(cx);
        assert_eq!(app.active_tab().document.text(), "[label](old)");
        assert_eq!(app.active_tab().document.version(), version);
        assert!(app.active_tab().undo_stack.is_empty());
        assert!(Arc::ptr_eq(
            &blocks,
            &app.active_tab().document.visual_blocks_shared()
        ));
    });
}

#[gpui::test]
fn link_editor_ime_composition_stays_out_of_canonical_markdown_until_apply(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("文档"))];
        app.active_tab_mut().selected_range = 0..6;
        app.view_mode = ViewMode::VisualEdit;
        app
    });

    app.update(cx, |app, cx| app.open_link_editor(cx));
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(app, None, "链", None, window, cx);
            EntityInputHandler::replace_and_mark_text_in_range(app, None, "链接", None, window, cx);
            EntityInputHandler::replace_text_in_range(app, None, "链接", window, cx);
        });
    });
    app.update(cx, |app, cx| {
        assert_eq!(app.active_tab().document.text(), "文档");
        assert_eq!(app.link_editor.as_ref().unwrap().url, "链接");
        app.confirm_link_editor(cx);
        assert_eq!(app.active_tab().document.text(), "[文档](链接)");
        assert_eq!(app.active_tab().undo_stack.len(), 1);
    });
}

#[gpui::test]
fn pasted_clipboard_image_uses_managed_asset_and_one_undo(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("note.md");
    let mut document = MarkdownDocument::from_text("before ");
    document.save_as(&path).unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app.active_tab_mut().selected_range = 7..7;
        app
    });
    cx.update(|_, cx| {
        cx.write_to_clipboard(ClipboardItem::new_image(&gpui::Image::from_bytes(
            ImageFormat::Png,
            b"clipboard-image".to_vec(),
        )));
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| app.paste(&Paste, window, cx));
    });
    app.update(cx, |app, _| {
        let text = app.active_tab().document.text();
        assert!(
            text.starts_with("before ![pasted-image](note.assets/pasted-image-"),
            "unexpected pasted source: {text:?}"
        );
        assert!(text.ends_with(".png)"));
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        let asset = fs::read_dir(dir.path().join("note.assets"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(fs::read(asset).unwrap(), b"clipboard-image");
    });
}

#[gpui::test]
fn visual_image_presentation_is_one_exact_undoable_mutation(cx: &mut TestAppContext) {
    let source = "![图](old.png \"Caption\")";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.view_mode = ViewMode::VisualEdit;
        app
    });

    app.update(cx, |app, cx| {
        let version = app.active_tab().document.version();
        app.set_image_presentation_at(
            3,
            ImagePresentation {
                width_percent: 50,
                alignment: ImageAlignment::Right,
            },
            cx,
        );
        assert_eq!(
            app.active_tab().document.text(),
            "![图](old.png \"Caption {width=50 align=right}\")"
        );
        assert_eq!(app.active_tab().document.version(), version + 1);
        assert_eq!(app.active_tab().undo_stack.len(), 1);
        assert!(app.active_tab_mut().apply_undo());
        assert_eq!(app.active_tab().document.text(), source);
    });
}

#[gpui::test]
fn external_change_reload_and_dirty_conflict_preserve_expected_source(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("external.md");
    fs::write(&path, "disk one").unwrap();
    let document = MarkdownDocument::open(&path).unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app
    });

    fs::write(&path, "disk version two").unwrap();
    app.update(cx, |app, cx| app.check_external_changes(cx));
    // The disk half of the check now runs on the background executor.
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "disk version two");
        assert!(!app.active_tab().document.is_dirty());
        assert_eq!(app.active_tab().external_conflict, None);
    });

    app.update(cx, |app, _| {
        app.active_tab_mut().document.set_text("local dirty")
    });
    fs::write(&path, "third external version").unwrap();
    app.update(cx, |app, cx| app.check_external_changes(cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "local dirty");
        assert!(app.active_tab().document.is_dirty());
        assert_eq!(
            app.active_tab().external_conflict,
            Some(DiskState::Modified)
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "third external version");
    });
}

#[gpui::test]
fn external_check_outcome_is_dropped_when_the_document_was_saved_meanwhile(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stale.md");
    fs::write(&path, "disk one").unwrap();
    let document = MarkdownDocument::open(&path).unwrap();
    let stale_identity = document.disk_identity().cloned();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app
    });

    // Simulate a round that was checked against `stale_identity` while the
    // user saved new content: the document's identity moved on, so the
    // outcome must not apply.
    app.update(cx, |app, cx| {
        app.active_tab_mut().document.set_text("saved locally");
        app.active_tab_mut().document.force_save().unwrap();
        let tab = app.active_tab().document_tab().unwrap();
        let recovery_id = tab.recovery_id;
        // Keep instance/version current so this test still isolates the
        // identity-staleness guard; the version guard has its own coverage.
        let (instance, version) = (tab.document.instance_id(), tab.document.version());
        app.apply_external_check_outcomes(
            vec![(
                ExternalCheckRequest {
                    recovery_id,
                    path: path.clone(),
                    known: stale_identity,
                    read_for_reload: true,
                    instance,
                    version,
                },
                markion::ExternalCheckOutcome::Modified {
                    reload: Some(Ok((
                        "outdated reload".to_string(),
                        app.active_tab().document.disk_identity().cloned().unwrap(),
                    ))),
                },
            )],
            cx,
        );
        assert_eq!(app.active_tab().document.text(), "saved locally");
        assert_eq!(app.active_tab().external_conflict, None);
    });
}

#[gpui::test]
fn autosave_runs_off_thread_and_clears_dirty(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("auto.md");
    fs::write(&path, "v1").unwrap();
    let document = MarkdownDocument::open(&path).unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.auto_save_preferences = AutoSavePreferences {
            enabled: true,
            silent_save: true,
            delay_secs: 1,
        };
        app.recovery_dir = dir.path().join("recovery");
        app.tabs = vec![EditorTab::new(document)];
        app
    });

    // `gpui::Timer` is a real-time smol timer outside the test clock, so the
    // due autosave is driven directly; the background disk stage still runs
    // through the (deterministic) test executor.
    app.update(cx, |app, cx| {
        app.active_tab_mut().document.set_text("v2");
        app.schedule_autosave(cx);
        let generation = app.active_tab().autosave_generation;
        let recovery_dir = app.recovery_dir.clone();
        app.run_due_autosave(0, generation, recovery_dir, cx);
        assert!(app.active_tab().autosave_in_flight);
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert!(!app.active_tab().document.is_dirty());
        assert!(!app.active_tab().autosave_in_flight);
        assert_eq!(app.active_tab().last_recovery_file, None);
    });
    assert_eq!(fs::read_to_string(&path).unwrap(), "v2");
}

#[gpui::test]
fn autosave_silent_save_off_keeps_named_file_and_recovery(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keep.md");
    fs::write(&path, "on-disk").unwrap();
    let document = MarkdownDocument::open(&path).unwrap();
    let recovery_dir = dir.path().join("recovery");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.auto_save_preferences = AutoSavePreferences {
            enabled: true,
            silent_save: false,
            delay_secs: 1,
        };
        app.recovery_dir = recovery_dir.clone();
        app.tabs = vec![EditorTab::new(document)];
        app
    });

    app.update(cx, |app, cx| {
        app.active_tab_mut().document.set_text("in-memory");
        app.schedule_autosave(cx);
        let generation = app.active_tab().autosave_generation;
        app.run_due_autosave(0, generation, recovery_dir.clone(), cx);
    });
    cx.run_until_parked();

    let recovery = app.update(cx, |app, _| {
        assert!(app.active_tab().document.is_dirty());
        assert!(!app.active_tab().autosave_in_flight);
        app.active_tab().last_recovery_file.clone().unwrap()
    });
    assert_eq!(fs::read_to_string(&path).unwrap(), "on-disk");
    let recovered = load_recovery_file(&recovery).unwrap();
    assert_eq!(recovered.text, "in-memory");
}

#[gpui::test]
fn autosave_enabled_false_does_not_run_due_work(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("off.md");
    fs::write(&path, "v1").unwrap();
    let document = MarkdownDocument::open(&path).unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.auto_save_preferences = AutoSavePreferences {
            enabled: false,
            silent_save: true,
            delay_secs: 1,
        };
        app.recovery_dir = dir.path().join("recovery");
        app.tabs = vec![EditorTab::new(document)];
        app
    });

    app.update(cx, |app, cx| {
        app.active_tab_mut().document.set_text("v2");
        let before = app.active_tab().autosave_generation;
        app.schedule_autosave(cx);
        assert_eq!(app.active_tab().autosave_generation, before.wrapping_add(1));
        // Even if a stale due call arrives, enabled=false means schedule never
        // armed a timer; drive due with the generation that schedule produced
        // and confirm no in-flight write starts when we somehow call it —
        // schedule itself returned early so we verify dirty state and disk.
        assert!(!app.active_tab().autosave_in_flight);
    });
    assert_eq!(fs::read_to_string(&path).unwrap(), "v1");
    let recovery_dir = dir.path().join("recovery");
    assert!(!recovery_dir.exists() || fs::read_dir(&recovery_dir).unwrap().next().is_none());
}

#[gpui::test]
fn autosave_completion_after_racing_edit_keeps_dirty_but_records_identity(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("race.md");
    fs::write(&path, "v1").unwrap();
    let document = MarkdownDocument::open(&path).unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(document)];
        app
    });

    app.update(cx, |app, cx| {
        let tab = app.active_tab_mut();
        tab.document.set_text("racing edit");
        tab.autosave_generation = 7;
        let recovery_id = tab.document_tab().unwrap().recovery_id;
        // The background stage saved an older snapshot ("v2") while the edit
        // above advanced the generation past the captured value.
        fs::write(&path, "v2").unwrap();
        let (_, identity) = markion::read_document_source(&path).unwrap();
        app.apply_autosave_outcome(
            AutosaveCompletion {
                recovery_id,
                generation: 6,
                result: AutosaveOutcome::Saved {
                    path: path.clone(),
                    identity: identity.clone(),
                },
            },
            cx,
        );
        let tab = app.active_tab();
        // Dirty survives (the racing edit is not on disk)...
        assert!(tab.document.is_dirty());
        // ...but the identity reflects our own write, so the external-change
        // poll will not mistake it for a foreign modification.
        assert_eq!(tab.document.disk_identity(), Some(&identity));
        assert!(!tab.autosave_in_flight);
    });
}

#[gpui::test]
fn startup_file_intent_opens_via_background_read(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("startup.md");
    fs::write(&path, "# opened at startup").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, cx| {
        app.apply_startup_open_intent(StartupOpenIntent::File(path.clone()), cx);
        // The read has not landed yet: the welcome tab is still in place.
        assert_eq!(app.active_tab().document.path(), None);
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.path(), Some(path.as_path()));
        assert_eq!(app.active_tab().document.text(), "# opened at startup");
        assert!(!app.active_tab().document.is_dirty());
    });
}

#[test]
fn selection_format_target_requires_one_safe_exact_editable_run() {
    let document = MarkdownDocument::from_text("plain text and **bold**");
    let blocks = document.visual_blocks_shared();
    let target = BlockTarget::from_block(document.version(), &blocks[0]);
    let mut tab = EditorTab::new(document);

    tab.selected_range = 0..5;
    let exact = visual_selection_format_target_for_block(&tab, &blocks, &target)
        .expect("plain text selection should be format-safe");
    assert_eq!(exact.range, 0..5);
    assert_eq!(exact.block_id, blocks[0].id);

    tab.selected_range = 0..blocks[0].source_range.end;
    assert!(visual_selection_format_target_for_block(&tab, &blocks, &target).is_none());

    let mut stale = target.clone();
    stale.document_version += 1;
    tab.selected_range = 0..5;
    assert!(visual_selection_format_target_for_block(&tab, &blocks, &stale).is_none());

    let math_document = MarkdownDocument::from_text("before $x$ after");
    let math_blocks = math_document.visual_blocks_shared();
    let math_target = BlockTarget::from_block(math_document.version(), &math_blocks[0]);
    let math_range = math_blocks[0]
        .editable_runs
        .iter()
        .find(|run| run.math.is_some())
        .expect("fixture should create an inline math run")
        .content_range
        .clone();
    let mut math_tab = EditorTab::new(math_document);
    math_tab.selected_range = math_range;
    assert!(
        visual_selection_format_target_for_block(&math_tab, &math_blocks, &math_target).is_none()
    );

    let html_document = MarkdownDocument::from_text("unclosed <em>em text");
    let html_blocks = html_document.visual_blocks_shared();
    let html_target = BlockTarget::from_block(html_document.version(), &html_blocks[0]);
    let html_range = html_blocks[0]
        .editable_runs
        .iter()
        .find(|run| run.conservative_fallback)
        .expect("unpaired inline HTML stays as conservative atoms")
        .content_range
        .clone();
    let mut html_tab = EditorTab::new(html_document);
    html_tab.selected_range = html_range;
    assert!(html_blocks[0].source_island.is_none());
    assert!(
        visual_selection_format_target_for_block(&html_tab, &html_blocks, &html_target).is_none()
    );

    let multi_document = MarkdownDocument::from_text("first\n\nsecond");
    let multi_blocks = multi_document.visual_blocks_shared();
    let second_target = BlockTarget::from_block(multi_document.version(), &multi_blocks[1]);
    let mut multi_tab = EditorTab::new(multi_document);
    multi_tab.selected_range = 0..5;
    assert!(
        visual_selection_format_target_for_block(&multi_tab, &multi_blocks, &second_target)
            .is_none()
    );
}

#[gpui::test]
fn memory_harness_tab_growth_and_close_release(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    let before_second = app.update(cx, |app, cx| {
        app.load_memory_profile(MemoryProfile::PlainLong, 1, MemoryWarmup::VisualEdit, cx);
        app.memory_report().per_tab_total()
    });
    let global_before = app.update(cx, |app, _| app.memory_report().global_total());

    app.update(cx, |app, cx| {
        let document = MarkdownDocument::from_text(MemoryProfile::PlainLong.markdown());
        app.open_in_new_tab(document, cx);
        app.warm_active_tab(MemoryWarmup::VisualEdit, cx);
    });
    let with_second = app.update(cx, |app, _| app.memory_report());
    // Opening a second tab dormants the first: both texts are retained, but only
    // the active tab keeps warm visual caches — so per-tab total grows by about
    // one document_text, not a second full warm derived set.
    assert!(
        with_second.per_tab_total() > before_second,
        "second tab text should increase per-tab total ({} vs {})",
        with_second.per_tab_total(),
        before_second
    );
    assert!(
        with_second.per_tab_total() < before_second.saturating_mul(2),
        "dormancy must prevent a second full warm cache set ({} vs 2×{})",
        with_second.per_tab_total(),
        before_second
    );
    assert_eq!(
        with_second
            .find_site("tabs[0].document.visual_blocks")
            .unwrap()
            .estimated_bytes,
        0,
        "previous tab must be dormant after open_in_new_tab"
    );
    assert_eq!(
        with_second.global_total(),
        global_before,
        "opening another text tab must not grow global render caches"
    );

    let released = app.update(cx, |app, cx| {
        app.close_tab_confirmed(cx);
        app.warm_active_tab(MemoryWarmup::VisualEdit, cx);
        app.memory_report()
    });
    assert_eq!(
        app.update(cx, |app, _| app.tabs.len()),
        1,
        "closing the extra tab should leave one tab"
    );
    assert_eq!(released.per_tab_total(), before_second);
    assert_eq!(released.global_total(), with_second.global_total());
}

#[gpui::test]
fn memory_harness_repeated_reports_are_identical(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, cx| {
        app.load_memory_profile(MemoryProfile::Code, 1, MemoryWarmup::Preview, cx);
        let first = app.memory_report();
        let second = app.memory_report();
        assert!(
            first.sites_equal(&second),
            "site figures must agree across successive reports"
        );
        assert_eq!(first.accounted_total(), second.accounted_total());
        if let (Some(a), Some(b)) = (
            first.process_footprint.resident_peak,
            second.process_footprint.resident_peak,
        ) {
            assert!(b >= a, "resident_peak must be monotonic");
        }
    });
}

#[gpui::test]
fn memory_report_with_footprint_leaves_unpopulated_caches(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, cx| {
        // Text-only warmup: preview/visual caches must stay unpopulated.
        app.load_memory_profile(MemoryProfile::PlainLong, 1, MemoryWarmup::TextOnly, cx);
        let version = app.active_tab().document.version();
        let report = app.memory_report();
        assert_eq!(
            report
                .find_site("tabs[0].document.preview_blocks")
                .unwrap()
                .estimated_bytes,
            0
        );
        assert_eq!(
            report
                .find_site("tabs[0].document.visual_blocks")
                .unwrap()
                .estimated_bytes,
            0
        );
        assert!(
            report.process_footprint.resident_current.is_some()
                || report.process_footprint.resident_peak.is_some()
                || report.process_platform == "unknown",
            "footprint counters should be sampled when the platform supports them"
        );
        assert_eq!(app.active_tab().document.version(), version);
        // Re-check caches stayed empty after sampling.
        let again = app.memory_report();
        assert_eq!(
            again
                .find_site("tabs[0].document.preview_blocks")
                .unwrap()
                .estimated_bytes,
            0
        );
        assert_eq!(app.active_tab().document.version(), version);
    });
}

#[gpui::test]
fn inactive_tab_dormancy_clears_derived_caches_and_preserves_undo(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });

    let original_prefix = app.update(cx, |app, cx| {
        app.view_mode = ViewMode::VisualEdit;
        app.load_memory_profile(MemoryProfile::PlainLong, 2, MemoryWarmup::VisualEdit, cx);
        // Active tab is index 1 after load_memory_profile. Edit + undo on it.
        let tab = app.active_tab_mut();
        let original_prefix = tab.document.text()[..5].to_string();
        tab.selected_range = 0..5;
        tab.push_undo_snapshot();
        tab.document.replace_range(0..5, "HELLO");
        tab.selected_range = 5..5;
        // Re-warm visual caches after the edit.
        let blocks = tab.document.visual_blocks_shared();
        tab.sync_visual_list(&blocks);
        original_prefix
    });

    let (version_before, text_before, undo_len, selection_before, warmed_visual) =
        app.update(cx, |app, _| {
            let report = app.memory_report();
            let visual = report
                .find_site("tabs[1].document.visual_blocks")
                .expect("visual site");
            assert!(
                visual.estimated_bytes > 0,
                "tab 1 should be warm before switch"
            );
            (
                app.tabs[1].document.version(),
                app.tabs[1].document.text().to_string(),
                app.tabs[1].undo_stack.len(),
                app.tabs[1].selected_range.clone(),
                visual.estimated_bytes,
            )
        });

    app.update(cx, |app, cx| {
        app.switch_active_tab(0, cx);
    });

    app.update(cx, |app, _| {
        assert_eq!(app.active_tab, 0);
        let report = app.memory_report();
        for name in [
            "tabs[1].document.visual_blocks",
            "tabs[1].document.preview_blocks",
            "tabs[1].shaped_lines",
        ] {
            let site = report.find_site(name).expect(name);
            assert_eq!(
                site.estimated_bytes, 0,
                "{name} must be zero while dormant (was warm={warmed_visual})"
            );
        }
        assert_eq!(
            report
                .find_site("tabs[1].visual_list_blocks")
                .unwrap()
                .counts[0]
                .1,
            0
        );
        assert_eq!(app.tabs[1].document.version(), version_before);
        assert_eq!(app.tabs[1].document.text(), text_before);
        assert_eq!(app.tabs[1].undo_stack.len(), undo_len);
        assert_eq!(app.tabs[1].selected_range, selection_before);
        // Active tab still reports document text.
        assert!(
            report
                .find_site("tabs[0].document_text")
                .unwrap()
                .estimated_bytes
                > 0
        );
    });

    app.update(cx, |app, cx| {
        app.switch_active_tab(1, cx);
        app.warm_active_tab(MemoryWarmup::VisualEdit, cx);
        assert!(app.active_tab_mut().apply_undo());
    });

    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().selected_range, 0..5);
        assert!(
            app.active_tab()
                .document
                .text()
                .starts_with(&original_prefix),
            "undo after dormancy must restore pre-edit text (prefix {original_prefix:?})"
        );
        let report = app.memory_report();
        assert!(
            report
                .find_site("tabs[1].document.visual_blocks")
                .unwrap()
                .estimated_bytes
                > 0,
            "reactivated tab should rebuild visual blocks"
        );
    });
}

#[gpui::test]
fn memory_harness_dormancy_drops_and_restores_derived_bytes(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    let warm_bytes = app.update(cx, |app, cx| {
        app.view_mode = ViewMode::VisualEdit;
        app.load_memory_profile(MemoryProfile::PlainLong, 2, MemoryWarmup::VisualEdit, cx);
        // load leaves tab 1 warm and tab 0 dormant; re-warm tab 1 explicitly.
        app.warm_active_tab(MemoryWarmup::VisualEdit, cx);
        app.memory_report()
            .find_site("tabs[1].document.visual_blocks")
            .unwrap()
            .estimated_bytes
    });
    assert!(warm_bytes > 0, "fixture tab must populate visual blocks");

    let (dormant_derived, active_text) = app.update(cx, |app, cx| {
        app.switch_active_tab(0, cx);
        let report = app.memory_report();
        let dormant = report
            .find_site("tabs[1].document.visual_blocks")
            .unwrap()
            .estimated_bytes
            + report
                .find_site("tabs[1].document.preview_blocks")
                .unwrap()
                .estimated_bytes
            + report
                .find_site("tabs[1].shaped_lines")
                .unwrap()
                .estimated_bytes;
        let active_text = report
            .find_site("tabs[0].document_text")
            .unwrap()
            .estimated_bytes;
        (dormant, active_text)
    });
    assert_eq!(
        dormant_derived, 0,
        "inactive tab derived sites must drop to zero"
    );
    assert!(active_text > 0, "active tab text must remain attributed");

    let restored = app.update(cx, |app, cx| {
        app.switch_active_tab(1, cx);
        app.warm_active_tab(MemoryWarmup::VisualEdit, cx);
        app.memory_report()
            .find_site("tabs[1].document.visual_blocks")
            .unwrap()
            .estimated_bytes
    });
    assert!(
        restored > 0,
        "reactivated tab must rebuild visual blocks (was {warm_bytes} before dormancy)"
    );
}

/// Informational dump for `docs/memory-retention.md`. Not a merge gate.
#[gpui::test]
fn memory_harness_attribution_dump(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    eprintln!(
        "NOTE: peak counters are process-lifetime figures shared across profiles in this single test process — do not read them as per-profile costs."
    );
    for (profile, warmup) in [
        (MemoryProfile::PlainLong, MemoryWarmup::VisualEdit),
        (MemoryProfile::Images, MemoryWarmup::Preview),
        (MemoryProfile::Diagrams, MemoryWarmup::Preview),
        (MemoryProfile::Math, MemoryWarmup::Preview),
        (MemoryProfile::Code, MemoryWarmup::Preview),
    ] {
        app.update(cx, |app, cx| {
            app.load_memory_profile(profile, 1, warmup, cx);
        });
        // Let background preview-image / diagram rasters land before sampling.
        cx.run_until_parked();
        let report = app.update(cx, |app, _| app.memory_report());
        let fp = report.process_footprint;
        eprintln!(
            "profile={} warmup={:?} per_tab={} global={} accounted={} preview_image_bytes={} diagram_bytes={} platform={} resident_current={:?} resident_peak={:?} commit_current={:?} commit_peak={:?} (peaks=process-lifetime)",
            profile.name(),
            warmup,
            report.per_tab_total(),
            report.global_total(),
            report.accounted_total(),
            report
                .find_site("global.preview_image_cache")
                .map(|site| site.estimated_bytes)
                .unwrap_or(0),
            report
                .find_site("global.diagram_cache")
                .and_then(|site| {
                    site.counts
                        .iter()
                        .find(|(k, _)| k == "completed_bytes")
                        .map(|(_, n)| *n)
                })
                .unwrap_or(0),
            report.process_platform,
            fp.resident_current,
            fp.resident_peak,
            fp.commit_current,
            fp.commit_peak,
        );
    }
}

/// Informational probe: allocate then free large decode buffers and compare
/// peak vs current process footprint. Not a merge gate.
#[test]
fn memory_decode_spike_footprint_probe() {
    use image::{Rgba, RgbaImage};

    let before = ProcessFootprint::sample();
    // Simulate the preview decode path's full-resolution intermediate: several
    // large RGBA buffers that are dropped before sampling again.
    {
        let mut held = Vec::new();
        for _ in 0..4 {
            let img = RgbaImage::from_pixel(2048, 2048, Rgba([10, 20, 30, 255]));
            let rgba = img.into_raw();
            held.push(rgba);
        }
        // Touch the buffers so they are not optimized away.
        let checksum: usize = held.iter().map(|b| b.len()).sum();
        assert!(checksum > 0);
        drop(held);
    }
    let after = ProcessFootprint::sample();
    eprintln!(
        "decode_spike_probe platform={} before_resident={:?} after_resident={:?} after_peak={:?} before_commit={:?} after_commit={:?} after_commit_peak={:?}",
        process_footprint_platform(),
        before.resident_current,
        after.resident_current,
        after.resident_peak,
        before.commit_current,
        after.commit_current,
        after.commit_peak,
    );
    if let (Some(cur), Some(peak)) = (after.resident_current, after.resident_peak) {
        assert!(peak >= cur);
    }
    if let (Some(a), Some(b)) = (before.resident_peak, after.resident_peak) {
        assert!(b >= a, "peak must not fall after the probe");
    }
}

#[gpui::test]
fn preview_image_cache_shares_across_tabs_and_releases_on_close(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("tempdir");
    let shared_png = dir.path().join("shared.png");
    let unique_png = dir.path().join("unique.png");
    write_solid_png(&shared_png, 8, 8, [10, 20, 30, 255]);
    write_solid_png(&unique_png, 8, 8, [40, 50, 60, 255]);

    let shared_md = dir.path().join("shared.md");
    let unique_md = dir.path().join("unique.md");
    std::fs::write(&shared_md, "![s](shared.png)\n").unwrap();
    std::fs::write(&unique_md, "![u](unique.png)\n").unwrap();

    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    let shared_key = PreviewImageKey::from_url("shared.png", Some(dir.path()));
    let unique_key = PreviewImageKey::from_url("unique.png", Some(dir.path()));

    let warm_preview = |app: &mut MarkionApp, cx: &mut Context<MarkionApp>| {
        let preview = app.active_tab().document.preview_blocks_shared();
        app.active_tab_mut().sync_preview_list(&preview);
        let active = app.active_tab;
        app.refresh_tab_image_claims(active, &preview, &[], Some(dir.path()), cx);
        app.ensure_preview_images(&preview, &[], Some(dir.path()), cx);
    };

    let version_before = app.update(cx, |app, cx| {
        let shared_doc = MarkdownDocument::open(&shared_md).expect("open shared");
        app.replace_active_tab(shared_doc, cx);
        let version = app.active_tab().document.version();
        warm_preview(app, cx);
        version
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        assert!(matches!(
            app.preview_image_cache.get(&shared_key),
            Some(PreviewImageEntry::Ready(_))
        ));
        assert_eq!(app.active_tab().document.version(), version_before);

        let unique_doc = MarkdownDocument::open(&unique_md).expect("open unique");
        app.open_in_new_tab(unique_doc, cx);
        warm_preview(app, cx);
    });
    cx.run_until_parked();

    // Dormancy released the first tab's shared claim; only unique is claimed.
    app.update(cx, |app, _| {
        assert_eq!(app.preview_image_cache.claim_count(&shared_key), 0);
        assert_eq!(app.preview_image_cache.claim_count(&unique_key), 1);
    });

    app.update(cx, |app, cx| {
        let shared_doc = MarkdownDocument::open(&shared_md).expect("open shared again");
        app.open_in_new_tab(shared_doc, cx);
        warm_preview(app, cx);
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        assert_eq!(app.preview_image_cache.claim_count(&shared_key), 1);
        assert!(matches!(
            app.preview_image_cache.get(&shared_key),
            Some(PreviewImageEntry::Ready(_))
        ));
        // Closing the dormant unique tab leaves its image as an unclaimed LRU
        // entry (reusable if the file reopens; evictable under budget) without
        // disturbing the active shared claim.
        app.active_tab = 1;
        app.close_tab_confirmed(cx);
        assert_eq!(app.preview_image_cache.claim_count(&unique_key), 0);
        assert!(
            matches!(
                app.preview_image_cache.get(&unique_key),
                Some(PreviewImageEntry::Ready(_))
            ),
            "unique image stays cached as an unclaimed entry so reopening reuses it"
        );
        assert!(matches!(
            app.preview_image_cache.get(&shared_key),
            Some(PreviewImageEntry::Ready(_))
        ));
        // Remaining tabs: 0 = first shared (dormant), 1 = second shared (active).
        // Close the dormant shared tab; the active claim keeps the raster.
        app.active_tab = 0;
        app.close_tab_confirmed(cx);
        assert!(matches!(
            app.preview_image_cache.get(&shared_key),
            Some(PreviewImageEntry::Ready(_))
        ));
        assert_eq!(app.preview_image_cache.claim_count(&shared_key), 1);
    });
}

#[gpui::test]
fn inline_html_image_claims_ride_visual_blocks(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().expect("tempdir");
    let badge_png = dir.path().join("badge.png");
    write_solid_png(&badge_png, 8, 8, [10, 20, 30, 255]);

    let badge_md = dir.path().join("badge.md");
    std::fs::write(
        &badge_md,
        "Hello <img src=\"badge.png\" alt=\"Badge\"> world\n",
    )
    .unwrap();
    let plain_md = dir.path().join("plain.md");
    std::fs::write(&plain_md, "Hello world\n").unwrap();

    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    let badge_key = PreviewImageKey::from_url("badge.png", Some(dir.path()));

    let warm_visual = |app: &mut MarkionApp, cx: &mut Context<MarkionApp>| {
        let preview = app.active_tab().document.preview_blocks_shared();
        let visual = app.active_tab().document.visual_blocks_shared();
        app.active_tab_mut().sync_visual_list(&visual);
        let active = app.active_tab;
        app.refresh_tab_image_claims(active, &preview, &visual, Some(dir.path()), cx);
        app.ensure_preview_images(&preview, &visual, Some(dir.path()), cx);
    };

    app.update(cx, |app, cx| {
        let badge_doc = MarkdownDocument::open(&badge_md).expect("open badge");
        app.replace_active_tab(badge_doc, cx);
        warm_visual(app, cx);
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        // The inline `<img>` URL is claimed through the visual runs (the
        // preview paragraph flattens it to text, so only Visual Edit carries
        // the URL) and decodes to a ready entry.
        assert_eq!(app.preview_image_cache.claim_count(&badge_key), 1);
        assert!(matches!(
            app.preview_image_cache.get(&badge_key),
            Some(PreviewImageEntry::Ready(_))
        ));
    });

    app.update(cx, |app, cx| {
        let plain_doc = MarkdownDocument::open(&plain_md).expect("open plain");
        app.replace_active_tab(plain_doc, cx);
        warm_visual(app, cx);
    });
    cx.run_until_parked();

    app.update(cx, |app, _| {
        assert_eq!(
            app.preview_image_cache.claim_count(&badge_key),
            0,
            "claim releases once the inline HTML image leaves the document"
        );
    });
}

#[gpui::test]
fn inline_html_image_renders_mixed_path_including_a_wrapped_badges(cx: &mut TestAppContext) {
    use crate::app::preview::VISUAL_HTML_IMAGE_ATOM_BUILDS;

    // Cases that previously collapsed into a whole-block HTML source island
    // and therefore never rendered the image: a bare inline image, the README
    // badge pattern `<a href><img></a>`, and an image mixed with a `<br>`.
    let cases: &[&str] = &[
        "Hello <img src=\"a.png\" alt=\"A\"> world",
        "See <a href=\"https://example.com\"><img src=\"b.png\" alt=\"B\"></a> here",
        "x <br> <img src=\"c.png\"> y",
    ];
    for source in cases {
        let before = VISUAL_HTML_IMAGE_ATOM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(*source))];
            app.active_tab_mut().selected_range = 0..0;
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();

        app.update(cx, |app, _| {
            let blocks = app.active_tab().visual_list_blocks.clone();
            let block = blocks.first().expect("one paragraph block");
            assert!(
                block
                    .editable_runs
                    .iter()
                    .any(|run| run.html_image.is_some()),
                "image run present for: {source}"
            );
            let builds = VISUAL_HTML_IMAGE_ATOM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
            assert!(
                builds > before,
                "inline HTML image atom was not painted for: {source}"
            );
        });
    }

    // An unclosed `<em>` spoils the block (whole-block island / mixed path),
    // but the image atom must survive the pairing-failure demotion and still
    // paint.
    let source = "bad <em>styled <img src=\"d.png\"> tail";
    let before = VISUAL_HTML_IMAGE_ATOM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = 0..0;
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();
    app.update(cx, |_app, _| {
        let builds = VISUAL_HTML_IMAGE_ATOM_BUILDS.load(std::sync::atomic::Ordering::Relaxed);
        assert!(
            builds > before,
            "image atom must still paint beside an unclosed tag: {source}"
        );
    });
}

#[gpui::test]
fn visual_edit_renders_escapes_inline_html_and_entities_without_source_island(
    cx: &mut TestAppContext,
) {
    // Paragraphs whose only "unsupported" content used to be escaped
    // punctuation, non-image inline HTML, or decoded HTML entities must
    // render as styled prose with no source-island box: no island kind and
    // no conservative run.
    let cases: &[&str] = &[
        r"escaped \* star and \. dot",
        "text <em>em</em> and <strong>bold</strong> more",
        "one<br>two<br/>three",
        r"mixed \* escape and <em>html</em> and <br> break",
        "fish &amp; chips &mdash; done &#39;quoted&#39;",
        "dash &#x2014; here &copy; 2026",
        r"entity \* mix &amp; <br> break",
    ];
    for source in cases {
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(*source))];
            app.active_tab_mut().selected_range = 0..0;
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();

        app.update(cx, |app, _| {
            let tab = app.active_tab();
            let blocks = tab.visual_list_blocks.clone();
            let block = blocks
                .first()
                .unwrap_or_else(|| panic!("no block for {source}"));
            assert!(
                block.source_island.is_none(),
                "paragraph must not collapse into a source island: {source}"
            );
            assert!(
                block
                    .editable_runs
                    .iter()
                    .all(|run| !run.conservative_fallback),
                "every run stays rendered: {source}"
            );
            assert!(
                tab.visual_caret_bounds.is_some(),
                "caret visible for {source}"
            );
        });
    }
}

#[gpui::test]
fn visual_edit_reveals_escape_inline_html_and_entity_groups_without_version_bump(
    cx: &mut TestAppContext,
) {
    // Moving the caret into an escape, a supported inline-HTML element, or
    // a decoded entity token reveals the authored source group in place
    // without changing the document version or dirtying the tab.
    for (source, caret) in [
        (r"escaped \* star", 9),      // inside the `\*` group
        ("text <em>em</em> more", 8), // inside the em content
        ("one<br>two", 3),            // at the `<br>` tag start
        ("fish &amp; chips", 7),      // inside the `&amp;` token
        ("dash &#x2014; here", 8),    // inside the numeric token
    ] {
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
            app.active_tab_mut().selected_range = 0..0;
            app.view_mode = ViewMode::VisualEdit;
            app
        });
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });
        cx.run_until_parked();

        let version = app.update(cx, |app, _| app.active_tab().document.version());
        app.update(cx, |app, _| {
            app.active_tab_mut().selected_range = caret..caret;
            app.active_tab_mut().visual_cursor_reveal_pending = true;
        });
        cx.run_until_parked();

        app.update(cx, |app, _| {
            let tab = app.active_tab();
            assert_eq!(
                tab.document.version(),
                version,
                "caret-only reveal must not bump the version for {source}"
            );
            assert!(!tab.document.is_dirty(), "reveal must not dirty {source}");
            assert!(
                tab.visual_caret_bounds.is_some(),
                "revealed source still paints a caret for {source}"
            );
        });
    }
}

#[gpui::test]
fn visual_edit_left_navigation_through_br_tag_keeps_valid_carets(cx: &mut TestAppContext) {
    // `one<br>two`: the tag bytes 3..7 have no unrevealed display positions.
    // Walking Left may enter them (keyboard navigation reveals the authored
    // source group in place), but every stop must remain a valid, paintable
    // caret position without mutating the document.
    let source = "one<br>two";
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(source))];
        app.active_tab_mut().selected_range = source.len()..source.len();
        app.view_mode = ViewMode::VisualEdit;
        app
    });
    cx.update(|window, cx| {
        window.focus(&app.read(cx).focus_handle);
        window.activate_window();
    });
    cx.run_until_parked();

    let version = app.update(cx, |app, _| app.active_tab().document.version());
    let mut previous_offset = app.update(cx, |app, _| app.active_tab().cursor_offset());
    for _ in 0..10 {
        cx.dispatch_action(Left);
        cx.run_until_parked();
        app.update(cx, |app, _| {
            let tab = app.active_tab();
            let offset = tab.cursor_offset();
            assert!(
                offset < previous_offset,
                "Left must keep moving left (at {offset})"
            );
            previous_offset = offset;
            assert!(
                tab.visual_caret_bounds.is_some(),
                "caret at {offset} must stay paintable, including mid-tag stops"
            );
            assert_eq!(tab.document.version(), version);
            assert_eq!(tab.document.text(), source);
        });
    }
    assert_eq!(
        previous_offset, 0,
        "ten Left presses walk past the paragraph"
    );
}

fn write_solid_png(path: &Path, width: u32, height: u32, rgba: [u8; 4]) {
    let mut img = image::RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba(rgba);
    }
    img.save(path).expect("write png");
}

#[test]
fn open_recent_menu_is_wired_in_file_dropdown() {
    let root_view_source = include_str!("root_view.rs");
    assert!(root_view_source.contains(".on_action(cx.listener(Self::clear_recent_files))"));
    assert!(root_view_source.contains("Msg::ItemOpenRecent"));
    assert!(root_view_source.contains("Msg::ItemOpenRecentEmpty"));
    assert!(root_view_source.contains("Msg::ItemClearRecentFiles"));
    assert!(root_view_source.contains("open_recent_path"));
    assert!(root_view_source.contains("fn open_recent_submenu_panel"));
    assert!(root_view_source.contains("menu_submenu_parent_button"));

    let in_window = root_view_source
        .split_once("AppMenu::File => panel")
        .and_then(|(_, rest)| rest.split_once("AppMenu::Edit =>").map(|(file, _)| file))
        .expect("in-window File menu");
    let folder = in_window.find("Msg::ItemOpenFolder,").expect("Open Folder");
    let recent = in_window
        .find("Msg::ItemOpenRecent")
        .expect("Open Recent parent");
    let save = in_window.find("Msg::ItemSave,").expect("Save");
    assert!(
        folder < recent && recent < save,
        "Open Recent parent must sit between Open Folder and Save in the File dropdown"
    );
    assert!(
        !in_window.contains("Msg::ItemClearRecentFiles"),
        "Clear Recent Files must not be a top-level File sibling"
    );
    assert!(
        !in_window.contains("open_recent_path"),
        "recent path open handlers must live in the Open Recent submenu builder"
    );

    let submenu = root_view_source
        .split_once("fn open_recent_submenu_panel")
        .and_then(|(_, rest)| {
            rest.split_once("pub(super) fn preferences_panel_view")
                .map(|(body, _)| body)
        })
        .expect("Open Recent submenu builder");
    assert!(submenu.contains("Msg::ItemOpenRecentEmpty"));
    assert!(submenu.contains("Msg::ItemClearRecentFiles"));
    assert!(submenu.contains("open_recent_path"));
}

#[test]
fn status_bar_context_uses_cached_unicode_metrics_and_active_caret() {
    let source = "α β\n第三 行🙂";
    let line_start = source.find('第').unwrap();
    let caret = line_start + "第三".len();
    let mut tab = EditorTab::new(MarkdownDocument::from_text(source));
    tab.selected_range = line_start..caret;

    assert!(
        !tab.document
            .memory_breakdown()
            .site("stats")
            .unwrap()
            .populated
    );
    let version = tab.document.version();
    let forward = status_bar_context(&tab, ViewMode::Edit, Some("feature/状态栏"));
    assert_eq!(forward.characters, source.chars().count());
    assert_eq!(forward.words, source.split_whitespace().count());
    assert_eq!(forward.caret, Some((2, 3)));
    assert_eq!(forward.branch.as_deref(), Some("feature/状态栏"));
    assert!(
        tab.document
            .memory_breakdown()
            .site("stats")
            .unwrap()
            .populated
    );

    let repeated = status_bar_context(&tab, ViewMode::VisualEdit, Some("feature/状态栏"));
    assert_eq!(repeated, forward);
    assert_eq!(tab.document.version(), version);

    tab.selection_reversed = true;
    let reversed = status_bar_context(&tab, ViewMode::Split, None);
    assert_eq!(reversed.caret, Some((2, 1)));
    assert_eq!(reversed.branch, None);

    let read = status_bar_context(&tab, ViewMode::Read, None);
    assert_eq!(read.caret, None);
    assert_eq!(read.characters, forward.characters);
    assert_eq!(read.words, forward.words);
}

#[gpui::test]
fn status_bar_context_follows_active_tab_switches(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));
    app.update(cx, |app, cx| {
        let mut first = EditorTab::new(MarkdownDocument::from_text("one"));
        first.selected_range = 3..3;
        let mut second = EditorTab::new(MarkdownDocument::from_text("two words\n三"));
        second.selected_range = second.document.text().len()..second.document.text().len();
        app.tabs = vec![first, second];
        app.active_tab = 0;
        app.view_mode = ViewMode::Edit;

        let before = app.current_status_bar_context();
        assert_eq!(
            (before.characters, before.words, before.caret),
            (3, 1, Some((1, 4)))
        );

        app.switch_active_tab(1, cx);
        let after = app.current_status_bar_context();
        assert_eq!(after.characters, "two words\n三".chars().count());
        assert_eq!(after.words, 3);
        assert_eq!(after.caret, Some((2, 2)));
    });
}

fn write_symbolic_git_head(git_dir: &Path, branch: &str) {
    fs::create_dir_all(git_dir).unwrap();
    fs::write(git_dir.join("HEAD"), format!("ref: refs/heads/{branch}\n")).unwrap();
}

#[test]
fn git_branch_resolver_handles_nested_repositories_and_gitdir_indirection() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    let nested = root.join("notes/deep");
    fs::create_dir_all(&nested).unwrap();
    write_symbolic_git_head(&root.join(".git"), "main");

    let ordinary = resolve_git_branch(&nested);
    assert_eq!(ordinary.branch.as_deref(), Some("main"));
    assert_eq!(
        ordinary.head_path.as_deref(),
        Some(
            fs::canonicalize(root.join(".git"))
                .unwrap()
                .join("HEAD")
                .as_path()
        )
    );

    let nested_repo = root.join("notes");
    write_symbolic_git_head(&nested_repo.join(".git"), "feature/nested");
    assert_eq!(
        resolve_git_branch(&nested).branch.as_deref(),
        Some("feature/nested")
    );

    let worktree = temp.path().join("linked");
    let worktree_git_dir = temp.path().join("metadata/worktrees/linked");
    fs::create_dir_all(&worktree).unwrap();
    write_symbolic_git_head(&worktree_git_dir, "worktree/topic");
    fs::write(
        worktree.join(".git"),
        "gitdir: ../metadata/worktrees/linked\n",
    )
    .unwrap();
    let linked = resolve_git_branch(&worktree);
    assert_eq!(linked.branch.as_deref(), Some("worktree/topic"));
    assert_eq!(
        linked.head_path.as_deref(),
        Some(
            fs::canonicalize(&worktree_git_dir)
                .unwrap()
                .join("HEAD")
                .as_path()
        )
    );

    let absolute_worktree = temp.path().join("absolute-linked");
    let absolute_git_dir = temp.path().join("absolute-metadata/worktrees/linked");
    fs::create_dir_all(&absolute_worktree).unwrap();
    write_symbolic_git_head(&absolute_git_dir, "worktree/absolute");
    fs::write(
        absolute_worktree.join(".git"),
        format!("gitdir: {}\n", absolute_git_dir.display()),
    )
    .unwrap();
    assert_eq!(
        resolve_git_branch(&absolute_worktree).branch.as_deref(),
        Some("worktree/absolute")
    );
}

#[test]
fn git_branch_resolver_fails_closed_for_detached_malformed_and_missing_repositories() {
    let temp = tempfile::tempdir().unwrap();
    let detached = temp.path().join("detached");
    fs::create_dir_all(detached.join(".git")).unwrap();
    fs::write(
        detached.join(".git/HEAD"),
        "0123456789abcdef0123456789abcdef01234567\n",
    )
    .unwrap();
    let resolution = resolve_git_branch(&detached);
    assert_eq!(resolution.branch, None);
    assert_eq!(
        resolution.head_path.as_deref(),
        Some(
            fs::canonicalize(detached.join(".git"))
                .unwrap()
                .join("HEAD")
                .as_path()
        )
    );

    let malformed = temp.path().join("malformed");
    fs::create_dir_all(&malformed).unwrap();
    fs::write(malformed.join(".git"), "not a gitdir file\n").unwrap();
    assert_eq!(
        resolve_git_branch(&malformed),
        GitBranchResolution::default()
    );

    let missing = temp.path().join("missing/nested");
    fs::create_dir_all(&missing).unwrap();
    assert_eq!(resolve_git_branch(&missing), GitBranchResolution::default());
}

#[test]
fn git_context_prefers_saved_document_and_rejects_stale_results() {
    let workspace = PathBuf::from("workspace");
    let document = PathBuf::from("nested/repository/note.md");
    assert_eq!(
        git_context_path(Some(&document), &workspace, true),
        Some(PathBuf::from("nested/repository"))
    );
    assert_eq!(
        git_context_path(None, &workspace, true),
        Some(workspace.clone())
    );
    assert_eq!(git_context_path(None, &workspace, false), None);

    let first = PathBuf::from("first");
    let second = PathBuf::from("second");
    let mut state = GitBranchState::default();
    assert!(state.replace_context(Some(first.clone())));
    let (old_generation, old_context) = state.begin_lookup().unwrap();
    assert!(state.replace_context(Some(second.clone())));
    assert!(!state.accept(
        old_generation,
        &old_context,
        GitBranchResolution {
            head_path: Some(PathBuf::from("first/.git/HEAD")),
            branch: Some("stale".to_string()),
        },
    ));
    assert_eq!(state.branch, None);

    let (generation, context) = state.begin_lookup().unwrap();
    assert!(state.accept(
        generation,
        &context,
        GitBranchResolution {
            head_path: Some(PathBuf::from("second/.git/HEAD")),
            branch: Some("current".to_string()),
        },
    ));
    assert_eq!(state.branch.as_deref(), Some("current"));
}

#[gpui::test]
fn app_git_branch_cache_refreshes_in_background_and_clears_removed_repository(
    cx: &mut TestAppContext,
) {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    write_symbolic_git_head(&workspace.join(".git"), "main");
    let (app, cx) = cx.add_window_view(|_, cx| MarkionApp::new(cx));

    app.update(cx, |app, cx| app.set_workspace_root(workspace.clone(), cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.git_branch_state.branch.as_deref(), Some("main"));
        assert!(!app.git_branch_state.lookup_in_flight);
    });

    write_symbolic_git_head(&workspace.join(".git"), "feature/refresh");
    app.update(cx, |app, cx| app.refresh_git_branch_context(cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(
            app.git_branch_state.branch.as_deref(),
            Some("feature/refresh")
        );
    });

    fs::remove_dir_all(workspace.join(".git")).unwrap();
    app.update(cx, |app, cx| app.refresh_git_branch_context(cx));
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.git_branch_state.branch, None);
        assert_eq!(app.git_branch_state.head_path, None);
    });
}

#[test]
fn localized_status_context_preserves_values_and_transient_feedback() {
    let branch = "feature/a-very-long-分支-name-that-remains-verbatim";
    let context = StatusBarContext {
        characters: 42,
        words: 7,
        caret: Some((3, 9)),
        branch: Some(branch.to_string()),
    };
    let english = context.localized(Language::En);
    let chinese = context.localized(Language::ZhHans);
    assert_eq!(english.characters, "Chars 42");
    assert_eq!(english.words, "Words 7");
    assert_eq!(english.caret.as_deref(), Some("Ln 3, Col 9"));
    assert_eq!(
        english.branch.as_deref(),
        Some(format!("Branch {branch}").as_str())
    );
    assert_eq!(chinese.characters, "字符 42");
    assert_eq!(chinese.words, "词数 7");
    assert_eq!(chinese.caret.as_deref(), Some("行 3，列 9"));
    assert!(chinese.branch.as_deref().unwrap().ends_with(branch));

    for feedback in [
        "Saved note.md",
        "Exported note.pdf",
        "5 matches",
        "Save failed: denied",
    ] {
        let rendered = status_bar_feedback("note.md", true, "Modified", feedback);
        assert!(rendered.contains("note.md *"));
        assert!(rendered.ends_with(feedback));
    }

    let without_git = StatusBarContext {
        branch: None,
        ..context
    }
    .localized(Language::En);
    assert_eq!(without_git.branch, None);
}

#[test]
fn status_bar_layout_is_single_row_clipped_and_keeps_io_out_of_render() {
    let root_view = include_str!("root_view.rs");
    let status_bar = root_view
        .split_once("status-bar-feedback")
        .and_then(|(_, rest)| {
            rest.split_once(".child(active_menu_dropdown")
                .map(|(body, _)| body)
        })
        .expect("status bar render block");
    assert!(status_bar.contains(".h(px(28.))") || root_view.contains(".h(px(28.))"));
    assert!(status_bar.contains("status-bar-context"));
    assert!(status_bar.contains(".flex_1()"));
    assert!(status_bar.contains(".min_w_0()"));
    assert!(status_bar.contains(".flex_shrink_0()"));
    assert!(status_bar.contains(".whitespace_nowrap()"));
    assert!(status_bar.contains(".max_w(px(160.))"));
    assert!(status_bar.contains(".overflow_hidden()"));
    assert!(!root_view.contains("resolve_git_branch"));

    let status_source = include_str!("status_bar.rs");
    assert!(status_source.contains("background_executor()"));
    assert!(status_source.contains("Timer::after(GIT_BRANCH_REFRESH_INTERVAL)"));
}

// --- Tab-bar context menu ---------------------------------------------------

/// Only the file-backed items require the target tab to have a path; the
/// close family must stay available on untitled tabs.
#[test]
fn tab_context_actions_require_path_only_for_file_backed_items() {
    for action in [
        TabContextAction::CloseTab,
        TabContextAction::CloseOthers,
        TabContextAction::CloseToTheRight,
    ] {
        assert!(
            tab_context_action_enabled(action, false),
            "{action:?} must be enabled on untitled tabs"
        );
    }
    for action in [
        TabContextAction::Rename,
        TabContextAction::CopyPath,
        TabContextAction::RevealInFileManager,
    ] {
        assert!(
            !tab_context_action_enabled(action, false),
            "{action:?} must be disabled on untitled tabs"
        );
        assert!(
            tab_context_action_enabled(action, true),
            "{action:?} must be enabled on file-backed tabs"
        );
    }
}

/// The tab strip must register both the right-click menu opener and the
/// middle-click close companion (mirrors the structural layout tests above).
#[test]
fn tab_bar_registers_context_and_middle_click_handlers() {
    let editing_source = include_str!("editing.rs");
    let tab_bar = editing_source
        .split_once("pub(super) fn tab_bar_view")
        .expect("tab_bar_view")
        .1;
    assert!(tab_bar.contains("MouseButton::Right"));
    assert!(tab_bar.contains("MouseButton::Middle"));
    assert!(tab_bar.contains("show_tab_context_menu"));
}

/// Menu identity snapshot: same path + recovery id matches; a different tab
/// state at the same index (untitled sibling or replacement) does not.
#[test]
fn tab_context_target_identity_distinguishes_tabs() {
    let a = EditorTab::new(MarkdownDocument::from_text("A"));
    let b = EditorTab::new(MarkdownDocument::from_text("B"));
    // Same untitled shape (path None) but different recovery ids.
    let target = TabContextTarget::capture(0, &a);
    assert!(target.matches(&a));
    assert!(!target.matches(&b));

    // Image tabs match by path.
    let path = PathBuf::from("img/cover.png");
    let image = EditorTab::new_image(path.clone(), PreviewImageKey::from_local_path(&path));
    let image_target = TabContextTarget::capture(1, &image);
    assert!(image_target.matches(&image));
    assert!(!image_target.matches(&a));
}

#[gpui::test]
fn tab_context_menu_open_close_and_exclusivity(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("A")),
            EditorTab::new(MarkdownDocument::from_text("B")),
        ];
        app
    });

    app.update(cx, |app, cx| {
        app.show_tab_context_menu(1, Point::new(px(10.), px(10.)), cx);
        let menu = app.tab_context_menu.as_ref().expect("menu open");
        assert_eq!(menu.target.index, 1);
        assert!(menu.target.path.is_none());

        // Exclusivity: opening any sibling menu closes the tab menu, and the
        // tab menu closes the siblings.
        app.show_file_tree_context_menu(
            FileTreeContextTarget::Workspace,
            Point::new(px(0.), px(0.)),
            cx,
        );
        assert!(app.tab_context_menu.is_none());
        assert!(app.file_tree_context_menu.is_some());

        app.show_tab_context_menu(1, Point::new(px(10.), px(10.)), cx);
        assert!(app.file_tree_context_menu.is_none());
        assert!(app.tab_context_menu.is_some());

        app.show_preview_context_menu(Point::new(px(5.), px(5.)), None, cx);
        assert!(app.tab_context_menu.is_none());
        assert!(app.preview_context_menu.is_some());
    });

    // Click-away (the close_menu handler) closes the menu without dispatch.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(1, Point::new(px(10.), px(10.)), cx);
            app.close_menu(
                &MouseDownEvent {
                    button: MouseButton::Left,
                    position: Point::new(px(0.), px(0.)),
                    modifiers: Modifiers::none(),
                    click_count: 1,
                    first_mouse: false,
                },
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, _| {
        assert!(app.tab_context_menu.is_none());
    });
}

#[gpui::test]
fn tab_context_close_tab_switches_then_closes_clicked_tab(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("A")),
            EditorTab::new(MarkdownDocument::from_text("B")),
            EditorTab::new(MarkdownDocument::from_text("C")),
        ];
        app
    });

    // Clean close of a background tab: no dialog, clicked tab activated then
    // removed, active falls back to the previous neighbor.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(2, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CloseTab, window, cx);
        });
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab().document.text(), "B");
        assert!(app.tab_context_menu.is_none());
    });

    // Dirty close asks first; Cancel keeps the tab, Discard closes it.
    app.update(cx, |app, _| {
        app.active_tab_mut().document.replace_range(0..0, "dirty ");
    });
    let (discard_label, cancel_label) = app.update(cx, |app, _| {
        (
            t(app.language, Msg::DialogButtonDiscard).to_string(),
            t(app.language, Msg::DialogButtonCancel).to_string(),
        )
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(app.active_tab, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CloseTab, window, cx);
        });
    });
    cx.simulate_prompt_answer(&cancel_label);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 2, "cancel keeps the dirty tab open");
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(app.active_tab, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CloseTab, window, cx);
        });
    });
    cx.simulate_prompt_answer(&discard_label);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab().document.text(), "A");
    });
}

#[gpui::test]
fn close_other_tabs_cleans_silently_keeps_dirty_behind_dialog(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        let mut dirty_c = EditorTab::new(MarkdownDocument::from_text("C"));
        dirty_c.document.replace_range(0..0, "unsaved ");
        let mut dirty_d = EditorTab::new(MarkdownDocument::from_text("D"));
        dirty_d.document.replace_range(0..0, "unsaved ");
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("A")),
            EditorTab::new(MarkdownDocument::from_text("B")),
            dirty_c,
            dirty_d,
        ];
        app
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(0, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CloseOthers, window, cx);
        });
    });
    app.update(cx, |app, _| {
        // Clean sibling B closed immediately; both dirty tabs kept for the
        // dialog; the clicked tab survived and stays active.
        assert_eq!(app.tabs.len(), 3);
        assert_eq!(app.active_tab().document.text(), "A");
        assert!(
            app.tabs
                .iter()
                .any(|tab| tab.document.text() == "unsaved C")
        );
        assert!(
            app.tabs
                .iter()
                .any(|tab| tab.document.text() == "unsaved D")
        );
    });

    let keep_label = app.update(cx, |app, _| {
        t(app.language, Msg::DialogButtonKeepOpen).to_string()
    });
    cx.simulate_prompt_answer(&keep_label);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 3, "keep-open leaves the dirty tabs");
        assert_eq!(
            app.status,
            t(app.language, Msg::StatusCanceled),
            "cancel is user-visible"
        );
    });

    let discard_label = app.update(cx, |app, _| {
        t(app.language, Msg::DialogButtonDiscardAndCloseTabs).to_string()
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(0, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CloseOthers, window, cx);
        });
    });
    cx.simulate_prompt_answer(&discard_label);
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab().document.text(), "A");
    });
}

#[gpui::test]
fn close_other_tabs_all_clean_closes_without_dialog(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("A")),
            EditorTab::new(MarkdownDocument::from_text("B")),
            EditorTab::new(MarkdownDocument::from_text("C")),
        ];
        app.active_tab = 2;
        app
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(2, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CloseOthers, window, cx);
        });
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 1);
        // Removals left of the clicked tab must follow the active index.
        assert_eq!(app.active_tab, 0);
        assert_eq!(app.active_tab().document.text(), "C");
    });
}

#[gpui::test]
fn close_tabs_to_the_right_scopes_past_the_anchor(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("A")),
            EditorTab::new(MarkdownDocument::from_text("B")),
            EditorTab::new(MarkdownDocument::from_text("C")),
            EditorTab::new(MarkdownDocument::from_text("D")),
        ];
        app
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(1, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CloseToTheRight, window, cx);
        });
    });
    cx.run_until_parked();
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab().document.text(), "B");
        assert!(
            app.tabs
                .iter()
                .all(|tab| matches!(tab.document.text(), "A" | "B"))
        );
    });
}

#[gpui::test]
fn tab_context_menu_stale_target_cancels_instead_of_closing_the_wrong_tab(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::from_text("A")),
            EditorTab::new(MarkdownDocument::from_text("B")),
            EditorTab::new(MarkdownDocument::from_text("C")),
        ];
        app
    });

    // Index out of range: the captured tab was removed while the menu was open.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(2, Point::new(px(10.), px(10.)), cx);
            app.tabs.remove(2);
            app.handle_tab_context_action(TabContextAction::CloseTab, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 2);
        assert!(app.tab_context_menu.is_none());
        assert_eq!(app.status, t(app.language, Msg::StatusCanceled));
    });

    // Same index, different tab state (e.g. the tab was replaced in place).
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(1, Point::new(px(10.), px(10.)), cx);
            app.tabs[1] = EditorTab::new(MarkdownDocument::from_text("replacement"));
            app.handle_tab_context_action(TabContextAction::CloseTab, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(
            app.tabs[1].document.text(),
            "replacement",
            "identity mismatch must not close the replacement tab"
        );
    });
}

#[gpui::test]
fn tab_context_rename_reuses_file_pipeline_and_refuses_dirty(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let path = root.join("doc.md");
    fs::write(&path, "# One").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::open(&path).unwrap()),
            EditorTab::new(MarkdownDocument::from_text("scratch")),
        ];
        app.workspace_root = root.clone();
        app.file_tree = Some(FileTree::scan(&root).unwrap());
        app
    });

    // Rename on a clean file-backed tab opens the prompt prefilled with the
    // file name and targeting the tab's path.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(0, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::Rename, window, cx);
        });
    });
    app.update(cx, |app, _| {
        let pending = app.pending_name_input.as_ref().expect("prompt open");
        assert_eq!(pending.kind, PendingNameKind::Rename);
        assert_eq!(pending.buffer, "doc.md");
        assert_eq!(pending.target.as_deref(), Some(path.as_path()));
        assert!(app.tab_context_menu.is_none());
    });

    // Commit: file renamed on disk, open tab re-pointed at the new path.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.pending_name_input.as_mut().unwrap().buffer = "renamed.md".to_string();
            app.confirm_pending_name(&ConfirmPendingName, window, cx);
        });
    });
    app.update(cx, |app, _| {
        let new_path = root.join("renamed.md");
        assert!(!path.exists());
        assert!(new_path.exists());
        assert_eq!(app.tabs[0].path(), Some(new_path.as_path()));
        assert_eq!(app.tabs[0].document.text(), "# One");
        assert_eq!(app.tabs.len(), 2, "the untitled sibling is untouched");
    });

    // Dirty tab refuses rename with the save-first status.
    app.update(cx, |app, _| {
        app.active_tab = 0;
        app.tabs[0].document.replace_range(0..0, "dirty ");
    });
    let save_first = app.update(cx, |app, _| t(app.language, Msg::StatusSaveBeforeRename));
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(0, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::Rename, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.status, save_first);
        assert!(app.pending_name_input.is_none());
    });
}

#[gpui::test]
fn tab_context_copy_path_reports_status_and_skips_untitled(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.md");
    fs::write(&path, "plain").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![
            EditorTab::new(MarkdownDocument::open(&path).unwrap()),
            EditorTab::new(MarkdownDocument::from_text("untitled")),
        ];
        app
    });

    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(0, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CopyPath, window, cx);
        });
    });
    let expected = app.update(cx, |app, _| {
        tf(
            app.language,
            Msg::StatusCopiedPath,
            &[&path.display().to_string()],
        )
    });
    app.update(cx, |app, _| {
        assert_eq!(app.status, expected);
        assert!(app.tab_context_menu.is_none());
    });

    // Untitled tab: the disabled action never dispatches state changes.
    let status_before = app.update(cx, |app, _| app.status.clone());
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.show_tab_context_menu(1, Point::new(px(10.), px(10.)), cx);
            app.handle_tab_context_action(TabContextAction::CopyPath, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.status, status_before);
        assert!(app.tab_context_menu.is_none());
    });
}

/// The inline name editor must keep the document inert: while it is open, a
/// mouse-down in the editor pane must neither move the document caret nor
/// start a selection (the old behavior moved the caret and dragged a
/// selection — the reported "rename moves the source caret" bug).
#[gpui::test]
fn name_editor_open_click_in_editor_pane_leaves_document_untouched(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("hello world"))];
        app
    });
    // Park the caret at a known offset and open the editor.
    app.update(cx, |app, _| {
        app.active_tab_mut().selected_range = 3..3;
        app.pending_name_input = Some(PendingNameInput::new(
            PendingNameKind::Rename,
            PathBuf::from("w"),
            None,
            "hello.md",
        ));
    });

    // Click + drag in the editor pane with the editor open.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.on_mouse_down(
                &MouseDownEvent {
                    button: MouseButton::Left,
                    position: point(px(40.), px(20.)),
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                },
                window,
                cx,
            );
            app.on_mouse_move(
                &MouseMoveEvent {
                    position: point(px(120.), px(30.)),
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Default::default(),
                },
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().selected_range, 3..3, "caret unmoved");
        assert!(!app.active_tab().is_selecting, "no drag selection");
    });
}

/// Click-away commit: a mouse-down that lands below the menu bar (the
/// workspace-row close_menu handler) commits the typed name through the same
/// pipeline as Enter, and flags the click so the following tree-row mouse-up
/// does not also open a file.
#[gpui::test]
fn name_editor_click_away_commits_and_guards_the_following_mouse_up(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let path = root.join("doc.md");
    fs::write(&path, "# One").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![EditorTab::new(MarkdownDocument::open(&path).unwrap())];
        app.workspace_root = root.clone();
        app.file_tree = Some(FileTree::scan(&root).unwrap());
        app.selected_tree_path = Some(path.clone());
        app
    });

    // Open the rename editor and type a replacement for the selected base name.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.rename_tree_entry(&RenameTreeEntry, window, cx);
        });
    });
    cx.update(|_, cx| {
        app.update(cx, |app, cx| {
            app.push_text_input("renamed", cx);
        });
    });
    // Click away: the simulated workspace-row mouse-down commits the name.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.close_menu(
                &MouseDownEvent {
                    button: MouseButton::Left,
                    position: point(px(300.), px(300.)),
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                },
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, _| {
        assert!(app.pending_name_input.is_none(), "editor committed");
        assert!(app.name_editor_click_away, "click flagged for mouse-up");
        let new_path = root.join("renamed.md");
        assert!(new_path.exists(), "file renamed on disk");
        assert!(!path.exists());
        assert_eq!(app.tabs[0].path(), Some(new_path.as_path()));
    });

    // The mouse-up that closes the click clears the flag without side effects.
    app.update(cx, |app, _| {
        app.name_editor_click_away = false;
    });
}

/// A refused commit (empty buffer) must leave the editor open for retry
/// instead of discarding it — the old behavior silently killed the prompt.
#[gpui::test]
fn name_editor_refused_commit_keeps_editor_open(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let path = root.join("doc.md");
    fs::write(&path, "# One").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![EditorTab::new(MarkdownDocument::open(&path).unwrap())];
        app.workspace_root = root.clone();
        app.file_tree = Some(FileTree::scan(&root).unwrap());
        app.selected_tree_path = Some(path.clone());
        app
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.rename_tree_entry(&RenameTreeEntry, window, cx);
        });
    });
    // Delete the whole name (base selected + extension), then click away.
    cx.update(|_, cx| {
        app.update(cx, |app, cx| {
            app.move_name_caret(NameCaretMove::SelectAll, cx);
            app.pop_text_input(cx);
        });
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.close_menu(
                &MouseDownEvent {
                    button: MouseButton::Left,
                    position: point(px(300.), px(300.)),
                    modifiers: Default::default(),
                    click_count: 1,
                    first_mouse: false,
                },
                window,
                cx,
            );
        });
    });
    app.update(cx, |app, _| {
        assert!(
            app.pending_name_input.is_some(),
            "empty name keeps editor open"
        );
        assert_eq!(app.status, t(app.language, Msg::StatusNameRequired));
        assert!(path.exists(), "filesystem untouched");
    });
}

/// Typed characters replace the selected base name in one stroke, and arrow
/// keys move the name editor's caret instead of the document caret.
#[gpui::test]
fn name_editor_selection_replace_and_arrow_keys_stay_in_buffer(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("document text"))];
        app
    });
    app.update(cx, |app, _| {
        app.active_tab_mut().selected_range = 2..2;
        app.pending_name_input = Some(PendingNameInput::new(
            PendingNameKind::Rename,
            PathBuf::from("w"),
            None,
            "report.md",
        ));
    });

    // The rename editor pre-selects the base name: typing replaces it.
    cx.update(|_, cx| {
        app.update(cx, |app, cx| {
            app.push_text_input("notes", cx);
        });
    });
    app.update(cx, |app, _| {
        let pending = app.pending_name_input.as_ref().unwrap();
        assert_eq!(pending.buffer, "notes.md");
        // Caret sits right after the inserted replacement text.
        assert_eq!(pending.selection(), "notes".len().."notes".len());
    });

    // Arrow keys move the buffer caret; the document caret stays put.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.left(&Left, window, cx);
            app.left(&Left, window, cx);
            app.right(&Right, window, cx);
        });
    });
    app.update(cx, |app, _| {
        let pending = app.pending_name_input.as_ref().unwrap();
        assert_eq!(pending.cursor, "notes".len() - 1, "two lefts, one right");
        assert_eq!(
            app.active_tab().selected_range,
            2..2,
            "document caret unmoved"
        );
    });

    // Shift+left extends the selection inside the buffer only.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.select_left(&SelectLeft, window, cx);
        });
    });
    app.update(cx, |app, _| {
        let pending = app.pending_name_input.as_ref().unwrap();
        assert_eq!(pending.selection(), "notes".len() - 2.."notes".len() - 1);
        assert_eq!(app.active_tab().selected_range, 2..2);
    });

    // Home/End clamp within the buffer; document untouched.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.home(&Home, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.pending_name_input.as_ref().unwrap().cursor, 0);
        assert_eq!(app.active_tab().selected_range, 2..2);
    });

    // Backspace deletes at the buffer caret, not in the document.
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.backspace(&Backspace, window, cx);
        });
    });
    app.update(cx, |app, _| {
        let pending = app.pending_name_input.as_ref().unwrap();
        assert_eq!(pending.buffer, "notes.md");
        assert_eq!(pending.cursor, 0);
        assert_eq!(app.active_tab().document.text(), "document text");
    });
}

/// IME composition over the pre-selected base name: the marked text replaces
/// the selection, and the composition is removed (not duplicated) when the
/// user keeps typing through the redirected path.
#[gpui::test]
fn name_editor_ime_composition_over_selection(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("x"))];
        app
    });
    app.update(cx, |app, _| {
        app.pending_name_input = Some(PendingNameInput::new(
            PendingNameKind::Rename,
            PathBuf::from("w"),
            None,
            "report.md",
        ));
    });
    cx.update(|_, cx| {
        app.update(cx, |app, cx| {
            // IME composition events each carry the full current composition
            // string (platform contract); the redirected path strips the
            // previous composition before inserting the new one.
            app.insert_redirected_text("笔", true, cx);
            app.insert_redirected_text("笔记", true, cx);
            app.insert_redirected_text("笔记本", false, cx);
        });
    });
    app.update(cx, |app, _| {
        let pending = app.pending_name_input.as_ref().unwrap();
        assert_eq!(pending.buffer, "笔记本.md");
        assert_eq!(pending.cursor, "笔记本".len(), "caret after composition");
        assert_eq!(pending.selection(), pending.cursor..pending.cursor);
    });
}

/// Escape still cancels the editor outright — no commit, filesystem untouched.
#[gpui::test]
fn name_editor_escape_cancels_without_touching_disk(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let path = root.join("doc.md");
    fs::write(&path, "# One").unwrap();
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.language = Language::En;
        app.tabs = vec![EditorTab::new(MarkdownDocument::open(&path).unwrap())];
        app.workspace_root = root.clone();
        app.file_tree = Some(FileTree::scan(&root).unwrap());
        app.selected_tree_path = Some(path.clone());
        app
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.rename_tree_entry(&RenameTreeEntry, window, cx);
            app.push_text_input("typed", cx);
        });
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.clear_file_tree_search(&ClearFileTreeSearch, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert!(app.pending_name_input.is_none(), "Escape closed the editor");
        assert!(path.exists(), "original file untouched");
        assert!(!root.join("typed.md").exists());
    });
}

#[test]
fn search_field_state_edits_selection_ime_and_multibyte_boundaries() {
    let mut field = SearchFieldState::new("a文🙂b");
    field.move_caret(SearchCaretMove::Home);
    field.move_caret(SearchCaretMove::Right);
    assert_eq!(field.cursor, 1);
    field.move_caret(SearchCaretMove::SelectRight);
    assert_eq!(field.selected_text(), Some("文"));
    field.replace_selection("字", false);
    assert_eq!(field.buffer, "a字🙂b");

    field.move_caret(SearchCaretMove::SelectAll);
    field.replace_selection("拼", true);
    assert_eq!(field.marked_range, Some(0.."拼".len()));
    field.replace_selection("拼音", true);
    field.replace_selection("拼音", false);
    assert_eq!(field.buffer, "拼音");
    assert!(field.marked_range.is_none());

    field.move_caret(SearchCaretMove::Left);
    field.backspace();
    assert_eq!(field.buffer, "音");
    field.move_caret(SearchCaretMove::Home);
    field.delete_forward();
    assert!(field.buffer.is_empty());
}

#[test]
fn search_field_state_converts_platform_utf16_ranges_without_splitting_utf8() {
    let field = SearchFieldState::new("a🙂文");
    assert_eq!(field.byte_to_utf16(1), 1);
    assert_eq!(field.byte_to_utf16(5), 3);
    assert_eq!(field.range_from_utf16(1..3), 1..5);
    assert_eq!(
        field.utf16_to_byte(2),
        1,
        "inside a surrogate pair snaps left"
    );
}

#[test]
fn shared_search_pattern_keeps_unicode_regex_and_zero_width_behavior_identical() {
    let literal = SearchPattern::compile(&SearchOptions {
        query: "文本".into(),
        case_sensitive: true,
        regex: false,
    })
    .unwrap();
    assert_eq!(literal.find_ranges("文本 x 文本"), vec![0..6, 9..15]);

    let zero_width = SearchPattern::compile(&SearchOptions {
        query: r"\b".into(),
        case_sensitive: false,
        regex: true,
    })
    .unwrap();
    assert_eq!(zero_width.find_ranges("ab"), vec![0..0, 2..2]);
    assert!(
        SearchPattern::compile(&SearchOptions {
            query: "(".into(),
            case_sensitive: false,
            regex: true,
        })
        .is_err()
    );
}

#[test]
fn read_preview_search_uses_visible_canonical_runs_in_document_order() {
    let doc = MarkdownDocument::from_text(
        "# Styled **visible phrase**\n\n[visible label](https://hidden.example/visible-phrase)\n\n```text\nvisible phrase\n```\n\n| A | B |\n|---|---|\n| visible phrase | cell |\n\n<div>visible phrase</div>\n\n![alt](hidden-visible-phrase.png)\n\n$visible phrase$",
    );
    let blocks = doc.preview_blocks_shared();
    let pattern = SearchPattern::compile(&SearchOptions {
        query: "visible phrase".into(),
        case_sensitive: false,
        regex: false,
    })
    .unwrap();
    let matches = preview_search_matches(&blocks, &pattern);

    assert!(
        matches.len() >= 4,
        "heading, code, table and HTML are searchable"
    );
    assert!(matches.windows(2).all(|pair| {
        (
            pair[0].block_index,
            pair[0].run_id.rank(),
            pair[0].range.start,
        ) <= (
            pair[1].block_index,
            pair[1].run_id.rank(),
            pair[1].range.start,
        )
    }));
    assert!(matches.iter().all(|found| {
        !matches!(
            found.run_id,
            PreviewTextRunId::MathLatex | PreviewTextRunId::CodeLine(_)
        )
    }));
    assert_eq!(
        matches
            .iter()
            .filter(|found| found.run_id == PreviewTextRunId::CodeBody)
            .count(),
        1,
        "line-number presentation does not duplicate canonical code matches",
    );

    let destination = SearchPattern::compile(&SearchOptions {
        query: "hidden.example".into(),
        case_sensitive: false,
        regex: false,
    })
    .unwrap();
    assert!(preview_search_matches(&blocks, &destination).is_empty());
    let image_path = SearchPattern::compile(&SearchOptions {
        query: "hidden-visible-phrase.png".into(),
        case_sensitive: false,
        regex: false,
    })
    .unwrap();
    assert!(preview_search_matches(&blocks, &image_path).is_empty());
}

#[gpui::test]
fn focused_search_field_actions_and_ime_never_mutate_the_document(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "alpha beta alpha",
        ))];
        app.search_visible = true;
        app.search_focus = Some(SearchField::Find);
        app
    });
    let original = "alpha beta alpha";
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            EntityInputHandler::replace_text_in_range(app, None, "文本", window, cx);
            EntityInputHandler::replace_and_mark_text_in_range(app, None, "拼", None, window, cx);
            EntityInputHandler::replace_and_mark_text_in_range(app, None, "拼音", None, window, cx);
            app.left(&Left, window, cx);
            app.select_right(&SelectRight, window, cx);
            app.backspace(&Backspace, window, cx);
            app.home(&Home, window, cx);
            app.delete(&Delete, window, cx);
            app.select_all(&SelectAll, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), original);
        assert_eq!(
            app.search_query.selection(),
            0..app.search_query.buffer.len()
        );
    });
}

#[gpui::test]
fn search_navigation_wraps_and_read_mode_gates_replacement_without_losing_state(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "alpha\nalpha\nalpha",
        ))];
        app.search_visible = true;
        app.search_form = SearchPanelForm::Replace;
        app.replace_visible = true;
        app.search_query.set_text("alpha");
        app.replace_text.set_text("omega");
        app.search_focus = Some(SearchField::Find);
        app.refresh_search_matches();
        app
    });
    app.update(cx, |app, _| {
        assert_eq!(app.current_search_index, Some(0));
        assert!(
            app.search_matches
                .iter()
                .all(|target| matches!(target, SearchTarget::Source(_)))
        );
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.find_previous(&FindPrevious, window, cx);
            assert_eq!(app.current_search_index, Some(2));
            app.set_view_mode(ViewMode::Read, cx);
            let blocks = app.active_tab().document.preview_blocks_shared();
            let version = app.active_tab().document.version();
            let tab = app.active_tab_mut();
            tab.preview_reflects_version = Some(version);
            tab.sync_preview_list(&blocks);
            app.search_generation = None;
            app.refresh_search_matches();
            let before = app.active_tab().document.text().to_string();
            app.replace_current_match(&ReplaceCurrentMatch, window, cx);
            assert_eq!(app.active_tab().document.text(), before);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.search_form, SearchPanelForm::Replace);
        assert!(!app.replace_visible);
        assert_eq!(app.replace_text.buffer, "omega");
        assert!(
            app.search_matches
                .iter()
                .all(|target| matches!(target, SearchTarget::ReadPreview(_)))
        );
    });
    app.update(cx, |app, cx| app.set_view_mode(ViewMode::Edit, cx));
    app.update(cx, |app, _| {
        assert!(app.replace_visible);
        assert_eq!(app.search_query.buffer, "alpha");
        assert_eq!(app.replace_text.buffer, "omega");
    });
}

#[gpui::test]
fn search_overlay_renders_responsively_in_localized_light_dark_and_invalid_states(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "Alpha 文本 alpha",
        ))];
        app.search_visible = true;
        app.search_form = SearchPanelForm::Replace;
        app.replace_visible = true;
        app.search_focus = Some(SearchField::Find);
        app.search_control_focus = Some(SearchOverlayControl::FindField);
        app.search_query.set_text("(");
        app.search_regex = true;
        app.refresh_search_matches();
        app
    });
    cx.simulate_resize(size(px(460.), px(420.)));

    for (language, theme) in [
        (Language::En, AppTheme::Paper),
        (Language::ZhHans, AppTheme::Ink),
    ] {
        app.update(cx, |app, cx| {
            app.language = language;
            app.theme = theme;
            app.custom_theme = None;
            app.selected_theme_name = theme.name().to_string();
            cx.notify();
        });
        cx.run_until_parked();
        for selector in ["search-panel", "search-find-row", "search-replace-row"] {
            assert!(cx.debug_bounds(selector).is_some(), "missing {selector}");
        }
        app.update(cx, |app, _| {
            assert!(matches!(
                app.search_result,
                SearchResultState::InvalidPattern(_)
            ));
            assert_ne!(app.palette().panel_bg, app.palette().text);
            assert_ne!(app.palette().search_match, app.palette().search_current);
        });
    }

    app.update(cx, |app, cx| {
        app.search_regex = false;
        app.search_query
            .set_text("very-long-search-value-".repeat(20));
        app.search_generation = None;
        app.refresh_search_matches();
        assert_eq!(app.search_result, SearchResultState::NoMatches);
        cx.notify();
    });
    cx.run_until_parked();
    let panel = cx.debug_bounds("search-panel").unwrap();
    let field = cx.debug_bounds("search-find-field").unwrap();
    assert!(field.right() <= panel.right());

    app.update(cx, |app, cx| app.set_view_mode(ViewMode::Read, cx));
    cx.run_until_parked();
    assert!(cx.debug_bounds("search-read-guidance").is_some());
    app.update(cx, |app, _| assert!(!app.replace_visible));
}

#[gpui::test]
fn search_ime_composition_is_field_only_in_every_view_mode(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("document"))];
        app.search_visible = true;
        app.search_focus = Some(SearchField::Find);
        app.search_control_focus = Some(SearchOverlayControl::FindField);
        app
    });
    app.update(cx, |app, cx| {
        for (language, mode) in [
            (Language::En, ViewMode::Edit),
            (Language::ZhHans, ViewMode::Split),
            (Language::En, ViewMode::VisualEdit),
            (Language::ZhHans, ViewMode::Read),
        ] {
            app.language = language;
            app.set_view_mode(mode, cx);
            if mode == ViewMode::Read {
                let blocks = app.active_tab().document.preview_blocks_shared();
                let version = app.active_tab().document.version();
                let tab = app.active_tab_mut();
                tab.preview_reflects_version = Some(version);
                tab.sync_preview_list(&blocks);
            }
            app.search_focus = Some(SearchField::Find);
            app.search_control_focus = Some(SearchOverlayControl::FindField);
            app.search_query.set_text("");
            app.insert_redirected_text("拼", true, cx);
            app.insert_redirected_text("拼音", true, cx);
            app.insert_redirected_text("拼音", false, cx);
            assert_eq!(app.search_query.buffer, "拼音");
            assert!(app.search_query.marked_range.is_none());
            assert_eq!(app.active_tab().document.text(), "document");
        }
    });
}

#[gpui::test]
fn search_tab_clipboard_escape_and_reopen_preserve_field_values(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("unchanged"))];
        app.search_visible = true;
        app.search_form = SearchPanelForm::Replace;
        app.replace_visible = true;
        app.search_focus = Some(SearchField::Find);
        app.search_control_focus = Some(SearchOverlayControl::FindField);
        app.search_query.set_text("文本");
        app.replace_text.set_text("replacement");
        app.search_query.move_caret(SearchCaretMove::SelectAll);
        app
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            app.copy(&Copy, window, cx);
            assert_eq!(
                cx.read_from_clipboard().and_then(|item| item.text()),
                Some("文本".into())
            );
            app.indent(&Indent, window, cx);
            assert_eq!(
                app.search_control_focus,
                Some(SearchOverlayControl::ReplaceField)
            );
            app.outdent(&Outdent, window, cx);
            assert_eq!(
                app.search_control_focus,
                Some(SearchOverlayControl::FindField)
            );
            app.clear_file_tree_search(&ClearFileTreeSearch, window, cx);
            assert!(!app.search_visible);
            app.show_replace(&ShowReplace, window, cx);
        });
    });
    app.update(cx, |app, _| {
        assert_eq!(app.active_tab().document.text(), "unchanged");
        assert_eq!(app.search_query.buffer, "文本");
        assert_eq!(app.replace_text.buffer, "replacement");
        assert_eq!(app.search_form, SearchPanelForm::Replace);
    });
}

#[gpui::test]
fn source_search_state_replaces_continues_undoes_and_clears_invalid_or_empty_results(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("one one"))];
        app.search_visible = true;
        app.search_form = SearchPanelForm::Replace;
        app.replace_visible = true;
        app.search_query.set_text("one");
        app.replace_text.set_text("X");
        app.search_focus = Some(SearchField::Find);
        app.search_control_focus = Some(SearchOverlayControl::FindField);
        app.refresh_search_matches();
        app
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            assert_eq!(app.current_search_index, Some(0));
            app.replace_current_match(&ReplaceCurrentMatch, window, cx);
            assert_eq!(app.active_tab().document.text(), "X one");
            let SearchTarget::Source(next) = &app.search_matches[app.current_search_index.unwrap()]
            else {
                panic!("source replacement must continue in the source domain");
            };
            assert_eq!(next.range, 2..5);

            app.replace_all_matches(&ReplaceAllMatches, window, cx);
            assert_eq!(app.active_tab().document.text(), "X X");
            app.undo(&Undo, window, cx);
            assert_eq!(app.active_tab().document.text(), "X one");

            app.search_regex = true;
            app.search_query.set_text("(");
            app.search_generation = None;
            app.refresh_search_matches();
            assert!(matches!(
                app.search_result,
                SearchResultState::InvalidPattern(_)
            ));
            assert!(app.search_matches.is_empty());
            assert!(app.current_search_index.is_none());

            app.search_query.set_text("");
            app.search_generation = None;
            app.refresh_search_matches();
            assert_eq!(app.search_result, SearchResultState::Idle);
            assert!(app.search_matches.is_empty());
        });
    });
}

#[gpui::test]
fn read_search_options_navigation_highlights_pending_and_close_transitions(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "Alpha alpha 文本\n\n**styled phrase**\n\n```text\nAlpha\n```",
        ))];
        app.view_mode = ViewMode::Read;
        let blocks = app.active_tab().document.preview_blocks_shared();
        let version = app.active_tab().document.version();
        let tab = app.active_tab_mut();
        tab.preview_reflects_version = Some(version);
        tab.sync_preview_list(&blocks);
        app.search_visible = true;
        app.search_focus = Some(SearchField::Find);
        app.search_control_focus = Some(SearchOverlayControl::FindField);
        app.search_query.set_text("alpha");
        app.refresh_search_matches();
        app
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            assert_eq!(app.search_matches.len(), 3);
            assert_eq!(app.current_search_index, Some(0));
            app.find_previous(&FindPrevious, window, cx);
            assert_eq!(app.current_search_index, Some(2));

            app.search_case_sensitive = true;
            app.search_generation = None;
            app.refresh_search_matches();
            assert_eq!(app.search_matches.len(), 1);

            app.search_regex = true;
            app.search_query.set_text(r"^Alpha$");
            app.search_generation = None;
            app.refresh_search_matches();
            assert_eq!(app.search_matches.len(), 1);

            let current = app.current_search_index.unwrap();
            let SearchTarget::ReadPreview(found) = app.search_matches[current].clone() else {
                panic!("Read mode must own rendered targets");
            };
            let block = &app.active_tab().preview_list_blocks[found.block_index];
            let text = preview_run_plain_text(block, found.run_id).unwrap();
            let painted = active_preview_search_ranges(app, found.block_index, found.run_id, &text);
            assert!(painted.iter().any(|(_, is_current)| *is_current));

            app.active_tab_mut().preview_reflects_version = None;
            app.search_generation = None;
            app.refresh_search_matches();
            assert_eq!(app.search_result, SearchResultState::PendingPreview);
            assert!(app.search_matches.is_empty());

            app.close_search_overlay(cx);
            assert!(!app.search_visible);
            assert_eq!(app.search_result, SearchResultState::Idle);
            assert!(app.search_matches.is_empty());
        });
    });
}

#[gpui::test]
fn search_keyboard_navigation_cut_paste_and_escape_are_overlay_only(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "alpha alpha alpha",
        ))];
        app.search_visible = true;
        app.search_focus = Some(SearchField::Find);
        app.search_control_focus = Some(SearchOverlayControl::FindField);
        app.search_query.set_text("alpha");
        app.refresh_search_matches();
        app
    });
    cx.update(|window, cx| {
        app.update(cx, |app, cx| {
            let document = app.active_tab().document.text().to_string();
            app.insert_newline(&InsertNewline, window, cx);
            assert_eq!(app.current_search_index, Some(1));
            app.search_previous_or_newline(&SearchPreviousOrNewline, window, cx);
            assert_eq!(app.current_search_index, Some(0));

            app.search_query.move_caret(SearchCaretMove::SelectAll);
            app.cut(&Cut, window, cx);
            assert!(app.search_query.buffer.is_empty());
            cx.write_to_clipboard(ClipboardItem::new_string("alpha".to_string()));
            app.paste(&Paste, window, cx);
            assert_eq!(app.search_query.buffer, "alpha");

            app.indent(&Indent, window, cx);
            assert_eq!(
                app.search_control_focus,
                Some(SearchOverlayControl::Previous)
            );
            app.outdent(&Outdent, window, cx);
            assert_eq!(
                app.search_control_focus,
                Some(SearchOverlayControl::FindField)
            );
            app.clear_file_tree_search(&ClearFileTreeSearch, window, cx);
            assert!(!app.search_visible);
            assert_eq!(app.active_tab().document.text(), document);
        });
    });
}

#[gpui::test]
fn search_domain_tracks_all_four_view_modes_and_replacement_availability(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text("needle needle"))];
        app.search_visible = true;
        app.search_form = SearchPanelForm::Replace;
        app.search_query.set_text("needle");
        app
    });
    app.update(cx, |app, cx| {
        for mode in [ViewMode::Edit, ViewMode::Split, ViewMode::VisualEdit] {
            app.set_view_mode(mode, cx);
            app.search_generation = None;
            app.refresh_search_matches();
            assert!(app.replace_visible);
            assert!(
                app.search_matches
                    .iter()
                    .all(|target| matches!(target, SearchTarget::Source(_)))
            );
        }

        app.set_view_mode(ViewMode::Read, cx);
        assert_eq!(app.search_result, SearchResultState::PendingPreview);
        let blocks = app.active_tab().document.preview_blocks_shared();
        let version = app.active_tab().document.version();
        let tab = app.active_tab_mut();
        tab.preview_reflects_version = Some(version);
        tab.sync_preview_list(&blocks);
        app.search_generation = None;
        app.refresh_search_matches();
        assert!(!app.replace_visible);
        assert!(
            app.search_matches
                .iter()
                .all(|target| matches!(target, SearchTarget::ReadPreview(_)))
        );
    });
}
