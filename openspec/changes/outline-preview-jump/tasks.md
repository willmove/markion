## 1. Preview Navigation

- [ ] 1.1 Map outline heading offsets to preview block indexes without recomputing derived Markdown state.
- [ ] 1.2 Add a preview scroll helper that focuses the rendered heading block in Read mode.
- [ ] 1.3 Route outline clicks to source navigation in Edit mode and preview navigation in Read mode.

## 2. Verification

- [ ] 2.1 Add focused coverage for heading-to-preview block lookup and mode-specific outline click behavior where the existing test harness allows it.
- [ ] 2.2 Run `openspec validate outline-preview-jump` and `cargo test`.
