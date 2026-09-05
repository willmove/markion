## 1. Lock In Failing Evidence

- [x] 1.1 Add a pure regression fixture with two valid equal-length data-URI SVGs whose complete bytes differ only outside the former head/middle/tail samples; prove the current keys alias before changing the implementation.
- [x] 1.2 Add Markdown front-matter round-trip tests for one-element sequences, one-entry mappings, nested values, multiline/control-character strings, Unicode line separators, YAML-looking scalars, and a standalone `---` line.
- [x] 1.3 Add DOCX and PDF pandoc-input tests that parse the complete generated front matter and compare hostile title overrides semantically, including newline, carriage return, quotes, backslashes, control characters, and standalone `---`.
- [x] 1.4 Characterize default `pulldown-cmark` event and source-offset shapes for trailing `H~2~`, escaped tildes, `~~strike~~`, Unicode content, and candidates adjacent to code/link/emphasis events; record the expected ownership in tests.

## 2. Make Image Identity Complete and Repaint-Bounded

- [x] 2.1 Add a typed `ImageSourceIdentity` using normalized local/remote identity and `{byte_len, SHA-256}` for complete data URIs; add `sha2.workspace = true` to the root package without adding GPUI dependencies to member crates.
- [x] 2.2 Compute and intern data-URI identities during per-document-version preview/visual derivation, propagate them through block-level, inline, and HTML image descriptors, and ensure clone/undo/cache-eviction paths preserve the existing derived-state invariants.
- [x] 2.3 Change `PreviewImageKey`, URL/source collection, cache lookup, claim reconciliation, and failed-source tracking to consume the precomputed identity; remove `DATA_URI_KEY_FULL_HASH_MAX`, sampled hashing, and every repaint-time full-payload hash.
- [x] 2.4 Preserve the pending `Arc<str>` payload side table so a newly reserved data URI is copied once for decode and released on success, error, cancellation/removal, or an unclaimed late completion; keep memory accounting accurate.
- [x] 2.5 Extend pure and GPUI tests to prove distinct keys and rasters for the adversarial equal-length SVG pair, key invalidation after an unsampled same-length edit, shared identity for identical sources, and zero payload hash/clone work across repeated repaints.
- [x] 2.6 Run the root image/cache suites and commit the complete image-identity repair as one reviewable change.

## 3. Centralize Typed YAML Serialization

- [x] 3.1 Replace scalar/container line construction in `crates/markdown` with one canonical helper that serializes the complete `YamlFrontMatter` mapping and wraps it in Markdown delimiters.
- [x] 3.2 Make `render_to_markdown` use the canonical helper and prove typed parse/render/parse equality for recognized fields, tags, empty values, single-element and nested custom containers, hostile strings, and multiple custom keys.
- [x] 3.3 Refactor DOCX and PDF pandoc-input construction to apply `title_override` to cloned or newly built typed metadata and call the canonical renderer; remove `escape_yaml_string` and line-oriented title replacement.
- [x] 3.4 Verify both exporters preserve the complete override value in parsed transient input, do not create unintended YAML boundaries, and do not mutate the source `Document`; retain existing behavior when no override is supplied.
- [x] 3.5 Run `cargo test -p markdown` and `cargo test -p export`, then commit the YAML/front-matter repair as one reviewable change.

## 4. Complete Extended Inline Parsing and Reframe UTF-8 Containment

- [x] 4.1 Implement the narrow contiguous-text-event adapter selected from task 1.4, using original source-offset provenance to reconstruct only proven single-tilde candidates and falling back to literal text whenever ownership is ambiguous.
- [x] 4.2 Add default-parser tests for subscript at the beginning, middle, and end of a paragraph; Unicode subscript content; escaped delimiters; GFM strikethrough; and adjacency to code, links, emphasis, HTML, soft/hard breaks, and other semantic events.
- [x] 4.3 Add preview and export assertions showing trailing subscript reaches downstream consumers as `Inline::Subscript` under default options.
- [x] 4.4 Keep the existing UTF-8 clamp and safe-selection implementation, but rename/rewrite synthetic Callout fixtures and comments as malformed/stale-range containment; do not claim an ordinary Callout reproduction without a real parsed user-input path.
- [x] 4.5 Run Markdown parser tests and the affected root GPUI editing/navigation tests, then commit the parser and test-accuracy repair as one reviewable change.

## 5. Integration and Merge Readiness

- [x] 5.1 Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`, the MarkNice `verify-bundle` command, and `openspec validate --all --strict --no-interactive`; record ignored tests and existing warnings separately from failures.
- [x] 5.2 Validate both `harden-audit-fixes-before-merge` and the overlapping `fix-visual-data-uri-source-toggle-freeze` change, confirming that complete identity supersedes only the old task 5.1 implementation assumption while retaining bounded-frame and elision behavior.
- [x] 5.3 Review `git diff main...HEAD` for unrelated scope, verify the adversarial cache and semantic YAML tests fail if their fixes are removed, and update the final defect report to distinguish proven user paths from defense-in-depth containment.
- [x] 5.4 Mark this change complete only after all merge-blocking regressions are fixed and every required gate passes; leave manual multi-megabyte image interaction verification explicitly reported if it remains outstanding.
