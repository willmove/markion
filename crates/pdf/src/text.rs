//! Text layout (task 2.4): IR styled runs → per-paragraph cosmic-text
//! `Buffer`s → krilla-ready glyph groups. Wrapping (UAX#14, CJK-aware) and
//! per-glyph font fallback come from cosmic-text; this module only maps
//! styles to attrs and normalizes the shaped output for emission.

use std::collections::HashMap;

use cosmic_text::fontdb;
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight};
use krilla::Data;
use krilla::text::{Font as KrillaFont, Glyph, GlyphId, KrillaGlyph};

use crate::ir::{ImageData, InlineImage, Rgb, Run, Style};
use crate::{PdfError, theme};

/// Points per CSS pixel for inline-image natural sizing (96 DPI).
const PX_TO_PT: f32 = 72.0 / 96.0;
/// Unbreakable placeholder occupying an inline image's advance during wrap.
/// A Latin letter is used because every bundled face has it; Unicode spaces
/// can measure as zero-width when the face has no EM/NBSP glyph.
const PLACEHOLDER: &str = "M";

/// Which family stack a paragraph uses (design D3: body Latin is the bundled
/// serif; heading/code generics resolve to system faces when present).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilyKind {
    /// Serif body text (bundled Libertinus Serif, not the host `serif` generic).
    Body,
    /// Sans-serif headings.
    Heading,
    /// Monospace code.
    Code,
}

impl FamilyKind {
    fn family(self) -> Family<'static> {
        match self {
            Self::Body => crate::fonts::body_family(),
            Self::Heading => Family::SansSerif,
            Self::Code => Family::Monospace,
        }
    }
}

/// Caches fontdb face → krilla font conversions (whole file data plus the
/// TTC face index, per the API spike: the face index comes from fontdb, not
/// from cosmic's font id).
#[derive(Default)]
pub struct FontCache {
    fonts: HashMap<fontdb::ID, KrillaFont>,
}

impl FontCache {
    fn get(
        &mut self,
        fs: &mut FontSystem,
        id: fontdb::ID,
        weight: fontdb::Weight,
    ) -> Result<KrillaFont, PdfError> {
        if let Some(font) = self.fonts.get(&id) {
            return Ok(font.clone());
        }
        let face = fs.db().face(id).ok_or_else(|| {
            PdfError::Fonts("cosmic-text used a face fontdb does not know".into())
        })?;
        let face_index = face.index;
        let cosmic_font = fs.get_font(id, weight).ok_or_else(|| {
            PdfError::Fonts(format!(
                "cosmic-text resolved face {face_index} but cannot load it"
            ))
        })?;
        let font = KrillaFont::new(Data::from(cosmic_font.data().to_vec()), face_index)
            .ok_or_else(|| PdfError::Fonts(format!("krilla rejects face {face_index}")))?;
        self.fonts.insert(id, font.clone());
        Ok(font)
    }
}

/// Resolved per-run information shared by shaping and emission.
#[derive(Debug, Clone)]
pub struct RunInfo {
    pub style: Style,
    pub link: Option<String>,
    pub footnote: Option<u32>,
    /// Effective text fill (default color, link color, or syntax color).
    pub fill: Rgb,
}

/// Consecutive glyphs sharing one font, size, and run — one krilla draw
/// call. Advances are normalized per em and super/subscript baseline shifts
/// are baked into `y_offset`.
#[derive(Clone)]
pub struct GlyphGroup {
    pub font: KrillaFont,
    pub font_size: f32,
    /// Pen x of the first glyph, relative to the line origin.
    pub start_x: f32,
    pub glyphs: Vec<KrillaGlyph>,
    /// The full line text; glyph `text_range`s index into it.
    pub text: String,
    pub fill: Rgb,
    /// Index into [`ShapedParagraph::runs`].
    pub run: usize,
}

impl GlyphGroup {
    /// Width covered by the group's advances at its font size.
    pub fn width(&self) -> f32 {
        self.glyphs
            .iter()
            .map(|g| g.x_advance(self.font_size))
            .sum()
    }
}

/// One laid-out line of a paragraph.
pub struct ShapedLine {
    pub groups: Vec<GlyphGroup>,
    /// Inline images on this line, x relative to the line origin.
    pub objects: Vec<ShapedInlineObject>,
    /// Line height in points.
    pub height: f32,
    /// Distance from the line's top to the text/math baseline.
    pub baseline_offset: f32,
    /// Visual width in points.
    pub width: f32,
    /// Footnote ids referenced by runs on this line (deduplicated, in order).
    pub footnotes: Vec<u32>,
}

/// A placed inline image (math SVG) after wrapping.
pub struct ShapedInlineObject {
    pub run: usize,
    pub x: f32,
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
    pub tree: Box<usvg::Tree>,
}

/// A fully shaped paragraph: lines plus the run table they reference.
pub struct ShapedParagraph {
    pub lines: Vec<ShapedLine>,
    pub runs: Vec<RunInfo>,
}

impl ShapedParagraph {
    /// Total height when the paragraph is placed unsplit.
    pub fn height(&self) -> f32 {
        self.lines.iter().map(|l| l.height).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// What to shape.
pub struct ParagraphSpec<'a> {
    pub runs: &'a [Run],
    /// Base font size in points.
    pub size: f32,
    /// Base line height in points.
    pub line_height: f32,
    pub family: FamilyKind,
    /// Default text color for runs without an explicit one.
    pub color: Rgb,
    /// Wrap width in points; `None` disables wrapping (code lines).
    pub width: Option<f32>,
}

/// Script class for CJK–Latin boundary spacing.
#[derive(PartialEq, Clone, Copy)]
enum ScriptClass {
    Han,
    Latin,
    Other,
}

fn classify(c: char) -> ScriptClass {
    if ('\u{4e00}'..='\u{9fff}').contains(&c)
        || ('\u{3400}'..='\u{4dbf}').contains(&c)
        || ('\u{f900}'..='\u{faff}').contains(&c)
    {
        ScriptClass::Han
    } else if c.is_ascii_alphanumeric() {
        ScriptClass::Latin
    } else {
        ScriptClass::Other
    }
}

struct PreparedObject {
    width: f32,
    height: f32,
    ascent: f32,
    tree: Box<usvg::Tree>,
}

fn image_size_pt(image: &InlineImage, max_width: Option<f32>) -> (f32, f32, f32) {
    let mut width = image.width_px * PX_TO_PT;
    let mut height = image.height_px * PX_TO_PT;
    let mut ascent = image.ascent_px * PX_TO_PT;
    if let Some(max_width) = max_width
        && width > max_width
        && width > 0.0
    {
        let scale = max_width / width;
        width *= scale;
        height *= scale;
        ascent *= scale;
    }
    (width.max(0.5), height, ascent)
}

fn parse_inline_svg(image: &InlineImage, max_width: Option<f32>) -> Option<PreparedObject> {
    let ImageData::Svg(svg) = &image.data else {
        return None;
    };
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).ok()?;
    let (width, height, ascent) = image_size_pt(image, max_width);
    Some(PreparedObject {
        width,
        height,
        ascent,
        tree: Box::new(tree),
    })
}

/// Shape one paragraph of styled runs.
pub fn shape_paragraph(
    fs: &mut FontSystem,
    cache: &mut FontCache,
    spec: &ParagraphSpec<'_>,
) -> Result<ShapedParagraph, PdfError> {
    let script_size = spec.size * 0.75;
    let mut texts: Vec<String> = Vec::with_capacity(spec.runs.len());
    let mut run_infos = Vec::with_capacity(spec.runs.len());
    let mut objects: Vec<Option<PreparedObject>> = Vec::with_capacity(spec.runs.len());
    let mut placeholder_em: Vec<Option<f32>> = Vec::with_capacity(spec.runs.len());

    for run in spec.runs {
        let fill = run.style.color.unwrap_or(if run.link.is_some() {
            theme::LINK
        } else {
            spec.color
        });
        let prepared = run
            .inline_image
            .as_ref()
            .and_then(|image| parse_inline_svg(image, spec.width));
        // A footnote reference renders as a superscript number (the 1-based
        // footnote id); the run's own text is replaced per the IR contract.
        let text = if prepared.is_some() {
            format!("\u{200B}{PLACEHOLDER}")
        } else if let Some(id) = run.footnote {
            id.to_string()
        } else if let Some(image) = &run.inline_image {
            image.alt.clone()
        } else {
            run.text.clone()
        };
        let em = prepared.as_ref().map(|object| object.width);
        texts.push(text);
        placeholder_em.push(em);
        objects.push(prepared);
        run_infos.push(RunInfo {
            style: run.style,
            link: run.link.clone(),
            footnote: run.footnote,
            fill,
        });
    }

    if texts.iter().all(|t| t.is_empty()) {
        return Ok(ShapedParagraph {
            lines: Vec::new(),
            runs: run_infos,
        });
    }

    let default_attrs = Attrs::new().family(spec.family.family());
    let mut buffer = Buffer::new(fs, Metrics::new(spec.size, spec.line_height));
    for pass in 0..3 {
        let spans: Vec<(&str, Attrs)> = spec
            .runs
            .iter()
            .zip(&texts)
            .enumerate()
            .map(|(i, (run, text))| {
                let style = &run.style;
                let family = if style.code {
                    Family::Monospace
                } else {
                    spec.family.family()
                };
                let mut attrs = default_attrs.clone().family(family).metadata(i).color(
                    cosmic_text::Color::rgb(
                        run_infos[i].fill.0,
                        run_infos[i].fill.1,
                        run_infos[i].fill.2,
                    ),
                );
                if style.bold {
                    attrs = attrs.weight(Weight::BOLD);
                }
                if style.italic {
                    attrs = attrs.style(cosmic_text::Style::Italic);
                }
                if let Some(em) = placeholder_em[i] {
                    let line_h = spec.line_height.max(
                        objects[i]
                            .as_ref()
                            .map(|object| object.height)
                            .unwrap_or(spec.line_height),
                    );
                    attrs.metrics_opt = Some(Metrics::new(em, line_h).into());
                } else if style.superscript || style.subscript || run.footnote.is_some() {
                    // Super/subscript and footnote references shape at a smaller
                    // size; the baseline offset is applied at emission time.
                    attrs.metrics_opt = Some(Metrics::new(script_size, spec.line_height).into());
                }
                (text.as_str(), attrs)
            })
            .collect();
        buffer.set_size(spec.width, None);
        buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(fs, true);
        if pass == 2 {
            break;
        }
        let mut changed = false;
        for (i, em) in placeholder_em.iter_mut().enumerate() {
            let Some(desired) = objects[i].as_ref().map(|object| object.width) else {
                continue;
            };
            let measured = placeholder_advance(&buffer, i);
            if measured <= 0.05 {
                continue;
            }
            let current = em.unwrap_or(desired);
            let new_em = current * desired / measured;
            if (new_em - current).abs() / current.max(0.5) > 0.02 {
                *em = Some(new_em);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut lines = Vec::new();
    for run in buffer.layout_runs() {
        let glyphs = run.glyphs;
        if glyphs.is_empty() {
            continue;
        }

        let mut groups: Vec<GlyphGroup> = Vec::new();
        let mut i = 0;
        while i < glyphs.len() {
            // Group consecutive glyphs sharing font, size, and source run.
            let first = i;
            let key = |g: &cosmic_text::LayoutGlyph| (g.font_id, g.font_size.to_bits(), g.metadata);
            let group_key = key(&glyphs[i]);
            while i + 1 < glyphs.len() && key(&glyphs[i + 1]) == group_key {
                i += 1;
            }
            let group = &glyphs[first..=i];
            let font_size = group[0].font_size;
            let info = &run_infos[group[0].metadata];
            let font = cache.get(fs, group[0].font_id, group[0].font_weight)?;

            let shift = if info.footnote.is_some() || info.style.superscript {
                spec.size * 0.33
            } else if info.style.subscript {
                -spec.size * 0.25
            } else {
                0.0
            };

            let krilla_glyphs: Vec<KrillaGlyph> = group
                .iter()
                .enumerate()
                .map(|(j, g)| {
                    let pen_x = g.x + g.x_offset;
                    // Derive the advance from the next glyph's absolute pen
                    // position so cosmic's layout is preserved exactly (per
                    // the spike); fall back to the hitbox width at group end.
                    let advance = if j + 1 < group.len() {
                        group[j + 1].x + group[j + 1].x_offset - pen_x
                    } else {
                        g.w
                    };
                    KrillaGlyph::new(
                        GlyphId::new(u32::from(g.glyph_id)),
                        // krilla expects advances/offsets normalized per em;
                        // cosmic gives points at `font_size`.
                        advance / font_size,
                        0.0,
                        // cosmic y_offset: positive = up; add the
                        // super/subscript shift before normalizing.
                        (g.y_offset + shift) / font_size,
                        0.0,
                        g.start..g.end,
                        None,
                    )
                })
                .collect();

            groups.push(GlyphGroup {
                font,
                font_size,
                start_x: group[0].x + group[0].x_offset,
                glyphs: krilla_glyphs,
                text: run.text.to_string(),
                fill: info.fill,
                run: group[0].metadata,
            });
            i += 1;
        }

        // CJK–Latin boundary spacing: insert a thin gap where a Han group
        // abuts a Latin group by shifting subsequent groups right.
        let mut shift = 0.0_f32;
        let pad = spec.size * 0.2;
        for k in 0..groups.len() {
            groups[k].start_x += shift;
            if k + 1 < groups.len() {
                let last_char = groups[k]
                    .glyphs
                    .last()
                    .and_then(|g| groups[k].text.get(g.text_range()))
                    .and_then(|s| s.chars().last());
                let next_char = groups[k + 1]
                    .glyphs
                    .first()
                    .and_then(|g| groups[k + 1].text.get(g.text_range()))
                    .and_then(|s| s.chars().next());
                if let (Some(a), Some(b)) = (last_char, next_char) {
                    let pair = (classify(a), classify(b));
                    if pair == (ScriptClass::Han, ScriptClass::Latin)
                        || pair == (ScriptClass::Latin, ScriptClass::Han)
                    {
                        shift += pad;
                    }
                }
            }
        }

        let mut footnotes: Vec<u32> = Vec::new();
        for g in &groups {
            if let Some(id) = run_infos[g.run].footnote
                && !footnotes.contains(&id)
            {
                footnotes.push(id);
            }
        }

        let mut line_objects = Vec::new();
        let mut drawn = Vec::new();
        for group in groups {
            if let Some(prepared) = objects.get(group.run).and_then(|object| object.as_ref()) {
                // The image run is "\u{200B}M": UAX#14 may leave the ZWSP on
                // the previous line. That group has ~zero advance and must
                // not duplicate the atom.
                if group.width() > 0.05 {
                    line_objects.push(ShapedInlineObject {
                        run: group.run,
                        x: group.start_x,
                        width: prepared.width,
                        height: prepared.height,
                        ascent: prepared.ascent,
                        tree: prepared.tree.clone(),
                    });
                }
            } else {
                drawn.push(group);
            }
        }

        let (height, baseline_offset) = if line_objects.is_empty() {
            (run.line_height, run.line_height * 0.8)
        } else {
            let text_ascent = run.line_height * 0.8;
            let text_descent = run.line_height * 0.2;
            let mut ascent = text_ascent;
            let mut descent = text_descent;
            for object in &line_objects {
                ascent = ascent.max(object.ascent);
                descent = descent.max((object.height - object.ascent).max(0.0));
            }
            (ascent + descent, ascent)
        };

        lines.push(ShapedLine {
            groups: drawn,
            objects: line_objects,
            height,
            baseline_offset,
            width: run.line_w + shift,
            footnotes,
        });
    }

    Ok(ShapedParagraph {
        lines,
        runs: run_infos,
    })
}

fn placeholder_advance(buffer: &Buffer, run_idx: usize) -> f32 {
    for layout in buffer.layout_runs() {
        let mut width = 0.0;
        let mut found = false;
        for glyph in layout.glyphs.iter() {
            if glyph.metadata == run_idx {
                found = true;
                width += glyph.w;
            }
        }
        if found {
            return width;
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::bundled_only_font_system;

    fn spec<'a>(runs: &'a [Run], width: Option<f32>) -> ParagraphSpec<'a> {
        ParagraphSpec {
            runs,
            size: theme::BODY_SIZE,
            line_height: theme::BODY_SIZE * theme::LINE_HEIGHT_MULT,
            family: FamilyKind::Body,
            color: theme::TEXT,
            width,
        }
    }

    /// A spaceless Chinese paragraph must wrap within the line width
    /// (UAX#14 allows breaks between Han characters).
    #[test]
    fn spaceless_chinese_wraps_within_line_width() {
        let mut fs = bundled_only_font_system();
        let mut cache = FontCache::default();
        let text: String = "汉".repeat(100);
        let runs = vec![Run {
            text,
            ..Run::default()
        }];
        let width = 120.0;
        let para = shape_paragraph(&mut fs, &mut cache, &spec(&runs, Some(width)))
            .expect("shape Chinese paragraph");
        assert!(para.lines.len() > 1, "100 Han chars must wrap at 120pt");
        for line in &para.lines {
            assert!(
                line.width <= width + 1.0,
                "line width {} exceeds wrap width {width}",
                line.width
            );
        }
    }

    /// Mixed CJK/Latin runs shape without panicking and switch fonts per
    /// glyph group (cosmic automatic fallback).
    #[test]
    fn mixed_cjk_latin_uses_per_glyph_font_fallback() {
        let mut fs = bundled_only_font_system();
        let mut cache = FontCache::default();
        let runs = vec![
            Run {
                text: "Hello ".to_string(),
                ..Run::default()
            },
            Run {
                text: "世界".to_string(),
                style: Style {
                    bold: true,
                    ..Style::default()
                },
                ..Run::default()
            },
            Run {
                text: " PDF".to_string(),
                ..Run::default()
            },
        ];
        let para = shape_paragraph(&mut fs, &mut cache, &spec(&runs, Some(300.0)))
            .expect("shape mixed paragraph");
        assert_eq!(para.lines.len(), 1);
        let fonts: Vec<&KrillaFont> = para.lines[0].groups.iter().map(|g| &g.font).collect();
        let distinct: std::collections::HashSet<_> = fonts.iter().collect();
        assert!(
            distinct.len() >= 2,
            "mixed CJK/Latin text must switch fonts, got {} group(s)",
            fonts.len()
        );
    }

    /// A footnote reference run is replaced by its superscript number.
    #[test]
    fn footnote_reference_becomes_superscript_number() {
        let mut fs = bundled_only_font_system();
        let mut cache = FontCache::default();
        let runs = vec![
            Run {
                text: "text".to_string(),
                ..Run::default()
            },
            Run {
                text: "[^ignored]".to_string(),
                footnote: Some(2),
                ..Run::default()
            },
        ];
        let para = shape_paragraph(&mut fs, &mut cache, &spec(&runs, Some(300.0)))
            .expect("shape footnote ref");
        assert_eq!(para.lines[0].footnotes, vec![2]);
        let ref_group = para.lines[0]
            .groups
            .iter()
            .find(|g| g.run == 1)
            .expect("reference group exists");
        let rendered: String = ref_group
            .glyphs
            .iter()
            .filter_map(|g| ref_group.text.get(g.text_range()))
            .collect();
        assert_eq!(rendered, "2", "reference text replaced by the footnote id");
        assert!(
            ref_group.font_size < theme::BODY_SIZE,
            "reference shapes at the smaller script size"
        );
    }

    const TINY_MATH_SVG: &str = concat!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="12" viewBox="0 0 80 12">"##,
        r##"<path d="M1 11 L79 1" stroke="#000" fill="none" stroke-width="1.5"/></svg>"##
    );

    fn inline_svg_run(width_px: f32, height_px: f32, ascent_px: f32) -> Run {
        Run {
            inline_image: Some(InlineImage {
                data: ImageData::Svg(TINY_MATH_SVG.to_string()),
                width_px,
                height_px,
                ascent_px,
                alt: "$x$".to_string(),
            }),
            ..Run::default()
        }
    }

    #[test]
    fn inline_image_sits_on_the_same_line_as_surrounding_text() {
        let mut fs = bundled_only_font_system();
        let mut cache = FontCache::default();
        let runs = vec![
            Run {
                text: "before ".to_string(),
                ..Run::default()
            },
            inline_svg_run(24.0, 12.0, 9.0),
            Run {
                text: " after".to_string(),
                ..Run::default()
            },
        ];
        let para = shape_paragraph(&mut fs, &mut cache, &spec(&runs, Some(300.0)))
            .expect("shape inline image");
        assert_eq!(para.lines.len(), 1);
        assert_eq!(para.lines[0].objects.len(), 1);
        assert!(
            para.lines[0].objects[0].x > 0.0,
            "inline image should follow the leading text"
        );
        assert!(
            para.lines[0].groups.iter().all(|g| g.run != 1),
            "placeholder glyphs must not be emitted"
        );
    }

    #[test]
    fn inline_image_wraps_as_an_atom() {
        let mut fs = bundled_only_font_system();
        let mut cache = FontCache::default();
        // "MMMMMMMM" at body size is wider than 50pt; a 60pt-wide atom cannot
        // stay on the same line if its placeholder advance is calibrated.
        let runs = vec![
            Run {
                text: "MMMMMMMM".to_string(),
                ..Run::default()
            },
            inline_svg_run(80.0, 12.0, 9.0),
            Run {
                text: " z".to_string(),
                ..Run::default()
            },
        ];
        let para = shape_paragraph(&mut fs, &mut cache, &spec(&runs, Some(80.0)))
            .expect("shape wrapping inline image");
        assert!(
            para.lines.len() >= 2,
            "wide inline image must wrap as a unit, got {} line(s)",
            para.lines.len()
        );
        let object_lines = para
            .lines
            .iter()
            .filter(|line| !line.objects.is_empty())
            .count();
        assert_eq!(object_lines, 1, "the atom must occupy exactly one line");
        assert_eq!(
            para.lines
                .iter()
                .map(|line| line.objects.len())
                .sum::<usize>(),
            1
        );
    }
}
