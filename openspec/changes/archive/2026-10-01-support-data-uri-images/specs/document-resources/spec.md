## ADDED Requirements

### Requirement: Data-URI image destinations SHALL be decoded and rendered inline

A Markdown image whose destination is a `data:` URI (`data:<mediatype>[;base64],<data>`) SHALL be rendered in preview, Visual Edit, and raw-HTML `<img>` surfaces by decoding the URI payload entirely in-process — without issuing a network request or reading from disk. Base64-encoded payloads SHALL be decoded to bytes and fed into the same decode pipeline used for local and remote images. Non-base64 (URL-encoded) data-URI payloads SHALL be percent-decoded to bytes and rendered the same way. Decoded data-URI images SHALL be cached and deduplicated under the same bounded preview-image cache as local and remote images, keyed by the full URI, so repeated occurrences do not re-decode.

#### Scenario: Base64 PNG data URI renders

- **WHEN** a document contains an image with destination `data:image/png;base64,<base64-encoded PNG bytes>`
- **THEN** preview, Visual Edit, and raw-HTML surfaces render the decoded PNG
- **AND** no outbound network request is made for the image

#### Scenario: SVG data URI renders as a vector image

- **WHEN** a document contains an image with destination `data:image/svg+xml;base64,<base64-encoded SVG>` or an equivalent non-base64 `data:image/svg+xml,...` URI
- **THEN** the surface renders the SVG through the vector rasterization path
- **AND** the result is presented at the SVG's intrinsic size, subject to the same maximum-edge clamp as file-based SVGs

#### Scenario: Non-base64 data URI is percent-decoded

- **WHEN** a document contains an image with a non-base64 data URI (no `;base64` marker)
- **THEN** the payload is percent-decoded to bytes and decoded as an image of the declared media type

#### Scenario: Repeated data URI deduplicates in cache

- **WHEN** the same data URI appears more than once in a document or across visible documents
- **THEN** the payload is decoded at most once per cache residency
- **AND** each occurrence renders the cached result

#### Scenario: Malformed data URI shows explicit recovery state

- **WHEN** a document contains a data URI that cannot be parsed or whose decoded bytes are not a supported image format
- **THEN** the surface shows an explicit missing-resource placeholder rather than silently dropping the image
- **AND** the placeholder does not mutate the document source
