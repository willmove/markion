//! Built-in diagram registry shared by preview and export consumers.

use std::sync::{Arc, OnceLock};

use markion_diagram::{DiagramRegistry, DiagramTheme, MermaidBackend};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag, TagEnd};

use crate::escape::{escape_html_attribute, escape_html_text};

/// Returns the immutable built-in registry. Presentation and scheduling stay
/// in the application crate; this function only defines available backends.
pub fn builtin_diagram_registry() -> Arc<DiagramRegistry> {
    static REGISTRY: OnceLock<Arc<DiagramRegistry>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| {
            let mut registry = DiagramRegistry::new();
            registry
                .register(MermaidBackend)
                .expect("built-in diagram identifiers must be unique");
            Arc::new(registry)
        })
        .clone()
}

/// Resolves the first normalized token from a fenced-code info string.
pub fn diagram_backend_id(info: Option<&str>) -> Option<String> {
    let registry = builtin_diagram_registry();
    registry.backend_id_for_info(info?).map(ToOwned::to_owned)
}

pub(crate) struct HtmlDiagramReplacement {
    marker: String,
    html: String,
}

impl HtmlDiagramReplacement {
    pub(crate) fn apply(self, html: &mut String) {
        *html = html.replace(&self.marker, &self.html);
    }
}

struct PendingHtmlDiagram {
    backend_id: String,
    source: String,
}

/// Replaces registered fenced-code events with opaque comments. Rendering is
/// performed now, but the SVG/fallback is inserted only after Markion's
/// extended-inline and math HTML passes so SVG labels cannot be rewritten.
pub(crate) fn collect_html_diagrams<'a>(
    events: impl IntoIterator<Item = Event<'a>>,
) -> (Vec<Event<'a>>, Vec<HtmlDiagramReplacement>) {
    let registry = builtin_diagram_registry();
    let mut output = Vec::new();
    let mut replacements = Vec::new();
    let mut pending: Option<PendingHtmlDiagram> = None;

    for event in events {
        if let Some(active) = pending.as_mut() {
            match event {
                Event::Text(text) | Event::Code(text) => active.source.push_str(&text),
                Event::SoftBreak | Event::HardBreak => active.source.push('\n'),
                Event::End(TagEnd::CodeBlock) => {
                    let active = pending.take().expect("diagram state is present");
                    let marker =
                        format!("<!--markion-diagram-placeholder:{}-->", replacements.len());
                    let html = match registry.render(
                        &active.backend_id,
                        &active.source,
                        DiagramTheme::Light,
                    ) {
                        Ok(render) => format!(
                            "<div class=\"markion-diagram\" data-diagram-backend=\"{}\">{}</div>\n",
                            escape_html_attribute(&active.backend_id),
                            render.svg()
                        ),
                        Err(_) => format!(
                            "<pre><code class=\"language-{}\">{}</code></pre>\n",
                            escape_html_attribute(&active.backend_id),
                            escape_html_text(&active.source)
                        ),
                    };
                    output.push(Event::Html(CowStr::Boxed(marker.clone().into_boxed_str())));
                    replacements.push(HtmlDiagramReplacement { marker, html });
                }
                _ => {}
            }
            continue;
        }

        match &event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                if let Some(backend_id) = registry.backend_id_for_info(info) {
                    pending = Some(PendingHtmlDiagram {
                        backend_id: backend_id.to_string(),
                        source: String::new(),
                    });
                } else {
                    output.push(event);
                }
            }
            _ => output.push(event),
        }
    }

    (output, replacements)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_registry_classifies_only_the_first_info_token() {
        assert_eq!(
            diagram_backend_id(Some("MerMaid linenos")),
            Some("mermaid".into())
        );
        assert_eq!(diagram_backend_id(Some("rust mermaid")), None);
        assert_eq!(diagram_backend_id(None), None);
    }
}
