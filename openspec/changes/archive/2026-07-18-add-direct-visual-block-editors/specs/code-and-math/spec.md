## ADDED Requirements

### Requirement: Direct fenced-code editing in Visual Edit
Visual Edit SHALL present an ordinary, exactly ranged fenced code block as a syntax-highlighted direct code editor whose editable text is the authored code payload. Payload editing SHALL preserve the authored opening fence, closing fence, fence length, info-string spacing, language token, indentation, blank lines, and final-newline semantics outside the replaced payload. Registered diagram fences, unclosed fences, and ambiguous fenced constructs MUST retain their source-backed island behavior.

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
- **THEN** Visual Edit uses the existing complete source-backed fence island
- **AND** it does not attach the ordinary code direct editor

#### Scenario: Ambiguous fence falls back without loss
- **WHEN** the fence is unclosed or exact payload and delimiter boundaries cannot be proven
- **THEN** Visual Edit exposes the complete authored source island
- **AND** no delimiter or whitespace is synthesized or removed

### Requirement: Direct block-math editing in Visual Edit
Visual Edit SHALL present an exactly ranged block-math construct as a rendered formula together with a direct monospaced LaTeX editor for its authored payload. Editing the payload SHALL preserve its complete delimiters and unrelated whitespace, SHALL remain available while rendering is pending or invalid, and SHALL use the same validation, source selection, IME, and undo paths as other text input.

#### Scenario: Formula and LaTeX editor remain visible together
- **WHEN** an exactly ranged block-math construct appears in Visual Edit
- **THEN** its rendered formula or readable error state is shown together with the editable LaTeX payload
- **AND** focusing the payload does not replace the whole block with raw delimiter source

#### Scenario: Invalid LaTeX remains directly editable
- **WHEN** a payload edit makes the formula invalid or its render cache reports an error
- **THEN** the editor shows the validation or render error without discarding the payload editor
- **AND** the authored source remains available for correction

#### Scenario: Math payload edit is atomic and lossless
- **WHEN** the user replaces text within the LaTeX payload
- **THEN** one canonical source replacement changes only that payload range
- **AND** one Undo restores the prior formula source, selection, and delimiters
