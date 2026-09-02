## 1. URL Helper and Localization

- [x] 1.1 Add Kenhuang tutorial URL constants (`https://kenhuang.com/markdown/` and `https://kenhuang.com/en/markdown/`) plus a helper that returns the Chinese URL for `ZhHans`/`ZhHant` and the English URL for every other `Language`, without touching document or cache state.
- [x] 1.2 Add a `Msg` variant for the tutorial-link label and provide non-empty translations for every supported interface language.

## 2. Overlay Row

- [x] 2.1 Render the localized label and verbatim selected URL at the top of the Markdown Reference overlay, below the title and above the scrollable syntax body, using theme-derived link styling and a stable debug selector.
- [x] 2.2 Wire pointer activation to `cx.open_url` with the same helper URL, keep the overlay open, and do not fetch remote tutorial content or mutate document/derived Markdown state.

## 3. Focused Verification

- [x] 3.1 Add tests that the helper selects the Chinese URL for both Chinese languages and the English URL for English, Japanese, French, German, and Spanish, and that activation leaves the overlay open without changing document text, dirty state, or derived caches.
- [x] 3.2 Add rendering/localization tests that the tutorial row appears above the first syntax section, the label is non-empty in every language, and the visible URL matches the helper for that language.

## 4. Quality Gates

- [x] 4.1 Run `cargo fmt --all -- --check` and `cargo test --workspace`, correcting any formatting, compilation, localization-exhaustiveness, or regression failures.
- [x] 4.2 Run `openspec validate add-markdown-tutorial-link` and confirm the implementation satisfies every scenario without modifying Markdown-derived caches or Help-menu structure.
