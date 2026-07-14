## Context

Markion's i18n is a hand-rolled, zero-dependency system: a closed `Msg` enum (~200 variants) paired with per-language `fn Xx(msg) -> &'static str` functions. Adding a language means adding one `Language` variant, one private translation function, match-arm extensions in `t()` / `tf()` / `shortcut_reference()` / `sidebar_tab_label()`, and code-parsing aliases in `from_code()`. The compile-time guarantee that every `Msg` variant has a non-empty translation for every language is enforced by the existing test `every_message_returns_non_empty_text_for_every_language`, which iterates all variants for each `Language::all()` entry.

The file `src/i18n.rs` is already ~1626 lines. Adding 4 languages will bring it to approximately 2400–2600 lines. This is manageable for a single file but warrants a consistent organizational convention.

## Goals / Non-Goals

**Goals:**
- Add Japanese (ja), French (fr), German (de), and Spanish (es) as fully supported interface languages
- Every existing `Msg` variant receives a natural, idiomatic translation in each new language
- Common preference-code aliases are accepted (e.g. `jp`, `francais`, `deutsch`, `espanol`)
- The compile-time exhaustiveness test covers all 6 languages
- Zero architectural changes: the existing `match`-based pattern is preserved

**Non-Goals:**
- No RTL language support (Arabic, Hebrew) — GPUI's text layout would need separate work
- No CJK font bundling changes — users must have appropriate system fonts
- No locale-aware number/date formatting — only UI chrome strings are translated
- No translation of document content — that invariant is unchanged
- No View → Language menu submenu (noted as a future-change candidate in the existing spec)

## Decisions

### Decision 1: Which 4 languages to add

**Chosen:** Japanese (ja), French (fr), German (de), Spanish (es)

**Rationale:** These are among the most widely spoken languages globally and cover three major language families (Japonic, Romance, Germanic). The user explicitly requested Japanese, French, and German; Spanish fills the fourth slot as the second-most natively spoken language worldwide. Korean and Portuguese are deferred to a future change to keep this one focused.

**Alternatives considered:**
- Korean + Portuguese instead of Spanish: valid but Spanish has broader reach
- Russian + Arabic: RTL (Arabic) would require GPUI layout work; Russian Cyrillic is fine but lower demand

### Decision 2: Translation approach — manual, AI-assisted

**Chosen:** Provide curated translations using AI assistance followed by manual review of every string for each language. The translations should be idiomatic and consistent with standard OS/menu conventions in each locale (e.g. macOS Japanese menu conventions, standard French software terminology).

**Rationale:** Automated batch translation without review would produce unnatural or inconsistent UI text. The ~200-msg × 4-languages = ~800 entries is a meaningful but manageable manual effort, especially since many entries are single words or short phrases.

### Decision 3: File organization — keep single file, interleave by language

**Chosen:** Keep all translations in `src/i18n.rs` with the existing layout: `Language` enum → `t`/`tf`/`shortcut_reference`/`sidebar_tab_label` → `fn en(...)` → `fn zh(...)` → `fn ja(...)` → `fn fr(...)` → `fn de(...)` → `fn es(...)` → `substitute` → tests. Each language function follows the identical `match msg { ... }` structure in the same `Msg` variant order as `en()`.

**Rationale:** The file is already laid out this way and it works. Splitting into `src/i18n/en.rs`, `src/i18n/zh.rs`, etc. would require `pub(crate)` visibility plumbing and add module boilerplate for zero semantic benefit. If the file ever exceeds ~4000 lines, restructure then.

### Decision 4: Language codes and aliases

**Chosen:**
| Language | Canonical code | Accepted aliases |
|----------|---------------|-------------------|
| Japanese | `ja` | `jp`, `japanese`, `jpn` |
| French | `fr` | `francais`, `français`, `french`, `fra` |
| German | `de` | `deutsch`, `german`, `ger`, `deu` |
| Spanish | `es` | `espanol`, `español`, `spanish`, `spa` |

**Rationale:** ISO 639-1 two-letter codes are canonical (matching the existing `en`/`zh` convention). Common user-facing aliases (language name in that language, English name, ISO 639-2/B codes) are accepted to tolerate hand-edited preference files.

### Decision 5: Display order in `Language::all()`

**Chosen:** `[En, Zh, De, Es, Fr, Ja]` — English first (default), then Chinese (existing second language), then Germanic/Romance alphabetically by code, then Japanese last. Each language's `native_name()` is its self-referential name: "English", "中文", "Deutsch", "Español", "Français", "日本語".

## Risks / Trade-offs

- **[Risk] Translation quality may be imperfect for some strings.** → Mitigation: every string is reviewed by at least one person familiar with that language's UI conventions. Users can report issues and we iterate.
- **[Risk] File size growth (~2400+ lines) may feel unwieldy.** → Mitigation: the `match` structure is highly regular and easy to navigate with code folding or `Ctrl+F`. No action needed unless the file exceeds ~4000 lines.
- **[Risk] New `Msg` variants added by other in-flight changes may conflict.** → Mitigation: this change should either be applied first (so other changes add their new Msg variants with all 6 language arms) or applied last (adding the 4 new languages to the merged Msg set). Since this change only adds data (no new Msg variants), applying it first is safest: other changes then see the 6-language requirement and add translations for all 6. The compile-time exhaustiveness test enforces this.
