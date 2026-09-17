//! Pure data and source-tracking primitives for explicit image operations.
//!
//! Network, process, filesystem, and GPUI concerns live in the application
//! layer. Keeping the operation contract here makes planning and stale-target
//! behavior independently testable.

use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
    path::PathBuf,
};

use pulldown_cmark::{BrokenLink, CowStr, Event, LinkType, Parser, Tag};

use crate::{
    DocumentInstanceId, ImagePreferences, MarkdownDocument, MutationOrigin, inline_edit,
    parse::markdown_options,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageInput {
    Local(PathBuf),
    Bytes {
        stem: String,
        extension: String,
        bytes: Vec<u8>,
    },
    Remote(String),
    DataUri(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOperationKind {
    Insert,
    Upload,
    SaveToResources,
    UploadLocalDocumentImages,
    DownloadRemoteDocumentImages,
    OrganizeDocumentImages,
    TestUpload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ImageOccurrenceId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageOccurrenceTarget {
    pub id: ImageOccurrenceId,
    pub source_range: Range<usize>,
    pub expected_source: String,
    pub semantic_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageReferenceKind {
    Full,
    Collapsed,
    Shortcut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOccurrenceSyntax {
    MarkdownInline,
    MarkdownReference(ImageReferenceKind),
    HtmlSrc { quote: char },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageOccurrence {
    pub id: ImageOccurrenceId,
    pub semantic_url: String,
    pub full_range: Range<usize>,
    pub destination_range: Option<Range<usize>>,
    pub expected_source: String,
    pub authored_alt: String,
    pub resolved_title: Option<String>,
    pub syntax: ImageOccurrenceSyntax,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSkipReason {
    UnresolvedReference,
    MissingHtmlSrc,
    AmbiguousHtmlSrc,
    UnsupportedAuthoredForm,
    MissingDocumentBase,
    MissingLocalSource,
    UnsupportedScheme,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedImageOccurrence {
    pub range: Range<usize>,
    pub reason: ImageSkipReason,
    pub expected_source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageOccurrenceScan {
    pub occurrences: Vec<ImageOccurrence>,
    pub skipped: Vec<SkippedImageOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageSourceReplacement {
    pub range: Range<usize>,
    pub expected_source: String,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImagePlanSelection {
    Single(ImageOccurrenceId),
    Occurrences(Vec<ImageOccurrenceId>),
    CurrentDocument,
    InsertedFragment(Range<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInputGroup {
    pub input: ImageInput,
    pub occurrences: Vec<ImageOccurrenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImagePlanIssue {
    pub occurrence: Option<ImageOccurrenceId>,
    pub reason: ImageSkipReason,
    pub source: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageOperationPlan {
    pub selected_occurrences: Vec<ImageOccurrence>,
    pub input_groups: Vec<ImageInputGroup>,
    pub issues: Vec<ImagePlanIssue>,
}

#[derive(Debug)]
pub struct ImageApplicationPlan {
    pub mutation: Option<crate::CheckedMutation>,
    pub edits: Vec<Range<usize>>,
    pub replacement_lengths: Vec<usize>,
    pub applied: Vec<ImageOccurrenceId>,
    pub stale: Vec<ImageOccurrenceId>,
}

pub fn build_image_application_plan(
    document: &MarkdownDocument,
    operation: &PendingImageOperation,
    outputs: &HashMap<ImageOccurrenceId, String>,
) -> ImageApplicationPlan {
    let mut applied = Vec::new();
    let mut stale = Vec::new();
    let mut edits = Vec::new();
    if operation.config.document != document.instance_id() || !operation.auto_apply_allowed {
        stale.extend(outputs.keys().copied());
        return ImageApplicationPlan {
            mutation: None,
            edits: Vec::new(),
            replacement_lengths: Vec::new(),
            applied,
            stale,
        };
    }
    for occurrence in &operation.config.occurrences {
        let Some(url) = outputs.get(&occurrence.id) else {
            continue;
        };
        let Some(anchor) = operation
            .anchors
            .iter()
            .find(|anchor| anchor.occurrence == occurrence.id)
        else {
            stale.push(occurrence.id);
            continue;
        };
        if !anchor.valid
            || document.text().get(anchor.range.clone()) != Some(anchor.expected_source.as_str())
        {
            stale.push(occurrence.id);
            continue;
        }
        let original = occurrence.replacement(url);
        let relative_start = original
            .range
            .start
            .saturating_sub(occurrence.full_range.start);
        let relative_end = original
            .range
            .end
            .saturating_sub(occurrence.full_range.start);
        let range = anchor.range.start + relative_start..anchor.range.start + relative_end;
        if document.text().get(range.clone()) != Some(original.expected_source.as_str()) {
            stale.push(occurrence.id);
            continue;
        }
        edits.push((range, original.replacement, occurrence.id));
    }
    edits.sort_by_key(|(range, _, _)| range.start);
    let mut filtered = Vec::new();
    for edit in edits {
        if filtered.last().is_some_and(
            |(range, _, _): &(Range<usize>, String, ImageOccurrenceId)| range.end > edit.0.start,
        ) {
            stale.push(edit.2);
        } else {
            filtered.push(edit);
        }
    }
    let Some(first) = filtered.first() else {
        return ImageApplicationPlan {
            mutation: None,
            edits: Vec::new(),
            replacement_lengths: Vec::new(),
            applied,
            stale,
        };
    };
    let span = first.0.start..filtered.last().expect("non-empty").0.end;
    let expected = document.text()[span.clone()].to_owned();
    let mut replacement = expected.clone();
    for (range, text, _) in filtered.iter().rev() {
        replacement.replace_range(range.start - span.start..range.end - span.start, text);
    }
    let ranges = filtered.iter().map(|(range, _, _)| range.clone()).collect();
    let replacement_lengths = filtered.iter().map(|(_, text, _)| text.len()).collect();
    applied.extend(filtered.iter().map(|(_, _, id)| *id));
    ImageApplicationPlan {
        mutation: Some(document.prepare_range_mutation(
            MutationOrigin::ImageOperation,
            span,
            replacement,
        )),
        edits: ranges,
        replacement_lengths,
        applied,
        stale,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageOperationId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageOperationStatus {
    Planned,
    Running,
    Completed,
    Canceled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImageOperationProgress {
    pub queued: usize,
    pub running: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub uncertain: usize,
    pub unapplied: usize,
}

#[derive(Debug, Clone)]
pub struct PendingImageOperation {
    pub id: ImageOperationId,
    pub config: ImageJobConfig,
    pub anchors: Vec<TrackedImageAnchor>,
    pub input_groups: Vec<ImageInputGroup>,
    pub status: ImageOperationStatus,
    pub generation: CancellationGeneration,
    pub auto_apply_allowed: bool,
    pub results: Vec<ImageItemResult>,
    tracked_mutation_sequence: u64,
    tracked_version: u64,
    group_states: Vec<ImageItemState>,
    known_outputs: HashMap<usize, ImageItemResult>,
}

impl PendingImageOperation {
    pub fn progress(&self) -> ImageOperationProgress {
        let mut progress = ImageOperationProgress::default();
        for state in &self.group_states {
            match state {
                ImageItemState::Queued => progress.queued += 1,
                ImageItemState::Running => progress.running += 1,
                ImageItemState::Applied => progress.succeeded += 1,
                ImageItemState::Failed => progress.failed += 1,
                ImageItemState::Uncertain => progress.uncertain += 1,
                ImageItemState::Unapplied => progress.unapplied += 1,
                ImageItemState::Canceled | ImageItemState::Skipped => {}
            }
        }
        progress
    }

    pub fn retryable_groups(&self) -> Vec<usize> {
        self.group_states
            .iter()
            .enumerate()
            .filter_map(|(index, state)| {
                matches!(
                    state,
                    ImageItemState::Failed | ImageItemState::Uncertain | ImageItemState::Canceled
                )
                .then_some(index)
            })
            .collect()
    }

    pub fn known_output(&self, group: usize) -> Option<&ImageItemResult> {
        self.known_outputs.get(&group)
    }
}

#[derive(Debug, Default)]
pub struct ImageOperationController {
    next_id: u64,
    operations: BTreeMap<ImageOperationId, PendingImageOperation>,
}

impl ImageOperationController {
    pub fn operations(&self) -> impl DoubleEndedIterator<Item = &PendingImageOperation> {
        self.operations.values()
    }

    pub fn begin(
        &mut self,
        document: &MarkdownDocument,
        config: ImageJobConfig,
        input_groups: Vec<ImageInputGroup>,
    ) -> ImageOperationId {
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let id = ImageOperationId(self.next_id);
        let group_states = vec![ImageItemState::Queued; input_groups.len()];
        self.operations.insert(
            id,
            PendingImageOperation {
                id,
                anchors: config.targets.iter().map(TrackedImageAnchor::new).collect(),
                tracked_mutation_sequence: document.mutation_sequence(),
                tracked_version: document.version(),
                config,
                input_groups,
                status: ImageOperationStatus::Planned,
                generation: CancellationGeneration::default(),
                auto_apply_allowed: true,
                results: Vec::new(),
                group_states,
                known_outputs: HashMap::new(),
            },
        );
        id
    }

    pub fn operation(&self, id: ImageOperationId) -> Option<&PendingImageOperation> {
        self.operations.get(&id)
    }

    pub fn operation_mut(&mut self, id: ImageOperationId) -> Option<&mut PendingImageOperation> {
        self.operations.get_mut(&id)
    }

    pub fn observe_document(&mut self, document: &MarkdownDocument) {
        for operation in self
            .operations
            .values_mut()
            .filter(|operation| operation.config.document == document.instance_id())
        {
            match (operation.config.document_path.as_deref(), document.path()) {
                (None, Some(path)) => operation.config.document_path = Some(path.to_path_buf()),
                (left, right) if left != right => {
                    invalidate_operation(operation);
                    operation.tracked_version = document.version();
                    operation.tracked_mutation_sequence = document.mutation_sequence();
                    continue;
                }
                _ => {}
            }
            let events = document.mutation_journal_since(operation.tracked_mutation_sequence);
            if document.version() != operation.tracked_version && events.is_empty() {
                invalidate_operation(operation);
                operation.tracked_version = document.version();
                operation.tracked_mutation_sequence = document.mutation_sequence();
                continue;
            }
            if let Some(first) = events.first()
                && first.sequence > operation.tracked_mutation_sequence.saturating_add(1)
            {
                invalidate_operation(operation);
            }
            for event in events {
                operation.tracked_mutation_sequence = event.sequence;
                if event.rejection.is_some() || event.before_version == event.after_version {
                    continue;
                }
                if matches!(
                    event.origin,
                    MutationOrigin::Undo
                        | MutationOrigin::Redo
                        | MutationOrigin::ExternalReload
                        | MutationOrigin::Recovery
                ) {
                    invalidate_operation(operation);
                    continue;
                }
                let Some(range) = event.range else {
                    invalidate_operation(operation);
                    continue;
                };
                for anchor in &mut operation.anchors {
                    anchor.rebase(range.clone(), event.replacement_len);
                }
            }
            operation.tracked_version = document.version();
        }
    }

    pub fn invalidate_document(&mut self, document: DocumentInstanceId) {
        for operation in self
            .operations
            .values_mut()
            .filter(|operation| operation.config.document == document)
        {
            invalidate_operation(operation);
        }
    }

    pub fn cancel(&mut self, id: ImageOperationId) -> Option<CancellationGeneration> {
        let operation = self.operations.get_mut(&id)?;
        operation.generation = operation.generation.next();
        operation.status = ImageOperationStatus::Canceled;
        operation.auto_apply_allowed = false;
        for state in &mut operation.group_states {
            if matches!(*state, ImageItemState::Queued | ImageItemState::Running) {
                *state = ImageItemState::Canceled;
            }
        }
        Some(operation.generation)
    }

    pub fn mark_running(
        &mut self,
        id: ImageOperationId,
        group: usize,
        generation: CancellationGeneration,
    ) -> bool {
        let Some(operation) = self.operations.get_mut(&id) else {
            return false;
        };
        if operation.generation != generation || operation.status == ImageOperationStatus::Canceled
        {
            return false;
        }
        let Some(state) = operation.group_states.get_mut(group) else {
            return false;
        };
        *state = ImageItemState::Running;
        operation.status = ImageOperationStatus::Running;
        true
    }

    pub fn record_group_result(
        &mut self,
        id: ImageOperationId,
        group: usize,
        generation: CancellationGeneration,
        result: ImageItemResult,
    ) -> bool {
        let Some(operation) = self.operations.get_mut(&id) else {
            return false;
        };
        if operation.generation != generation || operation.status == ImageOperationStatus::Canceled
        {
            return false;
        }
        let Some(state) = operation.group_states.get_mut(group) else {
            return false;
        };
        *state = result.state;
        if matches!(
            result.state,
            ImageItemState::Applied | ImageItemState::Unapplied
        ) && (result.output_url.is_some() || result.output_path.is_some())
        {
            operation.known_outputs.insert(group, result.clone());
        }
        operation.results.push(result);
        if operation
            .group_states
            .iter()
            .all(|state| !matches!(*state, ImageItemState::Queued | ImageItemState::Running))
        {
            operation.status = ImageOperationStatus::Completed;
        }
        true
    }
}

fn invalidate_operation(operation: &mut PendingImageOperation) {
    operation.auto_apply_allowed = false;
    for anchor in &mut operation.anchors {
        anchor.valid = false;
    }
}

impl ImageOperationPlan {
    pub fn eligible_count(&self) -> usize {
        self.selected_occurrences.len()
    }

    pub fn unique_input_count(&self) -> usize {
        self.input_groups.len()
    }

    pub fn repeated_input_count(&self) -> usize {
        self.input_groups
            .iter()
            .map(|group| group.occurrences.len().saturating_sub(1))
            .sum()
    }
}

pub fn plan_image_operation(
    scan: &ImageOccurrenceScan,
    selection: ImagePlanSelection,
    document_path: Option<&std::path::Path>,
    mut local_source_exists: impl FnMut(&std::path::Path) -> bool,
) -> ImageOperationPlan {
    let mut plan = ImageOperationPlan::default();
    let selected = |occurrence: &ImageOccurrence| match &selection {
        ImagePlanSelection::Single(id) => occurrence.id == *id,
        ImagePlanSelection::Occurrences(ids) => ids.contains(&occurrence.id),
        ImagePlanSelection::CurrentDocument => true,
        ImagePlanSelection::InsertedFragment(fragment) => {
            occurrence.full_range.start >= fragment.start
                && occurrence.full_range.end <= fragment.end
        }
    };
    for occurrence in scan
        .occurrences
        .iter()
        .filter(|occurrence| selected(occurrence))
    {
        let input = match classify_occurrence_input(occurrence, document_path) {
            Ok(input) => input,
            Err(reason) => {
                plan.issues.push(ImagePlanIssue {
                    occurrence: Some(occurrence.id),
                    reason,
                    source: occurrence.semantic_url.clone(),
                });
                continue;
            }
        };
        if let ImageInput::Local(path) = &input
            && !local_source_exists(path)
        {
            plan.issues.push(ImagePlanIssue {
                occurrence: Some(occurrence.id),
                reason: ImageSkipReason::MissingLocalSource,
                source: occurrence.semantic_url.clone(),
            });
            continue;
        }
        if let Some(group) = plan
            .input_groups
            .iter_mut()
            .find(|group| same_image_input(&group.input, &input))
        {
            group.occurrences.push(occurrence.id);
        } else {
            plan.input_groups.push(ImageInputGroup {
                input,
                occurrences: vec![occurrence.id],
            });
        }
        plan.selected_occurrences.push(occurrence.clone());
    }

    for skipped in &scan.skipped {
        let include = match &selection {
            ImagePlanSelection::Single(_) => false,
            ImagePlanSelection::Occurrences(_) => false,
            ImagePlanSelection::CurrentDocument => true,
            ImagePlanSelection::InsertedFragment(fragment) => {
                skipped.range.start >= fragment.start && skipped.range.end <= fragment.end
            }
        };
        if include {
            plan.issues.push(ImagePlanIssue {
                occurrence: None,
                reason: skipped.reason,
                source: skipped.expected_source.clone(),
            });
        }
    }
    plan
}

fn classify_occurrence_input(
    occurrence: &ImageOccurrence,
    document_path: Option<&std::path::Path>,
) -> Result<ImageInput, ImageSkipReason> {
    let source = occurrence.semantic_url.trim();
    let lowercase = source.to_ascii_lowercase();
    if lowercase.starts_with("http://") || lowercase.starts_with("https://") {
        return Ok(ImageInput::Remote(source.to_owned()));
    }
    if lowercase.starts_with("data:image/") {
        return Ok(ImageInput::DataUri(source.to_owned()));
    }
    if lowercase.starts_with("data:")
        || lowercase.starts_with("blob:")
        || source.starts_with("//")
        || has_explicit_url_scheme(source)
    {
        return Err(ImageSkipReason::UnsupportedScheme);
    }
    let document_path = document_path.ok_or(ImageSkipReason::MissingDocumentBase)?;
    let document_dir = document_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let path = crate::storage::resources::resolve_local_reference(document_dir, source)
        .ok_or(ImageSkipReason::UnsupportedAuthoredForm)?;
    Ok(ImageInput::Local(path))
}

fn has_explicit_url_scheme(source: &str) -> bool {
    let Some(colon) = source.find(':') else {
        return false;
    };
    colon > 1
        && source[..colon].bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'+' | b'-' | b'.'))
        })
}

fn same_image_input(left: &ImageInput, right: &ImageInput) -> bool {
    match (left, right) {
        (ImageInput::Local(left), ImageInput::Local(right)) => left == right,
        (ImageInput::Remote(left), ImageInput::Remote(right))
        | (ImageInput::DataUri(left), ImageInput::DataUri(right)) => left == right,
        (
            ImageInput::Bytes {
                extension: left_extension,
                bytes: left_bytes,
                ..
            },
            ImageInput::Bytes {
                extension: right_extension,
                bytes: right_bytes,
                ..
            },
        ) => left_extension.eq_ignore_ascii_case(right_extension) && left_bytes == right_bytes,
        _ => false,
    }
}

impl ImageOccurrence {
    pub fn target(&self) -> ImageOccurrenceTarget {
        ImageOccurrenceTarget {
            id: self.id,
            source_range: self.full_range.clone(),
            expected_source: self.expected_source.clone(),
            semantic_url: self.semantic_url.clone(),
        }
    }

    /// Builds an exact, syntax-aware source edit for this occurrence alone.
    pub fn replacement(&self, url: &str) -> ImageSourceReplacement {
        match self.syntax {
            ImageOccurrenceSyntax::MarkdownInline => {
                let range = self
                    .destination_range
                    .clone()
                    .expect("inline occurrences have an attributed destination");
                ImageSourceReplacement {
                    expected_source: self.expected_destination_source(),
                    range,
                    replacement: inline_edit::escape_destination(url),
                }
            }
            ImageOccurrenceSyntax::MarkdownReference(_) => ImageSourceReplacement {
                range: self.full_range.clone(),
                expected_source: self.expected_source.clone(),
                replacement: inline_edit::serialize_inline_image_with_authored_alt(
                    &self.authored_alt,
                    url,
                    self.resolved_title.as_deref(),
                ),
            },
            ImageOccurrenceSyntax::HtmlSrc { quote } => {
                let range = self
                    .destination_range
                    .clone()
                    .expect("HTML src occurrences have an attributed destination");
                ImageSourceReplacement {
                    expected_source: self.expected_destination_source(),
                    range,
                    replacement: escape_html_attribute_value(url, quote),
                }
            }
        }
    }

    fn expected_destination_source(&self) -> String {
        let relative = self
            .destination_range
            .as_ref()
            .expect("destination range is present");
        let start = relative.start.saturating_sub(self.full_range.start);
        let end = relative.end.saturating_sub(self.full_range.start);
        self.expected_source
            .get(start..end)
            .unwrap_or_default()
            .to_owned()
    }
}

pub(crate) fn scan_image_occurrences(body: &str, base: usize) -> ImageOccurrenceScan {
    let mut scan = ImageOccurrenceScan::default();
    let parser = Parser::new_with_broken_link_callback(
        body,
        markdown_options(),
        Some(|_link: BrokenLink<'_>| Some((CowStr::Borrowed(""), CowStr::Borrowed("")))),
    );
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                ..
            }) => {
                let Some(authored) = body.get(range.clone()) else {
                    continue;
                };
                let full_range = base + range.start..base + range.end;
                let Some(authored_alt) = inline_edit::authored_image_alt_source(authored) else {
                    scan.skipped.push(SkippedImageOccurrence {
                        range: full_range,
                        reason: ImageSkipReason::UnsupportedAuthoredForm,
                        expected_source: authored.to_owned(),
                    });
                    continue;
                };
                let (syntax, destination_range) = match link_type {
                    LinkType::Inline => {
                        let Some(relative) =
                            inline_edit::authored_image_destination_range(authored)
                        else {
                            scan.skipped.push(SkippedImageOccurrence {
                                range: full_range,
                                reason: ImageSkipReason::UnsupportedAuthoredForm,
                                expected_source: authored.to_owned(),
                            });
                            continue;
                        };
                        if inline_edit::unescape_markdown(&authored[relative.clone()])
                            != dest_url.as_ref()
                        {
                            scan.skipped.push(SkippedImageOccurrence {
                                range: full_range,
                                reason: ImageSkipReason::UnsupportedAuthoredForm,
                                expected_source: authored.to_owned(),
                            });
                            continue;
                        }
                        (
                            ImageOccurrenceSyntax::MarkdownInline,
                            Some(
                                base + range.start + relative.start
                                    ..base + range.start + relative.end,
                            ),
                        )
                    }
                    LinkType::Reference => (
                        ImageOccurrenceSyntax::MarkdownReference(ImageReferenceKind::Full),
                        None,
                    ),
                    LinkType::Collapsed => (
                        ImageOccurrenceSyntax::MarkdownReference(ImageReferenceKind::Collapsed),
                        None,
                    ),
                    LinkType::Shortcut => (
                        ImageOccurrenceSyntax::MarkdownReference(ImageReferenceKind::Shortcut),
                        None,
                    ),
                    LinkType::ReferenceUnknown
                    | LinkType::CollapsedUnknown
                    | LinkType::ShortcutUnknown => {
                        scan.skipped.push(SkippedImageOccurrence {
                            range: full_range,
                            reason: ImageSkipReason::UnresolvedReference,
                            expected_source: authored.to_owned(),
                        });
                        continue;
                    }
                    _ => {
                        scan.skipped.push(SkippedImageOccurrence {
                            range: full_range,
                            reason: ImageSkipReason::UnsupportedAuthoredForm,
                            expected_source: authored.to_owned(),
                        });
                        continue;
                    }
                };
                scan.occurrences.push(ImageOccurrence {
                    id: ImageOccurrenceId(scan.occurrences.len() as u64 + 1),
                    semantic_url: dest_url.into_string(),
                    full_range,
                    destination_range,
                    expected_source: authored.to_owned(),
                    authored_alt: authored_alt.to_owned(),
                    resolved_title: (!title.is_empty()).then(|| title.into_string()),
                    syntax,
                });
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                scan_html_image_occurrences(html.as_ref(), base + range.start, &mut scan);
            }
            _ => {}
        }
    }
    scan.occurrences
        .sort_by_key(|occurrence| occurrence.full_range.start);
    for (index, occurrence) in scan.occurrences.iter_mut().enumerate() {
        occurrence.id = ImageOccurrenceId(index as u64 + 1);
    }
    scan.skipped.sort_by_key(|skipped| skipped.range.start);
    scan
}

fn scan_html_image_occurrences(html: &str, base: usize, scan: &mut ImageOccurrenceScan) {
    let bytes = html.as_bytes();
    let mut cursor = 0usize;
    while let Some(relative_start) = html[cursor..].find('<') {
        let tag_start = cursor + relative_start;
        let Some(tag_end) = html_tag_end(html, tag_start) else {
            break;
        };
        cursor = tag_end;
        let mut name_start = tag_start + 1;
        while bytes
            .get(name_start)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            name_start += 1;
        }
        if bytes.get(name_start) == Some(&b'/') {
            continue;
        }
        let mut name_end = name_start;
        while bytes
            .get(name_end)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        {
            name_end += 1;
        }
        if !html[name_start..name_end].eq_ignore_ascii_case("img") {
            continue;
        }
        let attrs = html_attributes(html, name_end, tag_end.saturating_sub(1));
        let sources: Vec<_> = attrs
            .iter()
            .filter(|attribute| attribute.name.eq_ignore_ascii_case("src"))
            .collect();
        let full_range = base + tag_start..base + tag_end;
        if sources.len() != 1 {
            let reason = if sources.is_empty() {
                ImageSkipReason::MissingHtmlSrc
            } else {
                ImageSkipReason::AmbiguousHtmlSrc
            };
            scan.skipped.push(SkippedImageOccurrence {
                range: full_range,
                reason,
                expected_source: html[tag_start..tag_end].to_owned(),
            });
            continue;
        }
        let source = sources[0];
        let Some(value_range) = source.value_range.clone() else {
            scan.skipped.push(SkippedImageOccurrence {
                range: full_range,
                reason: ImageSkipReason::UnsupportedAuthoredForm,
                expected_source: html[tag_start..tag_end].to_owned(),
            });
            continue;
        };
        let raw_value = &html[value_range.clone()];
        scan.occurrences.push(ImageOccurrence {
            id: ImageOccurrenceId(0),
            semantic_url: crate::parse::decode_html_entities(raw_value),
            full_range: full_range.clone(),
            destination_range: Some(base + value_range.start..base + value_range.end),
            expected_source: html[tag_start..tag_end].to_owned(),
            authored_alt: String::new(),
            resolved_title: None,
            syntax: ImageOccurrenceSyntax::HtmlSrc {
                quote: source.quote.expect("quoted HTML source"),
            },
        });
    }
}

#[derive(Debug)]
struct HtmlAttribute {
    name: String,
    value_range: Option<Range<usize>>,
    quote: Option<char>,
}

fn html_attributes(html: &str, mut cursor: usize, end: usize) -> Vec<HtmlAttribute> {
    let bytes = html.as_bytes();
    let mut attributes = Vec::new();
    while cursor < end {
        while cursor < end && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b'/') {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < end
            && !bytes[cursor].is_ascii_whitespace()
            && !matches!(bytes[cursor], b'=' | b'/' | b'>')
        {
            cursor += 1;
        }
        if name_start == cursor {
            cursor += 1;
            continue;
        }
        let name = html[name_start..cursor].to_owned();
        while cursor < end && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= end || bytes[cursor] != b'=' {
            attributes.push(HtmlAttribute {
                name,
                value_range: None,
                quote: None,
            });
            continue;
        }
        cursor += 1;
        while cursor < end && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= end || !matches!(bytes[cursor], b'\'' | b'"') {
            let value_start = cursor;
            while cursor < end && !bytes[cursor].is_ascii_whitespace() && bytes[cursor] != b'>' {
                cursor += 1;
            }
            attributes.push(HtmlAttribute {
                name,
                value_range: None,
                quote: None,
            });
            cursor = cursor.max(value_start + 1);
            continue;
        }
        let quote = bytes[cursor] as char;
        cursor += 1;
        let value_start = cursor;
        while cursor < end && bytes[cursor] != quote as u8 {
            cursor += 1;
        }
        if cursor >= end {
            attributes.push(HtmlAttribute {
                name,
                value_range: None,
                quote: None,
            });
            continue;
        }
        let value_end = cursor;
        cursor += 1;
        attributes.push(HtmlAttribute {
            name,
            value_range: Some(value_start..value_end),
            quote: Some(quote),
        });
    }
    attributes
}

fn html_tag_end(source: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, ch) in source[start..].char_indices() {
        match (quote, ch) {
            (Some(active), current) if current == active => quote = None,
            (None, '"' | '\'') => quote = Some(ch),
            (None, '>') => return Some(start + relative + 1),
            _ => {}
        }
    }
    None
}

fn escape_html_attribute_value(value: &str, quote: char) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' if quote == '"' => escaped.push_str("&quot;"),
            '\'' if quote == '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageJobConfig {
    pub document: DocumentInstanceId,
    pub document_path: Option<PathBuf>,
    pub document_version: u64,
    pub kind: ImageOperationKind,
    pub preferences: ImagePreferences,
    pub targets: Vec<ImageOccurrenceTarget>,
    pub occurrences: Vec<ImageOccurrence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ImageItemState {
    Queued,
    Running,
    Applied,
    Failed,
    Canceled,
    Uncertain,
    Unapplied,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImageItemResult {
    pub occurrence: Option<ImageOccurrenceId>,
    pub state: ImageItemState,
    pub source: String,
    pub output_url: Option<String>,
    pub output_path: Option<PathBuf>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImageRecoveryRecord {
    pub operation_id: String,
    pub document_path: Option<PathBuf>,
    pub created_unix_secs: u64,
    pub items: Vec<ImageItemResult>,
    pub owned_inputs: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageRecoveryAction {
    Retry,
    SaveLocally { destination: Option<PathBuf> },
    CopyUrl,
    RevealFile,
    SaveRecoveredInput { destination: Option<PathBuf> },
    ApplyKnownResults,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageRecoveryTransition {
    RetryTransfer,
    CopyText(String),
    Reveal(PathBuf),
    CopyOwnedInput {
        source: PathBuf,
        destination: PathBuf,
    },
    SaveKnownFile {
        source: PathBuf,
        destination: PathBuf,
    },
    ApplyKnownUrl {
        occurrence: ImageOccurrenceId,
        url: String,
    },
}

pub fn prepare_image_recovery_action(
    record: &ImageRecoveryRecord,
    item_index: usize,
    action: ImageRecoveryAction,
) -> Result<ImageRecoveryTransition, String> {
    let item = record
        .items
        .get(item_index)
        .ok_or_else(|| "recovery item no longer exists".to_owned())?;
    match action {
        ImageRecoveryAction::Retry => {
            if matches!(
                item.state,
                ImageItemState::Failed | ImageItemState::Canceled | ImageItemState::Uncertain
            ) {
                Ok(ImageRecoveryTransition::RetryTransfer)
            } else {
                Err("this result does not require another transfer".into())
            }
        }
        ImageRecoveryAction::CopyUrl => item
            .output_url
            .clone()
            .map(ImageRecoveryTransition::CopyText)
            .ok_or_else(|| "no known output URL is available".into()),
        ImageRecoveryAction::RevealFile => item
            .output_path
            .clone()
            .map(ImageRecoveryTransition::Reveal)
            .ok_or_else(|| "no saved output file is available".into()),
        ImageRecoveryAction::ApplyKnownResults => Ok(ImageRecoveryTransition::ApplyKnownUrl {
            occurrence: item
                .occurrence
                .ok_or_else(|| "the original occurrence is unavailable".to_owned())?,
            url: item
                .output_url
                .clone()
                .ok_or_else(|| "no known output URL is available".to_owned())?,
        }),
        ImageRecoveryAction::SaveRecoveredInput { destination } => {
            let destination = destination
                .ok_or_else(|| "a destination is required for the recovered input".to_owned())?;
            let source = record
                .owned_inputs
                .get(item_index)
                .cloned()
                .ok_or_else(|| "the captured input is no longer available".to_owned())?;
            Ok(ImageRecoveryTransition::CopyOwnedInput {
                source,
                destination,
            })
        }
        ImageRecoveryAction::SaveLocally { destination } => {
            let destination = destination
                .ok_or_else(|| "a saved document or explicit destination is required".to_owned())?;
            let source = record
                .owned_inputs
                .get(item_index)
                .cloned()
                .or_else(|| item.output_path.clone())
                .ok_or_else(|| "no local image bytes are available".to_owned())?;
            Ok(ImageRecoveryTransition::SaveKnownFile {
                source,
                destination,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CancellationGeneration(pub u64);

impl CancellationGeneration {
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedImageAnchor {
    pub occurrence: ImageOccurrenceId,
    pub range: Range<usize>,
    pub expected_source: String,
    pub valid: bool,
}

impl TrackedImageAnchor {
    pub fn new(target: &ImageOccurrenceTarget) -> Self {
        Self {
            occurrence: target.id,
            range: target.source_range.clone(),
            expected_source: target.expected_source.clone(),
            valid: true,
        }
    }

    /// Rebase over one accepted range edit. Edits wholly before the anchor
    /// shift it; edits wholly after it leave it alone. Touching/crossing the
    /// anchor invalidates it. For a zero-width pending insertion, edits at the
    /// same point move the anchor after inserted content (left affinity).
    pub fn rebase(&mut self, edit: Range<usize>, replacement_len: usize) {
        if !self.valid {
            return;
        }
        let removed_len = edit.end.saturating_sub(edit.start);
        let delta = replacement_len as isize - removed_len as isize;
        if self.range.is_empty() && edit.start == self.range.start && edit.end == edit.start {
            shift_range(&mut self.range, replacement_len as isize);
        } else if edit.end <= self.range.start {
            shift_range(&mut self.range, delta);
        } else if edit.start >= self.range.end {
            // Edit follows the anchor.
        } else {
            self.valid = false;
        }
    }
}

fn shift_range(range: &mut Range<usize>, delta: isize) {
    range.start = range.start.saturating_add_signed(delta);
    range.end = range.end.saturating_add_signed(delta);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarkdownDocument;

    fn target(range: Range<usize>) -> ImageOccurrenceTarget {
        ImageOccurrenceTarget {
            id: ImageOccurrenceId(1),
            source_range: range,
            expected_source: "![a](b.png)".into(),
            semantic_url: "b.png".into(),
        }
    }

    #[test]
    fn unrelated_edits_rebase_and_overlaps_invalidate() {
        let mut anchor = TrackedImageAnchor::new(&target(10..21));
        anchor.rebase(2..4, 5);
        assert_eq!(anchor.range, 13..24);
        assert!(anchor.valid);
        anchor.rebase(15..16, 0);
        assert!(!anchor.valid);
    }

    #[test]
    fn zero_width_anchor_has_left_affinity() {
        let mut anchor = TrackedImageAnchor::new(&target(4..4));
        anchor.rebase(4..4, 3);
        assert_eq!(anchor.range, 7..7);
        assert!(anchor.valid);
    }

    #[test]
    fn job_configuration_is_an_immutable_snapshot() {
        let mut preferences = ImagePreferences::default();
        let job = ImageJobConfig {
            document: crate::MarkdownDocument::from_text("").instance_id(),
            document_path: None,
            document_version: 1,
            kind: ImageOperationKind::Upload,
            preferences: preferences.clone(),
            targets: vec![],
            occurrences: vec![],
        };
        preferences.directory = "assets".into();
        assert_ne!(job.preferences.directory, preferences.directory);
    }

    #[test]
    fn scanner_attributes_markdown_and_html_occurrences_exactly() {
        let source = concat!(
            "---\ntitle: '![front](front.png)'\n---\n",
            "> ![nested](<images/a b.png>)\n\n",
            "| image |\n|---|\n| ![table](table.png) |\n\n",
            "`![code](code.png)`\n\n",
            "```md\n![fenced](fenced.png)\n```\n\n",
            "[ordinary](plain.png)\n",
            "![unresolved][missing-definition]\n",
            "<img class=\"hero\" src='a&amp;b.png' width=\"20\">\n",
            "<img srcset=\"a.png 1x, b.png 2x\">\n",
        );
        let scan = MarkdownDocument::from_text(source).image_occurrence_scan();
        let urls: Vec<_> = scan
            .occurrences
            .iter()
            .map(|occurrence| occurrence.semantic_url.as_str())
            .collect();
        assert_eq!(urls, ["images/a b.png", "table.png", "a&b.png"]);
        assert_eq!(scan.skipped.len(), 2);
        assert_eq!(scan.skipped[0].reason, ImageSkipReason::UnresolvedReference);
        assert_eq!(scan.skipped[1].reason, ImageSkipReason::MissingHtmlSrc);

        let html = scan.occurrences.last().unwrap();
        let replacement = html.replacement("https://example.test/a?x=1&y='two'");
        assert_eq!(
            replacement.replacement,
            "https://example.test/a?x=1&amp;y=&#39;two&#39;"
        );
        assert_eq!(&source[replacement.range], "a&amp;b.png");
    }

    #[test]
    fn resolved_reference_images_materialize_only_the_selected_use() {
        let source = concat!(
            "![**bold**][shared]\n",
            "![second][shared]\n",
            "![collapsed][]\n",
            "![shortcut]\n",
            "[ordinary][shared]\n\n",
            "[shared]: images/a.png \"Title\"\n",
            "[collapsed]: images/b.png\n",
            "[shortcut]: images/c.png\n",
        );
        let document = MarkdownDocument::from_text(source);
        let scan = document.image_occurrence_scan();
        assert_eq!(scan.occurrences.len(), 4);
        assert_eq!(
            scan.occurrences
                .iter()
                .filter(|occurrence| occurrence.semantic_url == "images/a.png")
                .count(),
            2
        );
        assert_eq!(
            scan.occurrences[0].syntax,
            ImageOccurrenceSyntax::MarkdownReference(ImageReferenceKind::Full)
        );
        assert_eq!(
            scan.occurrences[2].syntax,
            ImageOccurrenceSyntax::MarkdownReference(ImageReferenceKind::Collapsed)
        );
        assert_eq!(
            scan.occurrences[3].syntax,
            ImageOccurrenceSyntax::MarkdownReference(ImageReferenceKind::Shortcut)
        );

        let edit = scan.occurrences[0].replacement("https://cdn.test/new.png");
        let mut transformed = source.to_owned();
        transformed.replace_range(edit.range, &edit.replacement);
        assert!(
            transformed
                .starts_with("![**bold**](https://cdn.test/new.png \"Title\")\n![second][shared]")
        );
        assert!(transformed.contains("[ordinary][shared]"));
        assert!(transformed.contains("[shared]: images/a.png \"Title\""));
    }

    #[test]
    fn planning_selects_exact_occurrences_and_groups_only_transfer_inputs() {
        let source = concat!(
            "![old](same.png)\n",
            "![one](same.png) ![two](same.png)\n",
            "![web](https://example.test/a.png)\n",
            "![missing](missing.png) ![blob](blob:temporary)\n",
        );
        let scan = MarkdownDocument::from_text(source).image_occurrence_scan();
        let fragment = scan.occurrences[1].full_range.start..scan.occurrences[2].full_range.end;
        let document_path = std::path::Path::new("C:/notes/note.md");
        let inserted = plan_image_operation(
            &scan,
            ImagePlanSelection::InsertedFragment(fragment),
            Some(document_path),
            |_| true,
        );
        assert_eq!(inserted.eligible_count(), 2);
        assert_eq!(inserted.unique_input_count(), 1);
        assert_eq!(inserted.repeated_input_count(), 1);
        assert_eq!(inserted.input_groups[0].occurrences.len(), 2);

        let single = plan_image_operation(
            &scan,
            ImagePlanSelection::Single(scan.occurrences[2].id),
            Some(document_path),
            |_| true,
        );
        assert_eq!(single.eligible_count(), 1);
        assert_eq!(
            single.input_groups[0].occurrences,
            vec![scan.occurrences[2].id]
        );

        let all = plan_image_operation(
            &scan,
            ImagePlanSelection::CurrentDocument,
            Some(document_path),
            |path| !path.ends_with("missing.png"),
        );
        assert_eq!(all.eligible_count(), 4);
        assert_eq!(all.issues.len(), 2);
        assert!(
            all.issues
                .iter()
                .any(|issue| issue.reason == ImageSkipReason::MissingLocalSource)
        );
        assert!(
            all.issues
                .iter()
                .any(|issue| issue.reason == ImageSkipReason::UnsupportedScheme)
        );
    }

    fn operation_fixture(document: &MarkdownDocument) -> (ImageJobConfig, Vec<ImageInputGroup>) {
        let occurrence = document.image_occurrence_scan().occurrences.remove(0);
        (
            ImageJobConfig {
                document: document.instance_id(),
                document_path: document.path().map(PathBuf::from),
                document_version: document.version(),
                kind: ImageOperationKind::Upload,
                preferences: ImagePreferences::default(),
                targets: vec![occurrence.target()],
                occurrences: vec![occurrence.clone()],
            },
            vec![
                ImageInputGroup {
                    input: ImageInput::Remote("https://example.test/a.png".into()),
                    occurrences: vec![occurrence.id],
                },
                ImageInputGroup {
                    input: ImageInput::Remote("https://example.test/b.png".into()),
                    occurrences: vec![occurrence.id],
                },
            ],
        )
    }

    #[test]
    fn controller_rebases_canonical_edits_and_invalidates_unsafe_history() {
        let mut document = MarkdownDocument::from_text("xx ![a](b.png) yy");
        let (config, groups) = operation_fixture(&document);
        let mut controller = ImageOperationController::default();
        let id = controller.begin(&document, config, groups);
        let original = controller.operation(id).unwrap().anchors[0].range.clone();

        let mutation =
            document.prepare_range_mutation(MutationOrigin::PlatformTextInput, 0..0, "123");
        document.apply_checked_mutation(mutation).unwrap();
        controller.observe_document(&document);
        let operation = controller.operation(id).unwrap();
        assert_eq!(
            operation.anchors[0].range,
            original.start + 3..original.end + 3
        );
        assert!(operation.auto_apply_allowed);

        let overlap = operation.anchors[0].range.start + 2;
        let mutation = document.prepare_range_mutation(
            MutationOrigin::PlatformTextInput,
            overlap..overlap,
            "changed",
        );
        document.apply_checked_mutation(mutation).unwrap();
        controller.observe_document(&document);
        assert!(!controller.operation(id).unwrap().anchors[0].valid);

        let mut fresh = MarkdownDocument::from_text("![a](b.png)");
        let (config, groups) = operation_fixture(&fresh);
        let fresh_id = controller.begin(&fresh, config, groups);
        let mutation = fresh.prepare_whole_mutation(MutationOrigin::Undo, "other");
        fresh.apply_checked_mutation(mutation).unwrap();
        controller.observe_document(&fresh);
        assert!(!controller.operation(fresh_id).unwrap().auto_apply_allowed);
    }

    #[test]
    fn controller_cancellation_rejects_late_results_and_retry_reuses_successes() {
        let document = MarkdownDocument::from_text("![a](b.png)");
        let (config, groups) = operation_fixture(&document);
        let mut controller = ImageOperationController::default();
        let id = controller.begin(&document, config, groups);
        let generation = controller.operation(id).unwrap().generation;
        assert!(controller.mark_running(id, 0, generation));
        assert!(controller.record_group_result(
            id,
            0,
            generation,
            ImageItemResult {
                occurrence: None,
                state: ImageItemState::Unapplied,
                source: "a".into(),
                output_url: Some("https://cdn.test/a.png".into()),
                output_path: None,
                message: None,
            },
        ));
        assert!(controller.record_group_result(
            id,
            1,
            generation,
            ImageItemResult {
                occurrence: None,
                state: ImageItemState::Failed,
                source: "b".into(),
                output_url: None,
                output_path: None,
                message: Some("failed".into()),
            },
        ));
        let operation = controller.operation(id).unwrap();
        assert_eq!(operation.retryable_groups(), vec![1]);
        assert!(operation.known_output(0).is_some());

        let new_generation = controller.cancel(id).unwrap();
        assert_ne!(new_generation, generation);
        assert!(!controller.record_group_result(
            id,
            1,
            generation,
            ImageItemResult {
                occurrence: None,
                state: ImageItemState::Unapplied,
                source: "late".into(),
                output_url: Some("https://cdn.test/late.png".into()),
                output_path: None,
                message: None,
            },
        ));
        assert!(!controller.operation(id).unwrap().auto_apply_allowed);
    }

    #[test]
    fn application_plan_uses_one_current_version_mutation_and_skips_stale_targets() {
        let mut document = MarkdownDocument::from_text("![a](one.png) mid ![b](two.png)");
        let occurrences = document.image_occurrence_scan().occurrences;
        let config = ImageJobConfig {
            document: document.instance_id(),
            document_path: None,
            document_version: document.version(),
            kind: ImageOperationKind::Upload,
            preferences: ImagePreferences::default(),
            targets: occurrences.iter().map(ImageOccurrence::target).collect(),
            occurrences: occurrences.clone(),
        };
        let mut controller = ImageOperationController::default();
        let id = controller.begin(&document, config, vec![]);

        let mutation =
            document.prepare_range_mutation(MutationOrigin::PlatformTextInput, 0..0, "prefix ");
        document.apply_checked_mutation(mutation).unwrap();
        controller.observe_document(&document);
        let mut outputs = HashMap::new();
        outputs.insert(occurrences[0].id, "https://cdn.test/one.png".into());
        outputs.insert(occurrences[1].id, "https://cdn.test/two.png".into());
        let before_version = document.version();
        let plan =
            build_image_application_plan(&document, controller.operation(id).unwrap(), &outputs);
        assert_eq!(plan.applied.len(), 2);
        document
            .apply_checked_mutation(plan.mutation.expect("combined mutation"))
            .unwrap();
        assert_eq!(document.version(), before_version + 1);
        assert!(document.text().contains("https://cdn.test/one.png"));
        assert!(document.text().contains("https://cdn.test/two.png"));

        let mut partial = MarkdownDocument::from_text("![a](one.png) mid ![b](two.png)");
        let occurrences = partial.image_occurrence_scan().occurrences;
        let config = ImageJobConfig {
            document: partial.instance_id(),
            document_path: None,
            document_version: partial.version(),
            kind: ImageOperationKind::Upload,
            preferences: ImagePreferences::default(),
            targets: occurrences.iter().map(ImageOccurrence::target).collect(),
            occurrences: occurrences.clone(),
        };
        let id = controller.begin(&partial, config, vec![]);
        let first = occurrences[0].full_range.clone();
        let mutation = partial.prepare_range_mutation(
            MutationOrigin::PlatformTextInput,
            first.start + 2..first.start + 2,
            "x",
        );
        partial.apply_checked_mutation(mutation).unwrap();
        controller.observe_document(&partial);
        let mut outputs = HashMap::new();
        outputs.insert(occurrences[0].id, "https://cdn.test/one.png".into());
        outputs.insert(occurrences[1].id, "https://cdn.test/two.png".into());
        let plan =
            build_image_application_plan(&partial, controller.operation(id).unwrap(), &outputs);
        assert_eq!(plan.applied, vec![occurrences[1].id]);
        assert_eq!(plan.stale, vec![occurrences[0].id]);
    }

    #[test]
    fn recovery_actions_require_concrete_data_and_do_not_hide_transfer_work() {
        let record = ImageRecoveryRecord {
            operation_id: "job".into(),
            document_path: None,
            created_unix_secs: 1,
            items: vec![ImageItemResult {
                occurrence: Some(ImageOccurrenceId(3)),
                state: ImageItemState::Unapplied,
                source: "clipboard".into(),
                output_url: Some("https://cdn.test/a.png?sig=keep".into()),
                output_path: Some(PathBuf::from("saved/a.png")),
                message: None,
            }],
            owned_inputs: vec![PathBuf::from("private/input.png")],
        };
        assert_eq!(
            prepare_image_recovery_action(&record, 0, ImageRecoveryAction::CopyUrl).unwrap(),
            ImageRecoveryTransition::CopyText("https://cdn.test/a.png?sig=keep".into())
        );
        assert_eq!(
            prepare_image_recovery_action(&record, 0, ImageRecoveryAction::ApplyKnownResults)
                .unwrap(),
            ImageRecoveryTransition::ApplyKnownUrl {
                occurrence: ImageOccurrenceId(3),
                url: "https://cdn.test/a.png?sig=keep".into(),
            }
        );
        assert!(
            prepare_image_recovery_action(
                &record,
                0,
                ImageRecoveryAction::SaveRecoveredInput { destination: None }
            )
            .is_err()
        );
        assert!(
            prepare_image_recovery_action(
                &record,
                0,
                ImageRecoveryAction::SaveLocally { destination: None }
            )
            .is_err()
        );
    }

    #[test]
    fn save_as_and_missing_history_invalidate_application_but_keep_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let mut document = MarkdownDocument::from_text("![a](b.png)");
        let (config, groups) = operation_fixture(&document);
        let mut controller = ImageOperationController::default();
        let id = controller.begin(&document, config, groups);
        let generation = controller.operation(id).unwrap().generation;
        assert!(controller.record_group_result(
            id,
            0,
            generation,
            ImageItemResult {
                occurrence: Some(ImageOccurrenceId(1)),
                state: ImageItemState::Unapplied,
                source: "b.png".into(),
                output_url: Some("https://cdn.test/b.png".into()),
                output_path: None,
                message: None,
            }
        ));
        let first_path = temp.path().join("first.md");
        document.save_as(&first_path).unwrap();
        controller.observe_document(&document);
        assert!(controller.operation(id).unwrap().auto_apply_allowed);
        assert_eq!(
            controller
                .operation(id)
                .unwrap()
                .config
                .document_path
                .as_deref(),
            Some(first_path.as_path())
        );

        let second_path = temp.path().join("second.md");
        document.save_as(&second_path).unwrap();
        controller.observe_document(&document);
        let operation = controller.operation(id).unwrap();
        assert!(!operation.auto_apply_allowed);
        assert!(operation.known_output(0).is_some());

        let mut overflow = MarkdownDocument::from_text("![a](b.png)");
        let (config, groups) = operation_fixture(&overflow);
        let overflow_id = controller.begin(&overflow, config, groups);
        for _ in 0..(crate::MUTATION_JOURNAL_CAPACITY + 4) {
            let mutation =
                overflow.prepare_range_mutation(MutationOrigin::PlatformTextInput, 0..0, "x");
            overflow.apply_checked_mutation(mutation).unwrap();
        }
        controller.observe_document(&overflow);
        assert!(
            !controller
                .operation(overflow_id)
                .unwrap()
                .auto_apply_allowed
        );
    }
}
