## ADDED Requirements

### Requirement: Preferences SHALL expose an Images tab
The Preferences panel SHALL provide an Images tab alongside its existing tabs. It SHALL expose independent local-file, clipboard-byte, and remote-image insertion policies; a resource-directory template with presets and a resolved preview; and the active upload provider. Local-file choices SHALL be keep reference, copy, and upload; clipboard choices SHALL be save and upload; remote-image choices SHALL be keep URL, download, and download then upload. The default template SHALL be `{document}.assets`. The full settings pane SHALL scroll vertically when its content exceeds the viewport and SHALL show a visible scrollbar with the active theme. Controls and validation feedback SHALL use the active theme and interface language. Editing preferences SHALL NOT transfer images or migrate existing resources.

#### Scenario: Policy controls show the current defaults
- **WHEN** Images preferences is opened with no image settings present
- **THEN** local files use copy, clipboard bytes use save, remote images use keep URL, the directory is `{document}.assets`, and no uploader is configured

#### Scenario: Directory preview follows the current document
- **WHEN** the user selects `assets/{document}` while editing `notes/topic.md`
- **THEN** the tab previews `notes/assets/topic/` as the resolved destination
- **AND** an untitled document shows that a save location is needed for document-relative storage

#### Scenario: Invalid template is actionable
- **WHEN** the user enters an absolute path, parent traversal, unknown placeholder, or empty template
- **THEN** the tab explains the invalid value and does not commit it as a valid new setting
- **AND** the last valid stored value remains available

#### Scenario: Image settings exceed the panel height
- **WHEN** the Images tab contains more controls than fit in the Preferences panel viewport
- **THEN** the right side shows a draggable scrollbar representing the full image-settings body
- **AND** wheel and scrollbar movement reach every provider and recovery control

### Requirement: Images preferences SHALL configure and test upload providers
The tab SHALL expose PicGo HTTP endpoint, PicGo Core executable/launcher arguments/configuration path, and custom-command program/argument-list/timeout controls when the corresponding provider is selected. File pickers SHALL be offered for local programs and configuration files. The tab SHALL provide a Test upload action that lets the user select an image and explicitly initiates a real upload through the selected provider, displaying either the validated URL or an actionable error. Opening the tab or changing a setting SHALL NOT upload a test image. A test SHALL NOT insert into or modify any document.

#### Scenario: Provider controls follow selection
- **WHEN** the user switches from PicGo HTTP to a custom command
- **THEN** the relevant program, arguments, and timeout controls appear and retain previously configured values for other providers

#### Scenario: Successful explicit upload test
- **WHEN** the user selects a test image and starts Test upload with a working provider
- **THEN** the tab shows the resulting URL and a Copy URL action
- **AND** document text, selection, dirty state, and undo history are unchanged

#### Scenario: Test fails or is canceled
- **WHEN** the provider is unavailable, produces invalid output, times out, or the user cancels
- **THEN** the tab reports the corresponding state without claiming success
- **AND** the editor remains responsive

### Requirement: Image preferences SHALL persist with compatible defaults
Image preferences SHALL persist in optional `[images]` and provider-specific tables in `config.toml`. A missing table or field SHALL preserve the documented default. Invalid individual values SHALL NOT prevent startup or discard unrelated preferences; invalid transfer configuration SHALL remain disabled with actionable feedback instead of being silently executed. Changes SHALL apply to newly started operations; running operations SHALL retain their captured configuration. Reset SHALL restore default policies and directory and clear Markion's selected uploader/configuration without modifying PicGo's files or any document/resource. Preference summaries SHALL describe image policies, directory, and provider without dumping command arguments or external credentials.

#### Scenario: Existing config is loaded
- **WHEN** an older config has no image tables
- **THEN** existing local import behavior and remote-URL preservation remain the defaults

#### Scenario: Values round-trip
- **WHEN** the user changes a directory, policy, and provider configuration and restarts Markion
- **THEN** each valid value is restored and displayed in the Images tab

#### Scenario: Malformed image settings are isolated
- **WHEN** an image table contains an invalid policy, directory, program, or argument value
- **THEN** the app starts with documented safe policy defaults or a disabled invalid transfer setting as applicable
- **AND** unrelated preferences continue to load

#### Scenario: Settings change during a job
- **WHEN** a job is running and the user changes the destination or provider
- **THEN** that job retains its captured configuration and later jobs use the new values

#### Scenario: Preferences reset
- **WHEN** the user resets preferences
- **THEN** image policies and directory return to defaults and Markion's uploader configuration clears
- **AND** external tool settings, image files, and Markdown references remain intact
