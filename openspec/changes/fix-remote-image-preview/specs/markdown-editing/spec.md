## ADDED Requirements

### Requirement: Remote images render in preview surfaces
The editor SHALL render supported HTTP and HTTPS images referenced by Markdown or supported raw HTML in every rendered preview surface. When an authored image URL contains a URI fragment, the editor SHALL omit that fragment from the network request while preserving the authored URL in the Markdown source and preserving its scheme, authority, path, and query parameters for the request. A rendered Markdown image block SHALL present the image without appending its alt text, URL, or optional title as supplemental visible or selectable preview text. Remote-image rendering SHALL NOT invalidate or recompute document-derived Markdown state.

#### Scenario: Fragment-bearing Markdown image renders
- **WHEN** a Markdown image destination is an HTTP(S) URL containing a URI fragment such as `#imgIndex=0`
- **THEN** the rendered preview requests the image without the fragment and displays the returned supported image
- **AND** the Markdown source retains the authored URL

#### Scenario: Query parameters survive request normalization
- **WHEN** a remote image URL contains both query parameters and a fragment
- **THEN** the preview's image request retains the complete query string and removes only the fragment delimiter and fragment value

#### Scenario: Preview surfaces share remote-image behavior
- **WHEN** a supported remote image appears in Split Preview, Read mode, Visual Edit, or a supported raw-HTML image preview
- **THEN** each surface uses the same fragment-safe HTTP(S) request URL behavior

#### Scenario: Rendered image omits redundant metadata
- **WHEN** a Markdown image with alt text, a URL, and an optional title is rendered successfully
- **THEN** the preview displays the image without an additional caption, URL, or title block beneath it
- **AND** preview text selection and Select All do not include that hidden image metadata

#### Scenario: Local image resolution remains unchanged
- **WHEN** an image destination is a local absolute or document-relative path, including a filename containing `#`
- **THEN** the editor resolves it through the existing local-image path rules without applying remote URL fragment normalization

#### Scenario: Remote image loading preserves derived caches
- **WHEN** a remote image request starts, completes, or fails without a document text edit
- **THEN** the document version and cached preview blocks, outline, stats, syntax highlighting, and cached text handle remain unchanged
