## Why

Markion currently supports only English and Simplified Chinese as interface languages. Users who prefer Japanese, French, German, Spanish, or other mainstream languages cannot use the editor in their native tongue. Adding 4+ languages brings the total to 6+ and makes Markion accessible to a much broader international audience — a table-stakes feature for any editor targeting global users.

## What Changes

- Add 4 new `Language` variants to the i18n module: **Japanese (Ja)**, **French (Fr)**, **German (De)**, and **Spanish (Es)**
- Provide complete translation match arms in `t()` / `tf()` / `shortcut_reference()` / `sidebar_tab_label()` for every new language, covering all ~200 existing `Msg` variants
- Extend `Language::from_code()` to parse `ja`, `fr`, `de`, `es` preference codes (with common aliases like `jp`, `francais`, `deutsch`, `espanol`)
- Extend `Language::all()` to return all 6 languages in display order: English, 中文, Deutsch, Español, Français, 日本語
- Update the compile-time exhaustiveness test (`every_message_returns_non_empty_text_for_every_language`) to validate all 6 languages
- No architecture changes: the hand-rolled `Msg` enum + exhaustive `match` pattern remains the single source of truth

## Capabilities

### New Capabilities
- *none* — this change purely extends an existing capability with new data; no new system behavior

### Modified Capabilities
- `ui-i18n`: the set of supported interface languages expands from 2 (En, Zh) to 6 (En, Zh, Ja, Fr, De, Es); every requirement that references "supported languages" or "every language" now covers 6 languages instead of 2

## Impact

- **`src/i18n.rs`**: the primary (and only) file touched. New `Language` variants, new `match` arms (4 × ~200 entries), extended `from_code()` aliases, extended `all()`, extended test
- **Compile time**: adding 4 × ~200 match-arm entries may increase compilation time modestly; the file will grow from ~1600 lines to ~2400+ lines
- **No runtime impact**: the i18n lookup is still a simple `match` dispatch with no allocations
- **No dependency changes**: the hand-rolled approach is preserved — no `fluent`, `icu4x`, or other i18n crates are introduced
