## ADDED Requirements

### Requirement: Visual Edit partitions a leading same-line Markdown image

When Visual Edit presents a paragraph or heading that **starts** with a nested Markdown image (the image source range and the parent source range share the same start offset) and then continues with trailing prose in the same parent range, it SHALL partition into disjoint visual rows in document order: the image as a rendered image row, then leftover prose as a continuation row of the parent kind. The authored `![alt](url)` bytes SHALL belong only to the image row. Visual Edit SHALL NOT also present those bytes as a conservative source island, as visible syntax under the image preview, or as leaked alt/destination copy in the continuation row. List items that contain inline Markdown images remain out of scope.

#### Scenario: Leading same-line image plus trailing prose

- **WHEN** Visual Edit displays a paragraph of the form `![alt](url)trailing text` (the image is the first construct; trailing prose follows on the same line with no blank line)
- **THEN** the image renders as a bounded image preview
- **AND** the trailing text renders as a normal editable paragraph row below it
- **AND** the complete authored `![alt](url)` syntax does not appear as a source island or as visible copy under the preview
- **AND** no row is force-marked as an unsupported source island due to range overlap

#### Scenario: Leading image in a heading or quoted paragraph

- **WHEN** Visual Edit displays a heading or a blockquote paragraph that starts with a nested Markdown image and continues with trailing prose
- **THEN** the image and leftover prose still partition into disjoint rows
- **AND** quoted rows keep the same quote boundary
- **AND** the authored image syntax is not duplicated under the preview

#### Scenario: Prose-before-image and image-only cases stay unchanged

- **WHEN** Visual Edit displays `text ![alt](url) more`, an image-only paragraph, or a prose paragraph separated from an image by a blank line
- **THEN** those shapes keep their existing partitioned or standalone image presentation
- **AND** they do not regress into overlapping source islands
