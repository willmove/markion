## 1. Localization and Link Model

- [x] 1.1 Add `Msg::DialogAboutStarInvite` and `Msg::DialogAboutStarLink` with non-empty translations for every supported language. Simplified Chinese invitation: `觉得有用的话，欢迎给个 Star，谢谢！`; English invitation: `If Markion helps you, please give it a Star on GitHub. Thank you!`; keep the word `Star` in every language.
- [x] 1.2 Add `AboutLink::GithubStar` that reuses `GITHUB_REPO_URL`, has its own label and debug selectors, and is **not** included in `AboutLink::ALL`, so the existing website-then-GitHub order stays unchanged.

## 2. About Dialog UI

- [x] 2.1 In `about_dialog_view`, render the localized star invitation immediately after the product description, then a `GithubStar` link row (same `about_link_row` / `open_about_link` path), then the existing official project links. Activating the star link MUST open `https://github.com/willmove/markion` and MUST NOT dismiss the dialog or touch document/persisted state.

## 3. Focused Verification

- [x] 3.1 Extend About-dialog tests so the invitation and star-link selectors render, the star URL is exactly `GITHUB_REPO_URL`, activating it leaves the dialog open, `AboutLink::ALL` remains website-then-GitHub, and every supported language has non-empty invite/star-link strings.
- [x] 3.2 Confirm the star invitation and star-link row remain theme-derived and readable under representative light and dark themes, using stable debug selectors.

## 4. Quality Gates

- [x] 4.1 Run `cargo fmt --all -- --check` and `cargo test --workspace`, correcting any formatting, compilation, localization-exhaustiveness, or regression failures.
- [x] 4.2 Run `openspec validate about-dialog-github-star` and confirm the implementation satisfies every scenario without modifying Markdown-derived caches, persisted preferences/session data, or unrelated Help-menu links.
