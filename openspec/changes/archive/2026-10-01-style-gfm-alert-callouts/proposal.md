## Why

GFM alerts (`> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`) are parsed, but Split Preview and Read mode render them exactly like plain blockquotes: gray rule, no title, no color. Visual Edit shows only a colored title word, while the body keeps the gray quote rule. Technical notes and docs lean heavily on these callouts, and GitHub and Obsidian both present them as distinct, color-coded cards, so the current presentation loses meaning.

## What Changes

- Split Preview and Read mode render each GFM alert as a callout card. The card has a kind-colored left border, a subtle kind-tinted background gradient, and a title row with the kind's icon and its label (Note, Tip, Important, Warning, Caution). Plain blockquotes keep their current look.
- Visual Edit uses the same kind color for the alert's quote rule and background across the whole quote group, and adds the kind icon to the existing callout title row. Progressive source reveal of the marker line is unchanged.
- In Split Preview and Read mode, the callout title row folds and unfolds the alert body. Fold state is view-only, session-only UI state. It never edits the document.
- Icons are bundled SVGs (Lucide, already the project's icon set), so they render the same on every platform without emoji fonts.

Non-goals: Obsidian-only syntax (`[!NOTE]-` / `[!NOTE]+` fold markers, custom titles after the marker, extra callout types). pulldown-cmark recognizes only the five GFM markers, and those forms keep rendering as ordinary quotes. Also out of scope: folding inside Visual Edit, where every byte must stay reachable for editing; changes to export formats; localizing the alert labels, which are document content, as the existing title row already treats them.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: adds a requirement for styled GFM alert callouts across Split Preview, Read mode, and Visual Edit, including the preview fold toggle. The existing "Visual Edit blockquote and GFM alert fidelity" requirement (structure, byte ownership, reveal) is unchanged.

## Impact

- Code: `src/app/preview.rs` (preview blockquote rendering, Visual Edit quote-row chrome, title row), `src/model.rs` / `src/visual.rs` (the alert kind carried on the Visual Edit quote context), `src/app/state.rs` (per-tab folded-alert set), `src/ui/icon.rs` plus five SVGs under `assets/icons/ui/`, and tests.
- Invariants: folding and styling are render-time and UI-only. Document text, version, dirty state, undo history, and per-version derived caches (preview blocks, visual blocks, outline) are unaffected. The alert kind is already part of the cached preview blocks, and the Visual Edit quote context gains one copied field computed in the same pass.
