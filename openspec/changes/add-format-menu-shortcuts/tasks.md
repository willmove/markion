## 1. Shortcut Registry

- [x] 1.1 Add `UNORDERED_LIST`, `ORDERED_LIST`, `TASK_LIST`, `BLOCK_QUOTE`, and `CODE_FENCE` descriptors with the specified stable ids, GPUI defaults, and Windows/Linux and macOS labels, then include all five in `menu_shortcuts::ALL`
- [x] 1.2 Extend shortcut registry tests to assert the five exact default mappings, bracket-key parsing/formatting, stable-id lookup, canonical binding uniqueness, and acceptance by stored-override sanitization

## 2. Key Dispatch

- [x] 2.1 Bind the five descriptors to their existing GPUI formatting actions in `bind_app_keys` while preserving the complete-keymap/registry count invariant and all fixed editing bindings
- [x] 2.2 Add focused dispatch tests showing that each default keystroke reaches the matching existing Format action, including both bracket shortcuts, and that a customized binding replaces and resets to one new default without a restart

## 3. Menu and Shortcut Reference Integration

- [x] 3.1 Convert the five in-window Format menu rows to shortcut-bearing rows backed by their descriptors, and extend menu source-contract tests so all five effective labels are required while existing unbound items remain unmarked
- [x] 3.2 Add the five actions to the Editing shortcut category using the existing localized Format labels and stable ids; test every supported language, both heading-depth configurations, and Windows/Linux and macOS combinations
- [x] 3.3 Verify native Format-menu actions remain mapped to the same handlers and acquire their key equivalents from the shared keymap without changing formatting transformation semantics

## 4. Verification

- [x] 4.1 Run `cargo fmt --check` and `cargo test --workspace`, fixing any shortcut registry, localization, menu, or formatting regressions while preserving the cached-per-document-version Markdown invariants
- [x] 4.2 On Windows, verify all five default shortcuts transform a document, show beside the correct Format-menu items, appear in Preferences → Shortcuts → Editing, and continue to work after override and reset (covered by the Windows GPUI dispatch/live-rebind tests plus menu and catalog integration tests)
- [x] 4.3 Run `openspec validate add-format-menu-shortcuts` and reconcile the implementation checklist with the validated proposal, design, and delta spec
