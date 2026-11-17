## MODIFIED Requirements

### Requirement: The interface language SHALL be selectable and shall persist
The system SHALL let the user choose an interface language from the set of supported languages (`Language::all()`) via the Preferences panel. The chosen language SHALL persist across launches in the preferences file as a stable lowercase code (`language=<code>`).

#### Scenario: Unknown or missing language code falls back to English
- **WHEN** the preferences file has no `language=` line, an empty value, or an unrecognized code (e.g. `language=klingon`)
- **THEN** the system sets the interface language to English (`Language::default`) without error

#### Scenario: Common Chinese aliases are accepted
- **WHEN** the persisted language code is `zh`, `chs`, `zh-cn`, `zh-hans`, or `chinese` (case-insensitive)
- **THEN** the system selects Simplified Chinese as the interface language

#### Scenario: Common Japanese aliases are accepted
- **WHEN** the persisted language code is `ja`, `jp`, `japanese`, or `jpn` (case-insensitive)
- **THEN** the system selects Japanese as the interface language

#### Scenario: Common French aliases are accepted
- **WHEN** the persisted language code is `fr`, `francais`, `français`, `french`, or `fra` (case-insensitive)
- **THEN** the system selects French as the interface language

#### Scenario: Common German aliases are accepted
- **WHEN** the persisted language code is `de`, `deutsch`, `german`, `ger`, or `deu` (case-insensitive)
- **THEN** the system selects German as the interface language

#### Scenario: Common Spanish aliases are accepted
- **WHEN** the persisted language code is `es`, `espanol`, `español`, `spanish`, or `spa` (case-insensitive)
- **THEN** the system selects Spanish as the interface language

#### Scenario: Switching language takes effect immediately for menus
- **WHEN** the user selects a different language in the Preferences panel
- **THEN** the active language is updated in app state, preferences are persisted, and native OS menus are retranslated and reinstalled before the next render

#### Scenario: Common Korean aliases are accepted
- **WHEN** the persisted language is `ko`, `ko-kr`, `kor`, `korean`, or `한국어` (case-insensitive, with surrounding whitespace ignored)
- **THEN** Korean is selected and future preference saves use the stable code `ko`

### Requirement: Adding a UI string without a translation SHALL fail at compile time
The i18n module SHALL expose a closed `Msg` enum where each variant is a distinct user-visible string. The translation functions `t` and `tf` SHALL be exhaustive over `Msg` for every supported `Language`, so that adding a message key without providing translations for all languages is a compile error rather than a runtime fallback.

#### Scenario: New message key requires translations in every language
- **WHEN** a developer adds a new variant to `Msg`
- **THEN** the project fails to compile until every supported language, including Korean, covers the new variant (exhaustive `match`)

#### Scenario: Every message returns non-empty text for every language
- **WHEN** the i18n test suite runs the exhaustiveness guard
- **THEN** every `Msg` variant returns non-empty text for every supported language, including Korean

### Requirement: AI workflows SHALL be completely localized
All user-visible AI settings, setup, panel and composer, writing presets, context scopes, model capability states, tool activity, change review, cancellation and error, usage, history, and action-recovery strings SHALL use the localization catalog for English, Simplified Chinese, Traditional Chinese, Japanese, French, German, Spanish, and Korean. Per-field validation messages, dedicated save, test, discovery, and forget outcomes, draft-loss confirmations, misconfiguration guidance, and empty-result states SHALL likewise be localized in all eight languages. Context-size and output-size labels, their accepted ranges, the approximate token hint, the restore-recommended-limits action, the show and hide API key actions, localized protocol choice labels, and the notice that earlier conversation turns were omitted SHALL be localized in all eight languages. Provider and model identifiers MAY stay proper nouns, and the stored protocol codes SHALL remain `responses`, `chat`, and `anthropic`. The AI feature SHALL use one consistent product name across languages instead of divergent per-language coinages. Language changes SHALL relabel visible AI surfaces without modifying documents, conversation content, provider or model identifiers, authored instructions, or paths. Missing translations SHALL be caught by exhaustive compilation or an explicit all-language catalog completeness test. Ordinary labels SHALL describe writing and file actions in user language; raw protocol details SHALL remain in expandable diagnostics.

#### Scenario: Chinese AI workflow
- **WHEN** the interface is Simplified Chinese and the user opens AI settings, rewrites a selection, or reviews a file plan
- **THEN** labels, validation, progress, apply and reject controls, and outcome feedback are Simplified Chinese

#### Scenario: Language changes during a conversation
- **WHEN** the interface language changes with the AI panel visible
- **THEN** UI chrome follows the selected language while existing conversation and document content stays unchanged

#### Scenario: AI translation is missing
- **WHEN** an AI message is added without a non-empty translation in a supported language
- **THEN** compilation or the completeness test fails

#### Scenario: New settings labels follow the interface language
- **WHEN** the interface is German or Japanese and the user opens AI settings
- **THEN** the context-size label, show API key action, protocol choice labels, and limit-range messages are in that language
- **AND** the stored protocol code and model id are unchanged

## ADDED Requirements

### Requirement: Korean interface coverage
The system SHALL offer Korean as `한국어` in the existing language selector and persist it as `ko`. Every UI catalog SHALL provide Korean entries, including native menus, settings, status and validation, image/table controls, recovery and conflict handling, import/export, Backup and Sync, AI, shortcuts, and the Markdown reference. Korean SHALL preserve positional placeholders and authored identifiers, paths, model names, document text and history. Missing Korean entries SHALL fail compilation or catalog-completeness tests rather than fall back to English. Language switching SHALL not mutate document text, document version or undo history.

#### Scenario: Reader switches to Korean
- **WHEN** the user selects `한국어` in Preferences
- **THEN** native and in-app menus, settings, editor chrome and visible workflow controls use Korean
- **AND** the saved preference is `ko` and restarting retains Korean
- **AND** document text, version and undo history remain unchanged

#### Scenario: Korean catalog coverage is verified
- **WHEN** catalog tests iterate all supported languages and messages
- **THEN** every Korean entry is non-empty and uses the same positional-placeholder set as English
- **AND** AI and Backup and Sync messages are included
