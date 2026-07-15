## 1. Curate the default document

- [x] 1.1 Replace the short `# Welcome to Markion` string with a structured, Markion-specific Markdown tour covering every syntax category in the `markdown-editing` delta spec.
- [x] 1.2 Keep image syntax deterministic and remove or replace unrelated product, social-media, and messaging-platform references from the sample.
- [x] 1.3 Centralize the welcome-document text if needed so application construction and tests cannot drift, without changing document lifecycle or Markdown-cache behavior.

## 2. Protect editing behavior

- [x] 2.1 Update the focused Visual Edit test to use the expanded sample and verify normal editable content plus intentional conservative source islands.
- [x] 2.2 Add or adjust assertions that the fresh document begins with the welcome heading and contains representative supported syntax.

## 3. Validate

- [ ] 3.1 Run `cargo fmt --check` and the focused welcome/visual-editor tests.
- [x] 3.2 Run `cargo test` and confirm the expanded in-memory document does not affect per-version derived-state caching behavior.
