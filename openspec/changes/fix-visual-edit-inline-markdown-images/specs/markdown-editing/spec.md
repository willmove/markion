# Delta: markdown-editing

## ADDED Requirements

### Requirement: Visual Edit partitions prose around nested Markdown images

When Visual Edit presents a paragraph or heading whose source range contains one or more nested Markdown images that are also emitted as their own image blocks, it SHALL partition those ranges into disjoint visual rows in document order: the prose before each image, the image as a rendered image row, and any leftover prose after the last image as a continuation row. Each source byte in the region SHALL belong to exactly one visual row. The image row SHALL use the same Visual Edit image presentation as a standalone image block (bounded preview, caption, missing-resource placeholder) and SHALL NOT fall back to a conservative raw-source island solely because the image originated inside the prose block. Continuation rows SHALL keep the parent construct's kind for headings and paragraphs, including quote context when the parent is a blockquote leaf. List items that contain inline Markdown images are out of scope for this requirement.

#### Scenario: Adjacent-line image without a blank line

- **WHEN** Visual Edit displays a paragraph whose last line is a Markdown image and the preceding prose line has no blank line before it
- **THEN** the prose renders as a normal editable paragraph row
- **AND** the image renders below it as a bounded image preview, not as a gray source island
- **AND** the image alt text is not duplicated as ordinary paragraph copy

#### Scenario: Same-line text surrounding an image

- **WHEN** Visual Edit displays a paragraph of the form `text ![alt](url) more`
- **THEN** the leading text, the image, and the trailing text appear as three disjoint visual rows in that order
- **AND** no row is force-marked as an unsupported source island due to range overlap

#### Scenario: Multiple images in one paragraph

- **WHEN** Visual Edit displays a paragraph that contains two Markdown images with prose between them
- **THEN** visual rows alternate prose and image in source order
- **AND** every source byte of the paragraph is owned by exactly one row

#### Scenario: Image-only and blank-line-separated images stay unchanged

- **WHEN** Visual Edit displays a paragraph that is only a Markdown image, or a prose paragraph separated from an image by a blank line
- **THEN** the image still renders as a single image row
- **AND** the prose paragraph (when present) remains a separate row whose source range does not overlap the image

#### Scenario: Quoted paragraph leaves keep quote context

- **WHEN** Visual Edit displays a blockquote paragraph that contains a nested Markdown image
- **THEN** each partitioned prose row and the image row remain inside the same quote boundary
- **AND** the image still renders as a bounded preview rather than a raw-source island
