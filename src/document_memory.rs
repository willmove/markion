//! Observational retained-size estimates for [`MarkdownDocument`](crate::MarkdownDocument).
//!
//! These helpers never populate derived caches: they only read what is already
//! stored. Estimates are attribution instruments (order-of-magnitude), not
//! allocator-exact sizes.

use crate::model::{
    Heading, InlineSpan, MathSource, PreviewBlock, RichText, VisualBlock, VisualBlockKind,
    VisualInlineRun, VisualRevealGroup,
};
/// One named retained site inside a document's derived state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMemorySite {
    pub name: &'static str,
    pub estimated_bytes: usize,
    pub item_count: usize,
    /// True when the site is currently populated (even if estimated_bytes is 0).
    pub populated: bool,
}

/// Observational breakdown of a document's retained derived state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentMemoryBreakdown {
    pub text_bytes: usize,
    pub sites: Vec<DocumentMemorySite>,
}

impl DocumentMemoryBreakdown {
    pub fn site(&self, name: &str) -> Option<&DocumentMemorySite> {
        self.sites.iter().find(|site| site.name == name)
    }

    pub fn owned_total(&self) -> usize {
        self.text_bytes
            + self
                .sites
                .iter()
                .map(|site| site.estimated_bytes)
                .sum::<usize>()
    }
}

pub(crate) fn string_bytes(s: &str) -> usize {
    s.len() + std::mem::size_of::<String>()
}

pub(crate) fn option_string_bytes(value: Option<&str>) -> usize {
    value.map(string_bytes).unwrap_or(0)
}

fn math_source_bytes(math: &MathSource) -> usize {
    string_bytes(&math.latex) + string_bytes(&math.authored) + std::mem::size_of::<MathSource>()
}

pub(crate) fn rich_text_bytes(text: &RichText) -> usize {
    let mut total = string_bytes(&text.text) + std::mem::size_of::<RichText>();
    total += text
        .spans
        .capacity()
        .saturating_mul(std::mem::size_of::<InlineSpan>());
    for span in &text.spans {
        total += string_bytes(&span.text);
        total += option_string_bytes(span.link.as_deref());
        if let Some(math) = &span.math {
            total += math_source_bytes(math);
        }
    }
    total
}

fn heading_bytes(heading: &Heading) -> usize {
    string_bytes(&heading.title) + string_bytes(&heading.anchor) + std::mem::size_of::<Heading>()
}

pub(crate) fn preview_block_bytes(block: &PreviewBlock) -> usize {
    let base = std::mem::size_of::<PreviewBlock>();
    match block {
        PreviewBlock::Heading { text, .. }
        | PreviewBlock::Paragraph { text, .. }
        | PreviewBlock::ListItem { text, .. }
        | PreviewBlock::FootnoteDefinition { text, .. } => base + rich_text_bytes(text),
        PreviewBlock::BlockQuote { children, .. } => {
            base + children.iter().map(preview_block_bytes).sum::<usize>()
        }
        PreviewBlock::CodeBlock { language, code, .. } => {
            base + option_string_bytes(language.as_deref()) + string_bytes(code)
        }
        PreviewBlock::MathBlock {
            latex,
            authored,
            error,
            ..
        } => {
            base + string_bytes(latex)
                + string_bytes(authored)
                + option_string_bytes(error.as_deref())
        }
        PreviewBlock::Html { html, .. } => base + string_bytes(html),
        PreviewBlock::Image {
            alt, url, title, ..
        } => base + string_bytes(alt) + string_bytes(url) + option_string_bytes(title.as_deref()),
        PreviewBlock::Rule { .. } => base,
        PreviewBlock::Table { rows, .. } => {
            base + rows.iter().flatten().map(rich_text_bytes).sum::<usize>()
        }
    }
}

pub(crate) fn preview_blocks_bytes(blocks: &[PreviewBlock]) -> usize {
    blocks.iter().map(preview_block_bytes).sum::<usize>()
        + blocks
            .len()
            .saturating_mul(std::mem::size_of::<PreviewBlock>())
}

fn visual_run_bytes(run: &VisualInlineRun) -> usize {
    let mut total = string_bytes(&run.visible_text) + std::mem::size_of::<VisualInlineRun>();
    if let Some(math) = &run.math {
        total += math_source_bytes(math);
    }
    if let Some(crate::model::VisualNavigationTarget::Url(url)) = &run.navigation {
        total += string_bytes(url);
    } else if let Some(crate::model::VisualNavigationTarget::Footnote { label }) = &run.navigation {
        total += string_bytes(label);
    }
    total
}

fn visual_reveal_bytes(group: &VisualRevealGroup) -> usize {
    std::mem::size_of::<VisualRevealGroup>()
        + group
            .content_ranges
            .capacity()
            .saturating_mul(std::mem::size_of::<std::ops::Range<usize>>())
}

pub(crate) fn visual_block_bytes(block: &VisualBlock) -> usize {
    let mut total = std::mem::size_of::<VisualBlock>();
    total += block
        .editable_runs
        .iter()
        .map(visual_run_bytes)
        .sum::<usize>();
    total += block
        .reveal_groups
        .iter()
        .map(visual_reveal_bytes)
        .sum::<usize>();
    total += block
        .marker_ranges
        .capacity()
        .saturating_mul(std::mem::size_of::<std::ops::Range<usize>>());
    if let Some(quote) = &block.quote_context {
        total += quote
            .marker_ranges
            .capacity()
            .saturating_mul(std::mem::size_of::<std::ops::Range<usize>>());
    }
    match &block.kind {
        VisualBlockKind::CodeBlock { language } => {
            total += option_string_bytes(language.as_deref());
        }
        VisualBlockKind::MathBlock {
            latex, authored, ..
        } => {
            total += string_bytes(latex) + string_bytes(authored);
        }
        VisualBlockKind::Image {
            alt, url, title, ..
        } => {
            total += string_bytes(alt) + string_bytes(url) + option_string_bytes(title.as_deref());
        }
        VisualBlockKind::Table { rows, .. } => {
            total += rows.iter().flatten().map(rich_text_bytes).sum::<usize>();
        }
        VisualBlockKind::FootnoteDefinition { label } => {
            total += string_bytes(label);
        }
        _ => {}
    }
    total
}

pub(crate) fn visual_blocks_bytes(blocks: &[VisualBlock]) -> usize {
    blocks.iter().map(visual_block_bytes).sum::<usize>()
        + blocks
            .len()
            .saturating_mul(std::mem::size_of::<VisualBlock>())
}

pub(crate) fn headings_bytes(headings: &[Heading]) -> usize {
    headings.iter().map(heading_bytes).sum::<usize>()
        + headings
            .len()
            .saturating_mul(std::mem::size_of::<Heading>())
}

/// Collect image URL references reachable from already-populated derived caches
/// without forcing a parse.
pub(crate) fn image_refs_from_preview(blocks: &[PreviewBlock], out: &mut Vec<String>) {
    for block in blocks {
        match block {
            PreviewBlock::Image { url, .. } => out.push(url.clone()),
            PreviewBlock::Html { html, .. } => {
                for url in html_img_srcs(html) {
                    out.push(url);
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn image_refs_from_visual(blocks: &[VisualBlock], out: &mut Vec<String>) {
    for block in blocks {
        if let VisualBlockKind::Image { url, .. } = &block.kind {
            out.push(url.clone());
        }
    }
}

fn html_img_srcs(html: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut rest = html;
    while let Some(img_at) = rest.find("<img") {
        rest = &rest[img_at + 4..];
        let Some(src_at) = rest.find("src=") else {
            continue;
        };
        rest = &rest[src_at + 4..];
        let quote = rest.chars().next();
        if quote != Some('"') && quote != Some('\'') {
            continue;
        }
        let quote = quote.unwrap();
        rest = &rest[1..];
        if let Some(end) = rest.find(quote) {
            urls.push(rest[..end].to_string());
            rest = &rest[end + 1..];
        }
    }
    urls
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::MarkdownDocument;

    #[test]
    fn fresh_document_reports_empty_derived_caches() {
        let doc = MarkdownDocument::from_text("# Hello\n\nParagraph.");
        let version = doc.version();
        let breakdown = doc.memory_breakdown();
        assert_eq!(breakdown.text_bytes, doc.text().len());
        for name in [
            "preview_blocks",
            "visual_blocks",
            "outline",
            "stats",
            "line_count",
            "source_mapped_cache",
        ] {
            let site = breakdown.site(name).expect(name);
            assert!(!site.populated, "{name} should be unpopulated");
            assert_eq!(site.estimated_bytes, 0, "{name} should report zero bytes");
        }
        assert_eq!(doc.version(), version);
        // Still unpopulated after accounting.
        let again = doc.memory_breakdown();
        assert!(!again.site("preview_blocks").unwrap().populated);
        assert!(!again.site("visual_blocks").unwrap().populated);
    }

    #[test]
    fn populated_caches_each_report_nonzero() {
        let doc = MarkdownDocument::from_text(
            "# Title\n\nParagraph with **bold**.\n\n```rust\nfn main() {}\n```\n\n$$x^2$$\n",
        );
        let _ = doc.preview_blocks_shared();
        let _ = doc.visual_blocks_shared();
        let _ = doc.outline();
        let _ = doc.stats();
        let _ = doc.line_count();
        let breakdown = doc.memory_breakdown();
        for name in [
            "preview_blocks",
            "visual_blocks",
            "outline",
            "stats",
            "line_count",
            "source_mapped_cache",
        ] {
            let site = breakdown.site(name).expect(name);
            assert!(site.populated, "{name} should be populated");
            assert!(
                site.estimated_bytes > 0 || name == "stats" || name == "line_count",
                "{name} should report a size (got {})",
                site.estimated_bytes
            );
        }
        // stats / line_count are tiny structs; still require populated + item_count.
        assert!(breakdown.site("stats").unwrap().item_count >= 1);
        assert!(breakdown.site("line_count").unwrap().item_count >= 1);
        assert!(breakdown.site("preview_blocks").unwrap().estimated_bytes > 0);
        assert!(breakdown.site("visual_blocks").unwrap().estimated_bytes > 0);
        assert!(
            breakdown
                .site("source_mapped_cache")
                .unwrap()
                .estimated_bytes
                > 0
        );
    }

    #[test]
    fn evict_derived_caches_clears_without_bumping_version() {
        let mut doc = MarkdownDocument::from_text(
            "# Title\n\nParagraph with **bold**.\n\n```rust\nfn main() {}\n```\n\n$$x^2$$\n",
        );
        let _ = doc.preview_blocks_shared();
        let _ = doc.visual_blocks_shared();
        let _ = doc.outline();
        let _ = doc.stats();
        let _ = doc.line_count();
        let version = doc.version();
        let text = doc.text().to_string();
        let dirty = doc.is_dirty();
        let path = doc.path().map(PathBuf::from);

        doc.evict_derived_caches();

        assert_eq!(doc.version(), version);
        assert_eq!(doc.text(), text);
        assert_eq!(doc.is_dirty(), dirty);
        assert_eq!(doc.path(), path.as_deref());
        let breakdown = doc.memory_breakdown();
        for name in [
            "preview_blocks",
            "visual_blocks",
            "outline",
            "stats",
            "line_count",
            "source_mapped_cache",
        ] {
            let site = breakdown.site(name).expect(name);
            assert!(
                !site.populated,
                "{name} should be unpopulated after eviction"
            );
            assert_eq!(site.estimated_bytes, 0, "{name} should report zero bytes");
        }

        // Accessors can repopulate after dormancy.
        let _ = doc.preview_blocks_shared();
        let _ = doc.visual_blocks_shared();
        assert!(
            doc.memory_breakdown()
                .site("preview_blocks")
                .unwrap()
                .populated
        );
        assert!(
            doc.memory_breakdown()
                .site("visual_blocks")
                .unwrap()
                .populated
        );
        assert_eq!(doc.version(), version);
    }
}
