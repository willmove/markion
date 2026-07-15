//! GUI-free diagram rendering contracts and built-in backend adapters.
//!
//! Applications provide presentation and scheduling. Backends implement
//! [`DiagramBackend`] and return SVG; [`DiagramRegistry`] owns dispatch,
//! resource limits, and the passive-SVG safety boundary shared by every caller.

use std::{collections::HashMap, fmt, io::Cursor, sync::Arc};

#[cfg(feature = "mermaid")]
mod mermaid;

#[cfg(feature = "mermaid")]
pub use mermaid::MermaidBackend;

pub const DEFAULT_MAX_SOURCE_BYTES: usize = 64 * 1024;
pub const DEFAULT_MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagramTheme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagramLimits {
    pub max_source_bytes: usize,
    pub max_output_bytes: usize,
}

impl Default for DiagramLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: DEFAULT_MAX_SOURCE_BYTES,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramRenderRequest {
    pub source: String,
    pub theme: DiagramTheme,
    pub limits: DiagramLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramMediaType {
    Svg,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDiagramRender {
    pub media_type: DiagramMediaType,
    pub bytes: Vec<u8>,
}

impl RawDiagramRender {
    pub fn svg(svg: impl Into<String>) -> Self {
        Self {
            media_type: DiagramMediaType::Svg,
            bytes: svg.into().into_bytes(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiagramSize {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiagramRender {
    svg: String,
    intrinsic_size: Option<DiagramSize>,
}

impl DiagramRender {
    pub fn svg(&self) -> &str {
        &self.svg
    }

    pub fn into_svg(self) -> String {
        self.svg
    }

    pub fn intrinsic_size(&self) -> Option<DiagramSize> {
        self.intrinsic_size
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagramErrorKind {
    UnsupportedBackend,
    InputTooLarge,
    InvalidSource,
    UnsafeOutput,
    RenderFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticLocation {
    pub line: u32,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramError {
    kind: DiagramErrorKind,
    detail: String,
    location: Option<DiagnosticLocation>,
}

impl DiagramError {
    pub fn new(kind: DiagramErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
            location: None,
        }
    }

    pub fn at(mut self, line: u32, column: Option<u32>) -> Self {
        self.location = Some(DiagnosticLocation { line, column });
        self
    }

    pub fn kind(&self) -> DiagramErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn location(&self) -> Option<DiagnosticLocation> {
        self.location
    }
}

impl fmt::Display for DiagramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = self.location {
            if let Some(column) = location.column {
                return write!(formatter, "{} at {}:{}", self.detail, location.line, column);
            }
            return write!(formatter, "{} at line {}", self.detail, location.line);
        }
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for DiagramError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagramRegistryError {
    EmptyIdentifier,
    EmptyAlias,
    DuplicateIdentifier(String),
    DuplicateAlias(String),
}

impl fmt::Display for DiagramRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentifier => formatter.write_str("diagram backend identifier is empty"),
            Self::EmptyAlias => formatter.write_str("diagram backend alias is empty"),
            Self::DuplicateIdentifier(identifier) => {
                write!(
                    formatter,
                    "duplicate diagram backend identifier: {identifier}"
                )
            }
            Self::DuplicateAlias(alias) => write!(formatter, "duplicate diagram alias: {alias}"),
        }
    }
}

impl std::error::Error for DiagramRegistryError {}

pub trait DiagramBackend: Send + Sync {
    fn id(&self) -> &'static str;
    fn aliases(&self) -> &'static [&'static str];
    fn render(&self, request: &DiagramRenderRequest) -> Result<RawDiagramRender, DiagramError>;
}

#[derive(Clone)]
struct BackendEntry {
    id: String,
    backend: Arc<dyn DiagramBackend>,
}

#[derive(Clone)]
pub struct DiagramRegistry {
    limits: DiagramLimits,
    backends: HashMap<String, BackendEntry>,
    aliases: HashMap<String, String>,
}

impl Default for DiagramRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagramRegistry {
    pub fn new() -> Self {
        Self::with_limits(DiagramLimits::default())
    }

    pub fn with_limits(limits: DiagramLimits) -> Self {
        Self {
            limits,
            backends: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn limits(&self) -> DiagramLimits {
        self.limits
    }

    pub fn register<B>(&mut self, backend: B) -> Result<(), DiagramRegistryError>
    where
        B: DiagramBackend + 'static,
    {
        self.register_arc(Arc::new(backend))
    }

    pub fn register_arc(
        &mut self,
        backend: Arc<dyn DiagramBackend>,
    ) -> Result<(), DiagramRegistryError> {
        let id = normalize_name(backend.id()).ok_or(DiagramRegistryError::EmptyIdentifier)?;
        if self.backends.contains_key(&id) {
            return Err(DiagramRegistryError::DuplicateIdentifier(id));
        }

        let aliases = backend
            .aliases()
            .iter()
            .map(|alias| normalize_name(alias).ok_or(DiagramRegistryError::EmptyAlias))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(alias) = aliases
            .iter()
            .find(|alias| self.aliases.contains_key(alias.as_str()))
        {
            return Err(DiagramRegistryError::DuplicateAlias(alias.clone()));
        }
        let mut seen = std::collections::HashSet::new();
        if let Some(alias) = aliases.iter().find(|alias| !seen.insert(alias.as_str())) {
            return Err(DiagramRegistryError::DuplicateAlias(alias.clone()));
        }

        let entry = BackendEntry {
            id: id.clone(),
            backend,
        };
        self.backends.insert(id.clone(), entry);
        for alias in aliases {
            self.aliases.insert(alias, id.clone());
        }
        Ok(())
    }

    pub fn backend_id_for_info(&self, info: &str) -> Option<&str> {
        let alias = normalized_info_token(info)?;
        self.aliases.get(&alias).map(String::as_str)
    }

    pub fn render_info(
        &self,
        info: &str,
        source: &str,
        theme: DiagramTheme,
    ) -> Result<DiagramRender, DiagramError> {
        let id = self.backend_id_for_info(info).ok_or_else(|| {
            DiagramError::new(
                DiagramErrorKind::UnsupportedBackend,
                format!("no diagram backend is registered for '{info}'"),
            )
        })?;
        self.render(id, source, theme)
    }

    pub fn render(
        &self,
        backend_id: &str,
        source: &str,
        theme: DiagramTheme,
    ) -> Result<DiagramRender, DiagramError> {
        if source.len() > self.limits.max_source_bytes {
            return Err(DiagramError::new(
                DiagramErrorKind::InputTooLarge,
                format!(
                    "diagram source is {} bytes; limit is {} bytes",
                    source.len(),
                    self.limits.max_source_bytes
                ),
            ));
        }
        let id = normalize_name(backend_id).ok_or_else(|| {
            DiagramError::new(
                DiagramErrorKind::UnsupportedBackend,
                "diagram backend identifier is empty",
            )
        })?;
        let entry = self.backends.get(&id).ok_or_else(|| {
            DiagramError::new(
                DiagramErrorKind::UnsupportedBackend,
                format!("diagram backend '{backend_id}' is not registered"),
            )
        })?;
        let request = DiagramRenderRequest {
            source: source.to_string(),
            theme,
            limits: self.limits,
        };
        let raw = entry.backend.render(&request)?;
        sanitize_svg(raw, self.limits)
    }

    pub fn backend_ids(&self) -> impl Iterator<Item = &str> {
        self.backends.values().map(|entry| entry.id.as_str())
    }
}

pub fn normalized_info_token(info: &str) -> Option<String> {
    info.split_whitespace().next().and_then(normalize_name)
}

fn normalize_name(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_ascii_lowercase())
}

fn sanitize_svg(
    raw: RawDiagramRender,
    limits: DiagramLimits,
) -> Result<DiagramRender, DiagramError> {
    if raw.media_type != DiagramMediaType::Svg {
        return Err(DiagramError::new(
            DiagramErrorKind::UnsafeOutput,
            "diagram backend returned a non-SVG media type",
        ));
    }

    let mut output = Vec::new();
    svg_hush::Filter::new()
        .filter(Cursor::new(raw.bytes), &mut output)
        .map_err(|error| {
            DiagramError::new(
                DiagramErrorKind::UnsafeOutput,
                format!("diagram SVG sanitization failed: {error}"),
            )
        })?;
    if output.len() > limits.max_output_bytes {
        return Err(DiagramError::new(
            DiagramErrorKind::UnsafeOutput,
            format!(
                "sanitized diagram output is {} bytes; limit is {} bytes",
                output.len(),
                limits.max_output_bytes
            ),
        ));
    }
    let svg = String::from_utf8(output).map_err(|_| {
        DiagramError::new(
            DiagramErrorKind::UnsafeOutput,
            "sanitized diagram SVG is not UTF-8",
        )
    })?;
    if !svg.contains("<svg") || !svg.contains("</svg>") {
        return Err(DiagramError::new(
            DiagramErrorKind::UnsafeOutput,
            "diagram output does not contain an SVG root",
        ));
    }
    if ["<a ", "<a>", "<animate", "<set ", "<set>"]
        .iter()
        .any(|needle| svg.contains(needle))
    {
        return Err(DiagramError::new(
            DiagramErrorKind::UnsafeOutput,
            "diagram SVG contains interactive or animated elements",
        ));
    }

    let intrinsic_size = extract_svg_size(&svg);
    Ok(DiagramRender {
        svg,
        intrinsic_size,
    })
}

fn extract_svg_size(svg: &str) -> Option<DiagramSize> {
    let start = svg.find("<svg")?;
    let end = svg[start..].find('>')? + start;
    let tag = &svg[start..=end];
    let width = numeric_attribute(tag, "width");
    let height = numeric_attribute(tag, "height");
    match (width, height) {
        (Some(width), Some(height)) if width > 0.0 && height > 0.0 => {
            Some(DiagramSize { width, height })
        }
        _ => {
            let view_box =
                attribute_value(tag, "viewBox").or_else(|| attribute_value(tag, "viewbox"))?;
            let values = view_box
                .split(|character: char| character.is_ascii_whitespace() || character == ',')
                .filter(|value| !value.is_empty())
                .map(str::parse::<f32>)
                .collect::<Result<Vec<_>, _>>()
                .ok()?;
            (values.len() == 4 && values[2] > 0.0 && values[3] > 0.0).then_some(DiagramSize {
                width: values[2],
                height: values[3],
            })
        }
    }
}

fn numeric_attribute(tag: &str, name: &str) -> Option<f32> {
    attribute_value(tag, name)?
        .trim_end_matches("px")
        .parse()
        .ok()
}

fn attribute_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const SAFE_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="40"><text x="2" y="20">ok</text></svg>"#;

    struct MockBackend {
        id: &'static str,
        aliases: &'static [&'static str],
        output: RawDiagramRender,
        calls: Arc<AtomicUsize>,
    }

    impl DiagramBackend for MockBackend {
        fn id(&self) -> &'static str {
            self.id
        }

        fn aliases(&self) -> &'static [&'static str] {
            self.aliases
        }

        fn render(
            &self,
            _request: &DiagramRenderRequest,
        ) -> Result<RawDiagramRender, DiagramError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.output.clone())
        }
    }

    fn registry_with(output: RawDiagramRender) -> (DiagramRegistry, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let backend = MockBackend {
            id: "mock",
            aliases: &["diagram", "mock-diagram"],
            output,
            calls: calls.clone(),
        };
        let mut registry = DiagramRegistry::new();
        registry.register(backend).unwrap();
        (registry, calls)
    }

    #[test]
    fn aliases_are_case_insensitive_and_use_only_first_info_token() {
        let (registry, _) = registry_with(RawDiagramRender::svg(SAFE_SVG));
        assert_eq!(
            registry.backend_id_for_info("Diagram title=x"),
            Some("mock")
        );
        assert_eq!(registry.backend_id_for_info("unknown"), None);
    }

    #[test]
    fn contributor_backend_dispatches_through_public_trait() {
        let (registry, calls) = registry_with(RawDiagramRender::svg(SAFE_SVG));
        let render = registry
            .render_info("mock-diagram", "source", DiagramTheme::Light)
            .unwrap();
        assert!(render.svg().contains("<svg"));
        assert_eq!(
            render.intrinsic_size(),
            Some(DiagramSize {
                width: 120.0,
                height: 40.0
            })
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn duplicate_identifiers_and_aliases_are_rejected() {
        let calls = Arc::new(AtomicUsize::new(0));
        let backend = |id, aliases| MockBackend {
            id,
            aliases,
            output: RawDiagramRender::svg(SAFE_SVG),
            calls: calls.clone(),
        };
        let mut registry = DiagramRegistry::new();
        registry.register(backend("one", &["first"])).unwrap();
        assert_eq!(
            registry.register(backend("ONE", &["second"])),
            Err(DiagramRegistryError::DuplicateIdentifier("one".into()))
        );
        assert_eq!(
            registry.register(backend("two", &["FIRST"])),
            Err(DiagramRegistryError::DuplicateAlias("first".into()))
        );
    }

    #[test]
    fn unknown_backend_is_typed() {
        let registry = DiagramRegistry::new();
        let error = registry
            .render_info("unknown", "source", DiagramTheme::Light)
            .unwrap_err();
        assert_eq!(error.kind(), DiagramErrorKind::UnsupportedBackend);
    }

    #[test]
    fn source_limit_is_checked_before_backend_invocation() {
        let limits = DiagramLimits {
            max_source_bytes: 4,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let mut registry = DiagramRegistry::with_limits(limits);
        registry
            .register(MockBackend {
                id: "mock",
                aliases: &["mock"],
                output: RawDiagramRender::svg(SAFE_SVG),
                calls: calls.clone(),
            })
            .unwrap();
        assert!(registry.render("mock", "1234", DiagramTheme::Light).is_ok());
        let error = registry
            .render("mock", "12345", DiagramTheme::Light)
            .unwrap_err();
        assert_eq!(error.kind(), DiagramErrorKind::InputTooLarge);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn output_limit_accepts_exact_boundary_and_rejects_one_byte_less() {
        let (registry, _) = registry_with(RawDiagramRender::svg(SAFE_SVG));
        let sanitized = registry
            .render("mock", "source", DiagramTheme::Light)
            .unwrap();
        let exact = sanitized.svg().len();

        for (limit, succeeds) in [(exact, true), (exact - 1, false)] {
            let calls = Arc::new(AtomicUsize::new(0));
            let mut registry = DiagramRegistry::with_limits(DiagramLimits {
                max_source_bytes: DEFAULT_MAX_SOURCE_BYTES,
                max_output_bytes: limit,
            });
            registry
                .register(MockBackend {
                    id: "mock",
                    aliases: &["mock"],
                    output: RawDiagramRender::svg(SAFE_SVG),
                    calls,
                })
                .unwrap();
            assert_eq!(
                registry
                    .render("mock", "source", DiagramTheme::Light)
                    .is_ok(),
                succeeds
            );
        }
    }

    #[test]
    fn sanitizer_removes_scripts_events_and_external_resources() {
        let malicious = r#"<svg xmlns="http://www.w3.org/2000/svg" onload="steal()"><script>steal()</script><image href="https://example.com/t.png"/><rect onclick="steal()" width="1" height="1"/></svg>"#;
        let (registry, _) = registry_with(RawDiagramRender::svg(malicious));
        let render = registry
            .render("mock", "source", DiagramTheme::Light)
            .unwrap();
        assert!(!render.svg().contains("script"));
        assert!(!render.svg().contains("onload"));
        assert!(!render.svg().contains("onclick"));
        assert!(!render.svg().contains("example.com"));
    }

    #[test]
    fn interactive_animation_malformed_and_non_svg_outputs_are_rejected() {
        for raw in [
            RawDiagramRender::svg(
                r##"<svg xmlns="http://www.w3.org/2000/svg"><a href="#x"><text>x</text></a></svg>"##,
            ),
            RawDiagramRender::svg(
                r#"<svg xmlns="http://www.w3.org/2000/svg"><animate attributeName="x"/></svg>"#,
            ),
            RawDiagramRender::svg("<svg>"),
            RawDiagramRender {
                media_type: DiagramMediaType::Other("text/html".into()),
                bytes: b"<html></html>".to_vec(),
            },
        ] {
            let (registry, _) = registry_with(raw);
            let error = registry
                .render("mock", "source", DiagramTheme::Light)
                .unwrap_err();
            assert_eq!(error.kind(), DiagramErrorKind::UnsafeOutput);
        }
    }
}
