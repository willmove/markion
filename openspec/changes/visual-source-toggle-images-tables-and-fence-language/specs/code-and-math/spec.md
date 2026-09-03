## MODIFIED Requirements

### Requirement: Direct fenced-code editing in Visual Edit
Visual Edit SHALL present an ordinary, exactly ranged fenced code block as a syntax-highlighted direct code editor whose editable text is the authored code payload — the rendered form of a code block IS highlighted code with its fence hidden, so this is rendered WYSIWYG. Payload editing SHALL preserve the authored opening fence, closing fence, fence length, info-string spacing, language token, indentation, blank lines, and final-newline semantics outside the replaced payload. Registered diagram fences SHALL render as a diagram image with an editable source payload (see the `diagram-rendering` capability). Unclosed and ambiguous fenced constructs are WYSIWYG coverage gaps under the `markdown-editing` capability's `WYSIWYG coverage roadmap` and SHALL show raw source only as a transitional affordance until a future change closes the gap.

While an ordinary code fence owns the caret in Visual Edit, the language label in its block header SHALL become an editable field bound to the first authored info-string token. Committing that field SHALL replace only the first token's exact byte range — or insert a language token immediately after the opening fence when no info string is authored — preserving the remainder of the info string, both fences, fence length, spacing, and the payload byte-for-byte. Input through the language field SHALL be sanitized so it cannot introduce whitespace, backticks, or line breaks into the source; one Undo SHALL restore the prior info string. Editing the language token SHALL trigger ordinary re-parse dispatch (including diagram and math fences) without any special-cased transition. Read mode, Split Preview, and headers on fences that do not own the caret SHALL keep the static, non-editable language label.

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

#### Scenario: Language token edit rewrites only the token
- **WHEN** the fence owns the caret and the user changes the header language field from one identifier to another (e.g. `rust` to `toml`)
- **THEN** one exact source replacement covers only the first info-string token's byte range
- **AND** both fences, the remainder of the info string, spacing, and the payload remain byte-identical
- **AND** syntax highlighting follows the new language on re-parse

#### Scenario: Language insertion on a bare fence
- **WHEN** the fence owns the caret, the fence carries no info string, and the user commits a language in the header field
- **THEN** the token is inserted immediately after the opening fence with no other byte change
- **AND** the payload and both fences remain otherwise byte-identical

#### Scenario: Language field cannot corrupt the fence
- **WHEN** input into the language field contains whitespace, backticks, or line breaks
- **THEN** that input is not committed into the canonical source
- **AND** no fence, info-string spacing, or payload byte outside the token range changes through this field

#### Scenario: Reading surfaces keep the static label
- **WHEN** a fenced code block is rendered in Read mode or Split Preview, or a Visual Edit fence does not own the caret
- **THEN** the header shows the static, non-editable language label exactly as before

#### Scenario: Retyped language re-dispatches through ordinary re-parse
- **WHEN** the user changes the first info-string token to a diagram backend alias such as `mermaid` or to `math`
- **THEN** the block re-parses and presents through the corresponding Visual Edit editor on the next derivation
- **AND** no mode-specific transition mutates source beyond the token replacement
