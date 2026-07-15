## 1. Command and localization surface

- [x] 1.1 Add exhaustive `Msg` keys and translations for the Open Folder menu label, folder-picker prompt, and opening/success/cancel/failure statuses across every supported language; extend localization coverage tests.
- [x] 1.2 Declare and register the shared `OpenFolder` GPUI action, then place it immediately after Open in both the native File menu and the in-window File dropdown without adding a shortcut.

## 2. Folder selection and workspace behavior

- [x] 2.1 Add testable normalized-path containment/root-selection helpers that preserve a current workspace for contained documents and choose the parent for documents outside it, including Windows path/canonicalization fallbacks.
- [x] 2.2 Implement the single-directory picker flow (`files: false`, `directories: true`, `multiple: false`) so success changes only workspace/sidebar state, reveals the Files tab, persists existing sidebar preferences, and cancellation leaves workspace and document state intact.
- [x] 2.3 Reset root-specific file-tree selection/collapse/scroll state when the chosen root changes and start the existing Markdown-only scan on the background executor without touching document text, undo state, or derived Markdown caches.
- [x] 2.4 Make asynchronous scan completion root-aware so stale results are ignored, successful empty folders still install a valid empty tree, and scan failures produce localized non-destructive status feedback.
- [x] 2.5 Route all existing document-opening/focusing flows through the refined root update behavior so files inside the workspace retain its root while external files rebase to their parent and rescan.

## 3. Verification

- [x] 3.1 Add focused tests for menu/action wiring, folder-picker options and state transitions, contained/external path handling, empty-folder success, cancellation, scan failure, and stale-scan protection while preserving bounded tree rendering.
- [x] 3.2 Run `cargo fmt --check`, `cargo test`, and `cargo build`; resolve regressions without expanding the change beyond the proposal.
