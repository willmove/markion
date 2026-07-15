use mermaid_rs_renderer::{ParseError, RenderOptions, Theme, render_strict};

use crate::{
    DiagramBackend, DiagramError, DiagramErrorKind, DiagramRenderRequest, DiagramTheme,
    RawDiagramRender,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct MermaidBackend;

impl DiagramBackend for MermaidBackend {
    fn id(&self) -> &'static str {
        "mermaid"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["mermaid"]
    }

    fn render(&self, request: &DiagramRenderRequest) -> Result<RawDiagramRender, DiagramError> {
        let options = RenderOptions {
            theme: match request.theme {
                DiagramTheme::Light => Theme::mermaid_default(),
                DiagramTheme::Dark => Theme::dark(),
            },
            ..RenderOptions::default()
        };
        render_strict(&request.source, options)
            .map(RawDiagramRender::svg)
            .map_err(map_parse_error)
    }
}

fn map_parse_error(error: ParseError) -> DiagramError {
    match error {
        ParseError::UnknownParticipant { line, .. } => {
            DiagramError::new(DiagramErrorKind::InvalidSource, error.to_string()).at(line, None)
        }
        ParseError::UnclosedSubgraph { opened_at } => {
            DiagramError::new(DiagramErrorKind::InvalidSource, error.to_string())
                .at(opened_at, None)
        }
        ParseError::UnexpectedToken { line, col, .. }
        | ParseError::InvalidDirective { line, col, .. } => {
            DiagramError::new(DiagramErrorKind::InvalidSource, error.to_string())
                .at(line, Some(col))
        }
        _ => DiagramError::new(DiagramErrorKind::RenderFailed, error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiagramRegistry, DiagramSize};

    fn registry() -> DiagramRegistry {
        let mut registry = DiagramRegistry::new();
        registry.register(MermaidBackend).unwrap();
        registry
    }

    #[test]
    fn representative_diagram_families_render_to_sanitized_svg() {
        let fixtures = [
            "flowchart LR\nA[Start] --> B[End]",
            "sequenceDiagram\nAlice->>Bob: Hello",
            "classDiagram\nclass Animal\nAnimal : +name",
            "stateDiagram-v2\n[*] --> Ready\nReady --> [*]",
            "erDiagram\nCUSTOMER ||--o{ ORDER : places",
            "pie title Pets\n\"Dogs\" : 60\n\"Cats\" : 40",
        ];
        for fixture in fixtures {
            let render = registry()
                .render("mermaid", fixture, DiagramTheme::Light)
                .unwrap_or_else(|error| panic!("fixture failed: {fixture}\n{error}"));
            assert!(render.svg().contains("<svg"));
            assert!(render.intrinsic_size().is_some());
        }
    }

    #[test]
    fn dark_and_light_outputs_are_distinct_and_sized() {
        let registry = registry();
        let source = "flowchart LR\nA --> B";
        let light = registry
            .render("mermaid", source, DiagramTheme::Light)
            .unwrap();
        let dark = registry
            .render("mermaid", source, DiagramTheme::Dark)
            .unwrap();
        assert_ne!(light.svg(), dark.svg());
        assert!(matches!(
            light.intrinsic_size(),
            Some(DiagramSize { width, height }) if width > 0.0 && height > 0.0
        ));
    }

    #[test]
    fn invalid_source_keeps_structured_location() {
        let error = registry()
            .render(
                "mermaid",
                "flowchart LR\nsubgraph MissingEnd\nA --> B",
                DiagramTheme::Light,
            )
            .unwrap_err();
        assert_eq!(error.kind(), DiagramErrorKind::InvalidSource);
        assert!(error.location().is_some());
    }

    #[test]
    fn sanitized_output_is_decodable_by_gpui_usvg_version() {
        let render = registry()
            .render(
                "mermaid",
                "flowchart TD\nA[Safe] --> B[SVG]",
                DiagramTheme::Light,
            )
            .unwrap();
        let tree = usvg::Tree::from_data(render.svg().as_bytes(), &usvg::Options::default());
        assert!(tree.is_ok());
    }
}
