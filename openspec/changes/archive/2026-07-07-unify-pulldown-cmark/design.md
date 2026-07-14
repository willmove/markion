# Design & Evaluation: pulldown-cmark unification and the AST-adoption decision (P3)

## Context

The integration plan's Phase 4 / audit P3 bundles three ideas: unify pulldown-cmark (0.11 members / 0.13 root), evaluate replacing Markion's hand-rolled preview parsing (`src/parse.rs` + `PreviewBlock` model) with the absorbed `markdown` crate's AST, and — if adopted — unlock `incremental.rs`/`render_cache.rs` and deduplicate `table_ops`/`math`/`emoji`/`extended_inline`. The plan's own gate: **don't restructure unless there is a measured performance problem.**

## Part 1 — Version unification (implemented)

Migration surface measured by compiler: **one** breaking pattern in ~4.5k lines of parser/renderer code (`TagEnd::BlockQuote` → `TagEnd::BlockQuote(_)`, 0.13 added blockquote kinds for GitHub-style alerts). All 453 absorbed-crate tests pass unchanged on 0.13. The `simd` feature pin from Typune's manifest is dropped; 0.13 defaults apply workspace-wide.

## Part 2 — Math enablement (implemented; unlocked by 0.13 being pulled in anyway)

Reading the parser for the migration surfaced why P2a saw Typune's LaTeX exporter escape formulas into garbage: the parser *handles* `Event::InlineMath`/`DisplayMath`, the AST has `Inline::InlineMath`/`Block::MathBlock`, and the LaTeX exporter renders them — but `ParserOptions` never set `Options::ENABLE_MATH`, so the events never fired. Math only worked when a `$…$` run happened to be an *entire* text node (a heuristic in `parse_inlines`).

`enable_math: true` (default) closes that. Two subtleties, both regression-tested:

1. pulldown splits **rejected** math candidates (whitespace-edged `$ x $`, `$^ $`) into separate `$`/content/`$` text events, which would defeat the old heuristic. A trial-merge of `$`-led text-event runs consumes the run only when the merged text hits the heuristic.
2. An unconditional merge would break `\^`-escape handling (pulldown splits escapes into separate text runs; merging them re-exposes `^…^` pairs to the extended-inline parser). The conditional merge preserves the documented escape behavior (`test_escaped_delimiter`).

Consequence for the P2a decision: both of Typune-LaTeX's former advantages are now moot — Markion's native LaTeX gained inline styles/alignment/listings in `native-export-fidelity` (P2c), and the absorbed exporter's math defect is fixed but no longer buys anything. **Markion native stays; `latex.rs`/`html.rs` remain prune candidates for P5.**

## Part 3 — AST adoption evaluation (measured; decision: defer)

Benchmark: dev profile (root package opt-level 1, members opt-level 2 — note the bias *against* Markion), Linux container, synthetic documents mixing headings, styled prose, embedded math, task lists, aligned tables, and code fences.

| Document | Markion `from_text`+`preview_blocks` | Markion edit+full reparse | Typune full parse | Typune incremental edit |
|---|---|---|---|---|
| 37 KiB / 1200 blocks | 4.5 ms | 4.7 ms | 3.3 ms | 1.1 ms |
| 388 KiB / 12000 blocks | 44.9 ms | 45.5 ms | 31.5 ms | 14.1 ms |

Findings:

- At realistic note sizes (≤100 KiB), Markion's full reparse costs single-digit milliseconds — no user-perceivable problem exists. The plan's gate ("no performance problem → don't restructure") is not met.
- The incremental parser's win is ~3× at 388 KiB, not orders of magnitude (block re-splitting dominates its cost), and 14 ms per edit still isn't free. The gap that would justify rewriting the preview model (`PreviewBlock` consumers across `main.rs` rendering, editing helpers, exports, plus ~150 tests) does not appear until documents far beyond Markion's use case.
- Deduplication of `table_ops`/`math`/`emoji`/`extended_inline` remains desirable *in principle* but is priced at the same rewrite; it cannot be harvested separately.

**Decision: defer AST unification indefinitely, with explicit re-open triggers:**

1. user reports of typing/preview lag on large documents (first response: debounce/async preview derivation, which is far cheaper than AST unification and independently useful), or
2. a feature that genuinely needs the richer AST (e.g. block-level structure editing beyond what `PreviewBlock` carries).

Until a trigger fires, the absorbed crate's parser/AST serve the export engine path only, and `incremental.rs`/`render_cache.rs` stay tested inventory. This supersedes the plan's Phase 4 as its recorded outcome; P5 pruning decisions (`html.rs`, `latex.rs`, `image.rs`, `SyntaxHighlighter` facade) are unblocked.
