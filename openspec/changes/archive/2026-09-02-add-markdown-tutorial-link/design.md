## Context

See `proposal.md` for motivation. Help → Markdown Reference already opens a root-hosted overlay (`markdown_reference_view` in `src/app/root_view.rs`) with a localized title, a scrollable syntax cheat sheet from `markdown_reference(language)`, and an OK control. That overlay is presentation-only today: it never opens a tab, never fetches remote content, and never mutates document or derived Markdown state.

External navigation already exists elsewhere: About-dialog rows and Help-menu issue/docs items call `cx.open_url` and let the platform shell open the system browser. The Kenhuang tutorial is a similar chrome concern, not document content.

## Goals / Non-Goals

**Goals:**

- Place a clearly identifiable Kenhuang Markdown tutorial link at the top of the Markdown Reference overlay so it is visible without scrolling the cheat sheet.
- Select the Chinese or English tutorial URL from the active interface language.
- Open that exact HTTPS destination in the system default browser without embedding a web view or fetching tutorial HTML into Markion.
- Keep overlay dismissal, syntax sections, shortcuts, and document/cache invariants unchanged.

**Non-Goals:**

- A new Help-menu item, shortcut, or shortcut-reference entry for the tutorial.
- Embedding, prefetching, or availability-checking the tutorial.
- Changing Markdown Reference sections, F1 binding, or scrollbar behavior.
- A reusable generic “external link row” widget beyond this overlay (reuse the About styling pattern in place).
- Additional locale-specific tutorial URLs beyond Chinese vs non-Chinese.

## Decisions

### 1. Pin the link above the scrollable cheat sheet

Render the tutorial row as a sibling below `DialogMarkdownReferenceTitle` and above the existing `markdown-reference-body` scroll region. Putting it inside the first syntax section would bury it after scrolling; putting it above the title would compete with the overlay heading.

The row uses the About-dialog link treatment: localized label, verbatim URL, pointer cursor, underline, theme-derived link/hover colors, and a stable debug selector. Activating the link does not close the overlay, matching About.

Rejected alternatives: a Help-menu item (the request is inside the Markdown Reference overlay) and a first cheat-sheet section (that would scroll away and mix tutorial navigation with syntax examples).

### 2. Choose the URL from the interface language, not from OS locale

Centralize two constants next to the existing Help/About URLs:

- `https://kenhuang.com/markdown/` for `Language::ZhHans` and `Language::ZhHant`
- `https://kenhuang.com/en/markdown/` for every other `Language` variant (English, Japanese, French, German, Spanish)

A small helper (for example `kenhuang_markdown_tutorial_url(language)`) returns the constant. The renderer and the click handler both use that helper so the visible URL and the `cx.open_url` argument cannot drift.

Rejected alternatives: always opening the Chinese URL; using the OS/system locale instead of Markion’s interface language; and inferring language from the tutorial site at click time. The overlay already keys off `app.language`.

### 3. Open with `cx.open_url`; do not fetch into the overlay

On pointer activation, call `cx.open_url` with the selected constant, the same path used by About and Help-menu external links. Markion performs no HTTP request for the tutorial. The overlay remains open so the user can still read the in-app cheat sheet.

This preserves the existing Markdown Reference rule that the overlay body is local content. Opening a system browser is shell delegation, not remote overlay content.

Rejected alternatives: an embedded web view, `webbrowser` or other new crates, and copying the URL only to the clipboard.

### 4. Localize the label; keep URLs as constants

Add one `Msg` variant for the tutorial-link label (for example `DialogMarkdownReferenceTutorial`) and provide non-empty translations for every supported language. Typical copy: Simplified Chinese “垦荒学园 Markdown 教程”, Traditional Chinese “墾荒學園 Markdown 教程”, and a Kenhuang Markdown Tutorial equivalent in the other languages. URLs stay out of translation strings.

This follows the compile-time-exhaustive `Msg` model. Embedding the URL inside translated prose was rejected because it can drift per language and makes the click target harder to keep exact.

### 5. Isolate from document and persisted state

Data flow:

`Help → Markdown Reference` (unchanged) → overlay render reads `app.language` → helper picks a URL constant → pointer event calls `cx.open_url` → overlay stays open until Escape/OK.

No document text, derived Markdown cache, syntax-highlight cache, undo snapshot, session value, or preference is read or invalidated. Language changes already rebuild chrome; the new row simply follows `app.language` on the next overlay open or re-render.

## Risks / Trade-offs

- **[Risk] A missing system browser or broken OS URL association can make `open_url` appear to do nothing** → Show the exact URL as visible text, as About already does; GPUI does not report launch success, so new error UI is out of scope.
- **[Risk] Traditional Chinese users land on a Simplified-Chinese tutorial page** → Accept the requested Chinese vs non-Chinese split; there is no Traditional-specific tutorial URL.
- **[Risk] The pinned row reduces cheat-sheet viewport height slightly** → Keep the row compact (label + one URL line) and leave the existing 420px scroll body and right-side scrollbar in place.
- **[Trade-off] Duplicating About-like link styling instead of extracting a shared widget** → Avoids a broader chrome refactor for a single extra row.

## Migration Plan

No data migration. Ship the overlay row and translations with the application update. Rollback is removing the row, helper/constants, `Msg` variant, and tests; no persisted data or external contract needs reversal.

## Open Questions

None. The destinations, language split, and overlay placement are specified by the request.
