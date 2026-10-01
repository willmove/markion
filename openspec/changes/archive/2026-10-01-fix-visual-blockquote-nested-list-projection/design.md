## Context

`MarkdownDocument::derive_preview_and_outline` currently represents a blockquote as one `PreviewBlock::BlockQuote` with a `RichText` containing all quoted paragraphs, a `children` vector containing nested list items, and one source range covering the complete quote. The representation fixed list containment in Split Preview / Read, but it discards paragraph/list interleaving and is not a valid flat input for Visual Edit.

`build_visual_blocks` compensates by emitting the parent quote followed by its list children. The parent owns the complete quote range while every child owns a nested range. The normal overlap guard therefore marks the children `Unsupported`; when smart punctuation also makes the parent byte-inexact, the UI renders the complete parent source plus the nested item sources as separate source islands.

Visual Edit's virtual list and text-input bridge are deliberately row-oriented: one `VisualBlock` owns one contiguous canonical source range, one row identity, and one editable projection. The fix must preserve that architecture, the per-document-version `Arc` caches, stable-ID reconciliation, and incremental/full derivation equivalence.

Current data flow:

```text
pulldown events
  -> BlockQuote { whole range, paragraph text, list children }
  -> [whole-range quote row, nested child rows]
  -> overlap guard marks children Unsupported
  -> source-island UI duplicates nested list source
```

Target data flow:

```text
pulldown events
  -> BlockQuote { ordered child blocks }
  -> partition into disjoint quoted leaf rows + quote context
  -> version-cached VisualBlock rows
  -> ordinary paragraph/list renderers decorated by quote context
```

## Goals / Non-Goals

**Goals:**

- Preserve quoted paragraphs and list items in authored order for Preview and Visual Edit.
- Give every Visual Edit leaf row one disjoint, contiguous, UTF-8-safe source range while retaining its blockquote depth and exact structural prefixes.
- Render quoted ordered, unordered, nested, and task lists exactly once inside continuous blockquote styling.
- Keep pointer, keyboard, IME, selection, undo/redo, structural Enter/Backspace, and stable row identity on the existing source-backed paths.
- Prevent smart-punctuation substitution alone from converting supported Visual Edit prose into a complete source island.
- Derive all new metadata only in the existing per-version cached model path.

**Non-Goals:**

- A general recursive visual DOM for every CommonMark container combination.
- Rendering arbitrary entities or other length-changing decoded text through a new non-identity projection model.
- Changing Split Preview / Read smart-punctuation presentation.
- Introducing a parallel editable rich-text tree or persisting derived quote metadata.

## Decisions

### 1. Make blockquote children the ordered semantic flow

`PreviewBlock::BlockQuote` will use an ordered `children: Vec<PreviewBlock>` as the semantic content of the quote. Ordinary quoted prose becomes `PreviewBlock::Paragraph`; quoted list items remain `PreviewBlock::ListItem`; all are appended when their end events arrive. The separate flattened `text` field will be removed rather than kept as a second representation.

`plain_text`, preview selection, statistics, export, math collection, memory accounting, and source-range shifting will fold the ordered children recursively. Split Preview / Read will render children in vector order, fixing the existing accepted reordering of `intro -> list -> outro` into `intro/outro -> list`.

Alternative considered: keep `text + children` and add source-range fragments only for Visual Edit. Rejected because the two representations can disagree about order and force every consumer to reconstruct a flow that the model has already discarded.

### 2. Flatten semantic children into disjoint quoted leaf rows

Visual Edit will not emit a parent `BlockQuote` row when that parent also has independently editable descendants. Instead it will flatten the supported ordered child flow into leaf `VisualBlock`s. A partition helper will assign each leaf a contiguous owned range bounded by the parent quote start, the next leaf start, and the parent quote end. Structural-only quote lines and blank quoted separators will be owned by the adjacent leaf or represented as quote-context whitespace; no non-whitespace gap may fall through to `Unsupported`.

Each resulting row receives quote context containing at least:

- quote depth;
- exact quote-marker ranges within the owned source;
- whether the row begins, continues, or ends a contiguous quote group, for spacing and border continuity.

Ordinary nested-list subtree overlap will still be partitioned at the first descendant item as today. After partitioning, visual row ranges must be monotonically ordered and non-overlapping; the existing overlap-to-`Unsupported` guard remains as a correctness backstop.

Alternative considered: one recursive GPUI quote row containing several editable child elements. Rejected because it would bypass the one-row/one-projection input, caret, virtualization, and stable-identity machinery.

Alternative considered: ignore the overlap guard for quote children. Rejected because two editable rows would claim the same canonical bytes and make hit testing and mutation ambiguous.

### 3. Represent the quote prefix separately from the leaf prefix

Quoted leaf rows need two structural layers: the outer quote marker and an optional inner heading/list/task marker. Add quote-context prefix metadata alongside the existing leaf `block_prefix` rather than pretending `> 1. ` is one list marker.

The complete prefix parser will recognize repeated quote markers followed by indentation and an optional ordered, unordered, or task-list prefix. Projection hides both proven layers while unfocused. Structural edits act on the innermost leaf prefix first: Backspace at the visible start of `> 1. item` removes `1. ` and leaves `> item`; a subsequent quote-prefix edit can demote the quoted paragraph. Enter on a non-empty quoted list item emits the combined continued prefix, including ordered-number advancement and an unchecked task marker.

Alternative considered: store a generic recursive prefix stack. Rejected for this focused change; explicit quote context plus the existing leaf prefix covers the affected container combination without redesigning unrelated headings and lists.

### 4. Keep Visual Edit punctuation parsing source-faithful

Add a Visual Edit inline option set equal to the semantic Markdown options except `ENABLE_SMART_PUNCTUATION`. Preview, Read, export, outline, and statistics retain the canonical parser options. Visual Edit therefore displays authored straight quotes/dashes as ordinary editable text with identity byte mapping instead of asking `push_run` to map substituted Unicode punctuation back to different source bytes.

This is an intentional fidelity trade-off: authored punctuation remains visually clean and editable, but Visual Edit does not reproduce Preview's curly smart-punctuation glyphs yet. A future non-identity projection change can render substituted glyphs while retaining reversible source mappings; entity decoding remains on that roadmap.

Alternative considered: special-case ASCII quotes after pulldown substitution. Rejected because the existing projection assumes equal-length identity segments and ad-hoc reverse mapping would fail for entities and multi-byte substitutions.

### 5. Preserve cached derivation and stable leaf identity

Ordered quote children, source partitions, quote context, and composite-prefix ranges are derived inside `derive_preview_and_outline` / `build_visual_blocks` and cached once per `MarkdownDocument.version()`. GPUI rendering consumes metadata only; it does not parse source during paint.

`shift_preview_block`, visual-range shifting, retained-size accounting, and stable-ID reconciliation will include the new recursive children and quote metadata. An edit to one quoted leaf may replace that leaf's identity, while text-identical quoted siblings outside the affected range remain eligible for reuse. Debug and release behavior must match fresh full derivation.

### 6. Render quote context by decorating ordinary leaf rows

`visual_block_view` will continue to use the existing paragraph, list, and task-list row renderers. Quote context adds left-border/padding/quote typography around each contiguous row and suppresses inter-row margins that would visibly break the border. First/middle/last metadata controls outer spacing; nested quote depth may draw repeated border/padding layers.

This keeps one editable element per virtual row and avoids duplicating list-marker logic in a second quote-specific renderer.

## Risks / Trade-offs

- [Changing the quote model touches many consumers and a `..` pattern could silently ignore children] -> Centralize text folding in `PreviewBlock::plain_text`, audit every `BlockQuote` match, and add consumer regressions for selection, stats, export, math, and memory accounting.
- [Range partitioning could drop or double-own `>`-only separator lines] -> Assert complete ordered source coverage, UTF-8 boundaries, no overlap, and no unexpected non-whitespace gap for quoted fixtures including CRLF.
- [Flat per-row borders could show seams] -> Carry quote-group edge metadata and use zero internal vertical spacing; verify the first, middle, and last rows in a rendered GPUI test.
- [Source-faithful straight quotes differ from Preview smart punctuation] -> Document the deliberate Visual Edit trade-off and keep the future non-identity projection gap explicit.
- [Changing child granularity could churn stable IDs after local edits] -> Reconcile by exact leaf ranges and source lineage, and test unaffected quoted siblings before and after prefix edits.
- [Incremental regions could parse a quote continuation differently from a full document] -> Reuse the conservative container-boundary rules and add incremental-versus-fresh-full tests around blank quoted lines and list continuations.

## Migration Plan

No persisted document, preference, recovery, or workspace format changes. Implement the semantic model and consumers first, then visual partition/prefix metadata, then GPUI styling and interactions. The change can be rolled back as one internal-model change; Markdown files remain untouched.

## Open Questions

None required before implementation. General non-identity decoded-text projection and arbitrary nested containers remain separate future changes.
