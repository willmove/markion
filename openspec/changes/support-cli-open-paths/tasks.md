## 1. Startup Intent Parsing

- [ ] 1.1 Add a testable startup-intent type/helper that consumes `std::env::args_os().skip(1)`, keeps only the first positional path, and returns none/file/folder/invalid variants.
- [ ] 1.2 Resolve relative startup paths against `env::current_dir()` before classification, and preserve absolute paths unchanged.
- [ ] 1.3 Classify existing supported Markdown files with the existing `is_markdown_path` helper, existing directories as folders, missing paths and unsupported files as invalid, and do not create any filesystem entries.

## 2. Startup Application Flow

- [ ] 2.1 Add a `run_with_startup_intent` entry path in the app bootstrap and keep `run()` as the environment-argument wrapper used by `src/main.rs`.
- [ ] 2.2 Apply the startup intent during initial `window.update` after focus/menu setup and before `check_recovery_on_startup`.
- [ ] 2.3 For a startup Markdown file, open it with `MarkdownDocument::open`, replace the welcome tab with `replace_active_tab`, update the workspace root through `update_workspace_root_from_document`, and set the existing `StatusOpened` feedback.
- [ ] 2.4 For a startup folder, reuse `set_workspace_root`, reveal the Files sidebar, persist the sidebar preference consistently with File -> Open Folder, and call `schedule_file_tree_scan(Some(display_path), cx)`.
- [ ] 2.5 For invalid startup paths or late file-open failures, keep the welcome document and empty tree, set existing open-failure status feedback, log the reason, and avoid scanning the process working directory.

## 3. Regression Coverage

- [ ] 3.1 Add unit tests for startup-intent parsing/classification: no args, relative Markdown file, absolute Markdown file, directory, unsupported file, missing path, case-insensitive Markdown extensions, and extra positional args ignored.
- [ ] 3.2 Add focused tests or source-wiring assertions that startup file application replaces the welcome tab rather than opening a second tab, and that startup folder application reveals the Files sidebar and schedules a scan without touching document text.
- [ ] 3.3 Verify the new behavior preserves existing startup-without-path behavior and does not introduce any new i18n keys.

## 4. Validation

- [ ] 4.1 Run `cargo fmt`.
- [ ] 4.2 Run `cargo test`.
- [ ] 4.3 Run `cargo build`.
- [ ] 4.4 Run `openspec validate support-cli-open-paths`.
- [ ] 4.5 Manually verify from a terminal: `markion path/to/file.md` opens that file as the first tab; `markion path/to/folder` opens the welcome document with the Files sidebar rooted at that folder; `markion` still starts without scanning the current directory; an unsupported or missing path keeps the welcome document and reports failure.
