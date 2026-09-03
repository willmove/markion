//! Layout tests for the code-block horizontal-scroll construction.
//!
//! gpui derives a scroll container's extent from its direct children's
//! *layout* bounds, so the wrap-off code path hosts its rows in a flex row
//! whose single `flex_none` child sizes to its (nowrap) content. A block
//! child would cap at the container width and never become scrollable —
//! these tests pin that distinction so the construction cannot silently
//! regress.

use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Pixels, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, StyledText, TestAppContext, Window, div, px,
};

const CONTAINER_WIDTH: Pixels = px(300.);
const CONTAINER_HEIGHT: Pixels = px(120.);

/// Renders the exact wrap-off scaffolding `code_block_view` builds: a flex
/// scroll container whose single `flex_none` child stacks gutter rows of
/// nowrap text. `flex_content: false` reproduces the naive block-child
/// construction that must NOT scroll (the historical bug).
struct CodeScrollProbe {
    scroll: ScrollHandle,
    line: &'static str,
    flex_content: bool,
}

impl Render for CodeScrollProbe {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let content = div()
            .min_w_full()
            .whitespace_nowrap()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .min_w_full()
                    .child(div().w(px(36.)).flex_none().child("  1"))
                    .child(div().flex_none().child(StyledText::new(self.line))),
            );
        let content = if self.flex_content {
            content.flex_none()
        } else {
            content
        };
        div().w(CONTAINER_WIDTH).h(CONTAINER_HEIGHT).child(
            div()
                .id("probe-scroll")
                .w_full()
                .flex()
                .overflow_x_scroll()
                .track_scroll(&self.scroll)
                .child(content),
        )
    }
}

fn scroll_extent_for(line: &'static str, flex_content: bool, cx: &mut TestAppContext) -> Pixels {
    let scroll = ScrollHandle::new();
    cx.add_window_view(|_, _| CodeScrollProbe {
        scroll: scroll.clone(),
        line,
        flex_content,
    });
    cx.refresh().expect("draw the probe window");
    scroll.max_offset().width
}

#[gpui::test]
fn flex_none_nowrap_content_produces_horizontal_scroll_extent(cx: &mut TestAppContext) {
    let long_line = "fn main() { println!(\"this line is deliberately much longer than the three hundred pixel wide container it lives in\"); }";
    let short_line = "let x = 1;";

    assert!(
        scroll_extent_for(long_line, true, cx) > px(0.),
        "a long nowrap line in a flex_none child must widen the scroll extent"
    );
    assert_eq!(
        scroll_extent_for(short_line, true, cx),
        px(0.),
        "content narrower than the container must not scroll (min_w_full keeps it filled)"
    );
    // Negative control: the block-child construction stays capped at the
    // container width and never scrolls — the wrap-off path must keep the
    // flex_none wrapper.
    assert_eq!(
        scroll_extent_for(long_line, false, cx),
        px(0.),
        "block child caps at the container width; the flex_none construction exists to avoid it"
    );
}
