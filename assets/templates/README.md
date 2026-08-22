# `reference.docx` — bundled pandoc reference document

The DOCX pandoc engine path passes this file as `--reference-doc`, so pandoc
styles the produced document from it. `word/styles.xml` carries every style
pandoc's docx writer looks up by name (Normal, Body Text, First Paragraph,
Compact, Title, Subtitle, Author, Date, Abstract, Bibliography, Heading 1–6,
Block Text, Footnote Text, Definition Term, Definition, Table Caption, Image
Caption, Figure, Captioned Figure, TOC Heading, Source Code; character styles
Hyperlink, Verbatim Char, Footnote Reference, Default Paragraph Font, and the
skylighting `*Tok` token styles used by `--highlight-style`), with CJK-friendly
fonts (`w:eastAsia` faces on every text style) and an A4 `sectPr`.

## Regenerating

Preferred route when pandoc is installed:

```sh
pandoc -o /tmp/custom-reference.docx --print-default-data-file reference.docx
# unzip, edit word/styles.xml (eastAsia fonts, heading sizes, code style), rezip
```

No-pandoc route (how the committed file was produced — this environment has no
pandoc): run the checked-in generator, which assembles a minimal but valid
OOXML package by hand and validates the archive round-trip:

```sh
python assets/templates/build_reference_docx.py
```

Keep the style *names* above intact when restyling: pandoc resolves styles by
`w:name`, and silently falls back to unstyled output for names it cannot find.
