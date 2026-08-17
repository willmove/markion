## Why

Visual Edit renders two common blockquote forms incorrectly. A GFM alert (`> [!NOTE]` … `> body`) splits into an unstyled raw-source island for the `> [!NOTE]` line plus a plain quote row below it, because pulldown-cmark consumes the alert marker as block structure, leaving those bytes unowned by any preview leaf. Separately, a multi-line quoted paragraph written with lazy continuation (`> line one` newline `> line two`) loses its line breaks and renders the lines run together, because the inline parse slice still contains the later lines' `>` markers, which split the paragraph without emitting a soft-break event. Both violate the "every source byte has exactly one visual owner, rendered form preserved" contract of Visual Edit.

## What Changes

- Model the GFM alert kind that pulldown-cmark already reports on the blockquote tag (`Note`/`Tip`/`Important`/`Warning`/`Caution`) instead of discarding it, and keep alert quotes with no body in the preview block model.
- Render the alert marker line in Visual Edit as a styled callout title row inside the quote group (label plus kind accent), replacing today's `Unsupported` source island. The row owns its exact source bytes; focusing it reveals `> [!NOTE]` verbatim through the existing marker-reveal mechanism.
- Preserve line breaks of multi-line quoted paragraphs in Visual Edit by giving the newline byte between lazy-continuation lines a synthetic soft-break run, so `> a` / `> b` renders as two lines instead of `ab`.
- Keep unknown markers such as `[!CUSTOM]` as literal paragraph text (on their own line once soft breaks are preserved), matching GitHub's behavior for unrecognized alert types.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: adds a requirement covering Visual Edit blockquote fidelity — GFM alert title rows render as part of the quote group with focus-time source reveal, multi-line quoted paragraphs keep their authored line breaks, and structural quote lines keep their current whitespace-row behavior.

## Impact

- `src/model.rs`: new alert-kind enum; `PreviewBlock::BlockQuote` gains the alert field; new `VisualBlockKind` variant for the callout title row.
- `src/lib.rs`, `src/parse.rs`: capture the blockquote tag's alert kind; retain body-less alert quotes.
- `src/visual.rs`: quote-group gap classification (alert marker line → title row instead of unsupported island) and soft-break synthesis for quoted leaves.
- `src/app/preview.rs`: view arm and accent styling for the callout title row.
- `src/document_memory.rs`, `src/source_mapped.rs`, `src/block_edit.rs`, `src/app/editing.rs`: exhaustive-match updates for the new block kind.
- Invariants preserved: every source byte keeps exactly one visual owner; display↔source mappings stay UTF-8-safe and monotonic; derived visual-block state stays cached per document version (no per-keystroke recompute).

Non-goals: Preview/Read mode and export styling for alerts (the parser still drops the `[!NOTE]` label there — unchanged); localizing alert labels; nested-quote alert layouts beyond current behavior; new block-insertion commands for alerts.
