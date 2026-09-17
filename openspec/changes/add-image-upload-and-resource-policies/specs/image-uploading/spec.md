## Purpose

Defines how Markion transfers image files through externally configured upload tools and obtains validated URLs, while keeping editing responsive and failures recoverable.

## ADDED Requirements

### Requirement: Image uploads SHALL use an explicitly configured external provider
Markion SHALL support PicGo HTTP, PicGo Core CLI, and custom-command providers. Uploading SHALL require a user-selected, valid provider configuration; Markion SHALL NOT silently select a cloud host, install software, or invoke an uploader merely because a document was opened or rendered. Cloud-provider credentials and host selection SHALL remain in PicGo or the custom tool. Each upload SHALL receive a captured image file rather than reading the system clipboard at execution time.

#### Scenario: OSS is configured through PicGo
- **WHEN** the user chooses PicGo configured to upload to Aliyun OSS and requests an image upload
- **THEN** Markion supplies the captured image to PicGo and uses the validated URL returned by PicGo
- **AND** no OSS access key is required in Markion preferences

#### Scenario: Upload configuration is missing
- **WHEN** an insertion policy or manual action requests upload without a valid selected provider
- **THEN** Markion reports the configuration problem and offers access to Images preferences
- **AND** it preserves the original reference or pending input without uploading elsewhere

#### Scenario: Clipboard changes during upload
- **WHEN** the user copies another image after an upload was queued
- **THEN** the queued operation still uploads the originally captured image
- **AND** Markion does not replace the current clipboard contents

### Requirement: PicGo HTTP uploads SHALL follow the local server contract
The PicGo HTTP provider SHALL expose a configurable loopback endpoint defaulting to `http://127.0.0.1:36677/upload`. It SHALL send a JSON POST with a non-empty `list` of absolute file paths, using one image per request for unambiguous result attribution. Success SHALL require a successful HTTP status, `success: true`, and exactly one valid image URL in `result`. Non-loopback endpoints and redirects SHALL be rejected for this local-file protocol.

#### Scenario: PicGo returns one URL
- **WHEN** the server accepts the requested file and returns `{"success":true,"result":["https://images.example.test/a.png"]}`
- **THEN** the provider reports that URL as the result for that file

#### Scenario: PicGo is not running
- **WHEN** the configured loopback server cannot be reached
- **THEN** Markion reports that PicGo must be started or its server settings corrected
- **AND** the document retains its original image reference or recoverable pending input

#### Scenario: Server response is invalid
- **WHEN** the server redirects, returns a non-success status, reports `success: false`, returns malformed JSON, or returns zero or multiple URLs
- **THEN** the operation fails without inserting a guessed URL

### Requirement: PicGo Core CLI uploads SHALL have a dedicated result adapter
The PicGo Core provider SHALL accept a local executable, optional launcher arguments, and an optional PicGo configuration path. It SHALL invoke an upload for one captured file and require a zero exit status plus exactly one valid result URL in the successful upload output. Documented PicGo diagnostic lines SHALL NOT be treated as URLs. Missing programs, unsupported launchers, nonzero exits, ambiguous output, and timeout SHALL produce actionable errors without falling back to another provider.

#### Scenario: PicGo logs precede the upload result
- **WHEN** PicGo Core exits successfully with diagnostic lines followed by its successful-upload marker and one URL
- **THEN** Markion extracts that URL and ignores the diagnostic lines

#### Scenario: CLI does not complete successfully
- **WHEN** the executable is absent, cannot be launched, exits unsuccessfully, or does not return a unique successful result
- **THEN** Markion reports the relevant failure and preserves the input/reference for recovery

### Requirement: Custom commands SHALL use an explicit file-to-URL contract
The custom provider SHALL let the user configure a program, an ordered argument list containing one `{file}` placeholder, and a timeout. The placeholder SHALL expand to one literal absolute file-path argument; Markion SHALL NOT concatenate it into implicitly evaluated shell text. A successful command SHALL exit with zero and output exactly one non-empty line containing an absolute HTTP or HTTPS URL on stdout. Diagnostics SHALL use stderr. Programs requiring a script interpreter SHALL name that interpreter and pass the script as an argument. Command configuration SHALL come only from application preferences, never document content or front matter.

#### Scenario: Paths contain spaces and shell metacharacters
- **WHEN** an image path contains spaces, Unicode characters, or shell metacharacters
- **THEN** the program receives the complete literal path as one argument
- **AND** those characters do not become additional commands or arguments

#### Scenario: Command output is ambiguous
- **WHEN** stdout contains Markdown image syntax, multiple non-empty lines, an invalid URL, or the program exits unsuccessfully
- **THEN** Markion reports an output/command error without modifying the image destination

### Requirement: Image transfers SHALL be bounded and cancellable
Image reads, remote downloads, upload requests, and external commands SHALL run outside the UI thread with finite timeouts, bounded response/output sizes, and bounded concurrency. Remote materialization SHALL support HTTP/HTTPS only, require a successful response and supported image content, enforce the byte limit while reading, and bound redirects. Cancellation SHALL stop queued work, stop owned requests/processes as far as possible, and prevent later automatic document edits. An upload with uncertain delivery SHALL be labeled as such and SHALL NOT be retried automatically.

#### Scenario: A server stalls or sends an oversized response
- **WHEN** a download/upload exceeds the configured timeout or the documented byte/output limit
- **THEN** Markion terminates that local operation and reports the bounded failure
- **AND** typing, navigation, and cancellation remain available

#### Scenario: User cancels an upload
- **WHEN** the user cancels before a pending upload result is applied
- **THEN** no later completion from that attempt changes document text
- **AND** already received URLs remain available in the result summary without claiming that remote objects were deleted

#### Scenario: Connection is lost after upload may have succeeded
- **WHEN** the upload tool may have accepted the file but no conclusive result arrives
- **THEN** the result is reported as uncertain
- **AND** a retry requires a new user action because it could create another remote object

### Requirement: Upload results and diagnostics SHALL be validated before use
All providers SHALL return an absolute HTTP/HTTPS URL with a host and without embedded credentials, control characters, or surrounding Markdown. Markion SHALL escape the URL for its destination syntax without discarding query parameters. Returned URLs SHALL NOT require an additional public GET to count as a successful upload. Routine logs and status messages SHALL exclude full signed-URL queries, environment values, and raw command output; users SHALL still be able to copy their exact successful URL from the operation result.

#### Scenario: Upload returns a signed URL
- **WHEN** a provider returns a valid URL containing signed query parameters
- **THEN** the exact URL is preserved in the image reference and Copy URL result
- **AND** routine diagnostics omit the sensitive query

#### Scenario: Upload returns an unsafe destination
- **WHEN** a provider returns a local path, `data:`, `javascript:`, or a URL containing credentials or control characters
- **THEN** Markion rejects the result and leaves the reference unchanged
