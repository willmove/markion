## 1. Regression Guards

- [x] 1.1 Add a focused `src/app/tests.rs` guard that isolates the Preferences language-picker source and requires a wrapping language row plus a dedicated language-pill renderer.
- [x] 1.2 Make the guard reject the current `format!("{}  {}", ...)` marker/label construction and require non-shrinking pill content with a separate fixed marker slot.

## 2. Responsive Language Picker

- [x] 2.1 Update the General Preferences modal to target 640 logical pixels while remaining capped to the overlay's available width with a safe horizontal inset; leave the Shortcuts target at 720 pixels and preserve the existing height/scroll bounds.
- [x] 2.2 Change the language-choice container to wrap complete children while preserving the supported-language order and current selection listeners.
- [x] 2.3 Add a language-specific pill helper with a 72-pixel minimum, zero flex shrink, the existing active/inactive palette behavior, and a centered marker-plus-label group.
- [x] 2.4 Render U+2713 only in a fixed leading marker slot for the active language, keep the inactive slot empty at the same width, and use a compact 2-pixel visual gap before the unmodified `native_name()` label.

## 3. Automated Verification

- [x] 3.1 Run `cargo fmt --check` and resolve any formatting differences introduced by this change.
- [x] 3.2 Run the focused app test covering the language picker, then run `cargo test` and `cargo build` for the root Markion package without altering unrelated user changes.
- [x] 3.3 Run `openspec validate fix-preferences-language-overflow` and resolve every proposal/spec/design/task consistency error.

## 4. Manual Layout Verification

- [x] 4.1 On Windows at 2560×1600 and 125% scaling with a normal proportional UI font, confirm all seven pills stay on one comfortable row and each language remains selectable.
- [x] 4.2 At the same resolution and scale with a wide or monospaced UI font (or an equivalent test override), select each language in turn and confirm exactly one compact check mark plus seven complete, non-overlapping native labels remain inside their borders.
- [x] 4.3 Narrow the window until the language row wraps and confirm whole pills move to the next line, remain clickable, and stay reachable through the existing vertically scrollable Preferences body.
- [x] 4.4 Confirm theme selection, language persistence, the Shortcuts tab width, and unrelated Preferences controls retain their existing behavior.
