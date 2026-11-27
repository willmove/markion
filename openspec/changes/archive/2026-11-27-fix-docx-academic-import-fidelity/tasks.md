## 1. Import fidelity

- [x] 1.1 Preserve heading bookmarks as unique authored IDs and verify heading parsing and fragment targets.
- [x] 1.2 Serialize continuous emphasis across Word runs and verify the rendered styles at punctuation and style transitions.
- [x] 1.3 Convert the reference document's Office Math structures and verify inline, display, and table formulas plus unsupported-equation diagnostics.
- [x] 1.4 Preserve Word quote styling, thematic breaks and table alignment and remove the false warning for an unreferenced endnotes part, with regression tests.
- [x] 1.5 Verify that a formula-heavy document settles on visible formulas and schedules newly visible ones after scrolling.
- [x] 1.6 Keep heading formulas readable in the outline and verify that both derivation paths produce the same labels and stable anchors.
- [x] 1.7 Preserve authored column alignment for rendered formula cells and verify their layout in the read-only table renderer.

## 2. Verification

- [x] 2.1 Compare the original DOCX with an import through the Windows Markion UI and verify the reported defects and additional formatting differences.
- [x] 2.2 Run the workspace tests, validate the change, archive its specification, and verify the final diff.

## Evidence

- The original Word sample contained 490 equations. Before the change, the converter reported 317 unsupported equations and the Markdown parser found 170 inline formulas and zero display formulas.
- After conversion, the native math renderer successfully processed all 490 equations. The Markdown parser found 449 inline and 41 display formulas. Three Word display equations in table cells became inline formulas within the GFM table.
- The Markdown saved by the Windows import dialog matched the directly tested converter output byte for byte. It retained 55 heading IDs, 13 thematic breaks, 10 quotes, and the centered columns of the first table. The report showed one page-layout normalization notice and zero possible content losses.
- Compared the abstract, reading guide, and sections 0.1 and 6.1 in Microsoft Word and the Windows Markion build. Verified bold phrases, thematic breaks, quotes, inline and display formulas, formulas inside a table, formula text in the outline, outline navigation, and centered mixed formula headers in Read and Visual Edit.
- The final Windows build passed 2,391 workspace tests, including 35 DOCX importer tests, with five existing ignored tests. Strict OpenSpec validation passed.
