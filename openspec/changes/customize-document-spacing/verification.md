# Verification — 2026-11-17

## Behavior

- Paragraph spacing now accepts **0–128px**, default **12px**; Appearance keeps 1px steps.
- Document line spacing accepts **75–300%** relative to the default line heights, default **100%**; Appearance uses 5-point steps and explicitly labels the default relationship in every supported language.
- Source, Visual Edit, Split Preview and Read share the resolved scale, including heading/table line metrics, lists, quotes, code and source islands. Glyph sizes, paragraph gaps and application chrome do not inherit that scale. Export layout is unchanged.
- Existing configuration without `line_spacing` keeps the original metrics exactly; malformed fields default safely, integers clamp, reset restores 100%, and values persist as `line_spacing` in `config.toml`.

## Checks

| Check | Result |
| --- | --- |
| `cargo test --workspace -- --quiet` | **2,224 passed, 0 failed, 5 existing ignored** across 51 reported suites |
| Persistence | Old/malformed config, numeric extremes, 96px paragraph spacing and 150% line spacing round-trip passed |
| Geometry | Default equality, 75/150/300% scaling, unchanged font/gap sizes, source caret and vertical hit-testing at 200% passed |
| Presentation invariants | Reflow retains document version, derived preview `Arc`, selection, undo history, highlights, cached text and virtual-list reading anchor |
| Reset | Non-default paragraph and line spacing restore defaults without changing document text/version |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace` | Passed with the project's existing warnings (99 app-binary warnings); none reported for the new spacing helpers |
| Pinned MarkNice bundle | 23 files, 3,140,387 bytes verified |
| `openspec validate --all --strict --no-interactive` | **93 passed, 0 failed** |
| `git diff --check` | Passed |

## Windows native verification

Used an isolated copy of the development binary with separate `APPDATA` and `LOCALAPPDATA` under `target/tmp/document-spacing-native`, a newly created temporary demo workspace, Paper theme, Chinese UI, 16px reading text and an installed Consolas code/source font. Startup update checks and auto-save were disabled. Daily documents and preferences were not used.

Through actual native menu and mouse input:

1. Opened **File → Preferences → Appearance** and observed both controls, initially **63px / 145%**.
2. Clicked paragraph + once: **64px**, proving the former 32px limit is lifted.
3. Clicked line-spacing + once: **150%**. The rendered document reflowed immediately while the preferences chrome retained its size.
4. Checked `config.toml`: `paragraph_spacing = 64` and `line_spacing = 150`.
5. Closed and restarted the isolated app, reopened Appearance, and observed **64px / 150%** again.
6. Compared the whole Markdown file hash before/after: unchanged, SHA-256 `5364BD917A04AC058DD50856A7F533968F4DC93E2D160FAC563B7EF29ECC9BAF`.

The native executable SHA-256 was `C2C4FF266E9C2B76EDE46159B82A66A12FE2126D6E8B0EEA7C62F1CDF7626BE4`. [Actual post-restart preferences screenshot](../../../docs/verification/document-spacing-windows-2026-11-17.png). The isolated instance was closed after capture.

Native macOS/Linux interaction was not performed. No dependencies, package versions, release metadata or public tags changed. Earlier README/Korean work and unrelated release-acceptance changes remain intact.
