//! krilla emission (task 2.8): draws the placed page model (glyphs, fills,
//! strokes, images, SVG), adds link annotations, builds the document
//! outline, and writes metadata. All positioning was resolved during
//! layout; this module only translates primitives into krilla calls.

use krilla::Document;
use krilla::SerializeSettings;
use krilla::action::{Action, LinkAction};
use krilla::annotation::{Annotation, LinkAnnotation, Target};
use krilla::color::rgb;
use krilla::destination::{Destination, XyzDestination};
use krilla::geom::{Path, PathBuilder, Point, Rect, Size, Transform};
use krilla::metadata::{DateTime, Metadata};
use krilla::outline::{Outline, OutlineNode};
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, Stroke};
use krilla::surface::Surface;
use krilla::tagging::{ContentTag, Tag, TagGroup, TagTree};
use krilla_svg::{SurfaceExt, SvgSettings};

use crate::PdfError;
use crate::ir::{PdfMetadata, Rgb};
use crate::layout::{AnnotTarget, LayoutResult, OutlineEntry, PageLayout, PlacedItem, RectF};

fn fill(color: Rgb) -> Fill {
    Fill {
        paint: rgb::Color::new(color.0, color.1, color.2).into(),
        ..Fill::default()
    }
}

fn stroke(color: Rgb, width: f32) -> Stroke {
    Stroke {
        paint: rgb::Color::new(color.0, color.1, color.2).into(),
        width,
        ..Stroke::default()
    }
}

fn rect_path(rect: RectF) -> Option<Path> {
    let mut builder = PathBuilder::new();
    builder.push_rect(Rect::from_xywh(rect.x, rect.y, rect.w, rect.h)?);
    builder.finish()
}

fn line_path(x1: f32, y1: f32, x2: f32, y2: f32) -> Option<Path> {
    let mut builder = PathBuilder::new();
    builder.move_to(x1, y1);
    builder.line_to(x2, y2);
    builder.finish()
}

fn draw_item(surface: &mut Surface, item: &PlacedItem, tagging: bool) -> Option<TagGroup> {
    match item {
        PlacedItem::Line(line) => {
            surface.set_stroke(None);
            for group in &line.groups {
                surface.set_fill(Some(fill(group.fill)));
                surface.draw_glyphs(
                    Point::from_xy(line.x + group.start_x, line.baseline),
                    &group.glyphs,
                    group.font.clone(),
                    &group.text,
                    group.font_size,
                    false,
                );
            }
        }
        PlacedItem::Rect { rect, fill: color } => {
            if let Some(path) = rect_path(*rect) {
                surface.set_stroke(None);
                surface.set_fill(Some(fill(*color)));
                surface.draw_path(&path);
            }
        }
        PlacedItem::Stroke {
            x1,
            y1,
            x2,
            y2,
            width,
            color,
        } => {
            if let Some(path) = line_path(*x1, *y1, *x2, *y2) {
                surface.set_fill(None);
                surface.set_stroke(Some(stroke(*color, *width)));
                surface.draw_path(&path);
            }
        }
        PlacedItem::Image {
            image,
            alt,
            x,
            y,
            w,
            h,
        } => {
            if let Some(size) = Size::from_wh(*w, *h) {
                let identifier =
                    (tagging && !alt.is_empty()).then(|| surface.start_tagged(ContentTag::Other));
                surface.push_transform(&Transform::from_translate(*x, *y));
                surface.draw_image(image.clone(), size);
                surface.pop();
                if let Some(identifier) = identifier {
                    surface.end_tagged();
                    let mut group = TagGroup::new(Tag::Figure(Some(alt.clone())));
                    group.push(identifier);
                    return Some(group);
                }
            }
        }
        PlacedItem::Svg {
            tree,
            alt,
            x,
            y,
            w,
            h,
        } => {
            if let Some(size) = Size::from_wh(*w, *h) {
                let identifier =
                    (tagging && !alt.is_empty()).then(|| surface.start_tagged(ContentTag::Other));
                surface.push_transform(&Transform::from_translate(*x, *y));
                // Math SVG is path-only; the default settings (embedded
                // text) only matter for SVGs with <text> elements.
                surface.draw_svg(tree, size, SvgSettings::default());
                surface.pop();
                if let Some(identifier) = identifier {
                    surface.end_tagged();
                    let mut group = TagGroup::new(Tag::Figure(Some(alt.clone())));
                    group.push(identifier);
                    return Some(group);
                }
            }
        }
        PlacedItem::PushClip(rect) => {
            if let Some(path) = rect_path(*rect) {
                surface.push_clip_path(&path, &FillRule::NonZero);
            }
        }
        PlacedItem::Pop => surface.pop(),
    }
    None
}

fn emit_page(
    document: &mut Document,
    settings: PageSettings,
    layout: &PageLayout,
    toc_offset: usize,
    tagging: bool,
) -> Vec<TagGroup> {
    let mut groups = Vec::new();
    let mut page = document.start_page_with(settings);
    {
        let mut surface = page.surface();
        for item in &layout.items {
            if let Some(group) = draw_item(&mut surface, item, tagging) {
                groups.push(group);
            }
        }
        surface.finish();
    }
    for annot in &layout.annotations {
        let Some(rect) = Rect::from_xywh(annot.rect.x, annot.rect.y, annot.rect.w, annot.rect.h)
        else {
            continue;
        };
        let target = match &annot.target {
            AnnotTarget::Url(url) => Target::Action(Action::Link(LinkAction::new(url.clone()))),
            AnnotTarget::Internal { page, y } => Target::Destination(Destination::Xyz(
                XyzDestination::new(page + toc_offset, Point::from_xy(0.0, *y)),
            )),
        };
        page.add_annotation(Annotation::new_link(
            LinkAnnotation::new(rect, target),
            None,
        ));
    }
    page.finish();
    groups
}

/// Recursively group heading entries into nested outline nodes.
fn build_nodes(entries: &[OutlineEntry], toc_offset: usize, parent_level: u8) -> Vec<OutlineNode> {
    let mut nodes = Vec::new();
    let mut i = 0;
    while i < entries.len() {
        let entry = &entries[i];
        if entry.level <= parent_level {
            break;
        }
        let dest = XyzDestination::new(entry.page + toc_offset, Point::from_xy(0.0, entry.y));
        let mut node = OutlineNode::new(entry.title.clone(), dest);
        let mut j = i + 1;
        while j < entries.len() && entries[j].level > entry.level {
            j += 1;
        }
        for child in build_nodes(&entries[i + 1..j], toc_offset, entry.level) {
            node.push_child(child);
        }
        nodes.push(node);
        i = j;
    }
    nodes
}

/// Parse a front-matter date string (`YYYY`, `YYYY-MM`, or `YYYY-MM-DD`)
/// into a krilla date time.
fn parse_date(date: &str) -> Option<DateTime> {
    let mut parts = date.trim().split('-');
    let year: u16 = parts.next()?.parse().ok()?;
    let dt = DateTime::new(year);
    let dt = match parts.next() {
        Some(m) => dt.month(m.parse().ok()?),
        None => return Some(dt),
    };
    match parts.next() {
        Some(d) => Some(dt.day(d.parse().ok()?)),
        None => Some(dt),
    }
}

/// Emit the laid-out document to PDF bytes.
pub fn emit_document(layout: &LayoutResult, metadata: &PdfMetadata) -> Result<Vec<u8>, PdfError> {
    let settings = PageSettings::from_wh(layout.page_width, layout.page_height)
        .ok_or_else(|| PdfError::Layout("invalid page dimensions".to_string()))?;
    let toc_offset = layout.toc_pages.len();

    let tagging = layout
        .toc_pages
        .iter()
        .chain(layout.body_pages.iter())
        .flat_map(|page| page.items.iter())
        .any(|item| match item {
            PlacedItem::Image { alt, .. } | PlacedItem::Svg { alt, .. } => !alt.is_empty(),
            _ => false,
        });
    let mut document = Document::new_with(SerializeSettings {
        enable_tagging: tagging,
        ..SerializeSettings::default()
    });
    let mut tag_tree = TagTree::new();
    for page in layout.toc_pages.iter().chain(layout.body_pages.iter()) {
        for group in emit_page(&mut document, settings.clone(), page, toc_offset, tagging) {
            tag_tree.push(group);
        }
    }
    if tagging {
        document.set_tag_tree(tag_tree);
    }

    if !layout.outline.is_empty() {
        let mut outline = Outline::new();
        for node in build_nodes(&layout.outline, toc_offset, 0) {
            outline.push_child(node);
        }
        document.set_outline(outline);
    }

    let mut meta = Metadata::new()
        .creator("Markion".to_string())
        .producer("Markion built-in PDF writer (markion-pdf)".to_string());
    if let Some(title) = &metadata.title {
        meta = meta.title(title.clone());
    }
    if let Some(author) = &metadata.author {
        meta = meta.authors(vec![author.clone()]);
    }
    if let Some(date) = &metadata.date
        && let Some(dt) = parse_date(date)
    {
        meta = meta.creation_date(dt);
    }
    document.set_metadata(meta);

    document
        .finish()
        .map_err(|e| PdfError::Emit(format!("{e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_image_alt_text_is_written_into_the_tag_tree() {
        fn document(alt: &str) -> crate::PdfDocument {
            crate::PdfDocument {
                blocks: vec![crate::Block::Image {
                    data: crate::ImageData::Svg(
                        r##"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="32"><rect width="48" height="32" fill="#38a"/></svg>"##
                            .into(),
                    ),
                    alt: alt.into(),
                    width_px: 48,
                    height_px: 32,
                }],
                ..crate::PdfDocument::default()
            }
        }
        fn contains(bytes: &[u8], needle: &[u8]) -> bool {
            bytes.windows(needle.len()).any(|window| window == needle)
        }
        fn pdf_text(text: &str) -> Vec<u8> {
            if text.bytes().all(|byte| matches!(byte, 32..=126)) {
                return text.as_bytes().to_vec();
            }
            [0xfe, 0xff]
                .into_iter()
                .chain(text.encode_utf16().flat_map(u16::to_be_bytes))
                .collect()
        }

        let first = crate::render(&document("Four colored quadrants")).unwrap();
        let second = crate::render(&document("Red green blue and yellow swatches")).unwrap();
        assert!(contains(&first, b"/Alt"));
        assert!(contains(&first, &pdf_text("Four colored quadrants")));
        assert!(contains(
            &second,
            &pdf_text("Red green blue and yellow swatches")
        ));
        assert!(!contains(&second, &pdf_text("Four colored quadrants")));
    }

    #[test]
    fn parses_front_matter_dates() {
        assert!(parse_date("2026-08-22").is_some());
        assert!(parse_date("2026-08").is_some());
        assert!(parse_date("2026").is_some());
        assert!(parse_date("not a date").is_none());
    }
}
