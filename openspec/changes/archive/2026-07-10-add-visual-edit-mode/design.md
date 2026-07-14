## Context

Markion's current editing model is source-first. The source editor owns text input, cursor/selection state, undo/redo, IME composition, and scroll state. The preview and read panes render `PreviewBlock` values derived from `MarkdownDocument.text`; those derived values are cached by document version and shared through `Arc`, with preview parsing debounced and list-rendered through `ListState`.

Visual Edit should feel closer to Obsidian Live Preview than Typora. The document remains plain Markdown, but the active editing surface renders common Markdown constructs visually and reveals or edits their source representation only where needed.

Current source and preview flow:

```text
MarkdownDocument.text
  -> document version
  -> cached preview blocks / outline / stats
  -> Edit, Split Preview, Read rendering
```

Target Visual Edit flow:

```text
MarkdownDocument.text
  -> source-ranged visual model
  -> single visual editing surface
  -> existing MarkdownDocument text mutations
  -> existing undo / dirty / autosave / derived-cache invalidation
```

## Goals / Non-Goals

**Goals:**

- Add a fourth `ViewMode::VisualEdit` without changing existing Edit, Split Preview, or Read behavior.
- Keep Markdown text as the only persisted representation.
- Make common prose editing feel visual: headings, paragraphs, inline strong/emphasis/code/link/image syntax, blockquotes, lists, task lists, and rules.
- Route all text mutations through the existing document mutation and undo paths so dirty state, recovery, autosave, and tab isolation remain consistent.
- Preserve derived-state caching, syntax highlighting memoization, preview virtualization, and cached shared text handles.
- Provide a clear upgrade path for richer table, math, and code editing after v1.

**Non-Goals:**

- Full Typora parity.
- Rich-text or HTML storage.
- A separate editable rendered tree that can diverge from Markdown source.
- Direct visual table cell editing in v1.
- Full fidelity for every extension while editing. Reading view remains the highest-fidelity rendered view.
- Cross-block rich selection and rich clipboard formats in v1.

## Decisions

### 1. Source-backed visual mode, not rich-document mode

Visual Edit will keep `MarkdownDocument.text` as the canonical model. Every operation ultimately applies a source-range replacement or uses an existing formatting/table command.

Alternatives considered:

| Approach | Pros | Cons |
|---|---|---|
| Rich document model with Markdown serialization | More flexible WYSIWYG editing | Large new model, serialization ambiguity, higher risk of Markdown drift |
| Editable preview blocks | Reuses preview rendering | `PreviewBlock` currently loses source detail for many inline constructs |
| Source-backed visual model | Preserves file format and undo/cache paths | Requires source ranges and careful hit testing |

Decision: use a source-backed visual model for v1.

### 2. Introduce a visual-edit model separate from `PreviewBlock`

`PreviewBlock` is optimized for rendering/export-facing preview behavior. Visual Edit needs source ranges for blocks, inline spans, markers, and editable text runs. Rather than overloading `PreviewBlock`, add a narrow visual edit representation such as:

```text
VisualBlock {
  kind,
  source_range,
  editable_runs,
  marker_ranges,
}

VisualInlineRun {
  visible_text,
  source_range,
  content_range,
  style,
  link_target_range,
}
```

The first implementation may build this model from a full parse plus source scans, then cache it by document version. Later it can move toward incremental source mapping if needed.

### 3. Reuse the existing editor input path where practical

Visual Edit should not create a second undo or IME stack. Cursor movement and text replacement can be visual-mode aware, but actual mutation should continue through `MarkdownDocument::replace_range`, `apply_markdown_format`, and the existing `EntityInputHandler` style path where possible.

For v1, two editing levels are acceptable:

- Inline visual editing for simple text runs where visible text maps cleanly to a source content range.
- Source islands for complex constructs or ambiguous ranges.

### 4. Reveal syntax near focus instead of hiding everything permanently

Obsidian-style editing is safer than total marker hiding. When the cursor enters formatted content, the relevant source markers may become visible or editable. This keeps Markdown understandable and avoids impossible cursor positions around hidden markers.

Examples:

- A bold span may render as bold text while unfocused, then show `**text**` or marker affordances when focused.
- A link may render as linked text while unfocused, then expose `[text](url)` or a small source island when focused.
- Code/math/table blocks may enter a source island on click or keyboard focus.

### 5. Keep complex blocks conservative in v1

Fenced code blocks, math blocks, HTML blocks/front matter, images, and tables have richer semantics than plain prose. V1 should keep them editable through source islands or existing table toolbar operations. This prevents the first milestone from becoming a full visual document editor.

### 6. Rendering and caching boundaries

Visual Edit may require a new per-document-version cache for source-ranged visual blocks. It must follow the same invalidation rule as existing derived state: recompute only when `MarkdownDocument.version()` changes. Selection changes, cursor movement, hover state, or focus changes must not invalidate Markdown-derived caches.

Preview parsing remains debounced for Split/Read. Visual Edit can choose either synchronous first render plus cached reuse or a debounced parse, but it must not reparse on every paint when the document version is unchanged.

## Risks / Trade-offs

- [Risk] Hidden marker hit testing can create surprising cursor jumps -> Start with focused syntax reveal and source islands so every cursor position corresponds to real source text.
- [Risk] Source mapping is hard for nested inline syntax -> Limit v1 to common inline spans and fall back to source display for ambiguous ranges.
- [Risk] Visual Edit duplicates preview rendering code -> Extract only small shared styling helpers; keep visual edit model separate from preview output.
- [Risk] Large documents could regress typing latency -> Cache visual blocks per document version and avoid recomputing on cursor/selection-only updates.
- [Risk] Users may expect Typora-level table and math editing -> Name and document the mode as Visual Edit / Live Preview style, not full WYSIWYG.
- [Risk] Mode switching may lose selection/scroll context -> Add per-tab visual edit scroll/selection state or map it to existing source selection where practical.

## Migration Plan

- No document migration is required.
- Add Visual Edit as an additive view mode.
- Existing user preferences and saved Markdown files remain valid.
- Rollback is removal of the mode, menu/shortcut entries, and visual edit rendering/input paths; existing documents remain plain Markdown.

## Open Questions

- Should Visual Edit get a persisted default-mode preference immediately, or ship first as a manual mode switch?
- Which direct shortcut should Visual Edit use so existing Edit/Split/Read shortcuts stay stable?
- Should link click open the link on single click, modifier click, or only from Reading view when Visual Edit is active?
