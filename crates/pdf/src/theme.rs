//! Print-friendly styling constants for the built-in PDF writer (design D5).
//! Seeded from a light theme so exports print well regardless of the
//! editing theme.

use crate::ir::{AlertKind, Rgb};

/// Body text size in points.
pub const BODY_SIZE: f32 = 11.0;
/// Body line height as a multiple of the font size.
pub const LINE_HEIGHT_MULT: f32 = 1.45;
/// Fenced code / inline code block size.
pub const CODE_SIZE: f32 = 9.5;
/// Footnote text size.
pub const NOTE_SIZE: f32 = 8.5;
/// Page-number footer size.
pub const FOOTER_SIZE: f32 = 9.0;

/// Heading sizes for levels 1–6.
pub const HEADING_SIZES: [f32; 6] = [22.0, 17.0, 14.0, 12.0, 11.0, 10.5];

/// Vertical space after a paragraph / list item.
pub const PARA_SPACING: f32 = 6.0;
/// Space before a heading.
pub const HEADING_BEFORE: f32 = 14.0;
/// Space after a heading.
pub const HEADING_AFTER: f32 = 6.0;
/// Space around non-paragraph blocks (code, tables, images, rules).
pub const BLOCK_SPACING: f32 = 8.0;

/// Horizontal indent per list nesting level.
pub const LIST_INDENT: f32 = 18.0;
/// Hanging indent reserved for generated list markers.
pub const MARKER_WIDTH: f32 = 16.0;
/// Extra content indent inside quotes and alerts.
pub const ACCENT_INDENT: f32 = 12.0;
/// Horizontal padding inside table cells.
pub const CELL_PAD: f32 = 4.0;
/// Vertical padding inside table cells.
pub const CELL_PAD_Y: f32 = 3.0;
/// Padding inside the code-block background.
pub const CODE_PAD: f32 = 6.0;
/// Gap between the footnote rule and the content bottom limit.
pub const NOTE_GAP: f32 = 10.0;
/// Maximum share of the content height the per-page footnote area may use.
pub const NOTE_AREA_CAP: f32 = 0.4;
/// Space around a horizontal rule.
pub const RULE_SPACING: f32 = 8.0;
/// Table-of-contents entry line height.
pub const TOC_LINE: f32 = 18.0;

pub const TEXT: Rgb = Rgb(0x1a, 0x1a, 0x1a);
pub const MUTED: Rgb = Rgb(0x66, 0x66, 0x66);
pub const LINK: Rgb = Rgb(0x0b, 0x5c, 0xff);
pub const RULE_COLOR: Rgb = Rgb(0xc8, 0xc8, 0xc8);
pub const HIGHLIGHT: Rgb = Rgb(0xff, 0xf1, 0xa8);
pub const CODE_BG: Rgb = Rgb(0xf5, 0xf5, 0xf3);
pub const TABLE_BORDER: Rgb = Rgb(0xd3, 0xd3, 0xd3);
pub const HEADER_BORDER: Rgb = Rgb(0x9a, 0x9a, 0x9a);
pub const QUOTE_ACCENT: Rgb = Rgb(0xb9, 0xb9, 0xb9);

/// Accent color for a GFM alert kind.
pub fn alert_accent(kind: AlertKind) -> Rgb {
    match kind {
        AlertKind::Note => Rgb(0x09, 0x69, 0xda),
        AlertKind::Tip => Rgb(0x1a, 0x7f, 0x37),
        AlertKind::Important => Rgb(0x82, 0x50, 0xdf),
        AlertKind::Warning => Rgb(0x9a, 0x67, 0x00),
        AlertKind::Caution => Rgb(0xcf, 0x22, 0x2e),
    }
}

/// Background tint for a GFM alert kind.
pub fn alert_tint(kind: AlertKind) -> Rgb {
    match kind {
        AlertKind::Note => Rgb(0xdf, 0xee, 0xff),
        AlertKind::Tip => Rgb(0xda, 0xfb, 0xe1),
        AlertKind::Important => Rgb(0xf1, 0xea, 0xff),
        AlertKind::Warning => Rgb(0xff, 0xf8, 0xc5),
        AlertKind::Caution => Rgb(0xff, 0xeb, 0xe9),
    }
}

/// Uppercase label for a GFM alert kind.
pub fn alert_label(kind: AlertKind) -> &'static str {
    match kind {
        AlertKind::Note => "NOTE",
        AlertKind::Tip => "TIP",
        AlertKind::Important => "IMPORTANT",
        AlertKind::Warning => "WARNING",
        AlertKind::Caution => "CAUTION",
    }
}
