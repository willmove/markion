# Image handling and uploads

Markion can keep an image reference, copy or download the image into the note's resource folder, or upload it through PicGo or a custom program. Open **Preferences → Images** to choose a policy for local files, clipboard images, and remote HTTP(S) images. Opening, rendering, saving, or exporting a document never starts a transfer.

## Resource folders

The default template is `{document}.assets`. For `notes/trip.md`, it resolves to `notes/trip.assets/`. The presets `assets` and `assets/{document}` are useful for a shared folder or a nested per-note folder. A template may start with `./` and may contain nested relative segments. `{document}` is the only variable.

Absolute paths, drive or UNC prefixes, `..`, unknown variables, and the document directory itself are rejected. Existing symlink and junction ancestors are checked before publication. Markdown URLs use the complete relative path with `/` separators and percent encoding. Existing equal bytes are reused; a different file with the same name receives a collision-resistant name and is never overwritten.

Chinese document names and resource folders are supported. Encoded destinations such as `%E4%B8%AD…` are normal URL syntax: preview decodes them when opening the local file, so there is no need to replace them manually. If an image still cannot load, the document shows a short message and filename instead of a full system error. In Visual Edit, inspect the image source or right-click the image to edit or replace it.

An untitled note can keep or upload an image without a local resource folder. Copy, save, or download first asks for the Markdown file location. Canceling that save writes no resource files. A later **Save As** invalidates an in-flight automatic edit so its result can be recovered instead of being applied against a different base.

## Insertion and existing documents

- Clipboard image bytes and dropped image files follow their configured insertion policies and retain input order.
- **Insert Image from File** and **Insert Image from URL** use the same pipeline. The original syntax-only **Image** formatting command remains available.
- Pasted Markdown or converted rich text is inserted immediately. Only image references inside the inserted fragment are considered for later copy, download, or upload, and that later rewrite is a separate undo step. Bare URLs are unchanged. The HTML converter currently discards embedded HTML data images before this stage.
- Visual image controls and the Format menu can upload or save one exact occurrence. Shared reference definitions are not globally rewritten.
- **Upload Document Images** and **Save Document Images Locally** process eligible images across the current document. The publishing/Git **Organize Images** command uses the same checked executor.

Inline Markdown images, resolved full/collapsed/shortcut reference images, and HTML `img src` attributes are supported. Code, front matter, ordinary links, unresolved references, `srcset` without a usable `src`, browser-only `blob:` references, and unsupported schemes are skipped with an item reason.

## PicGo HTTP

Select **PicGo HTTP** and leave the default endpoint `http://127.0.0.1:36677/upload`, or enter another loopback HTTP endpoint. Enable PicGo Server in the PicGo GUI and configure Aliyun OSS or another image host in PicGo. Markion sends one staged file per JSON request. It accepts only a successful response containing exactly one HTTP(S) URL. Redirects, non-loopback endpoints, ambiguous result counts, and malformed success responses are rejected.

Use **Test upload** to choose one file and verify the provider without changing a document.

## PicGo Core

Select **PicGo Core**, choose the executable and optional PicGo config file, and enter launcher arguments one per line. Markion appends the PicGo upload command and one staged file. On Windows, configure `node.exe` as the program and the installed PicGo CLI JavaScript entry point as a launcher argument when a `.cmd` shim would otherwise be required. Markion does not install Node or PicGo and does not invoke a shell.

Example configuration:

```toml
[images]
uploader = "picgo-core"

[images.picgo_core]
executable = "C:\\Program Files\\nodejs\\node.exe"
launcher_args = ["C:\\Users\\me\\AppData\\Roaming\\npm\\node_modules\\picgo\\bin\\picgo"]
config_path = "C:\\Users\\me\\.picgo\\config.json"
```

## Custom commands

Select **Custom command**, choose the program, and enter one literal argument per line. `{file}` must appear exactly once as a complete argument. Scripts require an explicit interpreter; Markion never applies shell quoting, environment expansion, pipes, or command substitution.

```toml
[images]
uploader = "command"
timeout_secs = 60

[images.command]
executable = "pwsh.exe"
args = ["-NoProfile", "-NonInteractive", "-File", "C:\\tools\\upload.ps1", "{file}"]
```

The command must exit with status 0 and write exactly one absolute HTTP(S) URL to stdout. Write diagnostics to stderr. Credentials should stay in the uploader's config, environment, or operating-system credential store; avoid placing secrets in arguments. Routine status and logs do not print argument values, raw provider output, signed query strings, or environment variables.

## Limits, cancellation, recovery, and undo

Markion validates image content rather than trusting a filename or server MIME type. Each input or download is limited to 32 MiB. Materialization uses at most two concurrent workers; external uploads run serially. Remote downloads allow at most five redirects. The configurable transfer/process timeout is clamped to 5–600 seconds, and provider output is bounded.

Pending state stays outside Markdown. A completed batch applies all still-valid replacements as one checked source edit on the originating tab, even if another tab is active. Unrelated typing rebases targets; edits that overlap a target, undo/redo, reload, close/reopen, rename, or Save As prevent automatic application. Canceling stops owned work, changes the operation generation, and blocks late automatic edits.

Transfer failures and successful results that could not be applied are recorded atomically in Markion's private recovery directory. **Preferences → Images** lists retained recovery entries and offers URL copying, file reveal, and explicit discard. Discard removes only the private manifest and private staged inputs. It never deletes a published document resource or a remote object. Undo and redo only change recorded Markdown destinations and never repeat network or file transfers.

## Persisted settings

The relevant `config.toml` keys are:

```toml
[images]
local_policy = "copy"          # keep | copy | upload
clipboard_policy = "save"      # save | upload
remote_policy = "keep"         # keep | download | upload
directory = "{document}.assets"
uploader = "none"              # none | picgo-http | picgo-core | command
timeout_secs = 60

[images.picgo_http]
endpoint = "http://127.0.0.1:36677/upload"

[images.picgo_core]
executable = "picgo"
launcher_args = []
# config_path = "..."

[images.command]
# executable = "..."
args = []
```

Invalid fields fall back independently, so one bad provider field does not erase other preferences.
