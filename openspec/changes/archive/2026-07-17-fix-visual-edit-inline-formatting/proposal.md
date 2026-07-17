## Why

Visual Edit currently falls back an entire prose block to raw Markdown when one inline run is nested or otherwise marked conservative. The default welcome document contains nested bold-and-italic syntax in its Inline formatting paragraph, so all formatting in that paragraph appears unrendered even though most runs are byte-exact and supported.

## What Changes

- Keep supported inline formatting rendered in Visual Edit when a prose block contains safely nested strong/emphasis syntax instead of degrading the whole block to a source island.
- Render the existing `==highlight==`, `^superscript^`, and `~subscript~` extensions in the Visual Edit projection with exact source/display mappings and local marker reveal.
- Preserve conservative full-source fallback for escapes, ambiguous overlaps, malformed markers, inline HTML, and other byte-inexact constructs.
- Add model and rendered-window regressions using the default Inline formatting paragraph so future onboarding-content changes cannot silently disable visual rendering.

Non-goals: changing Markdown persistence, introducing a mutable rich-text tree, adding new inline syntaxes, or relaxing conservative fallback when exact UTF-8 source ranges cannot be proven.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Clarify that supported simple and safely nested inline formatting, including Markion's existing highlight/superscript/subscript extensions, remains visually rendered in Visual Edit while precise edits reveal only the relevant source markers.

## Impact

- Affects Visual Edit inline parsing and projection in `src/visual.rs`, visual reveal metadata in `src/model.rs`, and focused GPUI regression coverage in `src/app/tests.rs` / `src/app/preview.rs`.
- May expose a small range-aware helper from `src/parse.rs` so preview and Visual Edit use the same extended-inline grammar without duplicating syntax rules.
- Preserves `MarkdownDocument.text` as the sole canonical source and the existing per-document-version `Arc<VisualBlock>` cache; cursor-only reveal remains ephemeral and does not invalidate derived Markdown state.
- No dependency, file-format, localization, or public API changes are expected.
