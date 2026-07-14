# Implementation Plan: pulldown-cmark unification + AST-adoption evaluation (audit P3)

## Overview

Unify the workspace on pulldown-cmark 0.13 (prerequisite), enable math parsing in the absorbed parser (the unlock this migration pays for), and run the measured AST-adoption evaluation the plan gated Phase 4 on. Decision recorded in design.md: defer AST unification.

## Tasks

- [x] 1. Version unification
  - [x] 1.1 `[workspace.dependencies] pulldown-cmark = "0.13"`; root package inherits; drop the 0.11 pin + simd feature.
  - [x] 1.2 Fix the one breaking pattern (`TagEnd::BlockQuote(_)`); all absorbed-crate tests green on 0.13.

- [x] 2. Math enablement
  - [x] 2.1 `ParserOptions.enable_math` (default true) → `Options::ENABLE_MATH`.
  - [x] 2.2 Trial-merge `$`-led text-event runs so rejected math candidates still hit the `$…$` heuristic without breaking `\^`-escape semantics; tests for embedded-prose math and whitespace-edged runs.

- [x] 3. AST-adoption evaluation (design.md)
  - [x] 3.1 Benchmark Markion preview pipeline vs absorbed full/incremental parse on 37 KiB and 388 KiB documents.
  - [x] 3.2 Record decision + re-open triggers; update audit/plan docs (Phase 4 outcome, P2a consequence).

- [x] 4. Verification
  - [x] 4.1 `cargo test --workspace` fully green.
