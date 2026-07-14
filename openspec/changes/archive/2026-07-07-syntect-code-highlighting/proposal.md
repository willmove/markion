## Why

Phase 2 of `docs/typune-integration-plan.md`: Markion's code-block highlighting is a hand-written token-class lexer (`src/highlight.rs`) with rudimentary per-language keyword tables. The `markdown` crate absorbed in Phase 1 carries a syntect-based highlighter with a `LanguageRegistry` (bundled Sublime grammars + a hand-crafted alias table). Grammar-driven highlighting is a large, user-visible quality jump for the languages syntect covers (Rust, Python, JS, SQL, Bash, HTML/CSS, YAML, …).

Two constraints shape the design:

1. **Markion's theme system stays authoritative for colors.** Typune's `SyntaxHighlighter` façade returns concrete RGBA colors baked from a hard-coded syntect theme (`base16-ocean.dark`) — unusable across Markion's 14 light/dark themes. Instead, Markion drives syntect's *parser* directly (`ParseState`/`ScopeStack`) and classifies scopes back into Markion's existing `HighlightKind` (Keyword/String/Number/Comment/Type/Plain), which each Markion theme already knows how to color. The public interface `highlight_code(code, lang) -> Vec<Vec<HighlightedSpan>>` and the per-`(lang, code)` memoization in `main.rs` are unchanged.
2. **syntect's default grammar set has gaps** (no TypeScript, TOML, Kotlin, Swift, Dockerfile, PowerShell, Elixir, Dart, Zig, …). For those, the existing lexer remains as the fallback — same engine-first-with-fallback shape as Phase 1 — so no language regresses from "some coloring" to "plain".

## What Changes

- **`src/highlight.rs`.** `highlight_code` tries the syntect path first: look up the language in the absorbed `LanguageRegistry`; if found, parse line-by-line with persistent `ParseState`/`ScopeStack` (multi-line strings/comments highlight correctly across lines) and classify each text segment's scope stack innermost-first into `HighlightKind` (comment→Comment, string→String, constant.numeric→Number, constant.language/keyword/storage→Keyword, entity.name.type/support.type & friends→Type; punctuation is transparent so quoted strings stay one span; `keyword.operator` deliberately stays Plain to match the current look). Registry misses and any syntect parse error fall back to the existing lexer, which is kept intact.
- **Lazy init + startup warm-up.** The `LanguageRegistry` (syntect grammar loading, ~100ms+) lives in a `OnceLock` initialized on first use; a new `warm_highlighter()` is called from a background thread at app startup so the first code block never blocks the typing path.
- **`crates/markdown` (absorbed crate, minimal additive change).** `LanguageRegistry` gains a public `syntax_set()` accessor — scope-level parsing needs `&SyntaxSet`, which was private. First deliberate evolution of an absorbed crate; covered by a small test.
- **Root `Cargo.toml`.** The root package adds `syntect` (already a workspace dependency) alongside the existing `typune-markdown` path dep.
- **`supported_highlight_languages()` unchanged** — it advertises the stable curated list; syntect widens actual quality behind it.
- **Non-goals:** no theme-driven syntect color pass-through, no tree-sitter, no async/batched highlighting (typing-path memoization already amortizes), no removal of the legacy lexer.

## Capabilities

### Modified Capabilities
- `code-and-math`: fenced code block highlighting becomes grammar-based (syntect) for registry-covered languages with the hand-written lexer as fallback; colors continue to come from Markion's theme-mapped token classes.

## Impact

- **Edited files:** `src/highlight.rs` (syntect path + fallback), `src/lib.rs` (re-export `warm_highlighter`; highlight tests adjusted only where lexer-specific), `src/main.rs` (startup warm-up thread), `crates/markdown/src/highlight.rs` (public accessor + test), root `Cargo.toml` (syntect dep), `docs/typune-integration-plan.md` (Phase 2 status).
- **Typing-path invariants preserved:** the `(language, code)` highlight memoization in `main.rs` is untouched; syntect work happens at most once per distinct code block per session, and grammar loading happens once on a background thread.
- **Behavior deltas within the same interface:** minor kind shifts for grammar-covered languages (e.g. C's `int` classified via `storage` as Keyword rather than Type); multi-line constructs now highlight correctly (the old lexer was line-local).
- **Tests:** existing highlight contract tests must pass as-is (rust/sql/sh assertions hold under the scope mapping; ts hits the fallback path unchanged); new tests cover scope classification, multi-line constructs, fallback for uncovered languages, and the accessor.
