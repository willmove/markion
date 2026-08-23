//! Block layout and pagination (tasks 2.5–2.8, design D7): flows IR blocks
//! into pages with keep rules, footnote areas, tables, and furniture, then
//! hands a fully placed, font-resolved page model to [`crate::emit`].
//!
//! Coordinates are PDF points, origin top-left, y growing downward; text
//! positions are glyph baselines.
//!
//! Footnote area algorithm: references reserve their note's height on the
//! page as content flows; the note area is capped at
//! [`theme::NOTE_AREA_CAP`] of the content height and notes that do not fit
//! carry to the next page's note area (a note taller than the cap is placed
//! whole on its reference's page rather than dropped). Content-fit checks
//! always account for the reserved note height, so notes never overlap
//! body text.

use std::collections::{HashMap, HashSet, VecDeque};

use cosmic_text::FontSystem;
use krilla::image::Image as KrillaImage;

use crate::ir::{AlertKind, Alignment, Block, ImageData, ListMarker, PdfDocument, Rgb, Run, Style};
use crate::text::{
    FamilyKind, FontCache, ParagraphSpec, RunInfo, ShapedLine, ShapedParagraph, shape_paragraph,
};
use crate::{PdfError, theme};

const MM_TO_PT: f32 = 72.0 / 25.4;
/// Points per CSS pixel for image natural sizing (96 DPI).
const PX_TO_PT: f32 = 72.0 / 96.0;

/// A rectangle in page coordinates.
#[derive(Debug, Clone, Copy)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectF {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            x,
            y,
            w: w.max(0.0),
            h: h.max(0.0),
        }
    }
}

/// One placed text line (absolute page coordinates).
pub struct PlacedLine {
    pub x: f32,
    pub baseline: f32,
    pub groups: Vec<crate::text::GlyphGroup>,
}

/// A placed primitive on a page.
pub enum PlacedItem {
    /// Text glyphs.
    Line(PlacedLine),
    /// Filled rectangle (highlight, alert tint, code/table background).
    Rect { rect: RectF, fill: Rgb },
    /// Straight stroke (rules, accent bars, underlines, table borders).
    Stroke {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color: Rgb,
    },
    /// Raster image, already decoded by krilla.
    Image {
        image: KrillaImage,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    /// Vector image parsed by usvg (math SVG is path-only).
    Svg {
        tree: Box<usvg::Tree>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    /// Clip subsequent items to this rect. Code lines that overflow the
    /// text column are clipped (not wrapped, not shrunk).
    PushClip(RectF),
    /// End the current clip.
    Pop,
}

/// Link target of a placed annotation.
#[derive(Debug, Clone)]
pub enum AnnotTarget {
    Url(String),
    /// Body-relative page index and y position; emission adds the TOC offset.
    Internal {
        page: usize,
        y: f32,
    },
}

/// A placed link annotation.
pub struct PlacedAnnotation {
    pub rect: RectF,
    pub target: AnnotTarget,
}

/// One laid-out page.
#[derive(Default)]
pub struct PageLayout {
    pub items: Vec<PlacedItem>,
    pub annotations: Vec<PlacedAnnotation>,
}

/// A collected heading, for the outline and the table of contents.
pub struct OutlineEntry {
    pub level: u8,
    pub title: String,
    /// Body-relative page index.
    pub page: usize,
    pub y: f32,
}

/// Everything the emitter needs.
pub struct LayoutResult {
    pub toc_pages: Vec<PageLayout>,
    pub body_pages: Vec<PageLayout>,
    pub outline: Vec<OutlineEntry>,
    pub page_width: f32,
    pub page_height: f32,
}

/// Page geometry derived from [`crate::ir::PdfOptions`].
#[derive(Clone, Copy)]
struct Geometry {
    page_w: f32,
    page_h: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Geometry {
    fn from_options(options: &crate::ir::PdfOptions) -> Self {
        let page_w = options.page_width_mm * MM_TO_PT;
        let page_h = options.page_height_mm * MM_TO_PT;
        let margin = options.margin_mm * MM_TO_PT;
        Self {
            page_w,
            page_h,
            left: margin,
            top: margin,
            right: page_w - margin,
            bottom: page_h - margin,
        }
    }

    fn content_w(&self) -> f32 {
        self.right - self.left
    }

    fn content_h(&self) -> f32 {
        self.bottom - self.top
    }
}

/// An open quote/alert accent spanning (possibly) several pages. Per page
/// segment, the background/border item is inserted at the position recorded
/// when the segment opened, so backgrounds stay behind the text.
struct Accent {
    x: f32,
    width: f32,
    fill: Option<Rgb>,
    stroke: Rgb,
    /// Page index of the open segment.
    page: usize,
    /// Insertion index for the background item on that page.
    insert_idx: usize,
    /// Top y of the open segment.
    seg_top: f32,
}

struct PendingHeading {
    para: ShapedParagraph,
    level: u8,
    title: String,
    spacing_before: f32,
}

struct Layouter<'a> {
    fs: &'a mut FontSystem,
    cache: &'a mut FontCache,
    geom: Geometry,
    pages: Vec<PageLayout>,
    /// Cursor: top of the next content slice on the current page.
    y: f32,
    /// Whether the current page has any content yet.
    page_empty: bool,
    pending_heading: Option<PendingHeading>,
    /// Headings collected during body layout (body-relative page indices).
    headings: Vec<OutlineEntry>,
    accents: Vec<Accent>,
    // --- Footnotes ---
    /// Pre-shaped note bodies, indexed by footnote id - 1.
    notes: Vec<ShapedParagraph>,
    note_heights: Vec<f32>,
    /// Note ids assigned to the current page, in order.
    page_notes: Vec<u32>,
    /// Conservatively reserved note-area height on the current page.
    notes_reserved: f32,
    /// Ids already assigned to a page or the carry queue.
    scheduled: HashSet<u32>,
    /// Notes that did not fit on their reference's page.
    carry: VecDeque<u32>,
    /// Reference rects awaiting note-anchor resolution:
    /// (footnote id, body page index, rect).
    ref_rects: Vec<(u32, usize, RectF)>,
    /// Note anchors: id → (body page index, y of the note's first baseline).
    note_anchors: HashMap<u32, (usize, f32)>,
}

impl<'a> Layouter<'a> {
    fn new(fs: &'a mut FontSystem, cache: &'a mut FontCache, geom: Geometry) -> Self {
        let top = geom.top;
        Self {
            fs,
            cache,
            geom,
            pages: vec![PageLayout::default()],
            y: top,
            page_empty: true,
            pending_heading: None,
            headings: Vec::new(),
            accents: Vec::new(),
            notes: Vec::new(),
            note_heights: Vec::new(),
            page_notes: Vec::new(),
            notes_reserved: 0.0,
            scheduled: HashSet::new(),
            carry: VecDeque::new(),
            ref_rects: Vec::new(),
            note_anchors: HashMap::new(),
        }
    }

    fn note_cap(&self) -> f32 {
        self.geom.content_h() * theme::NOTE_AREA_CAP
    }

    fn note_height(&self, id: u32) -> Option<f32> {
        self.note_heights.get(id.checked_sub(1)? as usize).copied()
    }

    /// Bottom limit for content given a reserved note-area height.
    fn content_limit(&self, reserved: f32) -> f32 {
        if reserved > 0.0 {
            self.geom.bottom - reserved - theme::NOTE_GAP
        } else {
            self.geom.bottom
        }
    }

    fn cur_page(&mut self) -> &mut PageLayout {
        self.pages.last_mut().expect("a page is always open")
    }

    /// Ensure `h` points of vertical space, breaking the page if needed.
    /// `refs` are footnote ids first referenced by the content about to be
    /// placed; their note heights are reserved on this page (or carried).
    fn ensure_space(&mut self, h: f32, refs: &[u32]) {
        loop {
            let extra: f32 = refs
                .iter()
                .filter(|id| !self.scheduled.contains(id))
                .filter_map(|id| self.note_height(*id))
                .sum();
            let projected = (self.notes_reserved + extra)
                .min(self.note_cap())
                .max(self.notes_reserved);
            if self.y + h <= self.content_limit(projected) || self.page_empty {
                // A block taller than a whole page is placed anyway on a
                // fresh page (overflowing visually) rather than looping.
                self.schedule_notes(refs);
                return;
            }
            self.finish_page();
        }
    }

    /// Reserve note space for first-time references on the current page.
    fn schedule_notes(&mut self, refs: &[u32]) {
        for &id in refs {
            if self.scheduled.contains(&id) {
                continue;
            }
            self.scheduled.insert(id);
            let Some(h) = self.note_height(id) else {
                continue;
            };
            if self.notes_reserved + h <= self.note_cap() || self.page_notes.is_empty() {
                self.page_notes.push(id);
                self.notes_reserved += h;
            } else {
                self.carry.push_back(id);
                self.notes_reserved = self.note_cap();
            }
        }
    }

    /// Close the current page: accent segments, footnote area, then open a
    /// fresh page and drain carried notes into its reserve.
    fn finish_page(&mut self) {
        let seg_end = self.y.min(self.content_limit(self.notes_reserved));
        for accent in &mut self.accents {
            let item = accent_item(accent, seg_end);
            self.pages[accent.page]
                .items
                .insert(accent.insert_idx, item);
        }
        self.layout_note_area();
        self.pages.push(PageLayout::default());
        // Reopen accent segments at the top of the new page.
        for accent in &mut self.accents {
            accent.page = self.pages.len() - 1;
            accent.insert_idx = 0;
            accent.seg_top = self.geom.top;
        }
        self.y = self.geom.top;
        self.page_empty = true;
        self.notes_reserved = 0.0;
        self.page_notes.clear();
        while let Some(&id) = self.carry.front() {
            let h = self.note_height(id).unwrap_or(0.0);
            if self.notes_reserved + h <= self.note_cap() || self.page_notes.is_empty() {
                self.carry.pop_front();
                self.page_notes.push(id);
                self.notes_reserved += h;
            } else {
                break;
            }
        }
    }

    /// Draw the footnote rule and the assigned notes at the page bottom.
    fn layout_note_area(&mut self) {
        if self.page_notes.is_empty() {
            return;
        }
        let actual_h: f32 = self
            .page_notes
            .iter()
            .filter_map(|id| self.note_height(*id))
            .sum();
        let rule_y = self.geom.bottom - actual_h - 4.0;
        let left = self.geom.left;
        let rule_end = left + self.geom.content_w() * 0.35;
        self.cur_page().items.push(PlacedItem::Stroke {
            x1: left,
            y1: rule_y,
            x2: rule_end,
            y2: rule_y,
            width: 0.5,
            color: theme::RULE_COLOR,
        });
        let page_idx = self.pages.len() - 1;
        let mut y = rule_y + 4.0;
        let notes = std::mem::take(&mut self.page_notes);
        let all_notes = std::mem::take(&mut self.notes);
        for id in &notes {
            let Some(note) = all_notes.get((*id - 1) as usize) else {
                continue;
            };
            let mut first = true;
            for line in &note.lines {
                let baseline = y + line.height * 0.8;
                if first {
                    self.note_anchors.insert(*id, (page_idx, baseline));
                    first = false;
                }
                self.place_line_raw(line, &note.runs, left, baseline);
                y += line.height;
            }
        }
        self.notes = all_notes;
        self.page_notes = notes;
    }

    /// Place one shaped line at an absolute position, emitting decoration
    /// items (highlight, strikethrough, link underline/annotation, footnote
    /// reference annotation) before the glyphs.
    fn place_line_raw(&mut self, line: &ShapedLine, runs: &[RunInfo], x: f32, baseline: f32) {
        let mut decorations: Vec<PlacedItem> = Vec::new();
        let mut annotations: Vec<PlacedAnnotation> = Vec::new();
        for group in &line.groups {
            let info = &runs[group.run];
            let gx = x + group.start_x;
            let gw = group.width();
            let size = group.font_size;
            if gw <= 0.0 {
                continue;
            }
            if info.style.highlight {
                decorations.push(PlacedItem::Rect {
                    rect: RectF::new(gx, baseline - size * 0.78, gw, size * 1.08),
                    fill: theme::HIGHLIGHT,
                });
            }
            if info.style.strikethrough {
                decorations.push(PlacedItem::Stroke {
                    x1: gx,
                    y1: baseline - size * 0.28,
                    x2: gx + gw,
                    y2: baseline - size * 0.28,
                    width: (size * 0.05).max(0.4),
                    color: group.fill,
                });
            }
            if let Some(url) = &info.link {
                decorations.push(PlacedItem::Stroke {
                    x1: gx,
                    y1: baseline + size * 0.14,
                    x2: gx + gw,
                    y2: baseline + size * 0.14,
                    width: (size * 0.05).max(0.4),
                    color: group.fill,
                });
                annotations.push(PlacedAnnotation {
                    rect: RectF::new(gx, baseline - size * 0.85, gw, size * 1.25),
                    target: AnnotTarget::Url(url.clone()),
                });
            }
            if let Some(id) = info.footnote {
                let page = self.pages.len() - 1;
                self.ref_rects.push((
                    id,
                    page,
                    RectF::new(gx, baseline - size * 0.85, gw.max(2.0), size * 1.25),
                ));
            }
        }
        let page = self.cur_page();
        page.items.append(&mut decorations);
        page.annotations.append(&mut annotations);
        page.items.push(PlacedItem::Line(PlacedLine {
            x,
            baseline,
            groups: line.groups.clone(),
        }));
    }

    /// Place a paragraph line by line, splitting across pages as needed.
    fn place_paragraph(&mut self, para: &ShapedParagraph, x: f32) {
        for line in &para.lines {
            self.ensure_space(line.height, &line.footnotes);
            let baseline = self.y + line.height * 0.8;
            self.place_line_raw(line, &para.runs, x, baseline);
            self.y += line.height;
            self.page_empty = false;
        }
    }

    /// Flush a pending keep-with-next heading, ensuring `next_h` points of
    /// the following block fit on the same page.
    fn flush_pending(&mut self, next_h: f32) {
        let Some(pending) = self.pending_heading.take() else {
            return;
        };
        let h = pending.para.height();
        let refs: Vec<u32> = pending
            .para
            .lines
            .iter()
            .flat_map(|l| l.footnotes.iter().copied())
            .collect();
        self.y += pending.spacing_before;
        self.ensure_space(h + next_h.min(self.geom.content_h() * 0.5), &refs);
        let page = self.pages.len() - 1;
        let y0 = self.y;
        let x = self.geom.left;
        self.place_paragraph(&pending.para, x);
        self.headings.push(OutlineEntry {
            level: pending.level,
            title: pending.title,
            page,
            y: y0,
        });
        self.y += theme::HEADING_AFTER;
    }

    #[allow(clippy::too_many_arguments)]
    fn shape(
        &mut self,
        runs: &[Run],
        size: f32,
        line_height: f32,
        family: FamilyKind,
        color: Rgb,
        width: Option<f32>,
    ) -> Result<ShapedParagraph, PdfError> {
        shape_paragraph(
            self.fs,
            self.cache,
            &ParagraphSpec {
                runs,
                size,
                line_height,
                family,
                color,
                width,
            },
        )
    }

    // --- Blocks ---------------------------------------------------------

    fn heading(&mut self, level: u8, content: &[Run]) -> Result<(), PdfError> {
        let level = level.clamp(1, 6);
        let size = theme::HEADING_SIZES[(level - 1) as usize];
        let runs: Vec<Run> = content
            .iter()
            .cloned()
            .map(|mut r| {
                r.style.bold = true;
                r
            })
            .collect();
        let width = self.geom.content_w();
        let para = self.shape(
            &runs,
            size,
            size * 1.25,
            FamilyKind::Heading,
            theme::TEXT,
            Some(width),
        )?;
        let title: String = content.iter().map(|r| r.text.as_str()).collect();
        // Stacked headings keep together: flush an older pending heading
        // requiring room for this one too.
        let h = para.height();
        self.flush_pending(h);
        self.pending_heading = Some(PendingHeading {
            para,
            level,
            title,
            spacing_before: if self.page_empty {
                0.0
            } else {
                theme::HEADING_BEFORE
            },
        });
        Ok(())
    }

    fn paragraph(&mut self, content: &[Run]) -> Result<(), PdfError> {
        let width = self.geom.content_w();
        let para = self.shape(
            content,
            theme::BODY_SIZE,
            theme::BODY_SIZE * theme::LINE_HEIGHT_MULT,
            FamilyKind::Body,
            theme::TEXT,
            Some(width),
        )?;
        if para.is_empty() {
            self.flush_pending(0.0);
            return Ok(());
        }
        self.flush_pending(para.lines[0].height);
        let x = self.geom.left;
        self.place_paragraph(&para, x);
        self.y += theme::PARA_SPACING;
        Ok(())
    }

    fn list_item(
        &mut self,
        indent_level: u8,
        marker: ListMarker,
        content: &[Run],
    ) -> Result<(), PdfError> {
        let indent_x = self.geom.left + indent_level as f32 * theme::LIST_INDENT;
        let content_x = indent_x + theme::MARKER_WIDTH;
        let width = (self.geom.right - content_x).max(20.0);
        let para = self.shape(
            content,
            theme::BODY_SIZE,
            theme::BODY_SIZE * theme::LINE_HEIGHT_MULT,
            FamilyKind::Body,
            theme::TEXT,
            Some(width),
        )?;
        let marker_text = match marker {
            ListMarker::Bullet => "•".to_string(),
            ListMarker::Number(n) => format!("{n}."),
            ListMarker::Task { checked } => if checked { "[x]" } else { "[ ]" }.to_string(),
        };
        let marker_run = Run {
            text: marker_text,
            style: Style {
                color: Some(theme::MUTED),
                ..Style::default()
            },
            ..Run::default()
        };
        let marker_para = self.shape(
            std::slice::from_ref(&marker_run),
            theme::BODY_SIZE,
            theme::BODY_SIZE * theme::LINE_HEIGHT_MULT,
            FamilyKind::Body,
            theme::MUTED,
            None,
        )?;
        let first_h = para
            .lines
            .first()
            .map(|l| l.height)
            .unwrap_or(theme::BODY_SIZE * theme::LINE_HEIGHT_MULT);
        self.flush_pending(first_h);
        self.ensure_space(first_h, &[]);
        // The marker aligns with the first content line's baseline.
        let marker_baseline = self.y + first_h * 0.8;
        if let Some(line) = marker_para.lines.first() {
            self.place_line_raw(line, &marker_para.runs, indent_x, marker_baseline);
        }
        self.place_paragraph(&para, content_x);
        self.y += theme::PARA_SPACING * 0.5;
        Ok(())
    }

    fn quote(&mut self, children: &[Block]) -> Result<(), PdfError> {
        self.flush_pending(0.0);
        self.y += theme::BLOCK_SPACING * 0.5;
        let insert_idx = self.cur_page().items.len();
        self.accents.push(Accent {
            x: self.geom.left + 2.0,
            width: self.geom.content_w(),
            fill: None,
            stroke: theme::QUOTE_ACCENT,
            page: self.pages.len() - 1,
            insert_idx,
            seg_top: self.y,
        });
        let saved_left = self.geom.left;
        self.geom.left += theme::ACCENT_INDENT;
        for child in children {
            self.block(child)?;
        }
        self.geom.left = saved_left;
        let accent = self.accents.pop().expect("quote accent open");
        let item = accent_item(&accent, self.y);
        self.pages[accent.page]
            .items
            .insert(accent.insert_idx, item);
        self.y += theme::BLOCK_SPACING * 0.5;
        Ok(())
    }

    fn alert(&mut self, kind: AlertKind, children: &[Block]) -> Result<(), PdfError> {
        self.flush_pending(0.0);
        self.y += theme::BLOCK_SPACING * 0.5;
        let insert_idx = self.cur_page().items.len();
        self.accents.push(Accent {
            x: self.geom.left,
            width: self.geom.content_w(),
            fill: Some(theme::alert_tint(kind)),
            stroke: theme::alert_accent(kind),
            page: self.pages.len() - 1,
            insert_idx,
            seg_top: self.y,
        });
        let saved_left = self.geom.left;
        self.geom.left += theme::ACCENT_INDENT;
        // Bold kind label.
        let label_run = Run {
            text: theme::alert_label(kind).to_string(),
            style: Style {
                bold: true,
                color: Some(theme::alert_accent(kind)),
                ..Style::default()
            },
            ..Run::default()
        };
        let label = self.shape(
            std::slice::from_ref(&label_run),
            theme::BODY_SIZE,
            theme::BODY_SIZE * theme::LINE_HEIGHT_MULT,
            FamilyKind::Heading,
            theme::alert_accent(kind),
            Some(self.geom.content_w()),
        )?;
        let x = self.geom.left;
        self.place_paragraph(&label, x);
        self.y += 2.0;
        for child in children {
            self.block(child)?;
        }
        self.geom.left = saved_left;
        self.y += 2.0;
        let accent = self.accents.pop().expect("alert accent open");
        let item = accent_item(&accent, self.y);
        self.pages[accent.page]
            .items
            .insert(accent.insert_idx, item);
        self.y += theme::BLOCK_SPACING * 0.5;
        Ok(())
    }

    /// Push the clip rect for a code segment starting at the cursor.
    fn push_code_clip(&mut self) {
        let rect = RectF::new(
            self.geom.left,
            self.y,
            self.geom.content_w(),
            self.content_limit(self.notes_reserved) - self.y,
        );
        self.cur_page().items.push(PlacedItem::PushClip(rect));
    }

    /// Close a code segment: pop its clip and insert the background behind
    /// the segment's text.
    fn close_code_segment(&mut self, seg_page: usize, seg_insert: usize, seg_top: f32) {
        self.pages[seg_page].items.push(PlacedItem::Pop);
        let bg = PlacedItem::Rect {
            rect: RectF::new(
                self.geom.left,
                seg_top - theme::CODE_PAD,
                self.geom.content_w(),
                self.y_for_page(seg_page) - seg_top + theme::CODE_PAD,
            ),
            fill: theme::CODE_BG,
        };
        self.pages[seg_page].items.insert(seg_insert, bg);
    }

    /// The y cursor to use when closing a segment: the live cursor for the
    /// current page, the content limit for a page that is already closed.
    fn y_for_page(&self, page: usize) -> f32 {
        if page == self.pages.len() - 1 {
            self.y
        } else {
            self.content_limit(self.notes_reserved)
        }
    }

    fn code_block(
        &mut self,
        _language: &Option<String>,
        lines: &[Vec<Run>],
    ) -> Result<(), PdfError> {
        let line_height = theme::CODE_SIZE * 1.4;
        let mut shaped = Vec::with_capacity(lines.len());
        for line in lines {
            shaped.push(self.shape(
                line,
                theme::CODE_SIZE,
                line_height,
                FamilyKind::Code,
                theme::TEXT,
                None, // code lines never wrap; overflow is clipped
            )?);
        }
        let total_h = shaped.iter().map(|p| p.height()).sum::<f32>() + theme::CODE_PAD * 2.0;
        let capacity = self.geom.content_h();
        self.flush_pending(total_h);
        self.y += theme::BLOCK_SPACING * 0.5;
        // Keep the block together when it fits on a single page.
        if total_h <= capacity {
            self.ensure_space(total_h, &[]);
        }
        let mut seg_page = self.pages.len() - 1;
        let mut seg_insert = self.cur_page().items.len();
        let mut seg_top = self.y;
        self.push_code_clip();
        for para in &shaped {
            for line in &para.lines {
                let page_before = self.pages.len();
                self.ensure_space(line.height, &[]);
                if self.pages.len() != page_before {
                    self.close_code_segment(seg_page, seg_insert, seg_top);
                    seg_page = self.pages.len() - 1;
                    seg_insert = self.cur_page().items.len();
                    seg_top = self.y;
                    self.push_code_clip();
                }
                let baseline = self.y + line.height * 0.8;
                self.place_line_raw(line, &para.runs, self.geom.left + theme::CODE_PAD, baseline);
                self.y += line.height;
                self.page_empty = false;
            }
        }
        self.close_code_segment(seg_page, seg_insert, seg_top);
        self.y += theme::CODE_PAD + theme::BLOCK_SPACING * 0.5;
        Ok(())
    }

    fn table(
        &mut self,
        header: &[crate::ir::Cell],
        rows: &[Vec<crate::ir::Cell>],
        alignments: &[Alignment],
    ) -> Result<(), PdfError> {
        let ncols = header
            .len()
            .max(rows.iter().map(|r| r.len()).max().unwrap_or(0))
            .max(1);
        let col_w = self.geom.content_w() / ncols as f32;
        let cell_w = (col_w - theme::CELL_PAD * 2.0).max(10.0);

        let shape_row = |lay: &mut Self, cells: &[crate::ir::Cell], bold: bool| {
            let mut out = Vec::with_capacity(ncols);
            for i in 0..ncols {
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
                out.push(lay.shape(
                    &runs,
                    theme::BODY_SIZE,
                    theme::BODY_SIZE * 1.3,
                    FamilyKind::Body,
                    theme::TEXT,
                    Some(cell_w),
                )?);
            }
            Ok::<Vec<ShapedParagraph>, PdfError>(out)
        };

        let row_height = |row: &[ShapedParagraph]| {
            row.iter().map(|c| c.height()).fold(0.0, f32::max) + theme::CELL_PAD_Y * 2.0
        };
        let row_refs = |row: &[ShapedParagraph]| -> Vec<u32> {
            row.iter()
                .flat_map(|c| c.lines.iter().flat_map(|l| l.footnotes.iter().copied()))
                .collect()
        };

        let header_cells = shape_row(self, header, true)?;
        let header_h = row_height(&header_cells);
        self.flush_pending(header_h);
        self.y += theme::BLOCK_SPACING * 0.5;

        // Top border + header row (repeated after each page break below).
        self.hairline(theme::TABLE_BORDER, 0.7);
        self.ensure_space(header_h, &row_refs(&header_cells));
        self.place_row(
            &header_cells,
            header_h,
            col_w,
            alignments,
            theme::HEADER_BORDER,
            1.0,
        );

        let mut first_row = true;
        for r in rows {
            let row = shape_row(self, r, false)?;
            let h = row_height(&row);
            let refs = row_refs(&row);
            let page_before = self.pages.len();
            self.ensure_space(h, &refs);
            if !first_row && self.pages.len() != page_before {
                // The row moved to a new page: repeat the header.
                self.hairline(theme::TABLE_BORDER, 0.7);
                self.place_row(
                    &header_cells,
                    header_h,
                    col_w,
                    alignments,
                    theme::HEADER_BORDER,
                    1.0,
                );
            }
            self.place_row(&row, h, col_w, alignments, theme::TABLE_BORDER, 0.5);
            first_row = false;
        }
        self.y += theme::BLOCK_SPACING * 0.5;
        Ok(())
    }

    /// Horizontal border across the content width at the cursor.
    fn hairline(&mut self, color: Rgb, width: f32) {
        let y = self.y;
        let left = self.geom.left;
        let right = self.geom.right;
        self.cur_page().items.push(PlacedItem::Stroke {
            x1: left,
            y1: y,
            x2: right,
            y2: y,
            width,
            color,
        });
    }

    /// Place one table row (never splits) and draw its bottom border.
    fn place_row(
        &mut self,
        row: &[ShapedParagraph],
        height: f32,
        col_w: f32,
        alignments: &[Alignment],
        border_color: Rgb,
        border_width: f32,
    ) {
        let row_top = self.y;
        for (i, cell) in row.iter().enumerate() {
            let align = alignments.get(i).copied().unwrap_or(Alignment::Left);
            let cell_x = self.geom.left + i as f32 * col_w + theme::CELL_PAD;
            let inner_w = col_w - theme::CELL_PAD * 2.0;
            let mut cy = row_top + theme::CELL_PAD_Y;
            for line in &cell.lines {
                let dx = match align {
                    Alignment::Left => 0.0,
                    Alignment::Center => (inner_w - line.width) / 2.0,
                    Alignment::Right => inner_w - line.width,
                }
                .max(0.0);
                let baseline = cy + line.height * 0.8;
                self.place_line_raw(line, &cell.runs, cell_x + dx, baseline);
                cy += line.height;
            }
        }
        self.y = row_top + height;
        self.hairline(border_color, border_width);
        self.page_empty = false;
    }

    fn image(
        &mut self,
        data: &ImageData,
        alt: &str,
        width_px: u32,
        height_px: u32,
    ) -> Result<(), PdfError> {
        let parsed = match data {
            ImageData::Png(bytes) => KrillaImage::from_png(krilla::Data::from(bytes.clone()), true)
                .map(ParsedImage::Raster)
                .map_err(|e| format!("PNG decode failed: {e}")),
            ImageData::Jpeg(bytes) => {
                KrillaImage::from_jpeg(krilla::Data::from(bytes.clone()), true)
                    .map(ParsedImage::Raster)
                    .map_err(|e| format!("JPEG decode failed: {e}"))
            }
            ImageData::Svg(svg) => usvg::Tree::from_str(svg, &usvg::Options::default())
                .map(Box::new)
                .map(ParsedImage::Vector)
                .map_err(|e| format!("SVG parse failed: {e}")),
        };

        let (parsed, natural_w, natural_h) = match parsed {
            Ok(parsed) => {
                let (w, h) = match (&parsed, width_px, height_px) {
                    (_, w, h) if w > 0 && h > 0 => (w as f32 * PX_TO_PT, h as f32 * PX_TO_PT),
                    (ParsedImage::Vector(tree), _, _) => {
                        let size = tree.size();
                        (size.width() * PX_TO_PT, size.height() * PX_TO_PT)
                    }
                    (ParsedImage::Raster(img), _, _) => {
                        let (w, h) = img.size();
                        (w as f32 * PX_TO_PT, h as f32 * PX_TO_PT)
                    }
                };
                (parsed, w, h)
            }
            Err(_) => {
                // Undecodable payload: degrade to an `alt` text line rather
                // than failing the export (remote/missing images are already
                // filtered by the IR builder).
                let run = Run {
                    text: alt.to_string(),
                    style: Style {
                        italic: true,
                        color: Some(theme::MUTED),
                        ..Style::default()
                    },
                    ..Run::default()
                };
                return self.paragraph(std::slice::from_ref(&run));
            }
        };

        if natural_w <= 0.0 || natural_h <= 0.0 {
            return Ok(());
        }
        // Wider than the text column → scale down; narrower keeps its
        // natural size at 96 DPI.
        let scale = (self.geom.content_w() / natural_w).min(1.0);
        let w = natural_w * scale;
        let h = natural_h * scale;
        self.flush_pending(h);
        self.y += theme::BLOCK_SPACING * 0.5;
        self.ensure_space(h, &[]);
        let item = match parsed {
            ParsedImage::Raster(image) => PlacedItem::Image {
                image,
                x: self.geom.left,
                y: self.y,
                w,
                h,
            },
            ParsedImage::Vector(tree) => PlacedItem::Svg {
                tree,
                x: self.geom.left,
                y: self.y,
                w,
                h,
            },
        };
        self.cur_page().items.push(item);
        self.y += h + theme::BLOCK_SPACING * 0.5;
        self.page_empty = false;
        Ok(())
    }

    fn rule(&mut self) -> Result<(), PdfError> {
        self.flush_pending(0.0);
        self.ensure_space(1.0, &[]);
        self.y += theme::RULE_SPACING;
        self.hairline(theme::RULE_COLOR, 0.7);
        self.y += theme::RULE_SPACING;
        self.page_empty = false;
        Ok(())
    }

    fn block(&mut self, block: &Block) -> Result<(), PdfError> {
        match block {
            Block::Heading { level, content } => self.heading(*level, content),
            Block::Paragraph { content } => self.paragraph(content),
            Block::ListItem {
                indent_level,
                marker,
                content,
            } => self.list_item(*indent_level, *marker, content),
            Block::Quote { children } => self.quote(children),
            Block::Alert { kind, children } => self.alert(*kind, children),
            Block::CodeBlock { language, lines } => self.code_block(language, lines),
            Block::Table {
                header,
                rows,
                alignments,
            } => self.table(header, rows, alignments),
            Block::Image {
                data,
                alt,
                width_px,
                height_px,
            } => self.image(data, alt, *width_px, *height_px),
            Block::Rule => self.rule(),
        }
    }
}

enum ParsedImage {
    Raster(KrillaImage),
    Vector(Box<usvg::Tree>),
}

/// Build the accent background/border item for one page segment.
fn accent_item(accent: &Accent, seg_end: f32) -> PlacedItem {
    let h = (seg_end - accent.seg_top).max(0.0);
    if let Some(fill) = accent.fill {
        PlacedItem::Rect {
            rect: RectF::new(accent.x, accent.seg_top, accent.width, h),
            fill,
        }
    } else {
        PlacedItem::Stroke {
            x1: accent.x,
            y1: accent.seg_top,
            x2: accent.x,
            y2: accent.seg_top + h,
            width: 2.5,
            color: accent.stroke,
        }
    }
}

/// Shape the per-page footnote bodies once (numbered, smaller size).
fn preshape_notes(
    fs: &mut FontSystem,
    cache: &mut FontCache,
    doc: &PdfDocument,
    width: f32,
) -> Result<(Vec<ShapedParagraph>, Vec<f32>), PdfError> {
    let mut notes = Vec::with_capacity(doc.footnotes.len());
    let mut heights = Vec::with_capacity(doc.footnotes.len());
    for (i, runs) in doc.footnotes.iter().enumerate() {
        let mut note_runs = vec![Run {
            text: format!("{}. ", i + 1),
            style: Style {
                bold: true,
                ..Style::default()
            },
            ..Run::default()
        }];
        note_runs.extend(runs.iter().cloned());
        let para = shape_paragraph(
            fs,
            cache,
            &ParagraphSpec {
                runs: &note_runs,
                size: theme::NOTE_SIZE,
                line_height: theme::NOTE_SIZE * 1.35,
                family: FamilyKind::Body,
                color: theme::TEXT,
                width: Some(width),
            },
        )?;
        heights.push(para.height());
        notes.push(para);
    }
    Ok((notes, heights))
}

/// Lay out the whole document: body flow, then the optional TOC (prepended,
/// with body page numbers shifted by the TOC page count), then footers.
pub fn layout_document(
    fs: &mut FontSystem,
    cache: &mut FontCache,
    doc: &PdfDocument,
) -> Result<LayoutResult, PdfError> {
    let geom = Geometry::from_options(&doc.options);
    let (notes, note_heights) = preshape_notes(fs, cache, doc, geom.content_w())?;
    let mut lay = Layouter::new(fs, cache, geom);
    lay.notes = notes;
    lay.note_heights = note_heights;

    for block in &doc.blocks {
        lay.block(block)?;
    }
    lay.flush_pending(0.0);
    // Close open accents and the note area on the final page. An empty
    // trailing page (a break right at the end) is dropped unless it carries
    // footnotes; an empty document still yields one valid page.
    let final_seg_end = lay.y.min(lay.content_limit(lay.notes_reserved));
    let accents = std::mem::take(&mut lay.accents);
    for accent in &accents {
        let item = accent_item(accent, final_seg_end);
        lay.pages[accent.page].items.insert(accent.insert_idx, item);
    }
    if lay.page_empty && lay.pages.len() > 1 && lay.page_notes.is_empty() {
        lay.pages.pop();
    } else {
        lay.layout_note_area();
    }
    let mut body_pages = std::mem::take(&mut lay.pages);
    if body_pages.is_empty() {
        body_pages.push(PageLayout::default());
    }

    // Resolve footnote reference annotations now that note anchors exist.
    for (id, page, rect) in &lay.ref_rects {
        if let Some(&(note_page, note_y)) = lay.note_anchors.get(id)
            && let Some(p) = body_pages.get_mut(*page)
        {
            p.annotations.push(PlacedAnnotation {
                rect: *rect,
                target: AnnotTarget::Internal {
                    page: note_page,
                    y: note_y,
                },
            });
        }
    }

    // Table of contents: the heading entries are known before the TOC is
    // rendered, so the TOC page count is deterministic and body page
    // numbers shift by it.
    let mut toc_pages: Vec<PageLayout> = Vec::new();
    if doc.options.toc && !lay.headings.is_empty() {
        let toc_title_h = 40.0;
        let total_h = toc_title_h + lay.headings.len() as f32 * theme::TOC_LINE;
        let toc_page_count = (total_h / geom.content_h()).ceil().max(1.0) as usize;

        let mut page = PageLayout::default();
        let mut y = geom.top;
        let title = lay.shape(
            &[Run {
                text: "Contents".to_string(),
                style: Style {
                    bold: true,
                    ..Style::default()
                },
                ..Run::default()
            }],
            theme::HEADING_SIZES[1],
            theme::HEADING_SIZES[1] * 1.25,
            FamilyKind::Heading,
            theme::TEXT,
            Some(geom.content_w()),
        )?;
        for line in &title.lines {
            page.items.push(PlacedItem::Line(PlacedLine {
                x: geom.left,
                baseline: y + line.height * 0.8,
                groups: line.groups.clone(),
            }));
            y += line.height;
        }
        y += 12.0;

        let headings = std::mem::take(&mut lay.headings);
        for entry in &headings {
            if y + theme::TOC_LINE > geom.bottom {
                toc_pages.push(std::mem::take(&mut page));
                y = geom.top;
            }
            let baseline = y + theme::TOC_LINE * 0.75;
            let indent = (entry.level.saturating_sub(1)) as f32 * 14.0;
            let title_para = lay.shape(
                &[Run {
                    text: entry.title.clone(),
                    ..Run::default()
                }],
                theme::BODY_SIZE,
                theme::TOC_LINE,
                FamilyKind::Body,
                theme::TEXT,
                Some((geom.content_w() - indent - 40.0).max(40.0)),
            )?;
            if let Some(line) = title_para.lines.first() {
                page.items.push(PlacedItem::Line(PlacedLine {
                    x: geom.left + indent,
                    baseline,
                    groups: line.groups.clone(),
                }));
            }
            let num_para = lay.shape(
                &[Run {
                    text: (entry.page + toc_page_count + 1).to_string(),
                    ..Run::default()
                }],
                theme::BODY_SIZE,
                theme::TOC_LINE,
                FamilyKind::Body,
                theme::MUTED,
                None,
            )?;
            if let Some(line) = num_para.lines.first() {
                page.items.push(PlacedItem::Line(PlacedLine {
                    x: geom.right - line.width,
                    baseline,
                    groups: line.groups.clone(),
                }));
            }
            // The whole entry row links to the heading's page.
            page.annotations.push(PlacedAnnotation {
                rect: RectF::new(geom.left, y, geom.content_w(), theme::TOC_LINE),
                target: AnnotTarget::Internal {
                    page: entry.page,
                    y: entry.y,
                },
            });
            y += theme::TOC_LINE;
        }
        toc_pages.push(page);
        lay.headings = headings;
    }

    let toc_offset = toc_pages.len();

    // Centered page-number footers over all physical pages ("3 / 12").
    if doc.options.page_numbers {
        let total = toc_offset + body_pages.len();
        let footer_y = geom.page_h - (geom.page_h - geom.bottom) * 0.45;
        for (i, page) in toc_pages
            .iter_mut()
            .chain(body_pages.iter_mut())
            .enumerate()
        {
            let para = lay.shape(
                &[Run {
                    text: format!("{} / {}", i + 1, total),
                    ..Run::default()
                }],
                theme::FOOTER_SIZE,
                theme::FOOTER_SIZE * 1.2,
                FamilyKind::Body,
                theme::MUTED,
                None,
            )?;
            if let Some(line) = para.lines.first() {
                page.items.push(PlacedItem::Line(PlacedLine {
                    x: geom.left + (geom.content_w() - line.width) / 2.0,
                    baseline: footer_y,
                    groups: line.groups.clone(),
                }));
            }
        }
    }

    Ok(LayoutResult {
        toc_pages,
        body_pages,
        outline: lay.headings,
        page_width: geom.page_w,
        page_height: geom.page_h,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fonts::bundled_only_font_system;
    use crate::ir::{Cell, PdfMetadata, PdfOptions};

    fn layout(doc: &PdfDocument) -> LayoutResult {
        let mut fs = bundled_only_font_system();
        let mut cache = FontCache::default();
        layout_document(&mut fs, &mut cache, doc).expect("layout")
    }

    fn long_paragraphs(n: usize) -> Vec<Block> {
        let text =
            "这是一个用于分页测试的段落，包含中英文混排 content to wrap across lines. ".repeat(6);
        (0..n)
            .map(|_| Block::Paragraph {
                content: vec![Run {
                    text: text.clone(),
                    ..Run::default()
                }],
            })
            .collect()
    }

    /// A long document flows across pages with no truncation.
    #[test]
    fn long_document_paginates() {
        let doc = PdfDocument {
            blocks: long_paragraphs(30),
            ..PdfDocument::default()
        };
        let result = layout(&doc);
        assert!(
            result.body_pages.len() > 1,
            "30 long paragraphs must paginate, got {} page(s)",
            result.body_pages.len()
        );
    }

    /// An empty document still produces one valid page.
    #[test]
    fn empty_document_yields_one_page() {
        let result = layout(&PdfDocument::default());
        assert_eq!(result.body_pages.len(), 1);
    }

    /// A table split across a page break repeats its header row.
    #[test]
    fn table_header_repeats_across_pages() {
        let mut blocks = long_paragraphs(12);
        let header = vec![
            Cell {
                content: vec![Run {
                    text: "HEADCELLA".to_string(),
                    ..Run::default()
                }],
            },
            Cell {
                content: vec![Run {
                    text: "HEADCELLB".to_string(),
                    ..Run::default()
                }],
            },
        ];
        let rows: Vec<Vec<Cell>> = (0..80)
            .map(|i| {
                vec![
                    Cell {
                        content: vec![Run {
                            text: format!("row {i} left"),
                            ..Run::default()
                        }],
                    },
                    Cell {
                        content: vec![Run {
                            text: format!("row {i} right"),
                            ..Run::default()
                        }],
                    },
                ]
            })
            .collect();
        blocks.push(Block::Table {
            header,
            rows,
            alignments: vec![Alignment::Left, Alignment::Right],
        });
        let doc = PdfDocument {
            blocks,
            ..PdfDocument::default()
        };
        let result = layout(&doc);
        assert!(result.body_pages.len() > 1);
        let header_occurrences = result
            .body_pages
            .iter()
            .flat_map(|p| &p.items)
            .filter(|item| match item {
                PlacedItem::Line(line) => line.groups.iter().any(|g| g.text.contains("HEADCELLA")),
                _ => false,
            })
            .count();
        assert!(
            header_occurrences >= 2,
            "header row must repeat on the continued page, got {header_occurrences}"
        );
    }

    /// Two footnotes referenced on page 1 render at its bottom, with their
    /// references linking to the notes.
    #[test]
    fn footnotes_render_on_reference_page() {
        let doc = PdfDocument {
            metadata: PdfMetadata::default(),
            options: PdfOptions {
                page_numbers: false,
                ..PdfOptions::default()
            },
            blocks: vec![Block::Paragraph {
                content: vec![
                    Run {
                        text: "first".to_string(),
                        footnote: Some(1),
                        ..Run::default()
                    },
                    Run {
                        text: " and ".to_string(),
                        ..Run::default()
                    },
                    Run {
                        text: "second".to_string(),
                        footnote: Some(2),
                        ..Run::default()
                    },
                ],
            }],
            footnotes: vec![
                vec![Run {
                    text: "note one body".to_string(),
                    ..Run::default()
                }],
                vec![Run {
                    text: "note two body".to_string(),
                    ..Run::default()
                }],
            ],
        };
        let result = layout(&doc);
        assert_eq!(result.body_pages.len(), 1);
        let page = &result.body_pages[0];
        let note_lines =
            page.items
                .iter()
                .filter(|item| match item {
                    PlacedItem::Line(line) => line.groups.iter().any(|g| {
                        g.text.contains("note one body") || g.text.contains("note two body")
                    }),
                    _ => false,
                })
                .count();
        assert_eq!(note_lines, 2, "both notes render at the page bottom");
        let internal_links = page
            .annotations
            .iter()
            .filter(|a| matches!(a.target, AnnotTarget::Internal { .. }))
            .count();
        assert_eq!(internal_links, 2, "both references link to their notes");
    }

    /// Enabling the TOC prepends a contents page and records the outline.
    #[test]
    fn toc_prepends_pages() {
        let mut blocks = vec![
            Block::Heading {
                level: 1,
                content: vec![Run {
                    text: "Chapter".to_string(),
                    ..Run::default()
                }],
            },
            Block::Heading {
                level: 2,
                content: vec![Run {
                    text: "Section".to_string(),
                    ..Run::default()
                }],
            },
        ];
        blocks.extend(long_paragraphs(20));
        let doc = PdfDocument {
            options: PdfOptions {
                toc: true,
                ..PdfOptions::default()
            },
            blocks,
            ..PdfDocument::default()
        };
        let result = layout(&doc);
        assert_eq!(result.toc_pages.len(), 1);
        assert!(result.body_pages.len() > 1);
        assert_eq!(result.outline.len(), 2);
        assert_eq!(result.outline[0].title, "Chapter");
    }
}
