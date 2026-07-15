## 1. Proposal & spec delta

- [x] 1.1 Create change folder `expand-chinese-variants` with `openspec new change`
- [x] 1.2 Write `proposal.md` (why / what / non-goals / capabilities / impact)
- [x] 1.3 Write `specs/ui-i18n/spec.md` delta (modified requirements + new scenarios)
- [x] 1.4 Write this `tasks.md`
- [x] 1.5 `openspec validate expand-chinese-variants`

## 2. Language enum & metadata (`src/i18n.rs`)

- [x] 2.1 Rename `Language::Zh` → `Language::ZhHans`; update the enum doc comment
- [x] 2.2 Add `Language::ZhHant` variant
- [x] 2.3 `code()`: `ZhHans => "zh-hans"`, `ZhHant => "zh-hant"`
- [x] 2.4 `from_code()`: keep Simplified aliases on `ZhHans`; add `zh-hant`/`zh-tw`/`zh-hk`/`cht`/`traditional chinese` → `ZhHant`; unknown still → `En`
- [x] 2.5 `native_name()`: `ZhHans => "简体中文"`, `ZhHant => "繁體中文"`
- [x] 2.6 `all()` → `[En, ZhHans, ZhHant, De, Es, Fr, Ja]`

## 3. Traditional-Chinese translation table (`src/i18n.rs`)

- [x] 3.1 Add exhaustive `fn zh_hant(msg: Msg) -> &'static str` using Taiwan-regional terminology
- [x] 3.2 Add `PREFERENCES_DETAIL_ZH_HANT` constant
- [x] 3.3 Add `SHORTCUTS_ZH_HANT` and `SHORTCUTS_ZH_HANT_EXTENDED` constants

## 4. Dispatch sites

- [x] 4.1 `t()`: add `Language::ZhHant => zh_hant(msg)` arm; rename existing `Zh` arm to `ZhHans`
- [x] 4.2 `shortcut_reference()`: add `(Language::ZhHant, false)` → `SHORTCUTS_ZH_HANT` and `(true)` → `SHORTCUTS_ZH_HANT_EXTENDED`; rename existing `Zh` arms to `ZhHans`
- [x] 4.3 `sidebar_tab_label()`: add Traditional arms (`檔案` / `大綱`); rename existing `Zh` arms to `ZhHans`
- [x] 4.4 `src/app/mod.rs::AppMenu::dropdown_left`: rename `Language::Zh` arms to `ZhHans` and fold `ZhHant` into the CJK glyph-width column (keep exhaustive match compiling)

## 5. Tests

- [x] 5.1 `language_from_code_accepts_common_aliases`: add `zh-hant` / `zh-tw` → `ZhHant` assertions
- [x] 5.2 `every_message_returns_non_empty_text_for_every_language`: add `Language::ZhHant` assertion line
- [x] 5.3 Add `traditional_chinese_shortcut_reference_is_translated` test mirroring the Simplified one
- [x] 5.4 Update any remaining `Language::Zh` references in tests → `ZhHans`

## 6. Validation

- [x] 6.1 `cargo build` compiles clean (exhaustive-match safety net catches any missed dispatch site)
- [x] 6.2 `cargo test` — i18n module tests + exhaustiveness guard green
- [x] 6.3 `openspec validate expand-chinese-variants` passes

## Notes

- Two unrelated pre-existing test failures (`tests::html_preview_parts_render_common_readme_html`,
  `tests::preview_keeps_raw_html_blocks_for_rendering`) exist in the working tree. Verified by
  temporarily reverting `src/i18n.rs` + `src/app/mod.rs` to HEAD: both fail identically. They are
  caused by *other* uncommitted work (README text-join spacing in `src/lib.rs`, and incomplete
  `PreviewBlock::Html` handling in `src/parse.rs` / `src/visual.rs`) and are **outside the scope**
  of this i18n change. All 13 i18n tests + all 8 preferences tests pass.
