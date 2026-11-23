//! Shared native popup styling for completion and block context controls.
use super::root_view::SearchTooltip;
use super::*;

pub(super) fn panel(palette: ThemePalette, id: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.to_string())
        .w(px(248.))
        .occlude()
        .p_2()
        .bg(palette.panel_bg)
        .text_color(palette.text)
        .border_1()
        .border_color(palette.border)
        .rounded_lg()
        .shadow_lg()
        .flex()
        .flex_col()
        .gap_2()
}

pub(super) fn button(
    palette: ThemePalette,
    id: &'static str,
    selected: bool,
    enabled: bool,
    destructive: bool,
    label: SharedString,
) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.to_string())
        .tooltip(move |_, cx| {
            cx.new(|_| SearchTooltip {
                palette,
                label: label.clone(),
            })
            .into()
        })
        .h(px(30.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .rounded_md()
        .text_size(px(15.))
        .when(destructive, |button| {
            button
                .bg(if selected {
                    palette.control_hover_bg
                } else {
                    palette.control_bg
                })
                .text_color(palette.destructive_text)
        })
        .when(selected && !destructive, |button| {
            button.bg(palette.active_bg).text_color(palette.active_text)
        })
        .when(!selected && enabled, |button| {
            let hover_bg = if destructive {
                palette.control_hover_bg
            } else {
                palette.surface_bg
            };
            button.hover(move |style| style.bg(hover_bg))
        })
}
