## Why

The current README repeats long technical descriptions before readers can see the product, download it, or understand the latest release. A compact visual overview with authentic screenshots, evidence-linked measurements, and a Korean edition will make Markion easier to evaluate and try.

## What Changes

- Rebuild the README editions around a visual hero, download links, concise benefits, v0.6.0 highlights, editing modes, measured performance tables, and short getting-started instructions.
- Use the presentation pattern of s1dashu/quiet-md as inspiration while describing only Markion's verified capabilities and measurements.
- Add genuine Windows application screenshots with source/capture provenance, and retain essential signing, first-launch, export, and WYSIWYG limitations.
- Move the long English and Simplified Chinese reference material into a linked guide and keep every existing language entry point; add `README.ko.md` and reciprocal language navigation.
- Replace the duplicated English/Chinese root layout with one concise English root plus dedicated language editions. Preserve old English, Simplified Chinese, and installation anchors.
- Add Korean application UI (`ko`, 한국어), including editor, preferences, import/export, recovery, backup/sync, shortcut/reference and AI catalogs; persist language selection and keep document content unchanged.

Non-goals: publishing a release, claiming unmeasured competitor speedups, or changing the editor architecture. The user confirmed Korean README plus application UI on 2026-11-17. Localization does not touch the cached-per-version Markdown, text-handle, or syntax-highlighting invariants.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-documentation`: compact synchronized language entry points, including Korean, with screenshots and traceable performance evidence rather than a duplicated root edition.
- `ui-i18n`: selectable, persistent Korean UI with complete catalogs and placeholder compatibility.

## Impact

Root README files; detailed guides and screenshots; `src/i18n.rs`, `src/i18n/`, `src/ai_i18n.rs` and localization tests; OpenSpec deltas for documentation and UI localization. The existing language preference gains the stable `ko` value. No new runtime dependencies.
