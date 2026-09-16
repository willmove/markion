## 1. Source Transformation

- [ ] 1.1 Add `MarkdownFormat::Paragraph` and implement an ATX-only line transform that removes heading prefixes, preserves mixed non-heading lines, keeps UTF-8 caret/selection endpoints anchored to content, and avoids applying a document mutation when nothing changes; verify focused `MarkdownDocument` unit tests pass
- [ ] 1.2 Add document-level regression cases for H1–H6, a caret inside a heading, mixed multi-line selections, ordinary paragraphs, invalid hash prefixes, indented/code-like lines, UTF-8 content, and no-op version/cache behavior; verify the new test subset passes

## 2. Action and Shortcut Integration

- [ ] 2.1 Add the GPUI Paragraph action and handler, route it through the existing formatting/undo/dirty-state lifecycle, and verify action-level tests cover both a successful conversion and the existing no-formatting-change status
- [ ] 2.2 Add the stable `paragraph` shortcut descriptor with `secondary-0` / `Ctrl+0` / `Cmd+0`, include it in the complete registry and keymap rebuild, and verify registry uniqueness, stored-override sanitization, default dispatch, live override, and reset tests pass

## 3. Menu and Localization Integration

- [ ] 3.1 Place Paragraph immediately before H1 in both native and in-window Format menus with its effective shortcut label, and verify menu construction/source-contract tests cover placement, action wiring, and image-document disabled behavior
- [ ] 3.2 Add localized Paragraph menu/reference and success-status strings for every supported language, include the `paragraph` id in the Editing shortcut catalog beside heading actions, and verify localization completeness plus both heading-depth catalog variants pass

## 4. Verification

- [ ] 4.1 Run `cargo fmt --check` and `cargo test --workspace`, fixing only regressions within this change while preserving the per-document-version derived-state, syntax-highlight, and cached-text-handle invariants
- [ ] 4.2 Run `openspec validate add-paragraph-format-action` and reconcile the completed checklist with the validated proposal, design, and delta specification
