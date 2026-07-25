//! Retained-memory attribution for open tabs and process-global render caches.
//!
//! Accounting is observational: reading a report never populates a cache or
//! mutates document versions. Estimates are order-of-magnitude instruments.

use super::*;

/// How a site contributes to the report total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SiteContribution {
    /// Bytes are retained by Markion and counted in the total.
    Owned,
    /// A shared handle whose pointee is counted elsewhere; appears in the
    /// report but contributes zero to the total.
    Shared,
    /// Storage owned outside Markion and not enumerable; listed without a
    /// fabricated byte figure.
    External,
}

/// One named retention site with its estimate and the counts behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MemorySite {
    pub(super) name: String,
    pub(super) estimated_bytes: usize,
    pub(super) counts: Vec<(String, usize)>,
    pub(super) contribution: SiteContribution,
}

impl MemorySite {
    pub(super) fn owned(
        name: impl Into<String>,
        estimated_bytes: usize,
        counts: Vec<(String, usize)>,
    ) -> Self {
        Self {
            name: name.into(),
            estimated_bytes,
            counts,
            contribution: SiteContribution::Owned,
        }
    }

    pub(super) fn shared(
        name: impl Into<String>,
        estimated_bytes: usize,
        counts: Vec<(String, usize)>,
    ) -> Self {
        Self {
            name: name.into(),
            estimated_bytes,
            counts,
            contribution: SiteContribution::Shared,
        }
    }

    pub(super) fn external(name: impl Into<String>, counts: Vec<(String, usize)>) -> Self {
        Self {
            name: name.into(),
            estimated_bytes: 0,
            counts,
            contribution: SiteContribution::External,
        }
    }

    pub(super) fn contributes_bytes(&self) -> usize {
        match self.contribution {
            SiteContribution::Owned => self.estimated_bytes,
            SiteContribution::Shared | SiteContribution::External => 0,
        }
    }
}

/// Complete per-site report for the running application (or a harness profile).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct MemoryReport {
    pub(super) tab_sites: Vec<MemorySite>,
    pub(super) global_sites: Vec<MemorySite>,
    /// OS-level process footprint; not a contributing site.
    pub(super) process_footprint: ProcessFootprint,
    /// Platform label for the footprint section (e.g. `"windows"`).
    pub(super) process_platform: &'static str,
    /// Explicit note when a remainder is known to be externally owned and
    /// unaccounted (e.g. GPUI's image asset table).
    pub(super) unaccounted_note: Option<&'static str>,
}

impl MemoryReport {
    pub(super) fn per_tab_total(&self) -> usize {
        self.tab_sites
            .iter()
            .map(MemorySite::contributes_bytes)
            .sum()
    }

    pub(super) fn global_total(&self) -> usize {
        self.global_sites
            .iter()
            .map(MemorySite::contributes_bytes)
            .sum()
    }

    pub(super) fn accounted_total(&self) -> usize {
        self.per_tab_total() + self.global_total()
    }

    /// Site figures only — process counters may differ between consecutive samples.
    pub(super) fn sites_equal(&self, other: &Self) -> bool {
        self.tab_sites == other.tab_sites
            && self.global_sites == other.global_sites
            && self.unaccounted_note == other.unaccounted_note
    }

    pub(super) fn site_names(&self) -> Vec<&str> {
        self.tab_sites
            .iter()
            .chain(self.global_sites.iter())
            .map(|site| site.name.as_str())
            .collect()
    }

    pub(super) fn find_site(&self, name: &str) -> Option<&MemorySite> {
        self.tab_sites
            .iter()
            .chain(self.global_sites.iter())
            .find(|site| site.name == name)
    }

    pub(super) fn format_log(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Markion memory report ===\n");
        out.push_str(&format!(
            "accounted_total={}  per_tab={}  global={}\n",
            self.accounted_total(),
            self.per_tab_total(),
            self.global_total()
        ));
        out.push_str("--- per-tab sites ---\n");
        for site in &self.tab_sites {
            out.push_str(&format_site(site));
        }
        out.push_str("--- global sites ---\n");
        for site in &self.global_sites {
            out.push_str(&format_site(site));
        }
        out.push_str(
            &self
                .process_footprint
                .format_log_section(self.process_platform),
        );
        if let Some(note) = self.unaccounted_note {
            out.push_str("--- unaccounted ---\n");
            out.push_str(note);
            out.push('\n');
        }
        out
    }
}

fn format_site(site: &MemorySite) -> String {
    let kind = match site.contribution {
        SiteContribution::Owned => "owned",
        SiteContribution::Shared => "shared",
        SiteContribution::External => "external",
    };
    let counts = site
        .counts
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "  [{kind}] {} bytes={} {}\n",
        site.name, site.estimated_bytes, counts
    )
}

/// Sites that can report their own retained size.
pub(super) trait MemoryFootprint {
    fn memory_sites(&self) -> Vec<MemorySite>;
}

/// Per-line structural cost of a retained `gpui::WrappedLine`.
///
/// `WrappedLine` embeds `SmallVec<[DecorationRun; 32]>` inline (~3 KB) even when
/// Markion only supplies one to three decoration runs. Glyph data behind the
/// shared layout Arc is not publicly enumerable, so this constant is the
/// attribution instrument for the shaped-line site.
pub(super) const SHAPED_LINE_STRUCTURAL_BYTES: usize = 3_200;

impl MemoryFootprint for EditorTab {
    fn memory_sites(&self) -> Vec<MemorySite> {
        let mut sites = Vec::new();
        let prefix = "tab";

        let breakdown = self.document.memory_breakdown();
        sites.push(MemorySite::owned(
            format!("{prefix}.document_text"),
            breakdown.text_bytes,
            vec![("bytes".into(), breakdown.text_bytes)],
        ));
        for site in &breakdown.sites {
            sites.push(MemorySite::owned(
                format!("{prefix}.document.{}", site.name),
                site.estimated_bytes,
                vec![
                    ("items".into(), site.item_count),
                    ("populated".into(), usize::from(site.populated)),
                ],
            ));
        }

        let (undo_bytes, undo_entries) = history_bytes(&self.undo_stack);
        sites.push(MemorySite::owned(
            format!("{prefix}.undo_stack"),
            undo_bytes,
            vec![("entries".into(), undo_entries)],
        ));
        let (redo_bytes, redo_entries) = history_bytes(&self.redo_stack);
        sites.push(MemorySite::owned(
            format!("{prefix}.redo_stack"),
            redo_bytes,
            vec![("entries".into(), redo_entries)],
        ));

        let display_bytes = self
            .display_text_cache
            .borrow()
            .as_ref()
            .map(|(_, text)| text.len())
            .unwrap_or(0);
        sites.push(MemorySite::owned(
            format!("{prefix}.display_text_cache"),
            display_bytes,
            vec![("populated".into(), usize::from(display_bytes > 0))],
        ));

        let line_offsets_bytes = self
            .line_offsets_cache
            .borrow()
            .as_ref()
            .map(|(_, offsets)| offsets.len() * std::mem::size_of::<usize>())
            .unwrap_or(0);
        sites.push(MemorySite::owned(
            format!("{prefix}.line_offsets_cache"),
            line_offsets_bytes,
            vec![(
                "entries".into(),
                self.line_offsets_cache
                    .borrow()
                    .as_ref()
                    .map(|(_, o)| o.len())
                    .unwrap_or(0),
            )],
        ));

        let measured_populated = self.measured_height_cache.borrow().is_some();
        sites.push(MemorySite::owned(
            format!("{prefix}.measured_height_cache"),
            if measured_populated {
                std::mem::size_of::<Pixels>()
            } else {
                0
            },
            vec![("populated".into(), usize::from(measured_populated))],
        ));

        let shaped_lines = self.last_lines.len();
        let shaped_bytes = shaped_lines.saturating_mul(SHAPED_LINE_STRUCTURAL_BYTES);
        sites.push(MemorySite::owned(
            format!("{prefix}.shaped_lines"),
            shaped_bytes,
            vec![("lines".into(), shaped_lines)],
        ));

        // Tab-level Arc handles to document derived blocks are shared; the
        // pointee is counted under document.preview_blocks / visual_blocks.
        sites.push(MemorySite::shared(
            format!("{prefix}.preview_list_blocks"),
            0,
            vec![("blocks".into(), self.preview_list_blocks.len())],
        ));
        sites.push(MemorySite::shared(
            format!("{prefix}.visual_list_blocks"),
            0,
            vec![("blocks".into(), self.visual_list_blocks.len())],
        ));

        sites
    }
}

fn history_bytes(stack: &[UndoEntry]) -> (usize, usize) {
    let mut bytes = 0usize;
    for entry in stack {
        match entry {
            UndoEntry::Full(snapshot) => {
                bytes = bytes.saturating_add(snapshot.document.text().len());
            }
            UndoEntry::Diff(diff) => {
                bytes = bytes.saturating_add(diff.insert.len());
            }
        }
    }
    (bytes, stack.len())
}

impl MemoryFootprint for DiagramCache {
    fn memory_sites(&self) -> Vec<MemorySite> {
        let mut entry_count = 0usize;
        let mut pending = 0usize;
        let mut ready = 0usize;
        let mut raster_bytes = 0usize;
        for entry in self.entries.values() {
            entry_count += 1;
            match entry {
                DiagramCacheEntry::Pending => pending += 1,
                DiagramCacheEntry::Ready(image, _) => {
                    ready += 1;
                    raster_bytes = raster_bytes.saturating_add(render_image_bytes(image));
                }
                DiagramCacheEntry::Error(_) => {}
            }
        }
        // Key source strings are also retained.
        let key_bytes: usize = self
            .entries
            .keys()
            .map(|key| key.backend_id.len() + key.source.len())
            .sum();
        vec![MemorySite::owned(
            "global.diagram_cache",
            self.completed_bytes.saturating_add(key_bytes),
            vec![
                ("entries".into(), entry_count),
                ("pending".into(), pending),
                ("ready".into(), ready),
                ("completed_bytes".into(), self.completed_bytes),
                ("budget_bytes".into(), self.max_completed_bytes),
                ("raster_bytes".into(), raster_bytes),
            ],
        )]
    }
}

impl MemoryFootprint for PreviewImageCache {
    fn memory_sites(&self) -> Vec<MemorySite> {
        let (entries, pending, ready, completed_bytes, budget_bytes) = self.accounting_counts();
        vec![MemorySite::owned(
            "global.preview_image_cache",
            completed_bytes,
            vec![
                ("entries".into(), entries),
                ("pending".into(), pending),
                ("ready".into(), ready),
                ("completed_bytes".into(), completed_bytes),
                ("budget_bytes".into(), budget_bytes),
            ],
        )]
    }
}

impl MemoryFootprint for MathCache {
    fn memory_sites(&self) -> Vec<MemorySite> {
        let entry_count = self.entries.len();
        let key_bytes: usize = self.entries.keys().map(|key| key.latex.len()).sum();
        vec![MemorySite::owned(
            "global.math_cache",
            self.completed_bytes.saturating_add(key_bytes),
            vec![
                ("entries".into(), entry_count),
                ("completed_bytes".into(), self.completed_bytes),
            ],
        )]
    }
}

pub(super) fn highlight_cache_sites(cache: &HighlightCache) -> Vec<MemorySite> {
    let map = cache.borrow();
    let entry_count = map.len();
    let mut key_bytes = 0usize;
    let mut value_bytes = 0usize;
    for ((lang, code), spans) in map.iter() {
        key_bytes = key_bytes
            .saturating_add(lang.as_ref().map(|s| s.len()).unwrap_or(0))
            .saturating_add(code.len());
        for line in spans.iter() {
            for span in line {
                value_bytes = value_bytes.saturating_add(span.text.len());
            }
        }
    }
    vec![MemorySite::owned(
        "global.highlight_cache",
        key_bytes.saturating_add(value_bytes),
        vec![
            ("entries".into(), entry_count),
            ("key_bytes".into(), key_bytes),
            ("value_bytes".into(), value_bytes),
        ],
    )]
}

fn render_image_bytes(image: &RenderImage) -> usize {
    let mut total = 0usize;
    for frame in 0..image.frame_count() {
        if let Some(bytes) = image.as_bytes(frame) {
            total = total.saturating_add(bytes.len());
        } else {
            let size = image.size(frame);
            let w: i32 = size.width.into();
            let h: i32 = size.height.into();
            total = total.saturating_add((w.max(0) as usize).saturating_mul(h.max(0) as usize) * 4);
        }
    }
    total
}

/// Document content profiles used by the headless attribution harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MemoryProfile {
    PlainLong,
    Images,
    Diagrams,
    Math,
    Code,
}

impl MemoryProfile {
    pub(super) fn all() -> &'static [Self] {
        &[
            Self::PlainLong,
            Self::Images,
            Self::Diagrams,
            Self::Math,
            Self::Code,
        ]
    }

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::PlainLong => "plain_long",
            Self::Images => "with_images",
            Self::Diagrams => "with_diagrams",
            Self::Math => "with_math",
            Self::Code => "with_code",
        }
    }

    pub(super) fn markdown(self) -> &'static str {
        match self {
            Self::PlainLong => {
                include_str!("../../examples/memory_fixtures/plain_long.md")
            }
            Self::Images => include_str!("../../examples/memory_fixtures/with_images.md"),
            Self::Diagrams => {
                include_str!("../../examples/memory_fixtures/with_diagrams.md")
            }
            Self::Math => include_str!("../../examples/memory_fixtures/with_math.md"),
            Self::Code => include_str!("../../examples/memory_fixtures/with_code.md"),
        }
    }

    /// Load a harness document. Image fixtures open from disk so relative
    /// `fixture.png` paths resolve against `examples/memory_fixtures/`.
    pub(super) fn document(self) -> MarkdownDocument {
        match self {
            Self::Images => {
                let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("examples/memory_fixtures/with_images.md");
                MarkdownDocument::open(&path)
                    .unwrap_or_else(|_| MarkdownDocument::from_text(self.markdown()))
            }
            _ => MarkdownDocument::from_text(self.markdown()),
        }
    }
}

/// How deeply to warm derived state when building a harness profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MemoryWarmup {
    /// Only load document text (no derived caches).
    TextOnly,
    /// Populate Visual Edit blocks (default view mode).
    VisualEdit,
    /// Populate preview blocks (Split/Read) without shaping editor lines.
    Preview,
    /// Populate preview blocks and leave shaped-line accounting at zero unless
    /// the caller synthesizes `last_lines`.
    Split,
}

impl MarkionApp {
    /// Build `tab_count` tabs from a fixture profile and warm derived state.
    pub(super) fn load_memory_profile(
        &mut self,
        profile: MemoryProfile,
        tab_count: usize,
        warmup: MemoryWarmup,
        cx: &mut Context<Self>,
    ) {
        assert!(tab_count >= 1);
        let first = profile.document();
        self.replace_active_tab(first, cx);
        self.warm_active_tab(warmup, cx);
        for _ in 1..tab_count {
            let document = profile.document();
            self.open_in_new_tab(document, cx);
            self.warm_active_tab(warmup, cx);
        }
        self.sync_and_persist_session();
    }

    pub(super) fn warm_active_tab(&mut self, warmup: MemoryWarmup, cx: &mut Context<Self>) {
        let document_dir = self
            .active_tab()
            .document
            .path()
            .and_then(Path::parent)
            .map(PathBuf::from);
        let (preview, visual) = match warmup {
            MemoryWarmup::TextOnly => {
                return;
            }
            MemoryWarmup::VisualEdit => {
                let blocks = self.active_tab().document.visual_blocks_shared();
                self.active_tab_mut().sync_visual_list(&blocks);
                (std::sync::Arc::new(Vec::new()), blocks)
            }
            MemoryWarmup::Preview | MemoryWarmup::Split => {
                let blocks = self.active_tab().document.preview_blocks_shared();
                self.active_tab_mut().sync_preview_list(&blocks);
                let _ = self.active_tab().document.outline();
                let _ = self.active_tab().document.stats();
                (blocks, std::sync::Arc::new(Vec::new()))
            }
        };
        let active = self.active_tab;
        self.refresh_tab_image_claims(active, &preview, &visual, document_dir.as_deref(), cx);
        self.ensure_preview_images(&preview, &visual, document_dir.as_deref(), cx);
        self.ensure_diagram_renders(&preview, &visual, cx);
    }

    /// Assemble a complete per-site retained-memory report.
    pub(super) fn memory_report(&self) -> MemoryReport {
        let tab_sites = self
            .tabs
            .iter()
            .enumerate()
            .flat_map(|(index, tab)| {
                tab.memory_sites().into_iter().map(move |mut site| {
                    let rest = site.name.strip_prefix("tab.").unwrap_or(site.name.as_str());
                    site.name = format!("tabs[{index}].{rest}");
                    site
                })
            })
            .collect();

        let mut global_sites = Vec::new();
        global_sites.extend(self.preview_image_cache.memory_sites());
        global_sites.extend(self.diagram_cache.memory_sites());
        global_sites.extend(self.math_cache.memory_sites());
        global_sites.extend(highlight_cache_sites(&self.highlight_cache));

        MemoryReport {
            tab_sites,
            global_sites,
            process_footprint: ProcessFootprint::sample(),
            process_platform: process_footprint_platform(),
            unaccounted_note: Some(
                "Layer A fixed baseline (GPUI renderer, grammar/font databases) is not attributed; compare process footprint counters to accounted_total using the interpretation rules in docs/memory-retention.md.",
            ),
        }
    }

    pub(super) fn report_memory(
        &mut self,
        _: &ReportMemory,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let report = self.memory_report();
        let body = report.format_log();
        tracing::info!(target: "markion::memory", "{body}");
        self.status = t(self.language, Msg::StatusReady).into();
        self.active_menu = None;
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use markion_diagram::DiagramTheme;

    #[test]
    fn all_profiles_and_warmups_are_reachable() {
        // Keep every harness variant referenced so dead_code stays honest.
        for profile in MemoryProfile::all() {
            assert!(!profile.markdown().is_empty(), "{}", profile.name());
        }
        let _ = MemoryWarmup::TextOnly;
        let _ = MemoryWarmup::Split;
    }

    #[test]
    fn empty_report_totals_zero() {
        let report = MemoryReport::default();
        assert_eq!(report.per_tab_total(), 0);
        assert_eq!(report.global_total(), 0);
        assert_eq!(report.accounted_total(), 0);
    }

    #[test]
    fn process_footprint_does_not_contribute_to_accounted_total() {
        let mut report = MemoryReport::default();
        report.tab_sites.push(MemorySite::owned(
            "tabs[0].document_text",
            500,
            vec![("bytes".into(), 500)],
        ));
        report.global_sites.push(MemorySite::owned(
            "global.math_cache",
            1_024,
            vec![("entries".into(), 1)],
        ));
        report.process_footprint = ProcessFootprint {
            resident_current: Some(50_000_000),
            resident_peak: Some(80_000_000),
            commit_current: Some(60_000_000),
            commit_peak: Some(90_000_000),
        };
        report.process_platform = "windows";
        assert_eq!(report.accounted_total(), 1_524);
        assert_eq!(
            report.accounted_total(),
            report.per_tab_total() + report.global_total()
        );
        assert!(!report.site_names().iter().any(|n| n.contains("resident")));
        let log = report.format_log();
        assert!(log.contains("--- process footprint (windows) ---"));
        assert!(log.contains("resident_current=50000000"));
        assert!(log.contains("resident_peak=80000000"));
        // Footprint section must appear after site lists.
        let sites_end = log
            .find("--- process footprint")
            .expect("footprint section");
        assert!(log[..sites_end].contains("--- global sites ---"));
    }

    #[test]
    fn external_site_appears_but_contributes_zero() {
        let mut report = MemoryReport::default();
        report.global_sites.push(MemorySite::external(
            "global.unattributed_baseline",
            vec![("note".into(), 1)],
        ));
        report.global_sites.push(MemorySite::owned(
            "global.math_cache",
            1_024,
            vec![("entries".into(), 1)],
        ));
        assert!(report.find_site("global.unattributed_baseline").is_some());
        assert_eq!(
            report
                .find_site("global.unattributed_baseline")
                .unwrap()
                .contributes_bytes(),
            0
        );
        assert_eq!(report.global_total(), 1_024);
    }

    #[test]
    fn shared_site_excluded_from_total() {
        let mut report = MemoryReport::default();
        report.tab_sites.push(MemorySite::owned(
            "tabs[0].document_text",
            100,
            vec![("bytes".into(), 100)],
        ));
        report.tab_sites.push(MemorySite::shared(
            "tabs[0].preview_list_blocks",
            50_000,
            vec![("blocks".into(), 10)],
        ));
        assert_eq!(report.per_tab_total(), 100);
    }

    #[test]
    fn visual_only_tab_reports_zero_shaped_lines() {
        let mut tab = EditorTab::new(MarkdownDocument::from_text(
            "# Hello\n\nA paragraph with **bold** text.\n",
        ));
        let blocks = tab.document.visual_blocks_shared();
        tab.sync_visual_list(&blocks);
        let sites = tab.memory_sites();
        let shaped = sites
            .iter()
            .find(|site| site.name.ends_with("shaped_lines"))
            .expect("shaped_lines site");
        assert_eq!(shaped.estimated_bytes, 0);
        assert_eq!(shaped.counts[0].1, 0);
        let visual_shared = sites
            .iter()
            .find(|site| site.name.ends_with("visual_list_blocks"))
            .expect("visual_list_blocks site");
        assert_eq!(visual_shared.contribution, SiteContribution::Shared);
        assert!(visual_shared.counts[0].1 > 0);
        // Document visual blocks are owned and non-zero; the shared handle
        // contributes nothing, so the pointee is counted once.
        let visual_owned = sites
            .iter()
            .find(|site| site.name.ends_with("document.visual_blocks"))
            .expect("document.visual_blocks");
        assert!(visual_owned.estimated_bytes > 0);
        assert_eq!(
            sites
                .iter()
                .map(MemorySite::contributes_bytes)
                .sum::<usize>(),
            sites
                .iter()
                .filter(|s| s.contribution == SiteContribution::Owned)
                .map(|s| s.estimated_bytes)
                .sum::<usize>()
        );
    }

    #[test]
    fn diagram_and_math_and_highlight_caches_grow_from_empty() {
        let empty_diagram = DiagramCache::new(8);
        let empty_site = &empty_diagram.memory_sites()[0];
        assert_eq!(empty_site.counts[0].1, 0);
        assert_eq!(empty_site.estimated_bytes, 0);

        let mut diagram = DiagramCache::new(8);
        let key = DiagramCacheKey {
            backend_id: "mermaid".into(),
            source: "A --> B".into(),
            theme: DiagramTheme::Light,
        };
        assert!(diagram.reserve_pending(key.clone()));
        let pending_site = &diagram.memory_sites()[0];
        assert_eq!(pending_site.counts[0].1, 1);
        assert_eq!(
            pending_site
                .counts
                .iter()
                .find(|(k, _)| k == "pending")
                .map(|(_, v)| *v),
            Some(1)
        );

        let empty_math = MathCache::new(8);
        assert_eq!(empty_math.memory_sites()[0].counts[0].1, 0);

        let highlight: HighlightCache = RefCell::new(HashMap::new());
        assert_eq!(highlight_cache_sites(&highlight)[0].counts[0].1, 0);
        highlight.borrow_mut().insert(
            (Some("rust".into()), "fn main() {}".into()),
            Rc::new(vec![vec![HighlightedSpan {
                text: "fn".into(),
                kind: HighlightKind::Keyword,
            }]]),
        );
        let hl = &highlight_cache_sites(&highlight)[0];
        assert_eq!(hl.counts[0].1, 1);
        assert!(hl.estimated_bytes > 0);
    }
}
