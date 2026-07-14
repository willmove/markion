## 1. Language enum and infrastructure

- [x] 1.1 Add Ja, Fr, De, Es variants to the Language enum
- [x] 1.2 Extend Language::code() with ja, fr, de, es codes
- [x] 1.3 Extend Language::from_code() with aliases for Japanese, French, German, Spanish
- [x] 1.4 Extend Language::native_name() with "日本語", "Français", "Deutsch", "Español"
- [x] 1.5 Extend Language::all() to return [En, Zh, De, Es, Fr, Ja]

## 2. Japanese translations (ja)

- [x] 2.1 Write fn ja(msg: Msg) -> &'static str with all ~200 Msg match arms, following the same variant order as en()

## 3. French translations (fr)

- [x] 3.1 Write fn fr(msg: Msg) -> &'static str with all ~200 Msg match arms, following the same variant order as en()

## 4. German translations (de)

- [x] 4.1 Write fn de(msg: Msg) -> &'static str with all ~200 Msg match arms, following the same variant order as en()

## 5. Spanish translations (es)

- [x] 5.1 Write fn es(msg: Msg) -> &'static str with all ~200 Msg match arms, following the same variant order as en()

## 6. Dispatch wiring

- [x] 6.1 Extend t() match to dispatch to ja(), fr(), de(), es() for the new Language variants
- [x] 6.2 Extend sidebar_tab_label() with (Ja, Fr, De, Es) match arms for both Files and Outline tabs
- [x] 6.3 Add shortcut reference constants SHORTCUTS_JA, SHORTCUTS_FR, SHORTCUTS_DE, SHORTCUTS_ES and wire them in shortcut_reference()

## 7. Tests

- [x] 7.1 Update language_round_trips_through_its_code to cover all 6 languages
- [x] 7.2 Update language_from_code_accepts_common_aliases with new language alias tests
- [x] 7.3 Update every_message_returns_non_empty_text_for_every_language to validate all 6 languages
- [x] 7.4 Add shortcut reference smoke tests for Japanese, French, German, Spanish

## 8. Validation

- [x] 8.1 Run cargo test and ensure all tests pass (compile-time exhaustiveness check enforces completeness)
- [x] 8.2 Run openspec validate add-multilingual-i18n
