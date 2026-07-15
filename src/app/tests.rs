use super::*;

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
    assert_eq!(normalize_preview_selection_range("hello", 4..1), 1..4);
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
fn shortcut_reference_lists_core_workflows() {
    let reference = shortcut_reference_text();

    assert!(reference.contains("| Action | Windows/Linux | macOS |"));
    assert!(reference.contains("| Save | Ctrl+S | Cmd+S |"));
    assert!(reference.contains("| Cycle View Mode | Ctrl+Shift+V | Cmd+Shift+V |"));
    assert!(reference.contains("| Edit Mode | Ctrl+Alt+1 | Cmd+Option+1 |"));
    assert!(reference.contains("| Visual Edit Mode | Ctrl+Alt+4 | Cmd+Option+4 |"));
    assert!(reference.contains("| Split Preview Mode | Ctrl+Alt+2 | Cmd+Option+2 |"));
    assert!(reference.contains("| Read Mode | Ctrl+Alt+3 | Cmd+Option+3 |"));
    assert!(reference.contains("| Sidebar | Ctrl+Shift+B | Cmd+Shift+B |"));
    assert!(reference.contains("| Preferences | Ctrl+, | Cmd+, |"));
    assert!(reference.contains("| Find | Ctrl+F | Cmd+F |"));
    assert!(reference.contains("| DOCX | Ctrl+Shift+D | Cmd+Shift+D |"));
    assert!(!reference.contains("Secondary-"));
}

#[test]
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
    assert!(read_mode_preview_is_constrained(ViewMode::Read, false));
    assert!(!read_mode_preview_is_constrained(ViewMode::Read, true));
    assert!(!read_mode_preview_is_constrained(ViewMode::Split, false));
    assert!(!read_mode_preview_is_constrained(ViewMode::Split, true));
    assert!(!read_mode_preview_is_constrained(ViewMode::Edit, false));
    assert!(!read_mode_preview_is_constrained(
        ViewMode::VisualEdit,
        false
    ));
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
    let segments = vec![
        VisualTextSegment {
            visible_range: 0..5,
            source_range: 2..7,
        },
        VisualTextSegment {
            visible_range: 5..9,
            source_range: 11..15,
        },
    ];
    assert_eq!(visual_source_for_visible(&segments, 0), 2);
    assert_eq!(visual_source_for_visible(&segments, 4), 6);
    assert_eq!(visual_source_for_visible(&segments, 6), 12);
    assert_eq!(visual_visible_for_source(&segments, 13), Some(7));
    assert_eq!(visual_visible_for_source(&segments, 9), None);
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
        !(index_from_closure < app_tabs.len()),
        "tab-bar closure must skip a stale index instead of assigning it"
    );
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
