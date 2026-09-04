## Why

People who already like Markion currently have no in-app prompt to support the project on GitHub. The Help → About Markion dialog is the natural place for a short, optional ask plus a one-click path to the repository Star button.

## What Changes

- After the About dialog's product description, show a short localized invitation asking users who find Markion useful to star the project. Canonical Simplified Chinese copy: `觉得有用的话，欢迎给个 Star，谢谢！` (a tighter version of the requested sentence).
- Directly under that invitation, show a clickable GitHub link that opens `https://github.com/willmove/markion` in the system default browser so the user can star the repository.
- Keep the existing title, version, product description, project-website link, GitHub repository link, confirmation control, theme styling, and localization model.
- Add translations for the new invitation and star-link label in every supported interface language.

**Non-goals:** no GitHub API authentication or in-app starring, no Help-menu item for starring, no change to the existing website/repository destinations, and no embedded browser.

This change does not touch per-document-version Markdown caches, syntax-highlight memoization, text-handle reuse, or any `crates/*` member.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `chrome-platform`: The About Markion dialog presents a localized GitHub-star invitation and a clickable repository link that opens in the system browser.

## Impact

- About dialog rendering in `src/app/root_view.rs` (`about_dialog_view`) and the `AboutLink` model / URL constants in `src/app/mod.rs`.
- New `Msg` variants and exhaustive per-language arms in `src/i18n.rs`.
- Focused tests in `src/app/tests.rs` for invitation visibility, exact GitHub destination, dialog-stays-open activation, localization coverage, and theme-derived chrome.
- No persisted-data migration, public API change, new dependency, or change to Markdown-derived caches.
