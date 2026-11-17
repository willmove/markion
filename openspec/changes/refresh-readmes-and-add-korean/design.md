## Context

See proposal.md for motivation. The root currently duplicates about 230 lines per English/Chinese edition; six standalone translations repeat similar detail. v0.6.0 is published with verified native download assets. Existing November 11 performance records cover model derivation and Windows system-event input separately. November 16 screenshots already verify v0.6.0 editing features. The in-flight `add-multilingual-readmes` change assumes a duplicated root; this change explicitly supersedes that layout, preserving its existing language entry points.

## Goals / Non-Goals

**Goals:** make the first screen explain value and offer a download; keep language editions structurally equivalent; use genuine screenshots and auditable measurements; retain detailed English/Chinese reference information.

**Non-Goals:** editor runtime/architecture changes, new release publication or new competitor benchmarks. Korean application UI is now explicitly requested. Document-version caches and rendering data flow remain unchanged.

## Decisions

1. Use one English root and standalone localized overviews. A duplicated bilingual root conflicts with the requested reduction in text. Keep compatibility anchors and language navigation, and move the original reference sections into `docs/user-guide.md` with corrected relative links.
2. Use repository-local PNG captures rather than external images or generated mockups. Capture a dedicated sample workspace with isolated app data; reference existing verified menu captures when useful. Record provenance separately to keep README captions short.
3. Compare Markion at several input sizes using existing recorded model measurements, and show the separate 1,080-key input result. Avoid a direct comparison with quiet-md: its platform, document, and frame metric differ. Document that these are November 11 measurements, not a fresh v0.6.0 run.
4. Put essential compatibility caveats near downloads and compact limitations near the feature table. Keep the non-notarized macOS command visible. Keep native Word/MarkNice browser behavior distinctions in a short disclosure and the linked guide.
5. Use the same section order, numeric data and evidence links in all eight editions, with shared screenshots or a localized screenshot where available. Distinguish the seven UI languages of published v0.6.0 from the eight UI languages in the working tree after this change.
6. Extend the current exhaustive language dispatch with `Ko`, persisted as `ko` and displayed as `한국어`. Translate every catalog, including Git/AI arrays, reference text and shortcut labels. Preserve positional placeholders, proper nouns, identifiers, paths and Markdown syntax. No English fallback is used for missing Korean messages.
7. Verify locale aliases and preferences round-trip, catalog completeness and placeholder parity, native language switching and document immutability. Use isolated app data for Windows checks; run workspace tests, formatting and Clippy for the runtime localization change.

## Risks / Trade-offs

- Translation drift → review shared tables, dates, links, signing instructions, and feature rows across editions together.
- Historical performance overclaim → explicitly date results and state stage boundaries adjacent to the tables; retain links to all raw data and earlier peaks.
- Loss of detailed reference content → relocate existing English/Chinese explanations before replacing READMEs and validate relative links.
- Screenshot platform bias → label Windows captures; do not suggest screenshots prove untested macOS/Linux behavior.
- Legacy links → retain root anchors and existing standalone file names.
