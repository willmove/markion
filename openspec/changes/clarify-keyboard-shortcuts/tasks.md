## 1. Shortcut Binding

- [x] 1.1 Change the sidebar toggle key binding from `secondary-alt-b` to `secondary-shift-b`.
- [x] 1.2 Update any nearby comments so they describe the new binding and the Bold shortcut remains unambiguous.

## 2. Shortcut Reference

- [x] 2.1 Convert the English and extended English keyboard shortcut reference text to a table with Windows/Linux and macOS columns.
- [x] 2.2 Convert the Simplified and Traditional Chinese shortcut reference text to the same platform-specific table shape.
- [x] 2.3 Convert the Japanese, French, German, and Spanish shortcut reference text to the same platform-specific table shape.
- [x] 2.4 Ensure the sidebar row documents Ctrl+Shift+B on Windows/Linux and Cmd+Shift+B on macOS.
- [x] 2.5 Render the shortcut tables as prompt-safe plain text instead of Markdown table source.

## 3. Verification

- [x] 3.1 Update shortcut-reference tests to assert table headers, explicit platform keys, and absence of leaked `Secondary-` labels.
- [x] 3.2 Run `openspec validate clarify-keyboard-shortcuts`.
- [x] 3.3 Run `cargo test`.
