//! API spike for the built-in PDF export engine (OpenSpec change
//! `improve-pdf-export`, task 2.2). This test proves — against the pinned
//! dependency versions — every krilla / krilla-svg / cosmic-text API the
//! engine design relies on:
//!
//! 1. krilla document/page/surface lifecycle → `Vec<u8>` starting with `%PDF`.
//! 2. Pre-shaped glyph emission: cosmic-text (fontdb discovery + HarfBuzz
//!    shaping + automatic font fallback) lays out a mixed Latin/Han string,
//!    and the glyphs are emitted through `Surface::draw_glyphs` with one
//!    `krilla::text::Font` per glyph group — no `Surface::draw_text`
//!    (rustybuzz simple path) involved.
//! 3. A link annotation over the text rectangle.
//! 4. A two-level document outline pointing at the page.
//! 5. Document metadata (title/authors/creation date).
//! 6. SVG embedding through krilla-svg, in both shapes that matter to us:
//!    path-only SVG (what Markion's math renderer emits — see
//!    `crates/markdown/src/math.rs::is_self_contained_svg`, which rejects
//!    `<text`) and SVG with a `<text>` element (needs a fontdb).

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};
use krilla::geom::{Point, Rect, Size, Transform};
use krilla::page::PageSettings;
use krilla::text::{Font as KrillaFont, GlyphId, KrillaGlyph};
use krilla::Document;
use krilla::action::{Action, LinkAction};
use krilla::annotation::{Annotation, LinkAnnotation, Target};
use krilla::destination::XyzDestination;
use krilla::metadata::{DateTime, Metadata};
use krilla::outline::{Outline, OutlineNode};
use krilla::Data;
use krilla_svg::{SurfaceExt, SvgSettings};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Shaped glyphs for one line of text, plus the krilla fonts they use.
struct ShapedLine {
    /// (font group) — consecutive glyphs sharing one cosmic font, in order.
    groups: Vec<GlyphGroup>,
    /// Cosmic baseline offset of the line within the buffer.
    line_y: f32,
    /// Visual width of the line.
    line_w: f32,
}

struct GlyphGroup {
    font: KrillaFont,
    /// Absolute pen x of the first glyph (buffer coordinates).
    start_x: f32,
    font_size: f32,
    glyphs: Vec<KrillaGlyph>,
    /// The line text the glyph `text_range`s index into.
    text: String,
}

/// Shape `text` with cosmic-text and convert each layout run into
/// krilla-ready glyph groups, proving the shape-with-cosmic / emit-with-krilla
/// pipeline. Returns the shaped lines and the number of distinct fonts used.
fn shape_line(fs: &mut FontSystem, text: &str, font_size: f32) -> (Vec<ShapedLine>, usize) {
    let mut buffer = Buffer::new(fs, Metrics::new(font_size, font_size * 1.4));
    buffer.set_size(Some(500.0), None);
    buffer.set_text(
        text,
        &Attrs::new().family(Family::Name("Georgia")),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(fs, true);

    let mut fonts: HashMap<_, (KrillaFont, String)> = HashMap::new();
    let mut lines = Vec::new();

    for run in buffer.layout_runs() {
        let glyphs = run.glyphs;
        assert!(!glyphs.is_empty(), "layout run without glyphs");

        let mut groups = Vec::new();
        let mut i = 0;
        while i < glyphs.len() {
            // Group consecutive glyphs that share a font.
            let first = i;
            let font_id = glyphs[i].font_id;
            while i + 1 < glyphs.len() && glyphs[i + 1].font_id == font_id {
                i += 1;
            }
            let group = &glyphs[first..=i];
            let font_size = group[0].font_size;

            let font = fonts.entry(font_id).or_insert_with(|| {
                let cosmic_font = fs
                    .get_font(font_id, group[0].font_weight)
                    .expect("cosmic-text resolved a font it cannot load");
                // Face index inside a TTC comes from fontdb, NOT from cosmic's
                // `Font::id()`; `font.data()` is the whole font file (TTCs
                // included), which is exactly what krilla wants.
                let face = fs.db().face(font_id).expect("fontdb knows the face");
                let source_path = match &face.source {
                    usvg::fontdb::Source::File(path) => path.display().to_string(),
                    usvg::fontdb::Source::SharedFile(path, _) => path.display().to_string(),
                    _ => "<memory>".to_string(),
                };
                eprintln!(
                    "[spike]   face: {} index={} source={}",
                    face.post_script_name, face.index, source_path
                );
                let krilla_font = KrillaFont::new(
                    Data::from(cosmic_font.data().to_vec()),
                    face.index,
                )
                .expect("krilla loads the cosmic-selected font (incl. TTC face index)");
                (krilla_font, face.post_script_name.clone())
            });

            let krilla_glyphs: Vec<KrillaGlyph> = group
                .iter()
                .enumerate()
                .map(|(j, g)| {
                    let pen_x = g.x + g.x_offset;
                    // krilla places glyphs by accumulating advances from the
                    // start point; derive each advance from the *next* glyph's
                    // absolute pen position so cosmic's layout is preserved
                    // exactly (fall back to the hitbox width at group end).
                    let advance = if j + 1 < group.len() {
                        group[j + 1].x + group[j + 1].x_offset - pen_x
                    } else {
                        g.w
                    };
                    KrillaGlyph::new(
                        GlyphId::new(u32::from(g.glyph_id)),
                        // krilla expects advances/offsets normalized (per
                        // em); cosmic gives device px at `font_size`, so
                        // divide by `font_size`, not by units-per-em.
                        advance / font_size,
                        0.0,
                        // cosmic (harfrust) y_offset: positive = up, same
                        // convention as krilla's own naive shaping — pass
                        // through unchanged.
                        g.y_offset / font_size,
                        0.0,
                        g.start..g.end,
                        None,
                    )
                })
                .collect();

            groups.push(GlyphGroup {
                font: font.0.clone(),
                start_x: group[0].x + group[0].x_offset,
                font_size,
                glyphs: krilla_glyphs,
                text: run.text.to_string(),
            });
            i += 1;
        }

        lines.push(ShapedLine {
            groups,
            line_y: run.line_y,
            line_w: run.line_w,
        });
    }

    let font_count = fonts.len();
    for (_, name) in fonts.values() {
        eprintln!("[spike] font used: {name}");
    }
    (lines, font_count)
}

#[test]
fn spike_pdf_api_surface() {
    let out_dir = repo_root().join("target").join("tmp");
    fs::create_dir_all(&out_dir).expect("create target/tmp");

    // --- Capability 2: shape mixed Latin/Han text with cosmic-text ---
    let text = "Hello 世界, PDF!";
    let font_size = 16.0_f32;
    let mut font_system = FontSystem::new();
    let (lines, font_count) = shape_line(&mut font_system, text, font_size);
    assert_eq!(lines.len(), 1, "single layout run expected");
    assert!(
        font_count >= 2,
        "mixed Latin/Han text must use >= 2 fonts via fallback, got {font_count}"
    );
    let latin_font = &lines[0].groups[0].font;
    let han_font = &lines[0].groups[1].font;
    assert_ne!(
        latin_font, han_font,
        "Latin and Han glyph groups must come from different fonts"
    );

    // --- Capability 1: document/page/surface lifecycle (A4) ---
    let mut document = Document::new();
    let mut page =
        document.start_page_with(PageSettings::from_wh(595.3, 841.9).expect("A4 page settings"));
    let mut surface = page.surface();

    let margin = 50.0_f32;
    for line in &lines {
        for group in &line.groups {
            // y is the glyph baseline; krilla's surface origin is the
            // TOP-LEFT corner, y grows downward, units are PDF points.
            surface.draw_glyphs(
                Point::from_xy(margin + group.start_x, margin + line.line_y),
                &group.glyphs,
                group.font.clone(),
                &group.text,
                group.font_size,
                false,
            );
        }
    }

    // --- Capability 6a: path-only SVG (the math renderer's output shape) ---
    let path_svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="60"><path d="M10 50 L60 10 L110 50 Z" fill="rgba(18,52,86,1)" stroke="none"/></svg>"##;
    let path_tree =
        usvg::Tree::from_str(path_svg, &usvg::Options::default()).expect("parse path-only SVG");
    surface.push_transform(&Transform::from_translate(margin, 120.0));
    let drawn = surface.draw_svg(
        &path_tree,
        Size::from_wh(120.0, 60.0).unwrap(),
        SvgSettings::default(),
    );
    surface.pop();
    assert!(drawn.is_some(), "path-only SVG draws without any fontdb");

    surface.finish();

    // --- Capability 3: link annotation over the text rectangle ---
    let link = LinkAnnotation::new(
        Rect::from_xywh(margin, margin + lines[0].line_y - font_size, lines[0].line_w, font_size * 1.4)
            .expect("link rect"),
        Target::Action(Action::Link(LinkAction::new(
            "https://example.com".to_string(),
        ))),
    );
    page.add_annotation(Annotation::new_link(link, None));
    page.finish();

    // --- Capability 4: nested outline/bookmarks ---
    let mut chapter = OutlineNode::new(
        "Spike chapter".to_string(),
        XyzDestination::new(0, Point::from_xy(0.0, 0.0)),
    );
    chapter.push_child(OutlineNode::new(
        "Spike section".to_string(),
        XyzDestination::new(0, Point::from_xy(0.0, 400.0)),
    ));
    let mut outline = Outline::new();
    outline.push_child(chapter);
    document.set_outline(outline);

    // --- Capability 5: metadata ---
    document.set_metadata(
        Metadata::new()
            .title("Markion PDF API spike".to_string())
            .authors(vec!["markion-pdf".to_string()])
            .creation_date(DateTime::new(2026).month(8).day(22)),
    );

    let pdf = document.finish().expect("krilla serializes the document");
    assert!(pdf.starts_with(b"%PDF"), "output is a PDF");
    assert!(
        pdf.len() > 4_000,
        "PDF with subsetted fonts should be non-trivial, got {} bytes",
        pdf.len()
    );
    let out_path = out_dir.join("markion-pdf-spike.pdf");
    fs::write(&out_path, &pdf).expect("write spike PDF");
    eprintln!(
        "[spike] wrote {} ({} bytes, {} font(s) embedded)",
        out_path.display(),
        pdf.len(),
        font_count
    );
}

/// Capability 6b: an SVG containing a `<text>` element. usvg lays out text at
/// parse time against `Options::fontdb`, which is EMPTY by default — so the
/// SVG needs a fontdb, and krilla-svg silently skips glyphs whose font is
/// missing. `SvgSettings::embed_text` chooses embedded subsetted text (true,
/// the default) vs. outlined paths (false).
#[test]
fn spike_svg_with_text_element() {
    let out_dir = repo_root().join("target").join("tmp");
    fs::create_dir_all(&out_dir).expect("create target/tmp");

    let text_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="40"><text x="10" y="28" font-family="Georgia" font-size="20">Hi 世界</text></svg>"#;

    // Variant b1: empty default fontdb → the text node resolves no font.
    let tree_no_fonts = usvg::Tree::from_str(text_svg, &usvg::Options::default())
        .expect("parse SVG with text, empty fontdb");
    let spans_no_fonts = count_text_spans(&tree_no_fonts);

    // Variant b2: fontdb with system fonts → text is laid out and drawable.
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    let options = usvg::Options {
        fontdb: Arc::new(fontdb),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(text_svg, &options).expect("parse SVG with text");
    let spans_with_fonts = count_text_spans(&tree);
    eprintln!("[spike] text spans laid out: empty fontdb = {spans_no_fonts}, system fontdb = {spans_with_fonts}");
    assert!(
        spans_with_fonts > spans_no_fonts,
        "<text> only lays out when Options::fontdb has fonts"
    );

    let mut document = Document::new();
    let mut page =
        document.start_page_with(PageSettings::from_wh(595.3, 841.9).expect("A4 page settings"));
    let mut surface = page.surface();
    surface.push_transform(&Transform::from_translate(50.0, 50.0));
    let drawn = surface.draw_svg(
        &tree,
        Size::from_wh(200.0, 40.0).unwrap(),
        SvgSettings {
            embed_text: true,
            ..Default::default()
        },
    );
    surface.pop();
    assert!(drawn.is_some(), "SVG with <text> draws when fonts exist");
    surface.finish();
    page.finish();

    let pdf = document.finish().expect("krilla serializes the document");
    assert!(pdf.starts_with(b"%PDF"));
    fs::write(out_dir.join("markion-pdf-spike-svg-text.pdf"), &pdf).expect("write spike PDF");
    eprintln!("[spike] SVG <text> variant: {} bytes", pdf.len());
}

fn count_text_spans(tree: &usvg::Tree) -> usize {
    fn walk(node: &usvg::Node, count: &mut usize) {
        if let usvg::Node::Text(text) = node {
            *count += text.layouted().len();
        }
        if let usvg::Node::Group(group) = node {
            for child in group.children() {
                walk(child, count);
            }
        }
    }
    let mut count = 0;
    for child in tree.root().children() {
        walk(child, &mut count);
    }
    count
}
