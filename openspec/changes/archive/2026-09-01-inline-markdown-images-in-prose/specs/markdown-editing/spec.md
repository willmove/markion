## ADDED Requirements

### Requirement: Mixed Markdown images stay inline with adjacent prose

When a paragraph, heading, quoted paragraph, or list item contains a Markdown image together with any other prose in the same construct, Visual Edit, Read mode, and Split Preview SHALL present the image as an inline atom on the same visual line as the adjacent text (wrapping only when the line does not fit). The authored `![alt](url)` bytes SHALL belong only to that atom. Those surfaces SHALL NOT stack the image on its own row above leftover prose, SHALL NOT paint the complete image syntax as a source island under the preview, and SHALL NOT leak alt text or the destination URL as ordinary copy. Image-only paragraphs and images separated by a blank line remain block-level image rows with the existing image presentation.

#### Scenario: Leading same-line image plus trailing prose

- **WHEN** the document contains a paragraph of the form `![alt](url)trailing text`
- **THEN** Read mode and Split Preview show the image and the trailing text on the same line
- **AND** Visual Edit shows the same inline layout in one paragraph row
- **AND** the complete authored `![alt](url)` syntax does not appear as a source island or as visible copy under the preview while the atom is unfocused

#### Scenario: Text surrounding an image on one line

- **WHEN** the document contains `text ![alt](url) more`
- **THEN** Visual Edit, Read mode, and Split Preview keep leading text, the image atom, and trailing text in one prose row
- **AND** no row is force-marked as an unsupported source island due to range overlap

#### Scenario: Heading, quote, and list item keep the parent construct

- **WHEN** a heading, a blockquote paragraph, or a list item starts with or contains a mixed Markdown image and trailing prose
- **THEN** the image stays an inline atom inside that heading, quoted paragraph, or list item
- **AND** a list item does not emit a second bullet or a continuation paragraph
- **AND** quoted rows keep the same quote boundary

#### Scenario: Image-only and blank-line-separated images stay block-level

- **WHEN** Visual Edit, Read mode, or Split Preview displays a paragraph that is only a Markdown image, or a prose paragraph separated from an image by a blank line
- **THEN** the image still renders as a block-level image row
- **AND** the prose paragraph (when present) remains a separate row whose source range does not overlap the image
