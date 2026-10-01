## 1. Model + derivation

- [x] 1.1 Add `VisualNavigationTarget` and optional `navigation` on `VisualInlineRun`
- [x] 1.2 Populate URL navigation for link runs and footnote navigation for footnote-reference runs in `inline_runs`
- [x] 1.3 Unit-test that the Notes fixture attaches URL and footnote navigation targets

## 2. Visual Edit UI

- [x] 2.1 Render a compact clickable icon after navigable runs (fragmented flex layout when needed)
- [x] 2.2 Wire icon click: `open_url` for URLs; caret + scroll to matching `FootnoteDefinition` for footnotes
- [x] 2.3 Ensure label clicks still only update source selection

## 3. Verification

- [x] 3.1 Run related `visual::tests` / lib tests and `openspec validate`
