## Context

See proposal.md. Mixed Markdown currently flattens individual HTML events; Visual Edit has a separate allowlist and exact-range stack; HTML blocks and HTML cells have independent style counters. GPUI HighlightStyle supports a uniform font size, so actual script positioning requires separately shaped fragments rather than a color-only highlight.

## Goals / Non-Goals

**Goals:** One tag/attribute semantic classifier and nested style state, exact source-backed reveal and navigation, cross-context style parity, authored break fidelity, and real script geometry.

**Non-Goals:** Browser CSS/DOM, changing persistence, and interpreting escaped Text events as HTML.

## Decisions

- Share an inline element descriptor (normalized tag name, style contribution, optional decoded href) and stack across the root parsers. Support the existing style tags plus span/font/a/kbd/samp. Accept inert metadata (class/id/title/lang/data/aria and anchor rel/target); interpret bounded color and basic inline style declarations only. Unknown active/layout attributes stay conservative in Visual Edit.
- Keep Visual Edit's source ranges and exact-pair validation; enrich frames with shared semantic descriptors and inherit navigation from enclosing anchors. Closing tags restore the previous style/link. Malformed constructs preserve all source bytes. No DOM serialization.
- Retain Markdown parsing around HTML Text events and keep the existing HTML-block renderer for block markup. All supported inline tags use shared semantics there and in cells; cell state resets between cells.
- Separate explicit br from structural paragraph boundaries. Explicit breaks emit one newline each, including repeated/leading/trailing breaks; rendered rows reserve height for empty lines.
- Render super/subscript as smaller text fragments at a metric-derived shifted baseline. Use the existing source-backed fragment and selectable preview text machinery so hit testing and IME geometry use actual shaped bounds. Revealed source keeps normal body geometry.
- Data flow: canonical Markdown/version -> cached blocks and exact runs -> per-frame projection -> shaped fragment layout. Hover, selection and focus do not rebuild per-version caches. No workspace crate gains a GPUI dependency.

## Risks / Trade-offs

- Fragmentation can affect wrapping and selection -> preserve full-run byte offsets and test line/selection/caret geometry.
- Shared parser changes could leak state -> cross-context and adjacent-cell reset tests.
- Link/image composition and source reveal can overlap -> test exact outer anchor groups, preserved destinations and one-mutation undo.

## Migration Plan

No data migration; source bytes remain canonical. Revert the change to roll back presentation behavior.
