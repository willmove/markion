# Implementation Plan: Syntect code highlighting behind the existing interface (Typune integration Phase 2)

## Overview

Swap the implementation under `highlight_code()` to syntect scope parsing (via the absorbed `markdown` crate's `LanguageRegistry`) mapped back onto Markion's `HighlightKind`, keeping the hand-written lexer as fallback for grammar gaps. Interface, memoization, and theme-driven coloring are unchanged.

## Tasks

- [x] 1. Registry accessor (`crates/markdown`)
  - [x] 1.1 Add `pub fn syntax_set(&self) -> &SyntaxSet` to `LanguageRegistry` (+ unit test). First deliberate additive change to an absorbed crate.

- [x] 2. Syntect path (`src/highlight.rs`)
  - [x] 2.1 `OnceLock<LanguageRegistry>` + `warm_highlighter()`; root package gains the `syntect` workspace dep.
  - [x] 2.2 Scope classifier: innermost-first stack scan → HighlightKind (comment/string/constant.numeric/constant.language/keyword minus keyword.operator/storage/entity.name.type/support.type/support.class/entity.name.class/entity.other.inherited-class); punctuation transparent.
  - [x] 2.3 Line loop with persistent `ParseState` + `ScopeStack` (newline-terminated lines for the newlines-mode grammar set), adjacent same-kind span merging, empty-line contract preserved (`vec![empty Plain span]`).
  - [x] 2.4 Fallback wiring: registry miss or any syntect error → existing lexer path untouched.
  - [x] 2.5 `supported_highlight_languages()` unchanged.

- [x] 3. App wiring (`src/lib.rs`, `src/main.rs`)
  - [x] 3.1 Re-export `warm_highlighter`; spawn it on a background thread at the top of `main()`.

- [x] 4. Tests and verification
  - [x] 4.1 Existing highlight tests pass (rust/sql/sh via syntect; ts via fallback; empty/no-language contracts).
  - [x] 4.2 New tests: multi-line block comment / string continuity; registry-uncovered language (e.g. zig) matches legacy lexer output; string spans include their quotes as one String span; syntax_set accessor.
  - [x] 4.3 `cargo test --workspace` fully green.
  - [x] 4.4 Update `docs/typune-integration-plan.md` Phase 2 status.
