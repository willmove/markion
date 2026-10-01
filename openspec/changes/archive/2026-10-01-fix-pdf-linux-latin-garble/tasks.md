## 1. Font pin

- [x] 1.1 After loading system fonts and bundled faces in `crates/pdf/src/fonts.rs`, call `set_serif_family("Libertinus Serif")` so CSS generic `serif` cannot follow a host fontconfig Symbol/Pi alias.
- [x] 1.2 PDF body (`FamilyKind::Body`) and snapshot body text request `Family::Name("Libertinus Serif")` instead of the host `serif` generic.
- [x] 1.3 Strip Pi/Adobe-Symbol faces (Standard Symbols L, OpenSymbol, Symbols Nerd Font, …) from the font database after load.

## 2. Tests

- [x] 2.1 Assert bundled-only and process-wide databases resolve `Family::Serif` to `"Libertinus Serif"`; shape a regular Latin sentence and require Libertinus Serif PostScript names (not a Math or Symbol face).
- [x] 2.2 Render a one-paragraph PDF of `This starter document is a quick tour` and assert it embeds Libertinus Serif (not a Symbol face) and that uncompressed bytes do not contain Greek homoglyphs (`Τηις`, `σταρτερ`).
- [x] 2.3 Shape the same Latin sentence through the process-wide (system-font) database and require Libertinus Serif, proving the pin wins over the OS generic serif.
- [x] 2.4 `cargo test -p markion-pdf` stays green.
