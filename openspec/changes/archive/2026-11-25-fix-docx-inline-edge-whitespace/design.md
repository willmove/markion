## Context

`Converter::render_run` resolves all active inline wrappers, then its `flush_text` closure sends the raw text to `InlineBoundaryState` before wrapping the entire escaped string. `InlineBoundaryState` therefore already has the correct source-level information for bold compatibility spaces, but `ChunkWriter` receives delimiters adjacent to edge whitespace. See `proposal.md` for motivation and the delta spec for required behavior.

## Goals / Non-Goals

**Goals:**

- Preserve raw source-boundary decisions while serializing edge whitespace outside the complete wrapper stack.
- Keep the change local to the existing run-to-chunk data flow, with no additional allocations for ordinary non-empty styled content beyond those already used for escaping.
- Leave `PreparedImport`, application preview parsing, and all per-document-version caches unchanged.

**Non-Goals:**

- Rewriting generated Markdown after serialization or post-processing the application preview.
- Merging adjacent runs or changing Word style inheritance and whitespace normalization rules.

## Decisions

1. Split each flushed raw text segment into leading whitespace, a non-whitespace-containing core, and trailing whitespace before Markdown escaping. Emit `leading + before + escaped(core) + after + trailing` through `ChunkWriter`. This preserves every source character and puts all nested emphasis/HTML wrappers around the same core. A delimiter-specific fix was rejected because combined styles and safe-HTML wrappers need consistent boundaries.
2. Call `boundary.prepare_text` with the original unsplit text before emission. The existing pending-bold state thus continues to reflect the authored last character, especially a trailing source space. Recomputing boundaries from the trimmed core would add unwanted synthetic spaces.
3. Pass whitespace-only segments through without `before` or `after`. This avoids empty markup while retaining the existing recovered-content semantics.
4. Derive split offsets from Rust string trimming lengths rather than character counts so slicing remains on UTF-8 boundaries. Escaping each emitted text piece independently is equivalent for whitespace because Markdown escaping does not alter whitespace.
5. Keep the data flow as raw DOCX text → source boundary tracking → whitespace split and escaping → `MarkdownChunk` → `PreparedImport::render_markdown` → existing application `MarkdownDocument` parsing and caches. No cache key, document version, or derived-state behavior changes.

## Risks / Trade-offs

- [Whitespace-only or all-whitespace segments can produce invalid slice ranges] → Handle them before calculating the core range and cover them in regression tests.
- [Moving a trailing hard line break outside a wrapper can change formatting boundaries] → Treat all trailing whitespace uniformly and verify existing run-break tests remain green.
- [Previously imported documents may already contain literal delimiters] → Do not rewrite saved documents; users can re-import the DOCX to obtain corrected Markdown.

## Migration Plan

No dependency, API, schema, or stored-data migration is required. The fix takes effect on subsequent DOCX imports. Rollback is a revert of the localized serializer and test change.
