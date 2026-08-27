## MODIFIED Requirements

### Requirement: Built-in DOCX fallback embeds local images
The built-in DOCX writer SHALL embed image bytes into the package from three resolvable sources: a local file (relative paths resolved against the document's directory), a remote `http(s)` image whose bytes were prefetched by the export flow, and a `data:` URI decoded inline. An embedded image SHALL be copied into `word/media/` with a unique name, declared in `[Content_Types].xml`, and referenced from `word/document.xml` as a `w:drawing` sized in EMUs so that images wider than the text column are scaled down to fit while narrower images keep their natural pixel size (at 96 DPI). The image's alt text SHALL be preserved as the drawing's description. The export flow SHALL prefetch remote images concurrently off the main thread with bounded per-image timeouts and size caps before invoking the writer; images whose fetch fails (offline, HTTP error, oversized) or whose payload is not PNG/JPEG SHALL keep the existing `alt: url` text fallback, and the export SHALL still succeed. The `text-fallback` image policy SHALL continue to export every image (local and remote) as text on both backends.

#### Scenario: Local image is embedded
- **WHEN** a document references `![diagram](images/diagram.png)` and the file exists relative to the document
- **THEN** the package contains `word/media/` with the image bytes, a relationship, a content-type entry for the extension, and a `w:drawing` in the document flow

#### Scenario: Oversized image is scaled to the column
- **WHEN** an embedded image's natural width at 96 DPI exceeds the text column width
- **THEN** the `wp:extent` scales it down proportionally to fit the column

#### Scenario: Prefetched remote image is embedded
- **WHEN** a document references `![chart](https://example.com/chart.png)` and the export flow's prefetch delivered that URL's PNG bytes to the writer
- **THEN** the package contains `word/media/` with the fetched bytes and a `w:drawing` in the document flow, with the alt text as the description

#### Scenario: Data-URI image is embedded
- **WHEN** a document references a base64 PNG `data:` URI image
- **THEN** the writer decodes the payload inline and embeds it like a local image, without any network access

#### Scenario: Missing or remote images keep the text fallback
- **WHEN** an image source is remote and its bytes were not prefetched (fetch failed, timed out, was oversized), or a data URI that does not decode, or a local path that does not exist, or a payload the writer cannot embed
- **THEN** the writer emits the existing `alt: url` text paragraph and the export still succeeds
