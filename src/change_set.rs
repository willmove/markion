//! Conservative mutation effects shared by presentation subscribers.
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeKind {
    TextOnly,
    InlineStructure,
    BlockStructure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeSet {
    pub before_version: u64,
    pub after_version: u64,
    pub old_range: Range<usize>,
    pub new_range: Range<usize>,
    pub kind: ChangeKind,
    pub outline_changed: bool,
    pub references_changed: bool,
    pub assets_changed: bool,
}

impl ChangeSet {
    pub(crate) fn range(
        source: &str,
        range: Range<usize>,
        replacement: &str,
        version: u64,
    ) -> Self {
        let line_start = source[..range.start].rfind('\n').map_or(0, |n| n + 1);
        let line_end = source[range.end..]
            .find('\n')
            .map_or(source.len(), |n| range.end + n);
        let context = &source[line_start..line_end];
        let old = &source[range.clone()];
        let at_prefix = source[line_start..range.start]
            .bytes()
            .all(|b| matches!(b, b' ' | b'\t'));
        let block = old.contains(['\r', '\n'])
            || replacement.contains(['\r', '\n'])
            || context
                .trim_start()
                .starts_with(['#', '>', '-', '*', '+', '|', '`', '~', '<', '['])
            || (at_prefix
                && replacement
                    .trim_start()
                    .starts_with(['#', '>', '-', '*', '+', '|', '`', '~', '<', '[']));
        let inline = old.contains([
            '!', '*', '_', '`', '[', ']', '(', ')', '$', '<', '>', '~', '^', '=',
        ]) || replacement.contains([
            '!', '*', '_', '`', '[', ']', '(', ')', '$', '<', '>', '~', '^', '=',
        ]);
        let references = context.trim_start().starts_with('[')
            || context.contains("]:")
            || replacement.contains("]:");
        let assets = references
            || context.contains("![")
            || context.contains("<img")
            || replacement.contains("![")
            || replacement.contains("<img")
            || (replacement.contains('!') && context.contains('['));
        Self {
            before_version: version,
            after_version: version.wrapping_add(1),
            new_range: range.start..range.start + replacement.len(),
            old_range: range,
            kind: if block {
                ChangeKind::BlockStructure
            } else if inline {
                ChangeKind::InlineStructure
            } else {
                ChangeKind::TextOnly
            },
            outline_changed: block,
            references_changed: references,
            assets_changed: assets,
        }
    }

    pub(crate) fn whole(old_len: usize, new_len: usize, version: u64) -> Self {
        Self {
            before_version: version,
            after_version: version.wrapping_add(1),
            old_range: 0..old_len,
            new_range: 0..new_len,
            kind: ChangeKind::BlockStructure,
            outline_changed: true,
            references_changed: true,
            assets_changed: true,
        }
    }

    pub fn map_unchanged(&self, range: Range<usize>) -> Option<Range<usize>> {
        if range.end <= self.old_range.start {
            return Some(range);
        }
        if range.start < self.old_range.end {
            return None;
        }
        let shift = self.new_range.len() as isize - self.old_range.len() as isize;
        Some(range.start.checked_add_signed(shift)?..range.end.checked_add_signed(shift)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn effects_map_exact_ranges_and_classify_structural_or_global_edits() {
        let change = ChangeSet::range("first\n\nsecond", 1..2, "中文", 7);
        assert_eq!(change.kind, ChangeKind::TextOnly);
        assert_eq!(change.map_unchanged(7..13), Some(12..18));
        assert_eq!(change.map_unchanged(0..5), None);
        let change = ChangeSet::range("[ref]: old", 7..10, "new", 8);
        assert!(change.references_changed && change.assets_changed);
        assert!(ChangeSet::range("# title", 2..2, "new", 9).outline_changed);
        assert!(ChangeSet::range("title", 0..0, "# ", 9).outline_changed);
    }
}
