## ADDED Requirements

### Requirement: PDF font fallback excludes rejected faces
When the built-in PDF writer selects a font face it cannot load for embedding, it SHALL resolve the text using compatible fallback faces before emitting glyphs.

#### Scenario: Incompatible host font has a compatible fallback
- **WHEN** shaping selects a host face the PDF writer rejects and another face can render the text
- **THEN** the PDF is exported with glyphs shaped and embedded from the compatible fallback
