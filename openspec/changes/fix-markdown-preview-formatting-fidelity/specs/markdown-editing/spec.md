## ADDED Requirements

### Requirement: Blockquotes render all emitted nested child blocks in Preview
In Split Preview and Read mode, `PreviewBlock::BlockQuote` SHALL render every child block variant emitted into its `children` list — including `Paragraph`, `Heading`, `ListItem`, `Html`, `Image`, `DefinitionTerm`, `DefinitionDetail`, and `Rule` — inside the blockquote visual container rather than silently skipping non-paragraph/list/HTML children. Preview plain-text extraction SHALL also include top-level `DefinitionTerm` and `DefinitionDetail` blocks.

#### Scenario: Quoted heading, image, and definition list remain visible in Preview
- **WHEN** a Markdown blockquote contains a heading (`> # Title`), an image (`> ![alt](a.png)`), and a definition list (`> Term\n> : Detail`)
- **THEN** Split Preview and Read mode render all of them inside the blockquote container with their respective typography and layout

### Requirement: Read and Split preview footnote references use superscript geometry
Footnote references in Split Preview and Read mode SHALL render using the same reduced font size and raised baseline (`script_fragment`) as Visual Edit and inline superscripts (`^sup^`), while keeping hover tooltips and selection/copy fidelity.

#### Scenario: Footnote reference shrinks and raises in Read and Split Preview
- **WHEN** a paragraph or table cell contains a footnote reference `[^1]` in Split Preview or Read mode
- **THEN** the rendered footnote marker uses reduced superscript size and raised baseline and retains its footnote hover tooltip

### Requirement: Safe inline-HTML CSS formatting tolerance
Shared inline-HTML parsing (used by Split Preview, Read mode, Visual Edit, HTML blocks, and HTML tables) SHALL recognize `font-weight` values `bold`, `bolder`, and `600`–`900` as bold; `font-style` values `italic` and `oblique` as italic; `text-decoration` and `text-decoration-line` values `underline` and `line-through`; `background-color` with a valid color as highlight; and standard CSS named colors and `rgba(...)` in addition to `#RGB`, `#RRGGBB`, and `rgb(...)`. Harmless typography declarations (`font-size`, `font-family`, `line-height`, `letter-spacing`, `white-space`) co-occurring with recognized formatting declarations SHALL NOT cause the tag's recognized formatting to be discarded, while unsafe or layout-altering CSS properties (such as `position:fixed`, `url(...)`, `var(...)`, or `expression(...)`) and event-handler attributes SHALL remain rejected.

#### Scenario: Named colors, numeric bold weights, oblique, and mixed benign CSS apply
- **WHEN** Markdown or HTML contains `<span style="font-weight:600">a</span>`, `<span style="font-style:oblique">b</span>`, `<span style="color:red">c</span>`, `<span style="background-color:yellow">d</span>`, or `<span style="font-weight:bold; font-size:18px">e</span>`
- **THEN** Preview and Visual Edit render the corresponding bold, italic, color, and highlight styles while continuing to reject `<span style="position:fixed">`

### Requirement: Extended inline delimiters compose around standard Markdown formatting
Extended inline delimiters (`==highlight==`, `^superscript^`, `~subscript~`) SHALL apply their formatting when wrapping standard Markdown inline constructs (such as `==**bold**==`, `==*italic*==`, `==\`code\`==`, `H~**2**~O`, and `x^*2*^`) in Split Preview, Read mode, and Visual Edit. Backslash-escaped delimiters (such as `\==**bold**==` or `\~2~`) SHALL remain literal text, and Visual Edit SHALL reveal the complete exact authored source group when focused.

#### Scenario: Highlight wrapping bold or inline code renders both styles
- **WHEN** a prose block contains `==**marked bold**==` or `==\`hl code\`==`
- **THEN** Split Preview, Read mode, and Visual Edit hide the `==` delimiters and apply highlight styling together with bold or inline-code styling
- **AND** focusing the construct in Visual Edit reveals the complete exact source range
