use super::*;
use gpui::{TestAppContext, VisualTestContext};
use markion::FileTreeFileKind;

fn small_file_tree(root: &Path) -> FileTree {
    FileTree {
        root: root.to_path_buf(),
        show_hidden: false,
        entries: (0..3)
            .map(|index| FileTreeEntry {
                path: root.join(format!("note-{index}.md")),
                name: format!("note-{index}.md"),
                depth: 0,
                kind: FileTreeEntryKind::File,
                file_kind: Some(FileTreeFileKind::Markdown),
            })
            .collect(),
    }
}

#[gpui::test]
fn sidebar_tab_bar_keeps_content_gap_across_themes_widths_and_tabs(cx: &mut TestAppContext) {
    let root = PathBuf::from("/tmp/markion-sidebar-tab-gap");
    let (app, cx) = cx.add_window_view(|_, cx| {
        let mut app = MarkionApp::new(cx);
        app.tabs = vec![EditorTab::new(MarkdownDocument::from_text(
            "# Root\n\n## Child\n",
        ))];
        app.workspace_root = root.clone();
        app.file_tree = Some(small_file_tree(&root));
        app.sidebar_visible = true;
        app.sidebar_tab = SidebarTab::Files;
        app
    });

    for theme in [AppTheme::Paper, AppTheme::Ink] {
        for width in [520., 1000.] {
            cx.simulate_resize(size(px(width), px(500.)));
            app.update(cx, |app, cx| {
                app.theme = theme;
                app.custom_theme = None;
                app.selected_theme_name = theme.name().to_string();
                cx.notify();
            });
            cx.run_until_parked();

            for (tab, content_selector) in [
                (SidebarTab::Files, "file-tree-workspace-root"),
                (SidebarTab::Outline, "outline-heading-row-0"),
            ] {
                app.update(cx, |app, cx| app.set_sidebar_tab(tab, cx));
                cx.run_until_parked();

                let tab_bar = cx
                    .debug_bounds("sidebar-tab-bar")
                    .expect("sidebar tab bar should render");
                let content = cx
                    .debug_bounds(content_selector)
                    .unwrap_or_else(|| panic!("{content_selector} should render"));
                let actual_gap = f32::from(content.top()) - f32::from(tab_bar.bottom());
                assert!(
                    (actual_gap - SIDEBAR_TAB_CONTENT_GAP).abs() < 0.01,
                    "{tab:?} content should stay {SIDEBAR_TAB_CONTENT_GAP}px below the tab bar in {} at {width}px, got {actual_gap}px",
                    theme.name()
                );
            }
        }
    }
}
