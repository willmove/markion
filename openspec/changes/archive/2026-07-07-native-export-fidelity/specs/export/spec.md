## ADDED Requirements

### Requirement: Native HTML/LaTeX export fidelity
The built-in HTML export SHALL keep inline and display math payloads byte-identical to the authored LaTeX (modulo HTML escaping) — extended inline syntax (superscript, subscript, emoji, autolink, highlight) SHALL NOT rewrite text inside math containers. The built-in LaTeX export SHALL preserve resolved inline styling (bold, italic, strikethrough, highlight, superscript, subscript, inline code, links), derive table column alignment from the Markdown separator row, render fenced code as `lstlisting` blocks (naming the language only when listings supports it), and place consecutive list items of the same kind in a single list environment with task-list checkboxes rendered as checkbox symbols.

#### Scenario: Inline math survives the superscript extension
- **WHEN** a paragraph contains `$a^2+b^2=c^2$` together with extended inline syntax such as `x^2^`
- **THEN** the exported HTML carries `data-latex="a^2+b^2=c^2"` unmodified while `x^2^` still renders as `<sup>2</sup>`

#### Scenario: LaTeX preserves inline styles
- **WHEN** a paragraph with bold, strikethrough, highlight, superscript, and link spans is exported to LaTeX
- **THEN** the output uses `\textbf`, `\sout`, `\hl`, `\textsuperscript`, and `\href` rather than flattening to plain text

#### Scenario: LaTeX table alignment follows the separator row
- **WHEN** a table declares `|:--|:-:|--:|`
- **THEN** the LaTeX `longtable` column spec is `{lcr}`

#### Scenario: Task list renders as one environment with checkboxes
- **WHEN** consecutive task-list items are exported to LaTeX
- **THEN** they share a single `itemize` environment and render `$\boxtimes$`/`$\square$` markers
