## 1. Preferences Panel Controls

- [x] 1.1 Inspect current Preferences panel rendering and existing preference action methods in `src/main.rs`.
- [x] 1.2 Add compact reusable button/segmented controls for boolean and option preferences using active theme colors.
- [x] 1.3 Make focus mode, typewriter mode, code line numbers, Preview adaptive width, sidebar visibility, and sidebar tab interactive from the Preferences panel.
- [x] 1.4 Reorder the Preferences panel so Language renders before Theme.

## 2. Theme-Aware Menus

- [x] 2.1 Update in-window menu bar and dropdown styling to derive colors from the active theme palette.
- [x] 2.2 Ensure active, hover, separator, muted, and border states remain readable for dark and light themes.

## 3. Verification

- [x] 3.1 Add or update focused tests for Preferences control behavior where the existing test harness allows it.
- [x] 3.2 Run `openspec validate make-preferences-controls-interactive` and relevant Rust checks (`cargo test` or targeted tests).
