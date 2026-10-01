## Why

In Visual Edit, links and footnote references render with link/superscript styling but are not actionable: clicking the label places the caret for editing. Users therefore cannot open a URL or jump to a footnote definition without switching to Split Preview / Read or editing the raw destination. A dedicated clickable affordance beside the construct keeps caret editing intact while restoring navigation.

## What Changes

- Visual Edit attaches a resolved navigation target to each actionable inline link run (inline and reference-style) and each footnote reference run.
- Visual Edit paints a small icon beside those constructs; clicking the icon opens the URL (external/internal absolute URL via the platform opener) or moves the caret/scroll to the matching footnote definition block.
- Clicking the link/footnote label text continues to place or drag the source selection (editing path unchanged).

Non-goals: Ctrl/Cmd+click on the label itself; hover-only tooltips beyond the icon; back-links from footnote definitions to every reference; changing Split Preview/Read click behavior.

## Capabilities

### New Capabilities

<!-- None -->

### Modified Capabilities

- `markdown-editing`: Add Visual Edit navigation-affordance requirements for links and footnote references.

## Impact

- **Code:** `src/model.rs` (navigation target type on runs), `src/visual.rs` (populate targets during inline parse), `src/app/preview.rs` (icon UI + click handlers), possibly `src/app/editing.rs` (jump helper). Cached visual derivation only.
- **Invariants:** `MarkdownDocument.text` remains canonical; icons are presentation-only and do not alter source ranges or document version.
