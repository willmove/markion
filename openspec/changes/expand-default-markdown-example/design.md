## Context

`Application::new_document()` currently creates a short, duplicated string literal in the app source; a visual-editor unit test uses the same literal independently. The parser and preview already support CommonMark/GFM constructs and Markion's enabled extensions, but the initial document exercises only basic emphasis, a list, a table mention, a code-block mention, and a task list.

The revised sample is an in-memory onboarding document, not UI chrome. It must remain English and non-localized, as required for welcome Markdown by `ui-i18n`.

## Goals / Non-Goals

**Goals:**

- Provide a clear, readable sample that exercises Markion's already supported Markdown syntax.
- Use Markion-oriented, neutral examples with no references to social-media, messaging, or unrelated products.
- Preserve a source-backed Visual Edit test that distinguishes directly editable blocks from intentionally conservative source islands.

**Non-Goals:**

- Add parser, renderer, export, formatting-action, or localization functionality.
- Fetch an image or introduce network-dependent starter-document behavior.
- Turn the sample into a translated template or modify any user file.

## Decisions

### Keep the example as a fixed raw Markdown string in the application layer

`new_document()` will continue to construct the in-memory text directly. This preserves the current lifecycle for a fresh or last-closed tab and avoids a new asset-loading or persistence path. The sample will be organized into short sections covering headings, paragraphs and inline styles, links and image syntax, blockquotes and rules, ordered/nested/task lists, tables, inline/fenced code, inline/display math, footnotes, and supported extended inline syntax.

An image example will use a non-fetching, illustrative URL or clearly labelled syntax rather than a real remote asset. This keeps first launch deterministic while still demonstrating image Markdown.

Alternative considered: ship a `.md` template file. Rejected because it adds packaging/path concerns for a small built-in document and would change the current no-filesystem startup behavior.

### Make example prose product-specific and self-contained

Examples will refer to Markion and normal Markdown-writing tasks. Names of third-party tools, messaging platforms, promotional channels, or unrelated workflow instructions will be omitted or replaced with generic Markion-oriented language.

Alternative considered: retain source example wording verbatim. Rejected because it can promote irrelevant services and does not match the product's voice.

### Update the focused Visual Edit test for the richer document

The unit test will source the same expanded example content and assert the expected mix of ordinary visual blocks and conservative islands (for constructs such as images, fenced code, display math, and other unsupported direct-edit regions). Its assertions will test the documented behavior rather than relying on the previous five-block count.

Alternative considered: remove the test because the example is longer. Rejected because the default document is a useful regression surface for visual-block derivation.

## Risks / Trade-offs

- [A broad sample becomes hard to read] → Use descriptive section headings and concise examples rather than exhaustive variants of every marker.
- [A syntax sample renders as a conservative Visual Edit island] → Make the test explicitly account for intentional islands and retain an editable-prose assertion.
- [Duplicated string literals drift] → Update the application and test together; implementation may extract a private shared constant if that reduces duplication without changing behavior.
- [Remote image references affect preview] → Do not require a fetchable remote image for the sample.
