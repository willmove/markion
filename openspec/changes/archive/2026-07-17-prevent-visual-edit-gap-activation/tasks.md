## 1. Whitespace Interaction

- [x] 1.1 Derive each Visual Edit whitespace row's pointer interactivity from current source-caret ownership while preserving its source range and layout height.
- [x] 1.2 Keep active whitespace rows and heading-created insertion lines visibly editable without adding state to cached visual blocks.

## 2. Regression Coverage

- [x] 2.1 Add rendered pointer-interaction coverage proving heading-to-heading and heading-to-paragraph passive gap clicks do not change selection or document content.
- [x] 2.2 Add keyboard coverage proving Enter from a heading activates a source-backed insertion line and subsequent typing edits the exact source position.
- [x] 2.3 Assert the gap interactions preserve document version and cached derived-state identity when no edit occurs.

## 3. Verification

- [x] 3.1 Run Rust formatting and the focused Visual Edit test set.
- [x] 3.2 Run the full test suite and validate the OpenSpec change.
