## ADDED Requirements

### Requirement: Data-URI payloads are elided across every Visual Edit source surface
Every Visual Edit surface that displays authored data-URI bytes SHALL elide the opaque payload (the bytes after the RFC 2397 comma) into the same atomic summary token used by the image source toggle: `…{size}…` framed by ellipsis marks with a human-readable binary-unit size, rendered with distinct chip styling. This SHALL cover the image source payload editor, raw-HTML block payload editors, source-island fallback rows, caret-revealed inline Markdown image runs and inline `<img>` atoms in prose (paragraphs, headings, list items, quote leaves, footnote text), and revealed inline link destinations. The structural bytes around an elided payload (Markdown delimiters, `data:` scheme, media type, `;base64,` marker, HTML attribute quoting) SHALL stay verbatim and editable.

Each elided token SHALL behave as one atomic unit on every surface that shows it: the caret snaps to its boundaries and never rests inside, selection edits replace the entire elided byte range through one exact canonical source replacement, and adjacent Backspace/Delete removes the whole payload with a single Undo restoring it. Elision SHALL be deterministic per document version, not dependent on how the surface is opened, and a surface without any data-URI payload SHALL render byte-for-byte as before this requirement. Read mode and Split Preview are unaffected.

#### Scenario: Raw-HTML image block payload elides its src data URI

- **WHEN** Visual Edit shows a standalone raw-HTML block containing `<img src="data:image/png;base64,AAAA…">` and its source payload is expanded
- **THEN** the payload editor shows the tag with the `src` attribute's data-URI payload collapsed into one size-labeled token while the surrounding tag and attribute syntax stay verbatim and editable

#### Scenario: Caret-revealed inline image elides in prose

- **WHEN** the caret or a selection endpoint enters an inline Markdown image `![alt](data:…)` or inline `<img src="data:…">` atom inside prose and the authored bytes are revealed
- **THEN** the revealed run shows the structural syntax verbatim with the opaque payload collapsed into the same token
- **AND** leaving the range restores the rendered atom without changing the document version

#### Scenario: Revealed link destination with a data URI elides

- **WHEN** an inline link `[label](data:…)` is revealed by the caret in Visual Edit
- **THEN** its destination payload appears as the same token instead of verbatim bytes

#### Scenario: Source-island fallback elides unprovable image spans

- **WHEN** a data-URI image whose exact span cannot be proven renders through the source-island fallback
- **THEN** the island row shows the elided token instead of the raw payload

#### Scenario: Prose that merely mentions data does not elide

- **WHEN** revealed source contains text like `(see data:foo,bar)` whose `data:` run lacks a media type and URI delimiters
- **THEN** no elision is applied and the text renders verbatim

#### Scenario: Atomic deletion works on every surface

- **WHEN** the caret sits at an elided token's trailing edge in an HTML payload, source island, or revealed run and the user presses Backspace (or forward-Delete at the leading edge)
- **THEN** one exact canonical replacement removes the whole opaque payload and a single Undo restores it
