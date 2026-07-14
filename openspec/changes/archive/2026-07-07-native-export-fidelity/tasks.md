# Implementation Plan: Native export fidelity fixes (audit P2c)

## Overview

Two defects exposed by the P2a comparison: HTML inline math corrupted by the extended-inline pass, and LaTeX export far below the fidelity the preview model already carries (port of the Typune LaTeX renderer's strengths).

## Tasks

- [x] 1. HTML math protection (`src/parse.rs`)
  - [x] 1.1 `render_extended_html_text_nodes` treats math containers (`span|div` with `math-inline`/`math-display` class) as raw text; test: `$a^2+b^2=c^2$` + `x^2^` in one paragraph.

- [x] 2. LaTeX fidelity (`src/render.rs`, `src/lib.rs`)
  - [x] 2.1 `render_latex_rich_text` (span styles → `\textbf`/`\textit`/`\sout`/`\hl`/`\textsuperscript`/`\textsubscript`/`\texttt`/`\href`); wired for paragraphs, list items, quotes.
  - [x] 2.2 Table column spec from separator-row alignments — carried into `PreviewBlock::Table` from pulldown's `Tag::Table(alignments)` (the `source_range` re-parse route proved unreliable: preview blocks built via `from_text` get `0..0` ranges).
  - [x] 2.3 `lstlisting` code blocks with listings-supported language whitelist.
  - [x] 2.4 Merge consecutive same-kind list items into one environment; `$\boxtimes$`/`$\square$` checkboxes.
  - [x] 2.5 Preamble: `ulem` (normalem), `soul`, `listings`, `amssymb`; update existing verbatim assertion.

- [x] 3. Verification
  - [x] 3.1 New tests for all four scenarios; `cargo test --workspace` fully green.
