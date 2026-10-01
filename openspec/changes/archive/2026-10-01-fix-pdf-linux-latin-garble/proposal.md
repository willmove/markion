## Why

On Linux (reproduced on Omarchy), the built-in PDF writer renders regular and italic body Latin as Greek lookalikes (`This` → `Τηις`). Headings, bold, and monospace stay correct. The writer asks cosmic-text for the CSS generic `serif` family; fontdb takes fontconfig's last `serif` alias, which on some systems is a Pi/Symbol face whose cmap draws Adobe Symbol glyphs at Latin code points. Bold works only because those faces have no bold, so fallback hits a real serif.

## What Changes

- After loading system and bundled fonts, pin the PDF writer's generic `serif` family to the bundled Libertinus Serif Latin face so fontconfig cannot select a Symbol/Pi encoding for body text.
- Request that named family for PDF and snapshot body text (not the host `serif` generic), and drop Pi/Adobe-Symbol faces from the font database so fallback cannot pick them.
- Keep heading (`sans-serif`) and code (`monospace`) generic resolution unchanged; CJK still falls back through per-OS system fonts then the bundled Noto Sans SC subset.
- Add a regression that Latin body text is shaped by Libertinus Serif on the process-wide (system-font) path, not a Symbol/math face.
- **Non-goals**: changing preview/editor fonts; making PDF follow the user's theme font slots; bundling extra Latin sans faces; altering heading or code stacks; pandoc/XeLaTeX PDF.

## Capabilities

### New Capabilities

- none

### Modified Capabilities

- `export`: the built-in PDF font requirement gains a Latin-script fidelity scenario — regular and italic body letters must keep their authored script, not a Symbol/Pi encoding — and states that the body stack's Latin face is the bundled Libertinus Serif regardless of the host fontconfig `serif` alias.

## Impact

- `crates/pdf` font setup (`fonts.rs`), body Attrs (`text.rs`, `raster.rs`). No new dependencies. Does not touch per-version Markdown caches, highlighting, or the typing path — font setup is process-wide and already runs once per export/snapshot.
