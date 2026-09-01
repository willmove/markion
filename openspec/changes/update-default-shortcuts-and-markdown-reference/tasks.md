## 1. Shortcut Registry Defaults

- [x] 1.1 Change `SET_EDIT_MODE`, `SET_SPLIT_PREVIEW_MODE`, and `SET_READ_MODE` factory bindings to `secondary-/` (`Ctrl+/` / `Cmd+/`), `secondary-p` (`Ctrl+P` / `Cmd+P`), and `secondary-shift-r` (`Ctrl+Shift+R` / `Cmd+Shift+R`); leave Visual Edit and cycle-mode descriptors unchanged
- [x] 1.2 Add a `NEW_TAB` descriptor (`new-tab`, `secondary-shift-n`, `Ctrl+Shift+N` / `Cmd+Shift+N`) and include it in `menu_shortcuts::ALL` next to the other tab actions
- [x] 1.3 Add a `SHOW_MARKDOWN_REFERENCE` descriptor (`show-markdown-reference`, `f1`, `F1` / `F1`) to `ALL`, and make `show-shortcuts` factory-unbound so it is not installed in the keymap unless a valid override exists
- [x] 1.4 Extend registry tests for the five new/changed mappings, `/` key parse/format, canonical uniqueness (including no factory clash on `F1`), `new-tab` / `show-markdown-reference` lookup, and override sanitization of the new ids

## 2. Key Dispatch and Menus

- [x] 2.1 Bind the updated descriptors in `bind_app_keys` (`SetEditMode`, `SetSplitPreviewMode`, `SetReadMode`, `NewTab`, `ShowMarkdownReference`); skip `ShowShortcuts` when unbound; keep the complete-keymap/registry count invariant for bound entries
- [x] 2.2 Pass the New Tab descriptor into the in-window File row and keep native File → New Tab on `NewTab` so both surfaces show `Ctrl+Shift+N` / `Cmd+Shift+N`
- [x] 2.3 Confirm in-window View rows for Edit / Split / Read already consume the shared descriptors so their displayed labels follow the new defaults without a second keymap
- [x] 2.4 Add dispatch tests that the four requested chords reach `SetEditMode`, `SetSplitPreviewMode`, `NewTab`, and `SetReadMode`, that `F1` reaches `ShowMarkdownReference` rather than `ShowShortcuts`, and that one override/reset round-trip still live-rebinds

## 3. Markdown Reference Overlay

- [x] 3.1 Add `ShowMarkdownReference` / close actions, transient overlay visibility (not persisted), and a `markdown_reference(language)` section list covering headings, inline formatting, links/images, quotes, lists/task lists, tables, fenced code, math, and footnotes, with non-empty text in every supported language
- [x] 3.2 Add Help-menu chrome strings (`Msg` variants for the item, overlay title, close, and status) in English, Simplified Chinese, Traditional Chinese, Japanese, French, German, and Spanish
- [x] 3.3 Implement a root-hosted, theme-derived, occluding overlay that renders the localized sections, closes on Escape and the close control, and does not create a tab or touch document text, dirty state, or derived Markdown caches
- [x] 3.4 Insert Markdown Reference into both Help surfaces after Check for Updates and before Report an Issue, show the effective `F1` label on the in-window row, dismiss the dropdown on invoke, and widen `AppMenu::Help` only if the longest localized label would clip

## 4. Shortcut Catalog and User Docs

- [x] 4.1 Update the localized shortcut catalog so Tabs includes `new-tab`, View lists the new source/split/Read defaults, Help/View lists Markdown Reference as `F1`, and `show-shortcuts` no longer claims `F1` as a factory key
- [x] 4.2 Update `docs/keyboard-shortcuts.md` and `docs/faq.md` (and any README mention of `Ctrl+Alt+1/2/3` or `F1` opening shortcuts) so published defaults match the new bindings, including New Tab and Markdown Reference

## 5. Verification

- [x] 5.1 Add Help-menu source-scan tests (in-window and native) that Markdown Reference precedes Report an Issue and About stays last; add overlay tests for open/close, no new tab, unchanged document caches, theme-derived chrome, and per-language section completeness
- [x] 5.2 Run `cargo fmt --all -- --check` and `cargo test --workspace`, fixing registry, i18n exhaustiveness, menu, or overlay regressions without changing per-document-version Markdown cache behavior
- [x] 5.3 Run `openspec validate update-default-shortcuts-and-markdown-reference` and confirm the checklist covers every scenario in the proposal, design, and delta specs
