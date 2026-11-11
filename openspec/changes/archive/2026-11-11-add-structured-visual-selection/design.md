## Context

See proposal.md. Text/IME already use canonical UTF-8 ranges and checked mutations. Block targets carry version/ID; table cells carry exact source ranges. Keep those proofs as the single mutation authority.

## Goals / Non-Goals

Goals: explicit text, contiguous block and rectangular cell selections; one checked transaction/undo step; common operation resolution and multi-format clipboard.

Non-goals: persistent selection trees, disjoint arbitrary block lists, merged cells, or arbitrary rich block content inside GFM cells.

## Decisions

- Add pure `SelectionState`, block/cell targets and `EditTransaction` over the existing checked mutation boundary. Text range remains the platform IME representation; structured selection is exclusive and resets on ordinary text navigation/input and stale document versions.
- Escape selects the current source unit; Shift+Up/Down extends block selection, arrows move it, and a second Escape returns to text. Shift-click extends blocks or cells. Block grips carry the selected range when dragged.
- Resolve selections to exact source units/cell fields. Clipboard includes visible text and HTML; explicit Markdown copy preserves authored bytes. Cell matrices use TSV and HTML tables.
- Deletion and move compute a minimal contiguous replacement and resulting selection before application. Validate document identity/version and selected IDs again at commit.

Data flow: input intent -> selection transition -> pure source-range resolution -> transaction -> checked mutation -> one history snapshot -> text/structured selection repair -> versioned visual cache.

## Risks / Trade-offs

- Legacy direct assignments to text selection -> validate structured state at every consuming boundary and clear it on normal selection changes.
- Clipboard platforms vary -> use GPUI's supported plain/HTML flavors and test the actual Windows clipboard.
- Nested source ownership -> resolve complete source units and reject ambiguous boundaries.
