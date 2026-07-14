## Why

Executes the P1/P2 items of `docs/typune-integration-audit.md` (user-approved backlog):

- **P1a — grammar coverage gap.** syntect's bundled set (75 grammars) misses the modern mainstream: measured `find()` = NONE for typescript, toml, kotlin, swift, dockerfile, powershell, elixir, vue, graphql, terraform, dart, zig, nix, protobuf, julia, solidity. Users testing those languages see the legacy lexer and reasonably conclude "syntect isn't working". The `two-face` crate (0.5.1, syntect-onig backend matching ours) packages bat's curated extended syntax set and closes most of the gap.
- **P1b — silent engine fallback.** PDF/DOCX export currently gives no user-visible indication whether the pandoc engine or the built-in simple writer produced the file; a user getting the 707-byte text-dump PDF has no hint that installing pandoc(+engine) fixes it. Also `PdfExporter` hardcodes `--pdf-engine=xelatex`, locking out machines that have pandoc with a different PDF engine.
- **P2b — dishonest advertised list.** `supported_highlight_languages()` returns a fixed 53-item list unrelated to actual registry coverage (audit found it has no UI call site — it is an advertised-capability API and README claim only).
- **P2a — HTML/LaTeX evaluation** (Phase 1 leftover): compare Markion's native HTML/LaTeX exports against the absorbed `html.rs`/`latex.rs` on a representative document and record the keep-or-switch decision.

## What Changes

- **Grammar set** (`crates/markdown`, `src/highlight.rs`, root `Cargo.toml`): `LanguageRegistry` gains `with_syntax_set(SyntaxSet)` (second additive change to an absorbed crate; `new()` delegates to it with syntect defaults, so the crate's own behavior and tests are unchanged). Markion builds its registry from `two_face::syntax::extra_newlines()`. The registry's conditional alias table means dormant aliases (typescript/ts/tsx, toml, kotlin, swift, dockerfile, powershell, …) activate automatically once the canonical syntaxes exist. Fallback lexer stays for anything still uncovered.
- **Export feedback** (`src/model.rs`, `src/lib.rs`, `src/export.rs`, `src/main.rs`, `src/i18n.rs`): `export_to` returns which backend produced the file (`ExportBackend::PandocEngine | BuiltIn`); the status bar reports PDF/DOCX exports as "(pandoc engine)" or "(built-in writer; install pandoc for richer output)" in both languages. Other formats keep the plain "Exported {path}" message.
- **Configurable PDF engine** (`crates/export`, `src/storage/preferences.rs`, `src/model.rs`, `src/main.rs`): `PdfExporter::with_pdf_engine` (third additive change, with test); `config.toml` gains `[export] pdf_engine = "xelatex"` (config-file only, like `[auto_save]`); the app plumbs it through `export_to_with`.
- **Advertised language list** (`src/highlight.rs`): `supported_highlight_languages()` becomes the sorted, deduped union of the legacy identifier list and the active registry's grammar names (lowercased), built lazily once. Signature unchanged (`&'static [&'static str]`, leaked once).
- **HTML/LaTeX decision** (docs only): comparison executed on a representative fixture; result recorded in `docs/typune-integration-audit.md` §P2a-结论 and the integration plan. No exporter switch in this change.

## Capabilities

### Modified Capabilities
- `code-and-math`: grammar-based highlighting coverage extends to the two-face extended syntax set; the advertised language list reflects actual registry coverage plus lexer-fallback identifiers.
- `export`: PDF/DOCX status messages disclose the producing backend; the pandoc PDF engine is configurable via `[export] pdf_engine`.

## Impact

- **Deps:** `two-face` joins workspace + root deps (embedded serialized syntax set; registry load time re-measured via the existing warm-up debug event).
- **Edited:** `crates/markdown/src/highlight.rs` (+accessor), `crates/export/src/pdf.rs` (+engine option), `src/highlight.rs`, `src/export.rs`, `src/lib.rs`, `src/model.rs`, `src/storage/preferences.rs`, `src/main.rs`, `src/i18n.rs`, docs.
- **Behavior:** languages in the extended set switch from lexer to grammar coloring (the intended user-visible improvement); export status strings for PDF/DOCX gain a backend suffix; `config.toml` grows an `[export]` table on next save.
- **Tests:** existing fallback test retargeted if its probe language became covered; new tests for extended coverage, backend reporting, `[export]` round-trip, and the `with_pdf_engine` option.
