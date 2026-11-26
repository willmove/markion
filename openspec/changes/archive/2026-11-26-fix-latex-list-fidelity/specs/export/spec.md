## MODIFIED Requirements

### Requirement: Native HTML/LaTeX export fidelity
The built-in HTML export SHALL keep inline and display math payloads byte-identical to the authored LaTeX (modulo HTML escaping) — extended inline syntax (superscript, subscript, emoji, autolink, highlight) SHALL NOT rewrite text inside math containers. The built-in LaTeX export SHALL preserve resolved inline styling (bold, italic, strikethrough, highlight, superscript, subscript, inline code, links), derive table column alignment from the Markdown separator row, render fenced code as `lstlisting` blocks (naming the language only when listings supports it), and preserve resolved list structure. Consecutive sibling items of the same kind SHALL share an `itemize` or `enumerate` environment; nested items SHALL appear in child environments under their parent item rather than being flattened into the parent environment. Each ordered environment SHALL start at its resolved first item index, including after prose or a different list kind. Task-list checkboxes SHALL retain their checked/unchecked symbols at every depth, and existing rich item styling SHALL be preserved.

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
- **WHEN** consecutive task-list sibling items are exported to LaTeX
- **THEN** they share a single `itemize` environment and render `$\boxtimes$`/`$\square$` markers

#### Scenario: Nested lists retain their parent and kind
- **WHEN** a list item contains child items, including a child list whose ordered or unordered kind differs from its parent
- **THEN** the child items are exported in a nested environment of the appropriate kind beneath that parent item
- **AND** later top-level siblings remain in the parent environment rather than joining the child environment

#### Scenario: Ordered environments retain their starting values
- **WHEN** ordered lists start at values other than 1 or are separated by prose or another list kind
- **THEN** each ordered environment begins at its resolved first item index and subsequent sibling items increment from that value
- **AND** no ordered environment silently restarts at 1
