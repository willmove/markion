## MODIFIED Requirements

### Requirement: Built-in PDF writer embeds local images
The built-in PDF writer SHALL embed images whose bytes resolve — local files (paths resolved against the document's directory, accepted by payload content rather than extension), remote `http(s)` images prefetched by the export flow, and decoded `data:` URIs. Every resolved payload SHALL be normalized: PNG and JPEG bytes pass through with sniffed dimensions, SVG passes through as the native vector image variant, and other decodable raster payloads (GIF, WebP, …) are decoded and re-encoded as PNG. Images wider than the text column SHALL be scaled down to fit while narrower images keep their natural size at 96 DPI, and the alt text SHALL be preserved as the image's accessibility description. Unresolvable or undecodable image sources SHALL fall back to an `alt: url` text paragraph without failing the export.

#### Scenario: Local image is embedded and scaled
- **WHEN** a document references `![diagram](images/diagram.png)` wider than the text column and the file exists relative to the document
- **THEN** the PDF embeds the image scaled to the column width

#### Scenario: Prefetched remote image is embedded
- **WHEN** a document references `![chart](https://example.com/chart.png)` and the export flow's prefetch delivered that URL's bytes
- **THEN** the PDF embeds the image with the alt text as its accessibility description

#### Scenario: Remote SVG stays vector
- **WHEN** a prefetched remote image is an SVG payload
- **THEN** the PDF embeds it through the native vector image variant rather than dropping it to text

#### Scenario: Raster families are normalized
- **WHEN** a resolved image payload is GIF or WebP
- **THEN** the writer decodes it and embeds a PNG re-encoding instead of the text fallback

#### Scenario: Data-URI image is embedded
- **WHEN** a document references a base64 image `data:` URI
- **THEN** the writer decodes the payload inline and embeds it like a local image, without any network access

#### Scenario: Missing or remote images keep the text fallback
- **WHEN** an image source is remote and its bytes were not prefetched (fetch failed, timed out, was oversized), or a data URI that does not decode, or a local path that does not exist, or a payload the writer cannot decode
- **THEN** the writer emits the `alt: url` text and the export still succeeds

### Requirement: Built-in DOCX fallback embeds local images
The built-in DOCX writer SHALL embed image bytes into the package from three resolvable sources: a local file (relative paths resolved against the document's directory), a remote `http(s)` image whose bytes were prefetched by the export flow, and a `data:` URI decoded inline. Every resolved payload SHALL be normalized before embedding: PNG and JPEG bytes pass through with sniffed dimensions, other decodable raster payloads (GIF, WebP, …) are decoded and re-encoded as PNG, and SVG payloads are rasterized to PNG with system fonts loaded (supersampled for crispness, reported at the SVG's natural size). An embedded image SHALL be copied into `word/media/` with a unique name, declared in `[Content_Types].xml`, and referenced from `word/document.xml` as a `w:drawing` sized in EMUs so that images wider than the text column are scaled down to fit while narrower images keep their natural pixel size (at 96 DPI). The image's alt text SHALL be preserved as the drawing's description. The export flow SHALL prefetch remote images concurrently off the main thread with bounded per-image timeouts and size caps before invoking the writer; images whose fetch fails (offline, HTTP error, oversized) or whose payload cannot be normalized SHALL keep the existing `alt: url` text fallback, and the export SHALL still succeed. The `text-fallback` image policy SHALL continue to export every image (local and remote) as text on both backends.

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

#### Scenario: Raster and vector payloads are normalized
- **WHEN** a resolved image payload is GIF, WebP, or SVG
- **THEN** the writer embeds it as a PNG `w:drawing` (rasterized from SVG at natural size) instead of the text fallback

#### Scenario: Missing or remote images keep the text fallback
- **WHEN** an image source is remote and its bytes were not prefetched (fetch failed, timed out, was oversized), or a data URI that does not decode, or a local path that does not exist, or a payload the writer cannot normalize
- **THEN** the writer emits the existing `alt: url` text paragraph and the export still succeeds
