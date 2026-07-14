## Why

The P2a exporter comparison (`docs/typune-integration-audit.md`) exposed real defects in Markion's own export paths, filed as backlog P2c and now user-approved:

1. **HTML: inline math corrupted by the superscript extension.** `render_extended_html_text_nodes` skips `code`/`pre`/`script`/`style` content but not pulldown's math containers, so `$a^2+b^2=c^2$` reaches the math annotator as `a<sup>2+b</sup>2=c^2` — the visible text *and* the `data-latex` attribute are both corrupted.
2. **LaTeX: fidelity far below what the model already carries.** `render_latex_body` flattens `RichText` to `.text`, dropping bold/italic/strikethrough/highlight/super/subscript/links the preview model resolves; table alignment is hardcoded `l`; code uses bare `verbatim`; consecutive list items each open their own `itemize`/`enumerate` environment. The Typune LaTeX renderer demonstrated all four done right (its own math handling is broken, so we port the strengths rather than switch — P2a decision).

## What Changes

- **HTML** (`src/parse.rs`): the extended-inline pass treats `<span|div class="math …">` containers as raw text, exactly like `code`/`pre` — the LaTeX payload passes through untouched and superscript/subscript/emoji/autolink processing no longer applies inside formulas.
- **LaTeX** (`src/render.rs`, `src/lib.rs`):
  - New `render_latex_rich_text` renders spans with `\textbf`/`\textit`/`\sout`/`\hl`/`\textsuperscript`/`\textsubscript`/`\texttt`/`\href`, preserving `$…$` math runs in plain prose; used for paragraphs, list items, and block quotes.
  - Tables derive their column spec (`l`/`c`/`r`) from the separator row via the existing `parse_markdown_table` on the block's `source_range`.
  - Code blocks emit `lstlisting` (with `[language=…]` only for identifiers in listings' supported set) instead of bare `verbatim`.
  - Consecutive list items of the same kind share one `itemize`/`enumerate` environment; task-list checkboxes render as `$\boxtimes$`/`$\square$`.
  - Preamble gains `ulem` (with `normalem`), `soul`, `listings`, `amssymb`.

## Capabilities

### Modified Capabilities
- `export`: HTML export keeps inline math intact alongside extended inline syntax; LaTeX export preserves inline styling, table alignment, listings-based code blocks, and merged list environments.

## Impact

- Edited: `src/parse.rs`, `src/render.rs`, `src/lib.rs` (+ test updates: existing LaTeX test asserts `verbatim`, now `lstlisting`).
- No absorbed-crate changes; no config/UI surface changes.
