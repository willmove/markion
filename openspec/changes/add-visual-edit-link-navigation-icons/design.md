## Context

Visual Edit already styles links and footnote references, and Split Preview opens URLs on click-without-drag. Visual Edit cannot reuse that click-on-text model because the same hit target must place the caret. An adjacent icon separates navigation from editing.

## Goals / Non-Goals

**Goals:**

- Store resolved destinations on visual runs for links (`Url`) and footnotes (`Footnote { label }`).
- Render a compact clickable icon after each navigable construct.
- Icon click opens the URL or jumps to the footnote definition block (caret + scroll).

**Non-Goals:**

- Changing progressive-reveal of link syntax.
- Icon packs / themeable SVG assets (Unicode/CSS glyph is enough).
- Making the label itself navigate on plain click.

## Decisions

1. **Put `navigation: Option<VisualNavigationTarget>` on `VisualInlineRun`.**
   - Links: set from `Tag::Link.dest_url` for every text/code run inside the link; UI shows one icon after the last consecutive run sharing the same target.
   - Footnotes: set on the `FootnoteReference` run from its label.
2. **Fragment layout when any run has navigation** (reuse the math flex-wrap pattern) so the icon is a sibling element, not part of the source projection.
3. **Footnote jump:** find `VisualBlockKind::FootnoteDefinition { label }` and `move_to` + `scroll_to_reveal_item`.
4. **URL open:** `cx.open_url` (same as preview). Empty/invalid URLs no-op.

## Risks / Trade-offs

- **[Icon vs caret hit testing]** Icon must stop mouse propagation → Mitigation: dedicated `on_mouse_down` that focuses and navigates without updating selection from the text hitbox.
- **[Wrapped link labels]** Multiple fragments for one link → Mitigation: emit icon only after the last fragment whose source end equals the last run with that navigation target.
