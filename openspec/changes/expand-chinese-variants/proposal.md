## Why

The interface language picker currently exposes a single entry labelled "中文". That label is ambiguous — it does not tell users whether they are getting Simplified or Traditional Chinese, and Traditional-Chinese readers have no way to choose their script. As Markion's chrome (menus, status bar, dialogs, file tree, preferences, keyboard-shortcut reference) is fully translated, the absence of a Traditional variant is a real gap for the large Traditional-Chinese readership (Taiwan / Hong Kong / Macau).

## What Changes

- **Rename the existing Chinese variant for clarity:** `Language::Zh` becomes `Language::ZhHans`, and its native display name changes from `"中文"` to `"简体中文"`. The persisted code becomes `"zh-hans"`. All previously-accepted Simplified aliases (`zh`, `chs`, `zh-cn`, `zh-hans`, `chinese`) continue to map to Simplified Chinese, so existing preference files keep working unchanged.
- **Add a Traditional-Chinese variant:** new `Language::ZhHant`, persisted as `"zh-hant"`, native display name `"繁體中文"`. `from_code` accepts `zh-hant`, `zh-tw`, `zh-hk`, `cht`, and `traditional chinese` aliases (in addition to the round-trip code itself). Unknown / empty / unrecognised values still fall back to English.
- **Full Traditional-Chinese translation:** a new exhaustive `zh_hant()` match table over every `Msg` variant, plus `PREFERENCES_DETAIL_ZH_HANT`, `SHORTCUTS_ZH_HANT`, and `SHORTCUTS_ZH_HANT_EXTENDED` constants, plus Traditional-Chinese arms in `sidebar_tab_label`. The Traditional variant uses Taiwan-regional terminology conventions (資料夾, 偏好設定, 預設, 剪貼簿, 結束, 匯出, 檢視 …).
- **Wire the new variant into every dispatch site:** `t()`, `shortcut_reference()`, `sidebar_tab_label()`, and the in-app menu `dropdown_left` pixel table in `src/app/mod.rs` (Traditional Chinese shares the CJK glyph-width column with Simplified Chinese, since 檔案/編輯/檢視/格式/匯出/說明 occupy roughly the same width as their Simplified counterparts).
- **Display order:** `Language::all()` returns `[En, ZhHans, ZhHant, De, Es, Fr, Ja]` so the two Chinese entries sit next to each other in the Preferences panel.
- **Tests:** extend the alias / round-trip / exhaustiveness guards in the `i18n` test module and add a `traditional_chinese_shortcut_reference_is_translated` regression test mirroring the existing Simplified-Chinese one.

### Non-goals

- No change to the preferences *file format* — `language=` still stores one raw string. Older builds tolerate the new codes (they simply fall back to English); this build tolerates the old codes (they keep mapping to Simplified Chinese).
- No translation of document content, the welcome Markdown, or user files — UI chrome only.
- No right-to-left layout, no ICU/pluralization rules.
- No View → Language submenu (still a separate, deferred future-change candidate — language selection stays in the Preferences panel).
- No new external crate; i18n stays hand-rolled.

## Capabilities

### Modified Capabilities
- `ui-i18n`: the supported language set expands from one ambiguous Chinese entry to two script-specific entries (Simplified + Traditional). Aliases, persistence codes, the exhaustive-match test surface, and the language-selection display all reflect the split.

## Impact

- **Code:** `src/i18n.rs` (rename `Zh`→`ZhHans`, add `ZhHant`, new `zh_hant()` table + 3 constants, extend `code`/`from_code`/`native_name`/`all`/`t`/`shortcut_reference`/`sidebar_tab_label`, test-module updates), `src/app/mod.rs` (rename the `Language::Zh` arms in `dropdown_left` to `ZhHans` and fold `ZhHant` into the CJK group so the match stays exhaustive). No other call sites reference `Language::Zh` directly.
- **Invariants touched:** None of the cached-per-version Markdown invariants are affected — this is purely a UI-chrome / i18n concern.
- **Persistence format:** Additive and backwards-compatible. New canonical codes are `zh-hans` / `zh-hant`; the legacy `zh` / `zh-cn` / `chs` aliases continue to resolve to Simplified Chinese, so no preference file migration is required.
- **APIs / dependencies:** No new dependencies. `Language::ZhHant` and the `zh_hant` translation table become part of the crate's public i18n surface (the enum is already re-exported).