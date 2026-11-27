#![cfg(target_os = "linux")]

use gpui::{
    Application, Font, FontFallbacks, FontFeatures, FontStyle, FontWeight, SharedString, px,
};
use markion::{DEFAULT_CODE_FONT_FAMILY, SYSTEM_UI_FONT_FAMILY};

#[test]
fn linux_system_ui_and_code_font_fallbacks_preserve_styles() {
    Application::headless().run(|cx| {
        let ts = cx.text_system();
        let make = |family: &str, weight: FontWeight, style: FontStyle| Font {
            family: SharedString::from(family.to_string()),
            features: FontFeatures::default(),
            fallbacks: None,
            weight,
            style,
        };

        let regular = ts.resolve_font(&make(
            SYSTEM_UI_FONT_FAMILY,
            FontWeight::NORMAL,
            FontStyle::Normal,
        ));
        let bold = ts.resolve_font(&make(
            SYSTEM_UI_FONT_FAMILY,
            FontWeight::BOLD,
            FontStyle::Normal,
        ));
        let italic = ts.resolve_font(&make(
            SYSTEM_UI_FONT_FAMILY,
            FontWeight::NORMAL,
            FontStyle::Italic,
        ));
        let bold_italic = ts.resolve_font(&make(
            SYSTEM_UI_FONT_FAMILY,
            FontWeight::BOLD,
            FontStyle::Italic,
        ));

        assert_ne!(
            regular, bold,
            ".SystemUIFont must resolve a distinct bold FontId"
        );
        assert_ne!(
            regular, italic,
            ".SystemUIFont must resolve a distinct italic FontId"
        );
        assert_ne!(
            bold, bold_italic,
            ".SystemUIFont must resolve a distinct bold-italic FontId"
        );

        // Normal-only fonts (such as CJK faces or DejaVu Sans without an italic cut)
        // must still produce a distinct synthetic-italic FontId so upright and slanted
        // glyphs do not alias in the glyph atlas.
        let available = ts.all_font_names();
        for normal_only_candidate in ["Noto Sans CJK SC", "Source Han Sans SC", "DejaVu Sans"] {
            if available
                .iter()
                .any(|installed| installed == normal_only_candidate)
            {
                let upright = ts.resolve_font(&make(
                    normal_only_candidate,
                    FontWeight::NORMAL,
                    FontStyle::Normal,
                ));
                let slanted = ts.resolve_font(&make(
                    normal_only_candidate,
                    FontWeight::NORMAL,
                    FontStyle::Italic,
                ));
                assert_ne!(
                    upright, slanted,
                    "synthetic italic for {normal_only_candidate} must use a distinct FontId"
                );
            }
        }

        if ["DejaVu Sans Mono", "Liberation Mono", "Noto Sans Mono"]
            .iter()
            .any(|mono| available.iter().any(|installed| installed == mono))
        {
            let code_font = Font {
                family: SharedString::from(DEFAULT_CODE_FONT_FAMILY),
                features: FontFeatures::default(),
                fallbacks: Some(FontFallbacks::from_fonts(vec![
                    "Cascadia Code".to_string(),
                    "Consolas".to_string(),
                    "SFMono-Regular".to_string(),
                    "Menlo".to_string(),
                    "DejaVu Sans Mono".to_string(),
                    "Liberation Mono".to_string(),
                    "Noto Sans Mono".to_string(),
                ])),
                weight: FontWeight::NORMAL,
                style: FontStyle::Normal,
            };
            let code_id = ts.resolve_font(&code_font);
            let adv_i = ts.advance(code_id, px(14.), 'i').expect("advance for 'i'");
            let adv_w = ts.advance(code_id, px(14.), 'W').expect("advance for 'W'");
            assert_eq!(
                adv_i.width, adv_w.width,
                "code slot fallback must resolve to a monospace font on Linux"
            );
        }

        cx.spawn(async move |cx| {
            let _ = cx.update(|cx| cx.quit());
        })
        .detach();
    });
}
