# Installed Windows Typora comparison — 2026-11-17

Reference application: `C:/Applications/Typora/Typora.exe`. Owned fixture: `target/tmp/typora-context-parity-qa/typora-menu-comparison.md`, containing headings, Unicode prose, a three-column table, a local image and trailing prose. The user's already-open dirty Markion document is preserved; Markion comparison/builds use separate fixtures and profiles.

Observed through native screenshots and accessibility (not web documentation):

| Context | Installed Typora behavior | Markion target |
| --- | --- | --- |
| Table cell | Compact clipboard and inline controls; one Table hover flyout beside the root. It groups four row/column insertions, four movements, row/column deletion, Copy Table, source formatting and Delete Table. | One Table flyout; retain existing checked row/column operations and extra duplication/clear/alignment controls; whole-table copy/movement/duplicate/delete identify Table. |
| Heading | Root entry uses the current heading label, e.g. Heading 1. Hover exposes H1–H6 and Paragraph with current indication. Separate Insert flyout. | Context-labeled format flyout, current indication, preserved existing block commands and insertion; grip grids remain available. |
| Prose | Root uses Paragraph; selected text stays selected on right-click, enabling clipboard controls; inline controls and Insert remain compact. A browser search command appears for selection. | Preserve exact selection and clipboard scope, use Paragraph format/Insert flyouts; no browser search integration added. |
| Image | Context menu offers resource/location operations, upload, Scale Image hover submenu (25/33/50/67/80/100/150/200%), syntax conversion and clipboard/save controls. Image stays rendered; selection also reveals an editable Markdown line, but there is no image `</>` hover button. | Follow the user's explicit request to omit image raw-source expansion entirely; provide Properties, supported width presets up to the existing 100% column limit, alignment, replace/save/upload and scoped image commands. |

Zero-delta native pointer movement over a parent, without a left click, opened the Table, Heading and Scale flyouts. Image source-line selection in Typora is recorded as a difference deliberately overridden by the user's explicit request. No Google search, upload, image-file deletion, rename, or external transmission was invoked.

## Implemented and verified

- Editing roots now use one Table hover flyout, a current paragraph/heading format flyout plus Insert where appropriate, and an image scaling flyout. Parent panels remain in place. Combined Table retains all 17 existing exact row/column actions and checked whole-table commands. In-cell block transformations and block/image insertion remain excluded; Link appears once. Whole-unit labels/tooltips identify Table, Image, Heading or Paragraph. Clipboard controls and inline controls are separate compact rows.
- Flyouts open on pointer hover without clicking; Right/Enter enters, Left/Escape backs out, and another root control changes/closes the child. Root/child bounds drive placement, viewport flipping/independent scrolling and capture-phase click-away. Keyboard closure suppresses stationary-pointer synthetic re-entry; stale callbacks cannot address a different submenu. There is no additional document derivation on hover.
- Markdown images stay rendered on hover, selection and load failure, including reference/data-URI images and both stable-inline preference states. Removed the image `</>` button, Edit Source action and raw image payload surface. Selection targets the exact image object; typing replaces that object and Enter adds a paragraph below it. Properties edits URL/alternative text/title through checked occurrence/version ownership, preserving sizing metadata and one-step undo. Unchanged/canceled properties preserve authored bytes, including references.
- Native QA exposed and corrected stationary-pointer reopening after keyboard dismissal, duplicate whole-image clipboard/deletion controls, Escape not canceling image properties, repeated percentage text in scale rows, and presentation metadata leaking into the caption after resizing. The last two presentation fixes were rebuilt and observed in the final native executable.

## Verification results

| Check | Result |
| --- | --- |
| Root `cargo test --lib --bin markion -- --quiet` | Library 756 passed / 1 ignored; app 850 passed / 2 ignored; no failures (`root-complete.log`). |
| `cargo test --workspace --exclude markion -- --quiet` | 593 passed / 2 ignored across member, integration and doc-test suites; no failures (`workspace-members.log`). |
| Root standalone integration binaries | `stats_standalone` 2 passed; `windows_input_frame` 3 passed. |
| Focused menu rerun after scale-row cleanup | 49 passed, 0 failed (`context-final.log`); the subsequent caption fallback was compiled and verified natively. |
| Static/spec checks | `cargo fmt --all -- --check`, `git diff --check`, and strict OpenSpec validation passed. |
| Final native build | `cargo rustc --bin markion -- -o target/tmp/typora-context-parity-qa/verified/markion.exe` succeeded (`native-verified-final.log`). |

The full automated pass totals 2,204 passed, 0 failed and 5 existing ignored; the focused rerun is additional, not added to that total. Logs and owned fixtures are under `target/tmp/typora-context-parity-qa/`.

Native Windows checks on the corrected implementation confirmed Table hover without clicking, side placement without root overlap, child scrolling down to Delete Table, pointer traversal/clicking Insert Row Below and one-step Undo, Left/Escape closure without immediate reopening, and clicking another cell dismissing the entire menu. Ctrl+Shift+N while the Table flyout was open created a new tab without crashing; closing that tab returned to the unchanged fixture. Heading 1 → Heading 2 and Undo worked through the heading flyout. Right-click preserved a selected prose fragment and Bold changed only that fragment; Undo restored it.

Image hover/selection showed the picture with no source button or expanded Markdown. The menu showed one clipboard set, Properties, Scale Image, alignment/resource controls and Delete Image. Scale hover opened without clicking; the final executable showed each percentage once and retained the authored alternative-text caption after choosing 50%. Properties fields displayed URL/alternative text/title; Escape canceled. Changing alternative text and saving matched the exact expected complete document, and one Undo restored it. Enter on the selected image retained the image and added a blank paragraph below; Undo restored it. After reversing native edits and saving, the fixture was byte-identical to its baseline. Both observed final-build stderr logs were empty. Testing used isolated application data; the user's existing dirty Markion window and executable were preserved.

## Remaining intentional differences and unverified scope

This is supported-command parity for the observed Markdown contexts, not a complete Typora clone. Markion retains extra table duplication/clear/alignment and whole-unit movement commands, its existing 75% scale preset, and a maximum width of 100% of the document column (Typora also offers 150/200%). The user explicitly overrides Typora's selected-image Markdown line: Markion does not reveal it. Bitmap clipboard, Google search, image-file rename/move/delete, syntax conversion and Typora's table-source-formatting menu entry were not added; existing full Source mode and Markion table-format commands remain available. Properties may convert only an explicitly edited reference occurrence inline; opening/canceling/no-op preserves reference spelling.

New native checks used standalone Markdown images and local owned files. Automated image tests additionally cover reference/missing/data-URI forms, UTF-8 and LF/CRLF, no-op/cancel/stale targets, metadata preservation and undo. Existing HTML-block source editing remains unchanged. External upload and native file-picker workflows were not invoked; existing resource policies/handlers are retained. Earlier native records in `tasks.md` cover the table Shift+Enter/main editing/code-tab work; the intermittent historical code-background symptom was not reproduced there and its original cause is not claimed proven.
