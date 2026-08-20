## MODIFIED Requirements

### Requirement: Direct fenced-code editing in Visual Edit
Visual Edit SHALL present an ordinary, exactly ranged fenced code block as a syntax-highlighted direct code editor whose editable text is the authored code payload — the rendered form of a code block IS highlighted code with its fence hidden, so this is rendered WYSIWYG. Payload editing SHALL preserve the authored opening fence, closing fence, fence length, info-string spacing, language token, indentation, blank lines, and final-newline semantics outside the replaced payload. Registered diagram fences SHALL render as a diagram image with an editable source payload (see the `diagram-rendering` capability). Unclosed and ambiguous fenced constructs are WYSIWYG coverage gaps under the `markdown-editing` capability's `WYSIWYG coverage roadmap` and SHALL show raw source only as a transitional affordance until a future change closes the gap.

#### Scenario: Ordinary code payload is edited without exposing fences
- **WHEN** the user focuses and edits the payload of an exactly ranged ordinary fenced code block in Visual Edit
- **THEN** the code remains presented as a code editor with the appropriate memoized syntax highlighting
- **AND** one exact source replacement updates only the authored payload range
- **AND** the opening fence, info string, and closing fence remain byte-identical

#### Scenario: Code editor supports platform text input and IME
- **WHEN** normal input or an IME composition replaces a selection in the direct code editor
- **THEN** the canonical source selection and UTF-8-safe replacement path are used
- **AND** the composition is integrated with the existing candidate geometry and semantic undo contracts

#### Scenario: Registered diagram remains conservative
- **WHEN** a fenced info string resolves to a registered diagram backend
- **THEN** Visual Edit presents the rendered diagram image above its editable source payload editor (rendered WYSIWYG — see the `diagram-rendering` capability), staying source-backed in the sense that only the canonical Markdown source is ever mutated
- **AND** it does not attach the ordinary code direct editor

#### Scenario: Ambiguous fence falls back without loss
- **WHEN** the fence is unclosed or exact payload and delimiter boundaries cannot be proven
- **THEN** Visual Edit exposes the complete authored source as a transitional affordance and classifies the construct as a WYSIWYG coverage gap under the `markdown-editing` roadmap
- **AND** no delimiter or whitespace is synthesized or removed
