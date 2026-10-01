## MODIFIED Requirements

### Requirement: Built-in PDF writer fonts and CJK support
The built-in PDF writer SHALL embed subsetted fonts covering every rendered glyph; it SHALL NOT substitute placeholder characters (such as `?`) for any Unicode content, and it SHALL NOT draw Latin letters using a Symbol/Pi encoding (Adobe Symbol-style Greek lookalikes). Font resolution SHALL use ordered fallback stacks — a configured or per-OS system CJK font (Microsoft YaHei, PingFang SC, Noto Sans CJK SC) before a bundled OFL-licensed Noto Sans SC subset as the guaranteed fallback — declared separately for body, heading, and code text. The body stack's Latin face SHALL be the bundled Libertinus Serif family, even when the host fontconfig `serif` alias names a different face. Document language SHALL be detected well enough to select CJK-aware line breaking and CJK–Latin spacing when the document is predominantly Chinese.

#### Scenario: Chinese text renders without substitution
- **WHEN** a document containing Chinese text is exported via the built-in writer
- **THEN** the PDF embeds a font covering those glyphs and no character is replaced by a placeholder

#### Scenario: Export works without any system CJK font
- **WHEN** the built-in writer exports Chinese text on a system with no CJK system font
- **THEN** the bundled fallback font still renders the common Han glyphs

#### Scenario: Code blocks use a monospace stack
- **WHEN** a document contains a fenced code block
- **THEN** the code text renders with the monospace fallback stack, distinct from the body stack

#### Scenario: Latin body text keeps Latin letters
- **WHEN** a paragraph of regular or italic English is exported via the built-in writer on a host whose fontconfig `serif` alias is a Symbol or Pi family
- **THEN** the PDF draws and encodes those letters as Latin (for example `This` stays `This`), not as Greek homoglyphs from a Symbol encoding
