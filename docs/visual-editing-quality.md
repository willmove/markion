# Visual Edit Support and Engineering Contract

Markion's Visual Edit mode is WYSIWYG-oriented, not a second rich-text document model. `MarkdownDocument.text` remains the only persisted representation. A visual interaction is supported only when it can map to an exact, UTF-8-safe source mutation; uncertain syntax keeps a complete source-backed editing path.

## Support Matrix

| Construct | Normal Visual Edit presentation | Canonical editable range | Conservative fallback trigger | Required evidence |
|---|---|---|---|---|
| Paragraphs, headings, blockquotes, list/task items | Rendered direct text | Exact inline content and structural prefix ranges | Byte-inexact or crossing parser events | Projection round-trip, formatting, structural Enter/Backspace, pointer, IME, undo |
| Emphasis, strong, strike, inline code, links, highlight, super/subscript, inline math | Rendered with progressive source reveal | Smallest complete proven syntax group | Malformed, escaped, overlapping, or ambiguous syntax | Reveal-group containment, caret affinity, cross-run selection, UTF-8 input |
| Ordinary fenced code | `VisualBlockEditor::Code`: highlighted payload editor with fences hidden | Payload only; opening fence, info string, and closing fence are immutable metadata | Unclosed/ambiguous fence or registered diagram backend | Exact fence/payload ranges, memoized highlights, delimiter preservation, edge handoff, IME/history |
| Display/fenced block math | `VisualBlockEditor::Math`: rendered formula plus LaTeX payload editor | LaTeX payload only; delimiters remain authored | Inline-only/ambiguous form | Valid/invalid/pending render, delimiter preservation, CJK/emoji IME geometry, one-action undo |
| Inline Markdown image | `VisualBlockEditor::Image`: bounded preview plus alt, destination, and optional title fields | Individual proven field range | Reference, angle-bracket destination, multiline, or malformed form | Field escaping, broken/local/remote presentation, Tab traversal, stale-event rejection, multi-tab isolation |
| GFM table | `VisualBlockEditor::Table`: editable header/body cells plus row/column toolbar | Logical cell; one deterministic full-table replacement for width reflow | Unequal/ambiguous cell boundaries or malformed table | Escaped-pipe projection, UTF-8 width reflow, alignment/export preservation, traversal, IME/history |
| Horizontal rule | Passive rendered rule | Exact block boundary through ordinary navigation/format commands | N/A | Source coverage and navigation tests |
| Blank lines and trailing whitespace | Passive visual row; focused exact source caret/island | Exact whitespace range | N/A | Complete source coverage, insertion/deletion, no pointer-created phantom edits |
| Mermaid/registered diagrams | Complete source-backed fence island | Complete authored fence | Always; diagram preview is presentation-only | Registry classification, source preservation, async/cache/version isolation |
| HTML and YAML front matter | Complete source-backed island | Complete authored block | Always | Exact source preservation, preview/export behavior |
| Unsupported or malformed constructs | Complete conservative source island | Complete containing source range | Exact mapping cannot be proven | Lossless source-mode round-trip and no guessed mutation |

## Source-Range Invariants

Every derived range must:

1. Be contained by the current canonical document and start/end on UTF-8 boundaries.
2. Be contained by its owning `VisualBlock`; direct fields and delimiter metadata must not overlap unrelated source.
3. Round-trip to the intended authored slice, including CRLF, indentation, blank lines, escaping, and final-newline semantics.
4. Move only through a consecutive `SourceEdit` chain or be rebuilt by the full-derivation fallback.
5. Reject a `VisualBlockEdit` when document version, `VisualBlockId`, field metadata, or replacement policy is stale.
6. Preserve exact post-edit selection/marked ranges after sanitization or table reformatting.

Interaction-only state such as focus, hover, selection, caret affinity, layout geometry, Tab traversal, and scroll position must not change the document version or invalidate per-version derived caches.

## Parser Ownership

- `pulldown-cmark` in the root document model owns semantic Markdown classification.
- `src/visual.rs` proves byte boundaries only inside an already-classified preview block. It must return no direct metadata when round-trip proof fails.
- `src/table.rs` owns the shared GFM table range, parse, format, and mutation rules used by source commands and Visual Edit.
- `crates/markdown` owns its GPUI-free parser/AST and exporter contracts. It does not own a parallel Visual Edit mutation model.
- Exporters consume canonical Markdown semantics; a widget never mutates an exporter model, preview block, image object, or math cache directly.

A new boundary helper must not become a second document parser. Prefer an exact, narrow recognizer plus source-island fallback over broader guessed support.

## Verification Layers

- Pure document tests: exact ranges, escapes, stale edits, table reflow, full/source round-trips.
- Differential/property tests: randomized UTF-8 edits and incremental output versus a fresh full derivation.
- Rendered GPUI tests: projection, caret/selection, keyboard handoff, platform input, IME bounds, undo/redo, mode switching, multi-tab isolation, and virtualization.
- Workspace tests: parser, diagram, exporter, and doctest compatibility.
- Deterministic performance tests: parsed/reused-region counters, stable IDs, shared `Arc` identity, and bounded dirty-region work.
- Informational benchmark: `cargo run --release --example bench_large_doc`. Timing is diagnostic and is not a merge threshold without dedicated benchmark hardware.

## Quality Gate

Run the complete local gate from the repository root:

```powershell
pwsh ./scripts/check-quality.ps1
```

It runs formatting, `cargo test --workspace`, and strict validation of every OpenSpec change/spec. CI runs the same gate in `.github/workflows/quality.yml`. Tests that explicitly require external network access, pandoc, or a PDF engine remain reported as ignored when their prerequisites are absent.

## Change Checklist

Every proposal that adds or changes a visual strategy must state:

- The strategy: rendered direct text, progressive reveal, dedicated editor, passive exact position, or source island.
- The semantic parser owner and the exact source-range proof owner.
- The canonical edit range, post-edit selection, escaping/normalization, and stale-event policy.
- The malformed/ambiguous fallback trigger.
- UTF-8/CRLF, pointer/keyboard, CJK/emoji IME, semantic undo, multi-tab, cache/identity, large-document, source-mode, and exporter evidence affected by the change.
- The corresponding row update in this matrix and delta spec changes.
