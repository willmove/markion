## 1. About State and Localization

- [x] 1.1 Add the canonical `https://markion.app` constant and transient About-dialog visibility state, initialize it without persistence, and change the existing About action/close path to open and dismiss that state while preserving menu dismissal and status feedback.
- [x] 1.2 Replace the newline-delimited About detail message with composable version, description, project-website, and GitHub labels, providing non-empty translations for every supported interface language while reusing the existing localized title and OK label.

## 2. Interactive About Modal

- [x] 2.1 Implement a root-hosted, occluding About modal that uses the active theme palette, shows the running version and localized description, and provides the localized OK control without touching document or persisted state.
- [x] 2.2 Render one ordered link model with `https://markion.app` above `https://github.com/willmove/markion`; give both literal URLs link styling, hover feedback, stable debug selectors, and pointer handlers that pass the same constants to `cx.open_url` without dismissing the modal.
- [x] 2.3 Integrate the modal into the root overlay layer so either Help-menu surface opens the same view, underlying editor controls cannot receive its pointer events, and confirmation closes it cleanly.

## 3. Focused Verification

- [x] 3.1 Add unit tests for the ordered About-link destinations, exact visible/activation URLs, action-driven open state, menu dismissal, confirmation-driven close state, and the rule that link activation leaves the modal open.
- [x] 3.2 Add rendering/localization tests that verify the website row precedes GitHub, required labels and URLs are present for every supported language, and link/modal colors remain theme-derived and readable under representative light and dark themes.

## 4. Quality Gates

- [x] 4.1 Run `cargo fmt --all -- --check` and `cargo test --workspace`, correcting any formatting, compilation, localization-exhaustiveness, or regression failures.
- [x] 4.2 Run `openspec validate add-clickable-about-links` and confirm the implementation satisfies every scenario without modifying Markdown-derived caches, persisted preferences/session data, or unrelated Help-menu links.
