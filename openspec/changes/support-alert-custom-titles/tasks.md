## 1. Detection and model

- [ ] 1.1 Add the strict titled-marker parser and `split_titled_alert` (preview builder), `BlockQuote.alert_title`, and `CalloutTitle.title`; add lib tests for the user's sample, case-insensitive kinds, title-only alerts, lazy continuation, and non-alert markers (`[!CUSTOM] x`, `[!NOTE]x`); verify with `cargo test --lib titled_alert`

## 2. Rendering

- [ ] 2.1 Show the custom title in the preview callout title row and the Visual Edit `CalloutTitle` row; add app tests (Read mode shows the title and not the marker; Visual Edit reveals the authored line on focus and keeps single byte ownership); verify with `cargo test --bin markion titled_alert`

## 3. Export

- [ ] 3.1 PDF `Alert.title`, DOCX `render_docx_alert` title, LaTeX bold title line, and the HTML event rewrite (title paragraph for all alerts, retyping titled quotes); add export tests; verify with `cargo test --lib alert_title_export` and `cargo test -p markion-pdf`

## 4. Integration

- [ ] 4.1 Run `cargo fmt --check`, `cargo test --workspace`, and confirm no new clippy lints on changed lines; validate with `openspec validate support-alert-custom-titles --strict`
