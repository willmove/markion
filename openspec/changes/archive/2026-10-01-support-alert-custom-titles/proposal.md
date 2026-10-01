## Why

Users write alerts the Obsidian way, with a title after the marker: `> [!NOTE] 注意`. pulldown-cmark recognizes a GFM alert only when `[!TYPE]` is alone on its line, so these quotes render as plain blockquotes with the literal `[!NOTE] 注意` as the first line of text, in every surface and every export. A user report showed exactly this.

## What Changes

- A blockquote whose first line is `[!TYPE] Title` is a titled alert of that type and renders with the custom title. TYPE is one of `NOTE`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`, case-insensitive. The title replaces the default label (Note, Tip, …). The marker line is not part of the alert body.
- Split Preview, Read mode, and Visual Edit show the custom title in the callout title row. In Visual Edit, focusing the title row still reveals the exact authored marker line, including the title, for editing.
- Exports show the title: DOCX, PDF, and LaTeX label the alert with it. HTML export gives every alert, titled or not, a title paragraph, and turns a titled alert's quote into an alert quote.
- Untitled GFM alerts behave as before, except that HTML and LaTeX exports now also show their default label.

Non-goals (deferred by the user): Obsidian fold markers (`[!NOTE]-` / `[!NOTE]+`), callout types beyond the five GFM kinds, and inline formatting in titles (titles render as plain text).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: adds "GFM alerts SHALL accept a custom title after the marker".
- `export`: adds "Exports SHALL show alert titles".

## Impact

- Code: `src/lib.rs` (preview builder titled-alert split; HTML event rewrite; LaTeX alert label), `src/parse.rs` (marker parser), `src/model.rs` (`BlockQuote.alert_title`, `CalloutTitle.title`), `src/visual.rs`, `src/source_mapped.rs`, `src/app/preview.rs` (title display), `src/export.rs` (DOCX, PDF), `crates/pdf` (`Alert.title`), and tests.
- Invariants: detection runs once per parse, inside the existing preview-block pass, and is cached per version. Visual Edit ownership is unchanged: the marker line belongs to the callout title row, and every other byte keeps its owner.
