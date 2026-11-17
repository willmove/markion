## MODIFIED Requirements

### Requirement: Configurable rendered paragraph spacing
The editor SHALL use a global paragraph-spacing preference, expressed in logical pixels, as the bottom gap after rendered paragraph blocks in Visual Edit, Split Preview, and Read mode. The default SHALL be 12px and the supported range SHALL be 0–128px inclusive. Changing this preference MUST NOT insert, remove, or rewrite whitespace in the Markdown source, and MUST NOT add artificial paragraph gaps to the source editor.

#### Scenario: Paragraph spacing applies to rendered paragraphs
- **WHEN** the user changes rendered paragraph spacing
- **THEN** paragraph blocks in Visual Edit, Split Preview, and Read mode reflow immediately with the selected bottom gap
- **AND** non-paragraph block spacing retains its defined relationship to the selected typography

#### Scenario: Zero spacing does not change Markdown
- **WHEN** the user selects 0px paragraph spacing
- **THEN** adjacent rendered paragraph blocks have no added bottom gap
- **AND** the document text, dirty state, undo history, and authored blank lines remain unchanged

## ADDED Requirements

### Requirement: Configurable document line spacing
The editor SHALL provide a global line-spacing percentage, 75–300 inclusive, relative to the existing document layout. The default SHALL be 100%, preserving existing typography exactly when configuration omits the field. The preference SHALL scale document line heights in Source, Visual Edit, Split Preview and Read modes, including headings, lists, quotes, code/source islands, tables and authored blank lines, without scaling font glyph sizes, paragraph gaps or application chrome. Shaping, wrapping, hit testing, caret/IME bounds, selection and virtualized scroll measurements SHALL use the same resolved line metrics. Applying the preference SHALL preserve document and cache invariants and approximate scroll anchors through the existing typography reflow path. Export layout SHALL remain independent.

#### Scenario: Line spacing changes document geometry
- **WHEN** the user selects 150% line spacing
- **THEN** document line heights are 1.5 times their default values at the chosen font sizes in all document modes
- **AND** glyph sizes, paragraph gap and application chrome do not change
- **AND** caret, selection and pointer geometry follow the new painted lines

#### Scenario: Existing configuration keeps the current appearance
- **WHEN** a configuration written before this preference omits line spacing
- **THEN** every document plane uses its previous default line heights

#### Scenario: Line spacing is presentation only
- **WHEN** the user changes line spacing in a long document or switches to another open tab
- **THEN** documents retain text, versions, dirty state, selection, history and derived-cache identity
- **AND** changed source and rendered layout measurements refresh without resetting the reading position to the top
