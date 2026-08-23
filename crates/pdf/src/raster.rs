//! Raster snapshot export — renders the layout IR to a single RGBA bitmap.
//!
//! This is the built-in PNG/JPEG exporter. It reuses the same layout IR
//! ([`crate::ir`]) and the same cosmic-text font pipeline as the PDF writer,
//! so a PNG/JPEG snapshot matches the PDF typography (headings, paragraphs,
//! lists, quotes, alerts, code, tables, rules, and embedded images) and —
//! critically — renders CJK text with the bundled Noto Sans SC subset instead
//! of an ASCII-only bitmap font. There is no pagination: the document flows
//! into one continuous, tall canvas, which is what an "export as image"
//! snapshot is expected to look like.
//!
//! Shapes are laid out at `scale` pixels-per-point (cosmic-text sizes are
//! multiplied by `scale`), then rasterized at `scale = 1.0` with cosmic-text's
//! own baseline metrics, so the output is crisp at any resolution.

use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Style as CosmicStyle, SwashCache, Weight,
};
use image::RgbaImage;

use crate::ir::{AlertKind, Alignment, Block, ImageData, ListMarker, PdfDocument, Rgb, Run, Style};
use crate::{fonts, theme, PdfError};

/// Default pixels-per-point for a snapshot (≈ 144 DPI at an 11 pt body).
pub const DEFAULT_SCALE: f32 = 2.0;

/// Render a whole document to an RGBA bitmap.
///
/// `scale` is pixels per point; the document page geometry (and every font
/// size) is multiplied by it. Text is shaped with the process-wide font
/// system (system fonts plus the bundled Noto Sans SC / Libertinus / DejaVu
/// fallbacks), so CJK and Latin both resolve to real glyphs.
pub fn render_snapshot(doc: &PdfDocument, scale: f32) -> Result<RgbaImage, PdfError> {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        DEFAULT_SCALE
    };
    fonts::with_font_system(|fs| render_document(fs, doc, scale))
}

// --- Geometry -------------------------------------------------------------

/// Page geometry in pixels.
#[derive(Clone, Copy)]
struct Geometry {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

const MM_TO_PT: f32 = 72.0 / 25.4;

impl Geometry {
    fn from_options(options: &crate::ir::PdfOptions, scale: f32) -> Self {
        let page_w = options.page_width_mm * MM_TO_PT * scale;
        let page_h = options.page_height_mm * MM_TO_PT * scale;
        let margin = options.margin_mm * MM_TO_PT * scale;
        let left = margin;
        Geometry {
            left,
            top: margin,
            right: (page_w - margin).max(left + 1.0),
            bottom: page_h - margin,
        }
    }

    fn content_w(&self) -> f32 {
        (self.right - self.left).max(1.0)
    }
}

// --- Canvas ---------------------------------------------------------------

/// A growable, opaque RGBA canvas. Text and images are alpha-blended on top
/// of the opaque background.
struct Canvas {
    width: u32,
    height: u32,
    data: Vec<u8>, // RGBA, width * height * 4
}

const BG: Rgb = Rgb(0xff, 0xff, 0xff);

impl Canvas {
    fn new(width: u32) -> Self {
        let mut canvas = Canvas {
            width,
            height: 0,
            data: Vec::new(),
        };
        canvas.grow(512);
        canvas
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
            Some((y as usize * self.width as usize + x as usize) * 4)
        } else {
            None
        }
    }

    /// Ensure the canvas is at least `min_height`, extending with background.
    fn grow(&mut self, min_height: u32) {
        if min_height <= self.height {
            return;
        }
        let row = self.width as usize * 4;
        let added = (min_height - self.height) as usize;
        self.data.reserve(added * row);
        for _ in 0..(added * self.width as usize) {
            self.data.extend_from_slice(&[BG.0, BG.1, BG.2, 0xff]);
        }
        self.height = min_height;
    }

    /// Opaque fill of a rectangle, clamped to the canvas.
    fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
        let x0 = x.floor().max(0.0) as i32;
        let y0 = y.floor().max(0.0) as i32;
        let x1 = (x + w).ceil().max(0.0) as i32;
        let y1 = (y + h).ceil().max(0.0) as i32;
        for py in y0..y1 {
            for px in x0..x1 {
                if let Some(i) = self.index(px, py) {
                    self.data[i] = color.0;
                    self.data[i + 1] = color.1;
                    self.data[i + 2] = color.2;
                    self.data[i + 3] = 0xff;
                }
            }
        }
    }

    /// Alpha-blend one cosmic-text pixel (over, onto opaque background).
    fn blend(&mut self, x: i32, y: i32, c: cosmic_text::Color) {
        let a = ((c.0 >> 24) & 0xff) as u32;
        if a == 0 {
            return;
        }
        let sr = ((c.0 >> 16) & 0xff) as u32;
        let sg = ((c.0 >> 8) & 0xff) as u32;
        let sb = (c.0 & 0xff) as u32;
        let Some(i) = self.index(x, y) else {
            return;
        };
        let (dr, dg, db) = (self.data[i] as u32, self.data[i + 1] as u32, self.data[i + 2] as u32);
        let mix = |s: u32, d: u32| ((s * a + d * (255 - a) + 127) / 255) as u8;
        self.data[i] = mix(sr, dr);
        self.data[i + 1] = mix(sg, dg);
        self.data[i + 2] = mix(sb, db);
        self.data[i + 3] = 0xff;
    }

    /// Alpha-blend an RGBA image (straight alpha) at the given origin.
    fn draw_rgba(&mut self, x: i32, y: i32, img: &RgbaImage) {
        let (iw, ih) = img.dimensions();
        for py in 0..ih {
            for px in 0..iw {
                let src = img.get_pixel(px, py);
                let a = src.0[3] as u32;
                if a == 0 {
                    continue;
                }
                let Some(i) = self.index(x + px as i32, y + py as i32) else {
                    continue;
                };
                let (dr, dg, db) = (self.data[i] as u32, self.data[i + 1] as u32, self.data[i + 2] as u32);
                let mix = |s: u32, d: u32| ((s * a + d * (255 - a) + 127) / 255) as u8;
                self.data[i] = mix(src.0[0] as u32, dr);
                self.data[i + 1] = mix(src.0[1] as u32, dg);
                self.data[i + 2] = mix(src.0[2] as u32, db);
                self.data[i + 3] = 0xff;
            }
        }
    }

    /// Composite a premultiplied-RGBA image (tiny_skia/SVG output) at origin.
    fn draw_premul(&mut self, x: i32, y: i32, data: &[u8], w: u32, h: u32) {
        for py in 0..h {
            for px in 0..w {
                let idx = (py * w + px) as usize * 4;
                let a = data[idx + 3] as u32;
                if a == 0 {
                    continue;
                }
                let Some(i) = self.index(x + px as i32, y + py as i32) else {
                    continue;
                };
                let inv = 255 - a;
                let mix = |s: u32, d: u32| ((s * 255 + d * inv + 127) / 255) as u8;
                // src is premultiplied by alpha already.
                self.data[i] = mix(data[idx] as u32, self.data[i] as u32);
                self.data[i + 1] = mix(data[idx + 1] as u32, self.data[i + 1] as u32);
                self.data[i + 2] = mix(data[idx + 2] as u32, self.data[i + 2] as u32);
                self.data[i + 3] = 0xff;
            }
        }
    }

    fn into_rgba(self) -> RgbaImage {
        RgbaImage::from_raw(self.width, self.height, self.data)
            .expect("canvas dimensions match data length")
    }
}

// --- Text shaping / drawing ----------------------------------------------

/// Shape a run of styled runs into a cosmic-text buffer at pixel sizes.
fn shape_buffer(
    fs: &mut FontSystem,
    runs: &[Run],
    size: f32,
    line_height: f32,
    family: Family,
    default_color: Rgb,
    wrap_w: Option<f32>,
) -> Buffer {
    let default_attrs = Attrs::new().family(family);
    let spans: Vec<(&str, Attrs)> = runs
        .iter()
        .enumerate()
        .map(|(i, run)| {
            let fill = run.style.color.unwrap_or(if run.link.is_some() {
                theme::LINK
            } else {
                default_color
            });
            let run_family = if run.style.code { Family::Monospace } else { family };
            let mut attrs = default_attrs
                .clone()
                .family(run_family)
                .metadata(i)
                .color(cosmic_text::Color::rgb(fill.0, fill.1, fill.2));
            if run.style.bold {
                attrs = attrs.weight(Weight::BOLD);
            }
            if run.style.italic {
                attrs = attrs.style(CosmicStyle::Italic);
            }
            // Footnotes and super/subscripts shape smaller; cosmic-text folds
            // the baseline shift into the glyph y_offset.
            if run.style.superscript || run.style.subscript || run.footnote.is_some() {
                attrs = attrs.metrics(Metrics::new(size * 0.75, line_height));
            }
            (run.text.as_str(), attrs)
        })
        .collect();

    let mut buffer = Buffer::new(fs, Metrics::new(size, line_height));
    buffer.set_size(wrap_w, None);
    buffer.set_rich_text(spans, &default_attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(fs, true);
    buffer
}

/// Total pixel height a shaped buffer occupies: the sum of its line advances.
fn buffer_height(buffer: &Buffer) -> f32 {
    buffer.layout_runs().map(|r| r.line_height).sum()
}

/// Blit the glyphs of one shaped layout run at screen origin (x, top), where
/// `run.line_y` already carries the per-line baseline. `clip_right` truncates
/// pixels past it (code lines that overflow the text column).
#[allow(clippy::too_many_arguments)]
fn blit_run(
    fs: &mut FontSystem,
    cache: &mut SwashCache,
    canvas: &mut Canvas,
    run: &cosmic_text::LayoutRun<'_>,
    x: f32,
    top: f32,
    clip_right: Option<f32>,
    default_color: Rgb,
) {
    for glyph in run.glyphs {
        let color = glyph.color_opt.unwrap_or(cosmic_text::Color::rgb(
            default_color.0,
            default_color.1,
            default_color.2,
        ));
        let phys = glyph.physical((0.0, run.line_y), 1.0);
        let ox = x.round() as i32 + phys.x;
        let oy = top.round() as i32 + phys.y;
        cache.with_pixels(fs, phys.cache_key, color, |dx, dy, pc| {
            let sx = ox + dx;
            let sy = oy + dy;
            if let Some(clip) = clip_right {
                if sx as f32 >= clip {
                    return;
                }
            }
            canvas.blend(sx, sy, pc);
        });
    }
}

/// Draw a whole shaped buffer (with marker-pen highlights) at origin (x, top).
#[allow(clippy::too_many_arguments)]
fn draw_buffer(
    fs: &mut FontSystem,
    cache: &mut SwashCache,
    canvas: &mut Canvas,
    runs: &[Run],
    buffer: &Buffer,
    x: f32,
    top: f32,
    clip_right: Option<f32>,
    default_color: Rgb,
) {
    for run in buffer.layout_runs() {
        // Marker-pen highlight: fill behind the run's glyph span.
        let mut spans: Vec<(usize, f32, f32)> = Vec::new();
        for glyph in run.glyphs {
            let phys = glyph.physical((0.0, run.line_y), 1.0);
            let gx = phys.x as f32;
            let gw = glyph.w;
            if let Some(entry) = spans.iter_mut().find(|e| e.0 == glyph.metadata) {
                entry.1 = entry.1.min(gx);
                entry.2 = entry.2.max(gx + gw);
            } else {
                spans.push((glyph.metadata, gx, gx + gw));
            }
        }
        for (meta, min_x, max_x) in spans {
            if runs.get(meta).map(|r| r.style.highlight) == Some(true) {
                let size = run
                    .glyphs
                    .iter()
                    .find(|g| g.metadata == meta)
                    .map(|g| g.font_size)
                    .unwrap_or(theme::BODY_SIZE);
                let baseline = top + run.line_y;
                canvas.fill_rect(
                    x + min_x,
                    baseline - size * 0.75,
                    (max_x - min_x).max(1.0),
                    size * 1.15,
                    theme::HIGHLIGHT,
                );
            }
        }
        blit_run(fs, cache, canvas, &run, x, top, clip_right, default_color);
    }
}

/// Draw one table cell buffer with horizontal alignment.
#[allow(clippy::too_many_arguments)]
fn draw_cell(
    fs: &mut FontSystem,
    cache: &mut SwashCache,
    canvas: &mut Canvas,
    buffer: &Buffer,
    cell_x: f32,
    cell_w: f32,
    align: Alignment,
    top: f32,
) {
    let mut y = top;
    for run in buffer.layout_runs() {
        let dx = match align {
            Alignment::Left => 0.0,
            Alignment::Center => (cell_w - run.line_w) / 2.0,
            Alignment::Right => cell_w - run.line_w,
        }
        .max(0.0);
        blit_run(fs, cache, canvas, &run, cell_x + dx, y, None, theme::TEXT);
        y += run.line_height;
    }
}

// --- Raster state ---------------------------------------------------------

struct Raster<'a> {
    fs: &'a mut FontSystem,
    cache: SwashCache,
    canvas: Canvas,
    geom: Geometry,
    scale: f32,
    /// Vertical flow cursor (top of the next block), in pixels.
    y: f32,
}

impl<'a> Raster<'a> {
    fn new(fs: &'a mut FontSystem, geom: Geometry, scale: f32) -> Self {
        let canvas = Canvas::new(geom.content_w().round().max(1.0) as u32);
        Raster {
            fs,
            cache: SwashCache::new(),
            canvas,
            geom,
            scale,
            y: geom.top,
        }
    }

    fn ensure(&mut self, h: f32) {
        self.canvas.grow((self.y + h + 4.0).ceil().max(1.0) as u32);
    }

    fn smul(&self, v: f32) -> f32 {
        v * self.scale
    }
}

// --- Block rendering ------------------------------------------------------

fn draw_paragraph(raster: &mut Raster, content: &[Run]) -> Result<(), PdfError> {
    let size = raster.smul(theme::BODY_SIZE);
    let line_h = raster.smul(theme::BODY_SIZE * theme::LINE_HEIGHT_MULT);
    let buffer = shape_buffer(
        raster.fs,
        content,
        size,
        line_h,
        Family::Serif,
        theme::TEXT,
        Some(raster.geom.content_w()),
    );
    let h = buffer_height(&buffer);
    if h <= 0.0 {
        return Ok(());
    }
    raster.ensure(h);
    let top = raster.y;
    draw_buffer(
        raster.fs,
        &mut raster.cache,
        &mut raster.canvas,
        content,
        &buffer,
        raster.geom.left,
        top,
        None,
        theme::TEXT,
    );
    raster.y += h + raster.smul(theme::PARA_SPACING);
    Ok(())
}

fn draw_heading(raster: &mut Raster, level: u8, content: &[Run]) -> Result<(), PdfError> {
    let level = level.clamp(1, 6);
    let size = raster.smul(theme::HEADING_SIZES[(level - 1) as usize]);
    let line_h = size * 1.25;
    let runs: Vec<Run> = content
        .iter()
        .cloned()
        .map(|mut r| {
            r.style.bold = true;
            r
        })
        .collect();
    let buffer = shape_buffer(
        raster.fs,
        &runs,
        size,
        line_h,
        Family::SansSerif,
        theme::TEXT,
        Some(raster.geom.content_w()),
    );
    let h = buffer_height(&buffer);
    if h <= 0.0 {
        return Ok(());
    }
    let before = if raster.y > raster.geom.top { raster.smul(theme::HEADING_BEFORE) } else { 0.0 };
    raster.ensure(h + before);
    raster.y += before;
    let top = raster.y;
    draw_buffer(
        raster.fs,
        &mut raster.cache,
        &mut raster.canvas,
        &runs,
        &buffer,
        raster.geom.left,
        top,
        None,
        theme::TEXT,
    );
    raster.y += h + raster.smul(theme::HEADING_AFTER);
    Ok(())
}

fn draw_list_item(
    raster: &mut Raster,
    indent_level: u8,
    marker: ListMarker,
    content: &[Run],
) -> Result<(), PdfError> {
    let indent_x = raster.geom.left + indent_level as f32 * raster.smul(theme::LIST_INDENT);
    let content_x = indent_x + raster.smul(theme::MARKER_WIDTH);
    let width = (raster.geom.right - content_x).max(20.0 * raster.scale);
    let size = raster.smul(theme::BODY_SIZE);
    let line_h = raster.smul(theme::BODY_SIZE * theme::LINE_HEIGHT_MULT);
    let buffer = shape_buffer(
        raster.fs,
        content,
        size,
        line_h,
        Family::Serif,
        theme::TEXT,
        Some(width),
    );
    let h = buffer_height(&buffer);
    let marker_text = match marker {
        ListMarker::Bullet => "•".to_string(),
        ListMarker::Number(n) => format!("{n}."),
        ListMarker::Task { checked } => {
            if checked {
                "[x]".to_string()
            } else {
                "[ ]".to_string()
            }
        }
    };
    let marker_run = Run {
        text: marker_text,
        style: Style {
            color: Some(theme::MUTED),
            ..Style::default()
        },
        ..Run::default()
    };
    let marker_buffer = shape_buffer(
        raster.fs,
        std::slice::from_ref(&marker_run),
        size,
        line_h,
        Family::Serif,
        theme::MUTED,
        None,
    );
    let marker_h = buffer_height(&marker_buffer).max(1.0);
    let h = h.max(marker_h);
    raster.ensure(h);
    let top = raster.y;
    draw_buffer(
        raster.fs,
        &mut raster.cache,
        &mut raster.canvas,
        std::slice::from_ref(&marker_run),
        &marker_buffer,
        indent_x,
        top,
        None,
        theme::MUTED,
    );
    draw_buffer(
        raster.fs,
        &mut raster.cache,
        &mut raster.canvas,
        content,
        &buffer,
        content_x,
        top,
        None,
        theme::TEXT,
    );
    raster.y += h + raster.smul(theme::PARA_SPACING * 0.5);
    Ok(())
}

fn draw_quote(raster: &mut Raster, children: &[Block]) -> Result<(), PdfError> {
    let start = raster.y;
    let left = raster.geom.left;
    let saved_left = raster.geom.left;
    raster.geom.left += raster.smul(theme::ACCENT_INDENT);
    for child in children {
        draw_block(raster, child)?;
    }
    let end = raster.y;
    raster.geom.left = saved_left;
    raster.canvas.fill_rect(
        left + raster.smul(1.0),
        start,
        raster.smul(2.5),
        (end - start).max(1.0),
        theme::QUOTE_ACCENT,
    );
    Ok(())
}

fn draw_alert(raster: &mut Raster, kind: AlertKind, children: &[Block]) -> Result<(), PdfError> {
    let start = raster.y;
    let left = raster.geom.left;
    let accent = theme::alert_accent(kind);
    let saved_left = raster.geom.left;
    let full_w = raster.geom.content_w();
    raster.geom.left += raster.smul(theme::ACCENT_INDENT);
    let inner_w = raster.geom.content_w();

    // Bold kind label, with a tint behind just the label line.
    let label_run = Run {
        text: theme::alert_label(kind).to_string(),
        style: Style {
            bold: true,
            color: Some(accent),
            ..Style::default()
        },
        ..Run::default()
    };
    let size = raster.smul(theme::BODY_SIZE);
    let line_h = raster.smul(theme::BODY_SIZE * theme::LINE_HEIGHT_MULT);
    let label = shape_buffer(
        raster.fs,
        std::slice::from_ref(&label_run),
        size,
        line_h,
        Family::SansSerif,
        accent,
        Some(inner_w),
    );
    let lh = buffer_height(&label).max(1.0);
    raster.ensure(lh + raster.smul(4.0));
    let label_top = raster.y;
    raster.canvas.fill_rect(
        left,
        label_top - raster.smul(1.0),
        full_w,
        lh + raster.smul(2.0),
        theme::alert_tint(kind),
    );
    draw_buffer(
        raster.fs,
        &mut raster.cache,
        &mut raster.canvas,
        std::slice::from_ref(&label_run),
        &label,
        raster.geom.left,
        label_top,
        None,
        accent,
    );
    raster.y += lh + raster.smul(2.0);

    for child in children {
        draw_block(raster, child)?;
    }
    let end = raster.y;
    raster.geom.left = saved_left;
    raster.canvas.fill_rect(
        left,
        start,
        raster.smul(4.0),
        (end - start).max(1.0),
        accent,
    );
    Ok(())
}

fn draw_code_block(raster: &mut Raster, lines: &[Vec<Run>]) -> Result<(), PdfError> {
    let size = raster.smul(theme::CODE_SIZE);
    let line_h = raster.smul(theme::CODE_SIZE * 1.4);
    let pad = raster.smul(theme::CODE_PAD);
    raster.y += raster.smul(theme::BLOCK_SPACING * 0.5);
    let seg_top = raster.y;

    // Shape all lines first so the background can be drawn behind the text.
    let mut buffers = Vec::with_capacity(lines.len());
    let mut total_h = 0.0f32;
    for line in lines {
        let buffer = shape_buffer(raster.fs, line, size, line_h, Family::Monospace, theme::TEXT, None);
        let h = buffer_height(&buffer);
        total_h += h;
        buffers.push((buffer, h));
    }
    let bg_h = total_h + pad * 2.0;
    raster.ensure(bg_h);
    raster.canvas.fill_rect(
        raster.geom.left,
        seg_top - pad * 0.5,
        raster.geom.content_w(),
        bg_h,
        theme::CODE_BG,
    );
    for (buffer, h) in buffers {
        draw_buffer(
            raster.fs,
            &mut raster.cache,
            &mut raster.canvas,
            &[],
            &buffer,
            raster.geom.left + pad,
            raster.y,
            Some(raster.geom.right),
            theme::TEXT,
        );
        raster.y += h;
    }
    raster.y += pad + raster.smul(theme::BLOCK_SPACING * 0.5);
    Ok(())
}

fn draw_rule(raster: &mut Raster) -> Result<(), PdfError> {
    raster.y += raster.smul(theme::RULE_SPACING);
    raster.ensure(1.0);
    raster.canvas.fill_rect(
        raster.geom.left,
        raster.y,
        raster.geom.content_w(),
        (raster.smul(0.7)).max(1.0),
        theme::RULE_COLOR,
    );
    raster.y += raster.smul(theme::RULE_SPACING);
    Ok(())
}

fn draw_table(
    raster: &mut Raster,
    header: &[crate::ir::Cell],
    rows: &[Vec<crate::ir::Cell>],
    alignments: &[Alignment],
) -> Result<(), PdfError> {
    let ncols = header
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0))
        .max(1);
    let col_w = raster.geom.content_w() / ncols as f32;
    let cell_w = (col_w - raster.smul(theme::CELL_PAD) * 2.0).max(10.0);
    let size = raster.smul(theme::BODY_SIZE);
    let line_h = raster.smul(theme::BODY_SIZE * 1.3);
    let cell_pad_y = raster.smul(theme::CELL_PAD_Y);

    let shape_row = |raster: &mut Raster, cells: &[crate::ir::Cell], bold: bool| -> Vec<Buffer> {
        (0..ncols)
            .map(|i| {
                let runs: Vec<Run> = cells
                    .get(i)
                    .map(|c| {
                        c.content
                            .iter()
                            .cloned()
                            .map(|mut r| {
                                if bold {
                                    r.style.bold = true;
                                }
                                r
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                shape_buffer(raster.fs, &runs, size, line_h, Family::Serif, theme::TEXT, Some(cell_w))
            })
            .collect()
    };
    let row_height = |row: &[Buffer]| {
        row.iter().map(buffer_height).fold(0.0_f32, f32::max) + cell_pad_y * 2.0
    };

    let hair = raster.smul(0.5).max(1.0);
    let hairline = |raster: &mut Raster, color: Rgb, width: f32| {
        raster.canvas.fill_rect(raster.geom.left, raster.y, raster.geom.content_w(), width, color);
    };
    let draw_row = |raster: &mut Raster, row: &[Buffer], height: f32| {
        let row_top = raster.y;
        for (i, cell) in row.iter().enumerate() {
            let align = alignments.get(i).copied().unwrap_or(Alignment::Left);
            let cell_x = raster.geom.left + i as f32 * col_w + raster.smul(theme::CELL_PAD);
            let inner_w = col_w - raster.smul(theme::CELL_PAD) * 2.0;
            draw_cell(
                raster.fs,
                &mut raster.cache,
                &mut raster.canvas,
                cell,
                cell_x,
                inner_w,
                align,
                row_top + cell_pad_y,
            );
        }
        raster.y += height;
    };

    raster.y += raster.smul(theme::BLOCK_SPACING * 0.5);

    // Header row.
    let header_cells = shape_row(raster, header, true);
    let header_h = row_height(&header_cells);
    raster.ensure(header_h);
    hairline(raster, theme::TABLE_BORDER, hair);
    draw_row(raster, &header_cells, header_h);
    hairline(raster, theme::HEADER_BORDER, hair);

    for r in rows {
        let row = shape_row(raster, r, false);
        let h = row_height(&row);
        raster.ensure(h);
        draw_row(raster, &row, h);
        hairline(raster, theme::TABLE_BORDER, hair);
    }
    raster.y += raster.smul(theme::BLOCK_SPACING * 0.5);
    Ok(())
}

fn draw_image(
    raster: &mut Raster,
    data: &ImageData,
    alt: &str,
    width_px: u32,
    height_px: u32,
) -> Result<(), PdfError> {
    // Natural size in pixels → points at 96 DPI, upscaled by `scale`.
    let px_to_pt = 72.0 / 96.0;
    let natural_w = width_px as f32 * px_to_pt * raster.scale;
    let natural_h = height_px as f32 * px_to_pt * raster.scale;
    if natural_w <= 0.0 || natural_h <= 0.0 {
        return Ok(());
    }
    let scale = (raster.geom.content_w() / natural_w).min(1.0);
    let w = natural_w * scale;
    let h = natural_h * scale;
    raster.ensure(h);
    raster.y += raster.smul(theme::BLOCK_SPACING * 0.5);
    let x0 = raster.geom.left.round() as i32;
    let y0 = raster.y.round() as i32;
    let (dw, dh) = (w.round().max(1.0) as u32, h.round().max(1.0) as u32);

    match data {
        ImageData::Png(bytes) | ImageData::Jpeg(bytes) => match image::load_from_memory(bytes) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let resized = image::imageops::resize(
                    &rgba,
                    dw,
                    dh,
                    image::imageops::FilterType::Triangle,
                );
                raster.canvas.draw_rgba(x0, y0, &resized);
            }
            Err(_) => {
                let run = Run {
                    text: alt.to_string(),
                    style: Style {
                        italic: true,
                        color: Some(theme::MUTED),
                        ..Style::default()
                    },
                    ..Run::default()
                };
                return draw_paragraph(raster, std::slice::from_ref(&run));
            }
        },
        ImageData::Svg(svg) => {
            let tree = usvg::Tree::from_str(svg, &usvg::Options::default())
                .map_err(|e| PdfError::Layout(format!("SVG image parse failed: {e}")))?;
            if let Some(mut pixmap) = resvg::tiny_skia::Pixmap::new(dw.max(1), dh.max(1)) {
                let ts = resvg::tiny_skia::Transform::from_scale(
                    dw as f32 / tree.size().width().max(1.0),
                    dh as f32 / tree.size().height().max(1.0),
                );
                resvg::render(&tree, ts, &mut pixmap.as_mut());
                raster
                    .canvas
                    .draw_premul(x0, y0, pixmap.data(), pixmap.width(), pixmap.height());
            }
        }
    }
    raster.y += h + raster.smul(theme::BLOCK_SPACING * 0.5);
    Ok(())
}

fn draw_block(raster: &mut Raster, block: &Block) -> Result<(), PdfError> {
    match block {
        Block::Heading { level, content } => draw_heading(raster, *level, content),
        Block::Paragraph { content } => draw_paragraph(raster, content),
        Block::ListItem {
            indent_level,
            marker,
            content,
        } => draw_list_item(raster, *indent_level, *marker, content),
        Block::Quote { children } => draw_quote(raster, children),
        Block::Alert { kind, children } => draw_alert(raster, *kind, children),
        Block::CodeBlock { language: _, lines } => draw_code_block(raster, lines),
        Block::Table {
            header,
            rows,
            alignments,
        } => draw_table(raster, header, rows, alignments),
        Block::Image {
            data,
            alt,
            width_px,
            height_px,
        } => draw_image(raster, data, alt, *width_px, *height_px),
        Block::Rule => draw_rule(raster),
    }
}

// --- Driver ---------------------------------------------------------------

fn render_document(fs: &mut FontSystem, doc: &PdfDocument, scale: f32) -> Result<RgbaImage, PdfError> {
    let geom = Geometry::from_options(&doc.options, scale);
    let mut raster = Raster::new(fs, geom, scale);
    for block in &doc.blocks {
        draw_block(&mut raster, block)?;
    }
    // Ensure at least one page-height of canvas (empty docs still export).
    raster.canvas.grow((geom.bottom).ceil().max(1.0) as u32);
    Ok(raster.canvas.into_rgba())
}

// --- Tests ----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::bundled_only_font_system;
    use crate::ir::{PdfMetadata, PdfOptions};

    fn doc_with_blocks(blocks: Vec<Block>) -> PdfDocument {
        PdfDocument {
            metadata: PdfMetadata::default(),
            options: PdfOptions::default(),
            blocks,
            footnotes: Vec::new(),
        }
    }

    fn run_of(text: &str) -> Run {
        Run {
            text: text.to_string(),
            ..Run::default()
        }
    }

    /// A document with a Chinese heading renders non-blank pixels (the old
    /// 8x8 ASCII-only font rendered every Han glyph as a tofu box).
    #[test]
    fn chinese_text_renders_glyphs_not_boxes() {
        let mut fs = bundled_only_font_system();
        let doc = doc_with_blocks(vec![Block::Heading {
            level: 1,
            content: vec![run_of("你好，世界")],
        }]);
        let img = render_document(&mut fs, &doc, 2.0).expect("snapshot");
        let (w, h) = img.dimensions();
        assert!(w > 0 && h > 0, "canvas must be non-empty");
        let mut dark = 0u32;
        for p in img.pixels() {
            if p.0[0] < 0x90 && p.0[1] < 0x90 && p.0[2] < 0x90 {
                dark += 1;
            }
        }
        assert!(dark > 50, "expected CJK glyphs to paint dark pixels, got {dark}");
    }

    /// Mixed CJK/Latin paragraph flows onto more than one line.
    #[test]
    fn mixed_text_wraps_and_renders() {
        let mut fs = bundled_only_font_system();
        let text: String = "这是一个用于测试的中文段落 content with English words 混排。".repeat(6);
        let doc = doc_with_blocks(vec![Block::Paragraph {
            content: vec![run_of(&text)],
        }]);
        let img = render_document(&mut fs, &doc, 2.0).expect("snapshot");
        let h = img.height();
        let body_line = theme::BODY_SIZE * theme::LINE_HEIGHT_MULT * 2.0;
        assert!(
            (h as f32) > body_line * 3.0,
            "a long paragraph must wrap, snapshot height {h}"
        );
    }

    /// Empty documents still yield one readable frame.
    #[test]
    fn empty_document_renders_one_frame() {
        let mut fs = bundled_only_font_system();
        let doc = doc_with_blocks(Vec::new());
        let img = render_document(&mut fs, &doc, 2.0).expect("snapshot");
        assert!(img.width() > 0 && img.height() > 0);
    }

    /// `render_snapshot` (the public wrapper) resolves fonts through the
    /// process-wide font system and returns an image for a CJK document.
    #[test]
    fn public_snapshot_resolves_cjk() {
        let doc = doc_with_blocks(vec![Block::Paragraph {
            content: vec![run_of("CJK 中文导出测试")],
        }]);
        let img = render_snapshot(&doc, 2.0).expect("snapshot");
        assert!(img.width() > 0);
    }
}
