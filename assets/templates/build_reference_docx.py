#!/usr/bin/env python3
"""Regenerate `reference.docx` — the bundled pandoc reference document used to
style DOCX exports on the pandoc engine path (`--reference-doc`).

This environment has no pandoc, so the package is built by hand from minimal,
valid OOXML parts. `word/styles.xml` carries every style pandoc's docx writer
looks up by name (Normal, Body Text, First Paragraph, Compact, Title,
Subtitle, Author, Date, Abstract, Bibliography, Heading 1-6, Block Text,
Footnote Text, Definition Term, Definition, Table Caption, Image Caption,
Figure, Captioned Figure, TOC Heading, Source Code; character styles Hyperlink,
Verbatim Char, Footnote Reference and the skylighting `*Tok` token styles) with
CJK-friendly fonts (eastAsia faces on every text style).

If pandoc IS available, prefer the canonical regeneration route documented in
README.md (extract pandoc's default reference doc and restyle it); this script
is the no-pandoc fallback.

Run from the repo root:  python assets/templates/build_reference_docx.py
"""

import zipfile
from pathlib import Path

OUT = Path(__file__).with_name("reference.docx")

W = 'xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'
R = 'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"'
PKG_REL = 'xmlns="http://schemas.openxmlformats.org/package/2006/relationships"'
CT = 'xmlns="http://schemas.openxmlformats.org/package/2006/content-types"'

BODY_FONT = ("Calibri", "DengXian")          # (ascii/hAnsi, eastAsia)
HEADING_FONT = ("Calibri Light", "Microsoft YaHei")
CODE_FONT = ("Consolas", "DengXian")

XML_HEAD = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n'


def rfonts(font):
    return (
        f'<w:rFonts w:ascii="{font[0]}" w:hAnsi="{font[0]}" '
        f'w:eastAsia="{font[1]}" w:cs="{font[0]}"/>'
    )


def para_style(style_id, name, font=BODY_FONT, size=None, bold=False,
               italic=False, color=None, based_on=None, next_style=None,
               qformat=True, ppr="", extra_rpr=""):
    based = f'<w:basedOn w:val="{based_on}"/>' if based_on else ""
    nxt = f'<w:next w:val="{next_style}"/>' if next_style else ""
    qfmt = "<w:qFormat/>" if qformat else ""
    rpr = rfonts(font)
    if bold:
        rpr += "<w:b/>"
    if italic:
        rpr += "<w:i/>"
    if color:
        rpr += f'<w:color w:val="{color}"/>'
    if size:
        rpr += f'<w:sz w:val="{size}"/><w:szCs w:val="{size}"/>'
    rpr += extra_rpr
    return (
        f'<w:style w:type="paragraph" w:styleId="{style_id}">'
        f'<w:name w:val="{name}"/>{based}{nxt}{qfmt}{ppr}'
        f"<w:rPr>{rpr}</w:rPr></w:style>"
    )


def char_style(style_id, name, font=None, size=None, bold=False, italic=False,
               color=None, extra_rpr=""):
    rpr = rfonts(font) if font else ""
    if bold:
        rpr += "<w:b/>"
    if italic:
        rpr += "<w:i/>"
    if color:
        rpr += f'<w:color w:val="{color}"/>'
    if size:
        rpr += f'<w:sz w:val="{size}"/><w:szCs w:val="{size}"/>'
    rpr += extra_rpr
    return (
        f'<w:style w:type="character" w:styleId="{style_id}">'
        f'<w:name w:val="{name}"/><w:rPr>{rpr}</w:rPr></w:style>'
    )


def heading(level, size):
    return para_style(
        f"Heading{level}", f"Heading {level}", font=HEADING_FONT, size=size,
        bold=True, color="2E74B5", based_on="Normal", next_style="BodyText",
        ppr=(
            "<w:keepNext/>"
            f'<w:spacing w:before="240" w:after="80"/>'
            f'<w:outlineLvl w:val="{level - 1}"/>'
        ),
    )


def build_styles_xml():
    styles = [
        # docDefaults: 10.5pt (五号) body with an eastAsia face for CJK text.
        "<w:docDefaults><w:rPrDefault><w:rPr>"
        + rfonts(BODY_FONT)
        + '<w:sz w:val="21"/><w:szCs w:val="21"/>'
        + '<w:lang w:val="en-US" w:eastAsia="zh-CN"/>'
        + "</w:rPr></w:rPrDefault><w:pPrDefault><w:pPr>"
        + '<w:spacing w:after="160" w:line="276" w:lineRule="auto"/>'
        + "</w:pPr></w:pPrDefault></w:docDefaults>",
        '<w:style w:type="paragraph" w:default="1" w:styleId="Normal">'
        '<w:name w:val="Normal"/><w:qFormat/></w:style>',
        para_style("BodyText", "Body Text", based_on="Normal",
                   ppr='<w:spacing w:before="0" w:after="160"/>'),
        para_style("FirstParagraph", "First Paragraph", based_on="BodyText"),
        para_style("Compact", "Compact", based_on="BodyText",
                   qformat=False,
                   ppr='<w:spacing w:before="0" w:after="0"/>'),
        para_style("Title", "Title", font=HEADING_FONT, size=56, bold=True,
                   based_on="Normal", next_style="BodyText",
                   ppr='<w:spacing w:before="240" w:after="240"/>'),
        para_style("Subtitle", "Subtitle", font=HEADING_FONT, size=26,
                   color="595959", based_on="Normal", next_style="BodyText"),
        para_style("Author", "Author", size=24, based_on="Normal",
                   next_style="BodyText",
                   ppr='<w:spacing w:before="0" w:after="80"/>'),
        para_style("Date", "Date", color="595959", based_on="Normal",
                   next_style="BodyText"),
        para_style("Abstract", "Abstract", italic=True, color="595959",
                   based_on="Normal",
                   ppr='<w:ind w:left="720" w:right="720"/>'),
        para_style("Bibliography", "Bibliography", based_on="Normal",
                   qformat=False,
                   ppr='<w:ind w:left="720" w:hanging="720"/>'),
        heading(1, 36), heading(2, 32), heading(3, 28),
        heading(4, 24), heading(5, 22), heading(6, 21),
        para_style("BlockText", "Block Text", italic=True, color="595959",
                   based_on="BodyText",
                   ppr='<w:ind w:left="720"/>'),
        para_style("FootnoteText", "Footnote Text", size=18,
                   based_on="Normal"),
        para_style("DefinitionTerm", "Definition Term", bold=True,
                   based_on="Normal"),
        para_style("Definition", "Definition", based_on="Normal",
                   ppr='<w:ind w:left="720"/>'),
        para_style("TableCaption", "Table Caption", size=18, italic=True,
                   color="595959", based_on="Normal", qformat=False),
        para_style("ImageCaption", "Image Caption", size=18, italic=True,
                   color="595959", based_on="Normal", qformat=False),
        para_style("Figure", "Figure", based_on="Normal", qformat=False,
                   ppr='<w:jc w:val="center"/>'),
        para_style("CaptionedFigure", "Captioned Figure", based_on="Figure",
                   qformat=False),
        para_style("TOCHeading", "TOC Heading", font=HEADING_FONT, size=32,
                   bold=True, color="2E74B5", based_on="Normal",
                   next_style="BodyText",
                   ppr='<w:spacing w:before="240" w:after="120"/>'),
        para_style("SourceCode", "Source Code", font=CODE_FONT, size=20,
                   based_on="Normal",
                   ppr='<w:spacing w:before="40" w:after="40" '
                       'w:line="240" w:lineRule="auto"/>'),
        char_style("DefaultParagraphFont", "Default Paragraph Font",
                   extra_rpr="<w:semiHidden/>"),
        char_style("Hyperlink", "Hyperlink", color="0563C1",
                   extra_rpr='<w:u w:val="single"/>'),
        char_style("VerbatimChar", "Verbatim Char", font=CODE_FONT, size=20),
        char_style("FootnoteReference", "Footnote Reference",
                   extra_rpr='<w:vertAlign w:val="superscript"/>'),
    ]

    # Skylighting token character styles with tango-ish colors; pandoc looks
    # these up by their exact `*Tok` names when `--highlight-style` is active.
    token_colors = {
        "KeywordTok": ("204A87", True, False),
        "DataTypeTok": ("204A87", False, False),
        "DecValTok": ("0000CF", False, False),
        "BaseNTok": ("0000CF", False, False),
        "FloatTok": ("0000CF", False, False),
        "ConstantTok": ("8F5902", False, False),
        "CharTok": ("4E9A06", False, False),
        "SpecialCharTok": ("4E9A06", False, False),
        "StringTok": ("4E9A06", False, False),
        "VerbatimStringTok": ("4E9A06", False, False),
        "SpecialStringTok": ("4E9A06", False, False),
        "ImportTok": ("8F5902", False, False),
        "CommentTok": ("8F5902", False, True),
        "DocumentationTok": ("8F5902", False, True),
        "AnnotationTok": ("8F5902", False, False),
        "CommentVarTok": ("8F5902", False, True),
        "OtherTok": ("000000", False, False),
        "FunctionTok": ("000000", False, False),
        "VariableTok": ("000000", False, False),
        "ControlFlowTok": ("204A87", True, False),
        "OperatorTok": ("000000", False, False),
        "BuiltInTok": ("000000", False, False),
        "ExtensionTok": ("000000", False, False),
        "PreprocessorTok": ("8F5902", False, False),
        "AttributeTok": ("000000", False, False),
        "RegionMarkerTok": ("000000", False, False),
        "InformationTok": ("8F5902", False, True),
        "WarningTok": ("A40000", True, False),
        "AlertTok": ("EF2929", False, False),
        "ErrorTok": ("A40000", True, False),
        "NormalTok": ("000000", False, False),
    }
    for name, (color, bold, italic) in token_colors.items():
        styles.append(char_style(name, name, font=CODE_FONT, size=20,
                                 bold=bold, italic=italic, color=color))

    return (
        XML_HEAD
        + f"<w:styles {W}>"
        + "".join(styles)
        + "</w:styles>"
    )


def build_numbering_xml():
    bullets = ["•", "◦", "▪"]

    def levels(num_fmt, text_fn):
        out = []
        for ilvl in range(9):
            left = 720 * (ilvl + 1)
            out.append(
                f'<w:lvl w:ilvl="{ilvl}"><w:start w:val="1"/>'
                f'<w:numFmt w:val="{num_fmt}"/>'
                f'<w:lvlText w:val="{text_fn(ilvl)}"/>'
                '<w:lvlJc w:val="left"/>'
                f'<w:pPr><w:ind w:left="{left}" w:hanging="360"/></w:pPr>'
                "</w:lvl>"
            )
        return "".join(out)

    bullet = (
        '<w:abstractNum w:abstractNumId="0">'
        + levels("bullet", lambda i: bullets[i % len(bullets)])
        + "</w:abstractNum>"
    )
    decimal = (
        '<w:abstractNum w:abstractNumId="1">'
        + levels("decimal", lambda i: f"%{i + 1}.")
        + "</w:abstractNum>"
    )
    nums = (
        '<w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num>'
        '<w:num w:numId="2"><w:abstractNumId w:val="1"/></w:num>'
    )
    return XML_HEAD + f"<w:numbering {W}>" + bullet + decimal + nums + "</w:numbering>"


CONTENT_TYPES = (
    XML_HEAD
    + f"<Types {CT}>"
    + '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
    + '<Default Extension="xml" ContentType="application/xml"/>'
    + '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
    + '<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>'
    + '<Override PartName="/word/settings.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml"/>'
    + '<Override PartName="/word/numbering.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml"/>'
    + '<Override PartName="/word/fontTable.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml"/>'
    + '<Override PartName="/word/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>'
    + "</Types>"
)

ROOT_RELS = (
    XML_HEAD
    + f"<Relationships {PKG_REL}>"
    + '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>'
    + "</Relationships>"
)

DOCUMENT_XML = (
    XML_HEAD
    + f"<w:document {W} {R}><w:body>"
    + "<w:p><w:r><w:t></w:t></w:r></w:p>"
    # A4 page, 2.54cm margins — pandoc takes the output sectPr from here.
    + '<w:sectPr><w:pgSz w:w="11906" w:h="16838"/>'
    + '<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" '
    + 'w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>'
    + "</w:body></w:document>"
)

DOCUMENT_RELS = (
    XML_HEAD
    + f"<Relationships {PKG_REL}>"
    + '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>'
    + '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings" Target="settings.xml"/>'
    + '<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering" Target="numbering.xml"/>'
    + '<Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable" Target="fontTable.xml"/>'
    + '<Relationship Id="rId5" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>'
    + "</Relationships>"
)

SETTINGS_XML = (
    XML_HEAD
    + f"<w:settings {W}>"
    + '<w:zoom w:percent="100"/><w:defaultTabStop w:val="720"/>'
    + "</w:settings>"
)

FONT_TABLE_XML = (
    XML_HEAD
    + f"<w:fonts {W}>"
    + '<w:font w:name="Calibri"><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>'
    + '<w:font w:name="Calibri Light"><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>'
    + '<w:font w:name="DengXian"><w:family w:val="auto"/><w:pitch w:val="variable"/></w:font>'
    + '<w:font w:name="Microsoft YaHei"><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>'
    + '<w:font w:name="Consolas"><w:family w:val="modern"/><w:pitch w:val="fixed"/></w:font>'
    + "</w:fonts>"
)

THEME_XML = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Office Theme">
  <a:themeElements>
    <a:clrScheme name="Office">
      <a:dk1>
        <a:sysClr val="windowText" lastClr="000000"/>
      </a:dk1>
      <a:lt1>
        <a:sysClr val="window" lastClr="FFFFFF"/>
      </a:lt1>
      <a:dk2>
        <a:srgbClr val="0E2841"/>
      </a:dk2>
      <a:lt2>
        <a:srgbClr val="E8E8E8"/>
      </a:lt2>
      <a:accent1>
        <a:srgbClr val="156082"/>
      </a:accent1>
      <a:accent2>
        <a:srgbClr val="E97132"/>
      </a:accent2>
      <a:accent3>
        <a:srgbClr val="196B24"/>
      </a:accent3>
      <a:accent4>
        <a:srgbClr val="0F9ED5"/>
      </a:accent4>
      <a:accent5>
        <a:srgbClr val="A02B93"/>
      </a:accent5>
      <a:accent6>
        <a:srgbClr val="4EA72E"/>
      </a:accent6>
      <a:hlink>
        <a:srgbClr val="467886"/>
      </a:hlink>
      <a:folHlink>
        <a:srgbClr val="96607D"/>
      </a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Office">
      <a:majorFont>
        <a:latin typeface="Aptos Display" panose="02110004020202020204"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <a:font script="Jpan" typeface="游ゴシック Light"/>
        <a:font script="Hang" typeface="맑은 고딕"/>
        <a:font script="Hans" typeface="等线 Light"/>
        <a:font script="Hant" typeface="新細明體"/>
        <a:font script="Arab" typeface="Times New Roman"/>
        <a:font script="Hebr" typeface="Times New Roman"/>
        <a:font script="Thai" typeface="Angsana New"/>
        <a:font script="Ethi" typeface="Nyala"/>
        <a:font script="Beng" typeface="Vrinda"/>
        <a:font script="Gujr" typeface="Shruti"/>
        <a:font script="Khmr" typeface="MoolBoran"/>
        <a:font script="Knda" typeface="Tunga"/>
        <a:font script="Guru" typeface="Raavi"/>
        <a:font script="Cans" typeface="Euphemia"/>
        <a:font script="Cher" typeface="Plantagenet Cherokee"/>
        <a:font script="Yiii" typeface="Microsoft Yi Baiti"/>
        <a:font script="Tibt" typeface="Microsoft Himalaya"/>
        <a:font script="Thaa" typeface="MV Boli"/>
        <a:font script="Deva" typeface="Mangal"/>
        <a:font script="Telu" typeface="Gautami"/>
        <a:font script="Taml" typeface="Latha"/>
        <a:font script="Syrc" typeface="Estrangelo Edessa"/>
        <a:font script="Orya" typeface="Kalinga"/>
        <a:font script="Mlym" typeface="Kartika"/>
        <a:font script="Laoo" typeface="DokChampa"/>
        <a:font script="Sinh" typeface="Iskoola Pota"/>
        <a:font script="Mong" typeface="Mongolian Baiti"/>
        <a:font script="Viet" typeface="Times New Roman"/>
        <a:font script="Uigh" typeface="Microsoft Uighur"/>
        <a:font script="Geor" typeface="Sylfaen"/>
        <a:font script="Armn" typeface="Arial"/>
        <a:font script="Bugi" typeface="Leelawadee UI"/>
        <a:font script="Bopo" typeface="Microsoft JhengHei"/>
        <a:font script="Java" typeface="Javanese Text"/>
        <a:font script="Lisu" typeface="Segoe UI"/>
        <a:font script="Mymr" typeface="Myanmar Text"/>
        <a:font script="Nkoo" typeface="Ebrima"/>
        <a:font script="Olck" typeface="Nirmala UI"/>
        <a:font script="Osma" typeface="Ebrima"/>
        <a:font script="Phag" typeface="Phagspa"/>
        <a:font script="Syrn" typeface="Estrangelo Edessa"/>
        <a:font script="Syrj" typeface="Estrangelo Edessa"/>
        <a:font script="Syre" typeface="Estrangelo Edessa"/>
        <a:font script="Sora" typeface="Nirmala UI"/>
        <a:font script="Tale" typeface="Microsoft Tai Le"/>
        <a:font script="Talu" typeface="Microsoft New Tai Lue"/>
        <a:font script="Tfng" typeface="Ebrima"/>
      </a:majorFont>
      <a:minorFont>
        <a:latin typeface="Aptos" panose="02110004020202020204"/>
        <a:ea typeface=""/>
        <a:cs typeface=""/>
        <a:font script="Jpan" typeface="游明朝"/>
        <a:font script="Hang" typeface="맑은 고딕"/>
        <a:font script="Hans" typeface="等线"/>
        <a:font script="Hant" typeface="新細明體"/>
        <a:font script="Arab" typeface="Arial"/>
        <a:font script="Hebr" typeface="Arial"/>
        <a:font script="Thai" typeface="Cordia New"/>
        <a:font script="Ethi" typeface="Nyala"/>
        <a:font script="Beng" typeface="Vrinda"/>
        <a:font script="Gujr" typeface="Shruti"/>
        <a:font script="Khmr" typeface="DaunPenh"/>
        <a:font script="Knda" typeface="Tunga"/>
        <a:font script="Guru" typeface="Raavi"/>
        <a:font script="Cans" typeface="Euphemia"/>
        <a:font script="Cher" typeface="Plantagenet Cherokee"/>
        <a:font script="Yiii" typeface="Microsoft Yi Baiti"/>
        <a:font script="Tibt" typeface="Microsoft Himalaya"/>
        <a:font script="Thaa" typeface="MV Boli"/>
        <a:font script="Deva" typeface="Mangal"/>
        <a:font script="Telu" typeface="Gautami"/>
        <a:font script="Taml" typeface="Latha"/>
        <a:font script="Syrc" typeface="Estrangelo Edessa"/>
        <a:font script="Orya" typeface="Kalinga"/>
        <a:font script="Mlym" typeface="Kartika"/>
        <a:font script="Laoo" typeface="DokChampa"/>
        <a:font script="Sinh" typeface="Iskoola Pota"/>
        <a:font script="Mong" typeface="Mongolian Baiti"/>
        <a:font script="Viet" typeface="Arial"/>
        <a:font script="Uigh" typeface="Microsoft Uighur"/>
        <a:font script="Geor" typeface="Sylfaen"/>
        <a:font script="Armn" typeface="Arial"/>
        <a:font script="Bugi" typeface="Leelawadee UI"/>
        <a:font script="Bopo" typeface="Microsoft JhengHei"/>
        <a:font script="Java" typeface="Javanese Text"/>
        <a:font script="Lisu" typeface="Segoe UI"/>
        <a:font script="Mymr" typeface="Myanmar Text"/>
        <a:font script="Nkoo" typeface="Ebrima"/>
        <a:font script="Olck" typeface="Nirmala UI"/>
        <a:font script="Osma" typeface="Ebrima"/>
        <a:font script="Phag" typeface="Phagspa"/>
        <a:font script="Syrn" typeface="Estrangelo Edessa"/>
        <a:font script="Syrj" typeface="Estrangelo Edessa"/>
        <a:font script="Syre" typeface="Estrangelo Edessa"/>
        <a:font script="Sora" typeface="Nirmala UI"/>
        <a:font script="Tale" typeface="Microsoft Tai Le"/>
        <a:font script="Talu" typeface="Microsoft New Tai Lue"/>
        <a:font script="Tfng" typeface="Ebrima"/>
      </a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Office">
      <a:fillStyleLst>
        <a:solidFill>
          <a:schemeClr val="phClr"/>
        </a:solidFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0">
              <a:schemeClr val="phClr">
                <a:lumMod val="110000"/>
                <a:satMod val="105000"/>
                <a:tint val="67000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="50000">
              <a:schemeClr val="phClr">
                <a:lumMod val="105000"/>
                <a:satMod val="103000"/>
                <a:tint val="73000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="100000">
              <a:schemeClr val="phClr">
                <a:lumMod val="105000"/>
                <a:satMod val="109000"/>
                <a:tint val="81000"/>
              </a:schemeClr>
            </a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0">
              <a:schemeClr val="phClr">
                <a:satMod val="103000"/>
                <a:lumMod val="102000"/>
                <a:tint val="94000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="50000">
              <a:schemeClr val="phClr">
                <a:satMod val="110000"/>
                <a:lumMod val="100000"/>
                <a:shade val="100000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="100000">
              <a:schemeClr val="phClr">
                <a:lumMod val="99000"/>
                <a:satMod val="120000"/>
                <a:shade val="78000"/>
              </a:schemeClr>
            </a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="6350" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill>
            <a:schemeClr val="phClr"/>
          </a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
        <a:ln w="12700" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill>
            <a:schemeClr val="phClr"/>
          </a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
        <a:ln w="19050" cap="flat" cmpd="sng" algn="ctr">
          <a:solidFill>
            <a:schemeClr val="phClr"/>
          </a:solidFill>
          <a:prstDash val="solid"/>
          <a:miter lim="800000"/>
        </a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle>
          <a:effectLst/>
        </a:effectStyle>
        <a:effectStyle>
          <a:effectLst/>
        </a:effectStyle>
        <a:effectStyle>
          <a:effectLst>
            <a:outerShdw blurRad="57150" dist="19050" dir="5400000" algn="ctr" rotWithShape="0">
              <a:srgbClr val="000000">
                <a:alpha val="63000"/>
              </a:srgbClr>
            </a:outerShdw>
          </a:effectLst>
        </a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill>
          <a:schemeClr val="phClr"/>
        </a:solidFill>
        <a:solidFill>
          <a:schemeClr val="phClr">
            <a:tint val="95000"/>
            <a:satMod val="170000"/>
          </a:schemeClr>
        </a:solidFill>
        <a:gradFill rotWithShape="1">
          <a:gsLst>
            <a:gs pos="0">
              <a:schemeClr val="phClr">
                <a:tint val="93000"/>
                <a:satMod val="150000"/>
                <a:shade val="98000"/>
                <a:lumMod val="102000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="50000">
              <a:schemeClr val="phClr">
                <a:tint val="98000"/>
                <a:satMod val="130000"/>
                <a:shade val="90000"/>
                <a:lumMod val="103000"/>
              </a:schemeClr>
            </a:gs>
            <a:gs pos="100000">
              <a:schemeClr val="phClr">
                <a:shade val="63000"/>
                <a:satMod val="120000"/>
              </a:schemeClr>
            </a:gs>
          </a:gsLst>
          <a:lin ang="5400000" scaled="0"/>
        </a:gradFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
  <a:objectDefaults/>
  <a:extraClrSchemeLst/>
  <a:extLst>
    <a:ext uri="{05A4C25C-085E-4340-85A3-A5531E510DB2}">
      <thm15:themeFamily xmlns:thm15="http://schemas.microsoft.com/office/thememl/2012/main" name="Office Theme" id="{2E142A2C-CD16-42D6-873A-C26D2A0506FA}" vid="{1BDDFF52-6CD6-40A5-AB3C-68EB2F1E4D0A}"/>
    </a:ext>
  </a:extLst>
</a:theme>
"""


PARTS = {
    "[Content_Types].xml": CONTENT_TYPES,
    "_rels/.rels": ROOT_RELS,
    "word/document.xml": DOCUMENT_XML,
    "word/_rels/document.xml.rels": DOCUMENT_RELS,
    "word/styles.xml": build_styles_xml(),
    "word/numbering.xml": build_numbering_xml(),
    "word/settings.xml": SETTINGS_XML,
    "word/fontTable.xml": FONT_TABLE_XML,
    "word/theme/theme1.xml": THEME_XML,
}


def main():
    import xml.etree.ElementTree as ET

    for name, xml in PARTS.items():
        ET.fromstring(xml)  # fail early on malformed XML

    with zipfile.ZipFile(OUT, "w", zipfile.ZIP_DEFLATED) as docx:
        for name, xml in PARTS.items():
            docx.writestr(name, xml)

    # Read back and validate the archive round-trips.
    with zipfile.ZipFile(OUT) as docx:
        assert docx.testzip() is None
        assert set(docx.namelist()) == set(PARTS)
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes, {len(PARTS)} parts)")


if __name__ == "__main__":
    main()
