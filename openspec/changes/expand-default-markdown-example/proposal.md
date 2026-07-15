## Why

The starter document currently demonstrates only a few Markdown constructs, so a first launch does not give users a useful tour of Markion's editing and preview support. A comprehensive, product-appropriate example will make the default document both an onboarding aid and a quick visual smoke-test surface.

## What Changes

- Replace the short in-memory `# Welcome to Markion` document with a structured Markdown example that covers the Markdown syntax supported by Markion, including headings, inline formatting, links, images, lists, task lists, quotations, tables, code, math, footnotes, and extended inline syntax where supported.
- Curate all prose, labels, links, image references, and tool names for Markion; remove content that is unrelated to a desktop Markdown editor, including social-media or messaging-platform promotion.
- Keep the welcome document as fixed, non-localized sample document content, and update focused visual-editor coverage to validate the expanded starter content remains compatible with the source-backed visual editing path.

Non-goals: This change does not add Markdown parser or preview capabilities, add localized variants of the sample, or alter user-created documents.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Define the comprehensive, Markion-curated Markdown example supplied in a fresh in-memory document.

## Impact

- Affects default-document construction in `src/app/application.rs` and its focused visual-editor test in `src/visual.rs`.
- No public API, dependency, parser, rendering, cache-invalidation, or localization changes are expected; existing per-document-version derived-state caching remains unchanged.
