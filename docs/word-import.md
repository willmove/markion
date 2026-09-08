# Built-in Word import

Markion can convert a WordprocessingML `.docx` document into a new, portable
Markdown document without Word, LibreOffice, Pandoc, Node, a browser, or a
network connection. Choose **File → Import Word (.docx)**. This native command
is separate from the session-only Word import in the MarkNice browser
workspace.

The source is read on a background worker. Markion validates the ZIP package,
content types, main-document relationship, XML namespaces, relationships, and
embedded images before it offers to write anything. The source DOCX and every
open tab remain unchanged.

## Review and save

Conversion first opens a report containing recovered paragraph, table, image,
and footnote counts. The report groups notes, formatting changes, and possible
content losses, and identifies the package part and document location where
available. Tracked changes use Word's accepted view: inserted and move-to text
is kept, while deleted and move-from text is omitted.

When the report contains possible content loss, **Cancel** is the safe default;
continuing requires the explicit **Continue with Content Loss** action. Other
reports offer **Save as Markdown**. The save picker suggests the DOCX source
stem and only accepts a new Markdown path. Existing Markdown files, managed
asset directories, and destinations already open in Markion are rejected.

Embedded PNG, JPEG, GIF, and WebP files are written below
`<markdown-stem>.assets/`. Markdown uses relative URLs, so move or copy the
`.md` file and its `.assets` directory together. Assets are completed first and
the Markdown path appears last. A successful import opens through Markion's
normal saved-document path in a new clean tab.

## Supported semantics

The first native importer covers paragraphs, explicit line breaks, headings,
bold/italic/strike/underline/superscript/subscript, safe web and email links,
bookmarks, nested list levels and starting numbers, rectangular GFM tables,
merged HTML tables, embedded raster images, footnotes, an OMML math subset,
and accepted-view tracked changes. Literal Markdown punctuation is escaped.

Page geometry, fonts, colors, floating layout, and exact pagination are
normalized because Markdown does not represent them. Comments, headers,
footers, endnotes, text boxes, embedded objects, `altChunk`, unsupported image
formats, unsafe links, and unsupported equations are diagnosed. Readable text
is retained where possible; otherwise the proposed Markdown contains a visible
fallback. A result containing only generated placeholders cannot be saved as a
successful import.

Legacy `.doc`, macro-enabled `.docm`, encrypted packages, PDF/OCR, batch
import, drag-and-drop import, and Word round trips are outside this workflow.

## Resource and cancellation limits

The built-in limits are:

| Resource | Limit |
| --- | ---: |
| Source DOCX | 20 MiB |
| Package entries | 4,096 |
| Actual decompressed package data | 128 MiB |
| One XML part | 16 MiB |
| XML nesting | 128 levels |
| Style inheritance | 64 levels |
| Embedded images | 200 |
| One embedded image | 16 MiB |
| All embedded images | 64 MiB |
| Decoded pixels per image | 40 million |
| Generated Markdown | 16 MiB |
| Conversion deadline | 30 seconds |

Use **File → Cancel Word Import** while conversion is running. Cancellation is
checked while package parts and XML events are consumed and between conversion
units. Once the status changes to saving, publication is intentionally short
and cannot be canceled.

A failure before the Markdown commit point attempts to remove import-owned
files. A process termination can leave a hidden sibling staging directory or
an asset directory without the Markdown file. Markion never removes unrelated
or substituted files; an error or successful-save status names any temporary
path that could not be cleaned automatically.

Compatibility evidence, reader selection, automated coverage, and outstanding
native-platform checks are maintained in
[`docx-import-compatibility.md`](docx-import-compatibility.md).
