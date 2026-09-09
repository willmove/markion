## MODIFIED Requirements

### Requirement: Configurable rendered paragraph spacing
The editor SHALL use a global paragraph-spacing preference, expressed in logical pixels, as the bottom gap after rendered paragraph blocks in Split Preview and Read mode and after rendered body-flow groups in Visual Edit. A Visual Edit body-flow group SHALL be a contiguous sequence of paragraph and source-backed whitespace rows in the same structural context; it SHALL contribute the selected paragraph spacing exactly once at the end of the group so replacing a blank line with text does not add or remove spacing. The default SHALL be 12px and the supported range SHALL be 0–32px inclusive. Changing this preference MUST NOT insert, remove, or rewrite whitespace in the Markdown source, and MUST NOT add artificial paragraph gaps to the source editor.

#### Scenario: Paragraph spacing applies to rendered paragraphs
- **WHEN** the user changes rendered paragraph spacing
- **THEN** paragraph blocks in Split Preview and Read mode and body-flow groups in Visual Edit reflow immediately with the selected bottom gap
- **AND** non-paragraph block spacing retains its defined relationship to the selected typography

#### Scenario: Zero spacing does not change Markdown
- **WHEN** the user selects 0px paragraph spacing
- **THEN** adjacent rendered paragraph blocks and Visual Edit body-flow groups have no added bottom gap
- **AND** the document text, dirty state, undo history, and authored blank lines remain unchanged

### Requirement: Visual Edit authored blank lines follow body line height
Visual Edit SHALL paint each authored blank-line (`Whitespace`) row at the rendered body paragraph line height derived from the rendered-document font size, one painted line per covered newline (floored at one line, capped at the existing pathological bound). A whitespace row SHALL participate with adjacent paragraph rows in one body-flow group, and the final row of that group SHALL contribute the paragraph spacing exactly once. Changing either the rendered-document font size or paragraph-spacing preference SHALL reflow the affected body-flow geometry without inserting, removing, or rewriting authored blank lines. Typography-only reflow MUST NOT increment the Markdown document version or rebuild per-version derived caches.

#### Scenario: Blank-line height matches a one-line paragraph
- **WHEN** Visual Edit displays an authored blank line between two rendered blocks
- **THEN** that `Whitespace` row uses the current body paragraph line height and its body-flow group contributes the current paragraph spacing once
- **AND** replacing the blank line with single-line text does not move the following rendered block

#### Scenario: Blank-line geometry follows typography preferences
- **WHEN** the user changes the rendered-document font size or paragraph-spacing preference while Visual Edit shows authored blank lines
- **THEN** those `Whitespace` line boxes and their body-flow group's trailing spacing reflow with the same geometry used for typed text
- **AND** the document text, dirty state, undo history, authored blank lines, and derived cache identity remain unchanged
