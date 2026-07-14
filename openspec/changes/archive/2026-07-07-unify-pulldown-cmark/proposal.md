## Why

P3 of `docs/typune-integration-audit.md` (user-approved): the workspace deliberately shipped two pulldown-cmark versions — 0.11 for the absorbed member crates (Typune parity) and 0.13 for the root package. The split was tolerable as a transition state but blocks everything downstream of the absorbed parser: the AST-unification evaluation, and (discovered during implementation) proper math parsing — the absorbed parser handles `Event::InlineMath`/`DisplayMath` and its AST + LaTeX exporter already support math nodes, but `ParserOptions` never sets `ENABLE_MATH`, which is exactly why the P2a comparison saw Typune LaTeX escape formulas into garbage.

P3's second half — adopting the absorbed AST/incremental parser inside Markion — is an *evaluation*, gated by the plan's own criterion ("不动，除非有性能问题"). This change carries the evaluation in `design.md` with measured numbers and records the go/no-go decision.

## What Changes

- **Version unification** (`Cargo.toml`): `[workspace.dependencies] pulldown-cmark = "0.13"`; the root package inherits via `.workspace = true`. Exactly one breaking API surface in the absorbed crate: `TagEnd::BlockQuote` gained a kind payload (pattern → `TagEnd::BlockQuote(_)`).
- **Math parsing enabled** (`crates/markdown/src/parser.rs`): `ParserOptions.enable_math` (default `true`) sets `Options::ENABLE_MATH`. Embedded math in prose (`prose $a^2+b^2$ end`) now produces `Inline::InlineMath` — previously only a whole-text-node `$…$` heuristic worked. The heuristic is kept as a fallback with a trial-merge for `$`-led text-event runs (pulldown splits *rejected* math candidates like whitespace-edged `$ x $` into separate `$`/content/`$` events); the merge consumes the run only when it hits the heuristic, preserving `\^`-escape semantics (regression-tested).
- **Evaluation recorded** (`design.md`): benchmark of Markion's preview pipeline vs the absorbed full/incremental parser; decision: **defer AST unification** (no performance problem at realistic sizes; triggers documented).

## Capabilities

### Modified Capabilities
- `crate-architecture`: the workspace SHALL use a single pulldown-cmark version for the root package and all members (no split-version transition state).

## Impact

- Edited: root `Cargo.toml` (+ lockfile), `crates/markdown/src/parser.rs` (fourth additive change to an absorbed crate: one pattern fix, math option, trial-merge; two new tests).
- Absorbed crate behavior: math nodes now parse by default — `render_to_markdown` round-trips them as `$…$`, so pandoc engine input is unchanged; the absorbed LaTeX exporter now emits correct math (its P2a defect), though Markion keeps its native LaTeX path (see design.md).
- All 453 absorbed-crate tests green on 0.13; full workspace suite green.
