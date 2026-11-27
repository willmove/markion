use super::*;

const PANEL_WIDTH: f32 = 912.;
const CARD_GAP: f32 = 12.;

pub(super) struct ExamplesGallery {
    entries: Vec<ExampleCard>,
    guide_count: usize,
    selected: Option<usize>,
    scroll: ScrollHandle,
    error: Option<SharedString>,
}

struct ExampleCard {
    path: PathBuf,
    name: SharedString,
    lines: Vec<ThumbnailLine>,
}

enum ThumbnailLine {
    Title(SharedString),
    Heading(SharedString),
    Body(SharedString),
}

impl ExamplesGallery {
    fn load() -> io::Result<Self> {
        let index = bundled_resource_path(PathBuf::from_iter(["assets", "examples", "README.md"]))
            .ok_or(io::ErrorKind::NotFound)?;
        let directory = index.parent().ok_or(io::ErrorKind::NotFound)?;
        let mut paths = fs::read_dir(directory)?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
            .collect::<Vec<_>>();
        if paths.is_empty() {
            return Err(io::ErrorKind::NotFound.into());
        }
        let is_template = |path: &Path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.split_once('-'))
                .and_then(|(number, _)| number.parse::<u16>().ok())
                .is_some_and(|number| number >= 10)
        };
        paths.sort_by(|a, b| {
            (is_template(a), a != &index, a).cmp(&(is_template(b), b != &index, b))
        });
        let guide_count = paths.iter().take_while(|path| !is_template(path)).count();
        Ok(Self {
            entries: paths.into_iter().map(ExampleCard::new).collect(),
            guide_count,
            selected: None,
            scroll: ScrollHandle::new(),
            error: None,
        })
    }
}

impl ExampleCard {
    fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
            .into();
        let mut lines = Vec::new();
        if let Ok(text) = fs::read_to_string(&path) {
            let document = MarkdownDocument::from_text(text);
            thumbnail_lines(&document.preview_blocks_shared(), &mut lines);
        }
        Self { path, name, lines }
    }
}

fn thumbnail_lines(blocks: &[PreviewBlock], lines: &mut Vec<ThumbnailLine>) {
    for block in blocks {
        if lines.len() >= 6 {
            break;
        }
        match block {
            PreviewBlock::Heading { level, text, .. } if *level == 1 => {
                lines.push(ThumbnailLine::Title(text.text.clone().into()));
            }
            PreviewBlock::Heading { text, .. } => {
                lines.push(ThumbnailLine::Heading(text.text.clone().into()));
            }
            PreviewBlock::Paragraph { text, .. } | PreviewBlock::ListItem { text, .. } => {
                lines.push(ThumbnailLine::Body(text.text.clone().into()));
            }
            PreviewBlock::BlockQuote { children, .. } => thumbnail_lines(children, lines),
            _ => {}
        }
    }
}

impl MarkionApp {
    pub(super) fn show_examples_templates(
        &mut self,
        _: &ShowExamplesTemplates,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.active_menu = None;
        if self.examples_gallery.is_none() {
            self.examples_gallery =
                Some(ExamplesGallery::load().unwrap_or_else(|_| ExamplesGallery {
                    entries: Vec::new(),
                    guide_count: 0,
                    selected: None,
                    scroll: ScrollHandle::new(),
                    error: Some(self.tr(Msg::StatusExamplesMissing).into()),
                }));
        }
        window.focus(&self.examples_gallery_focus);
        cx.notify();
    }

    pub(super) fn close_examples_gallery(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.examples_gallery = None;
        window.focus(&self.focus_handle);
        cx.notify();
    }

    fn activate_example(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(card) = self
            .examples_gallery
            .as_ref()
            .and_then(|gallery| gallery.entries.get(index))
        else {
            return;
        };
        match self.open_bundled_example(card.path.clone(), cx) {
            Ok(()) => self.close_examples_gallery(window, cx),
            Err(error) => {
                let message = self.trf(Msg::StatusOpenFailed, &[&error.to_string()]);
                if let Some(gallery) = &mut self.examples_gallery {
                    gallery.error = Some(message);
                }
                cx.notify();
            }
        }
    }

    fn open_bundled_example(&mut self, path: PathBuf, cx: &mut Context<Self>) -> io::Result<()> {
        if let Some(index) = self.tabs.iter().position(|tab| {
            tab.document_tab().is_some_and(|tab| {
                tab.document.path().is_none() && tab.bundled_example.as_ref() == Some(&path)
            })
        }) {
            self.switch_active_tab(index, cx);
        } else {
            let document = MarkdownDocument::from_text(fs::read_to_string(&path)?);
            let mut tab = self.editor_tab_for_document(document);
            tab.bundled_example = Some(path);
            self.open_tab_in_new_tab(tab, cx);
        }
        self.status = self.trf(Msg::StatusOpened, &[&self.active_tab().title()]);
        cx.notify();
        Ok(())
    }

    pub(super) fn open_bundled_example_link(&mut self, url: &str, cx: &mut Context<Self>) -> bool {
        if url.contains(':') {
            return false;
        }
        let Some(tab) = self.active_tab().document_tab() else {
            return false;
        };
        let Some(directory) = tab
            .bundled_example
            .as_deref()
            .filter(|_| tab.document.path().is_none())
            .and_then(Path::parent)
        else {
            return false;
        };
        let Some(relative) = markion::resolve_local_reference(Path::new(""), url) else {
            return false;
        };
        if relative.parent() != Some(Path::new(""))
            || relative
                .extension()
                .is_none_or(|extension| extension != "md")
        {
            return false;
        }
        match self.open_bundled_example(directory.join(relative), cx) {
            Ok(()) => {
                if let Some((_, fragment)) = url.split_once('#')
                    && let Some(anchor) =
                        markion::document_heading_fragment(&format!("#{fragment}"))
                    && let Some(offset) = self
                        .active_tab()
                        .document
                        .heading_offset_for_anchor(&anchor)
                {
                    self.navigate_to_outline_heading(offset, cx);
                }
            }
            Err(error) => {
                self.status = self.trf(Msg::StatusOpenFailed, &[&error.to_string()]);
                cx.notify();
            }
        }
        true
    }

    fn examples_gallery_key(
        &mut self,
        event: &KeyDownEvent,
        columns: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            self.close_examples_gallery(window, cx);
            return;
        }
        let Some(gallery) = &mut self.examples_gallery else {
            return;
        };
        let count = gallery.entries.len();
        if count == 0 {
            return;
        }
        if matches!(key, "enter" | "space") {
            let selected = gallery.selected.unwrap_or(0);
            self.activate_example(selected, window, cx);
            return;
        }
        let step = match key {
            "right" => 1,
            "left" => -1,
            "down" => columns as isize,
            "up" => -(columns as isize),
            "tab" if event.keystroke.modifiers.shift => -1,
            "tab" => 1,
            _ => return,
        };
        let next = match gallery.selected {
            Some(current) if key == "tab" => {
                (current as isize + step).rem_euclid(count as isize) as usize
            }
            Some(current) => current.saturating_add_signed(step).min(count - 1),
            None if step < 0 => count - 1,
            None => 0,
        };
        gallery.selected = Some(next);
        let heading_count = 1 + usize::from(next >= gallery.guide_count && gallery.guide_count > 0);
        gallery.scroll.scroll_to_item(next + heading_count);
        cx.notify();
    }
}

pub(super) fn gallery_view(
    app: &MarkionApp,
    gallery: &ExamplesGallery,
    window: &Window,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let palette = app.palette();
    let panel_width = (f32::from(window.viewport_size().width) - 32.).clamp(1., PANEL_WIDTH);
    let grid_width = (panel_width - 50. - PANE_SCROLLBAR_RESERVED_WIDTH).max(1.);
    let columns = (((grid_width + CARD_GAP) / (177. + CARD_GAP)) as usize).clamp(1, 4);
    let card_width = ((grid_width - CARD_GAP * (columns - 1) as f32) / columns as f32).floor();
    let mut body = div()
        .id("examples-gallery-body")
        .debug_selector(|| "examples-gallery-body".into())
        .h_full()
        .min_h_0()
        .overflow_y_scroll()
        .scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))
        .track_scroll(&gallery.scroll)
        .p_1()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(CARD_GAP));

    for (index, entry) in gallery.entries.iter().enumerate() {
        if index == 0 || index == gallery.guide_count {
            let label = if index < gallery.guide_count {
                Msg::ExamplesGalleryGuides
            } else {
                Msg::ExamplesGalleryTemplates
            };
            body = body.child(
                div()
                    .w_full()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(app.tr(label)),
            );
        }
        body = body.child(card_view(
            entry,
            index,
            gallery.selected == Some(index),
            card_width,
            palette,
            cx,
        ));
    }

    div()
        .id("examples-gallery-overlay")
        .debug_selector(|| "examples-gallery-overlay".into())
        .absolute()
        .inset_0()
        .occlude()
        .bg(rgba(0x00000055))
        .p_4()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("examples-gallery")
                .track_focus(&app.examples_gallery_focus)
                .occlude()
                .w(px(panel_width))
                .h_full()
                .max_h(px(656.))
                .p_5()
                .rounded_lg()
                .shadow_lg()
                .border_1()
                .border_color(palette.border)
                .bg(palette.panel_bg)
                .text_color(palette.text)
                .flex()
                .flex_col()
                .gap_3()
                .on_key_down(cx.listener(move |app, event, window, cx| {
                    app.examples_gallery_key(event, columns, window, cx);
                }))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_between()
                        .gap_3()
                        .child(
                            div()
                                .text_size(px(18.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(app.tr(Msg::ItemExamplesTemplates)),
                        )
                        .child(
                            div()
                                .id("examples-gallery-close")
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .border_1()
                                .border_color(palette.border)
                                .cursor_pointer()
                                .hover(move |button| button.bg(palette.surface_bg))
                                .child(app.tr(Msg::ExamplesGalleryClose))
                                .on_click(cx.listener(|app, _: &ClickEvent, window, cx| {
                                    app.close_examples_gallery(window, cx);
                                })),
                        ),
                )
                .when_some(gallery.error.clone(), |panel, error| {
                    panel.child(
                        div()
                            .text_size(px(13.))
                            .text_color(palette.muted)
                            .child(error),
                    )
                })
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .min_h_0()
                        .child(body)
                        .child(pane_scrollbar_view(
                            PaneScrollTarget::ExamplesGallery,
                            &gallery.scroll,
                            palette,
                            cx,
                        )),
                ),
        )
}

fn card_view(
    entry: &ExampleCard,
    index: usize,
    selected: bool,
    width: f32,
    palette: ThemePalette,
    cx: &mut Context<MarkionApp>,
) -> impl IntoElement {
    let mut page = div()
        .h(px(126.))
        .overflow_hidden()
        .rounded_sm()
        .bg(palette.surface_bg)
        .px_3()
        .py_2()
        .flex()
        .flex_col()
        .gap_1();
    for line in &entry.lines {
        let line_view = div().flex_none().line_height(px(14.));
        page = page.child(match line {
            ThumbnailLine::Title(text) => line_view
                .text_size(px(12.))
                .font_weight(FontWeight::BOLD)
                .text_color(palette.text)
                .line_clamp(2)
                .child(text.clone()),
            ThumbnailLine::Heading(text) => line_view
                .text_size(px(10.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text)
                .line_clamp(1)
                .child(text.clone()),
            ThumbnailLine::Body(text) => line_view
                .text_size(px(9.))
                .text_color(palette.muted)
                .line_clamp(2)
                .child(text.clone()),
        });
    }

    div()
        .id(("example-card", index))
        .debug_selector(move || format!("example-card-{index}"))
        .w(px(width))
        .min_w_0()
        .flex_none()
        .rounded_md()
        .border_1()
        .border_color(if selected {
            palette.active_text
        } else {
            palette.border
        })
        .bg(if selected {
            palette.active_bg
        } else {
            palette.panel_bg
        })
        .hover(move |card| card.bg(palette.active_bg).border_color(palette.active_text))
        .p_2()
        .cursor_pointer()
        .child(page)
        .child(
            div()
                .pt_2()
                .text_size(px(12.))
                .font_weight(FontWeight::MEDIUM)
                .text_ellipsis()
                .child(entry.name.clone()),
        )
        .on_click(cx.listener(move |app, _: &ClickEvent, window, cx| {
            app.activate_example(index, window, cx);
        }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Modifiers, TestAppContext};

    #[gpui::test]
    fn examples_gallery_opens_drafts_and_preserves_the_existing_workspace(cx: &mut TestAppContext) {
        let (app, cx) = cx.add_window_view(|_, cx| {
            let mut app = MarkionApp::new(cx);
            app.active_tab_mut()
                .document
                .set_text("unsaved workspace note");
            app
        });
        let original_root = app.update(cx, |app, _| app.workspace_root.clone());
        cx.update(|window, cx| {
            window.focus(&app.read(cx).focus_handle);
            window.activate_window();
        });

        cx.dispatch_action(ShowExamplesTemplates);
        cx.run_until_parked();
        app.update(cx, |app, _| {
            let gallery = app.examples_gallery.as_ref().unwrap();
            assert_eq!(gallery.guide_count, 6);
            assert_eq!(gallery.entries.len(), 18);
            assert!(gallery.entries.iter().all(|entry| !entry.lines.is_empty()));
            assert_eq!(app.tabs.len(), 1);
            assert!(app.active_tab().is_dirty());
        });

        let index_card = cx.debug_bounds("example-card-0").expect("index card");
        cx.simulate_click(index_card.center(), Modifiers::none());
        cx.run_until_parked();
        app.update(cx, |app, cx| {
            assert!(app.examples_gallery.is_none());
            assert_eq!(app.tabs.len(), 2);
            assert_eq!(app.active_tab().title(), "README.md");
            assert!(app.active_tab().path().is_none());
            assert_eq!(app.workspace_root, original_root);
            assert_eq!(app.tabs[0].document.text(), "unsaved workspace note");
            assert!(app.tabs[0].is_dirty());

            app.activate_document_or_external_link("05-images-links-export.md", cx);
            let guide = app.active_tab();
            assert_eq!(guide.title(), "05-images-links-export.md");
            assert!(guide.path().is_none());
            let image = guide
                .resource_dir()
                .unwrap()
                .join("images")
                .join("markion.png");
            assert!(image.is_file());
            app.activate_document_or_external_link("README.md", cx);
            assert_eq!(app.active_tab().title(), "README.md");
            assert_eq!(
                app.tabs.len(),
                3,
                "opening the index again reuses its draft"
            );
        });
        assert!(cx.opened_url().is_none());

        cx.dispatch_action(ShowExamplesTemplates);
        cx.run_until_parked();
        cx.simulate_keystrokes("tab tab enter");
        cx.run_until_parked();
        app.update(cx, |app, _| {
            assert!(app.examples_gallery.is_none());
            assert_eq!(app.active_tab().title(), "01-getting-started.md");
            assert!(app.active_tab().path().is_none());
        });
    }
}
