## 1. Relevance rules

- [ ] 1.1 Add `file_tree_watch_path_is_relevant(root, path, show_hidden)` to `src/storage/file_tree.rs` (always-excluded components and, when hidden entries are off, dot-named components make a path irrelevant; paths outside `root` are relevant), with unit tests for `.git`, `node_modules`, `.obsidian` with hidden on/off, nested visible paths, and an outside-root path; verify with `cargo test --lib file_tree_watch`
- [ ] 1.2 Add a pure `file_tree_watch_event_is_relevant(event, root, show_hidden)` over `notify::Event` (create/remove/rename candidates are path-filtered; data/metadata/any modifications and access are ignored; unknown kinds, `Flag::Rescan`, and empty path lists are relevant; any relevant path makes the event relevant), with unit tests for each branch including a rename out of `.obsidian`; verify with `cargo test --bin markion file_tree_watch_event`

## 2. Watcher integration

- [ ] 2.1 In `arm_file_tree_watch`, capture the workspace root and `show_hidden_files`, filter events in the `notify` callback with `file_tree_watch_event_is_relevant`, and treat watcher errors as relevant; verify with `cargo build` and the existing file-tree watch tests
- [ ] 2.2 Confirm that toggling hidden entries re-arms the watch (re-arm on toggle if it does not), and add an app-level test that a content-only modification event does not schedule a refresh while a create event does; verify with `cargo test --bin markion file_tree_watch`

## 3. Integration

- [ ] 3.1 Run `cargo fmt --check`, `cargo test --bin markion`, `cargo test --lib`, and confirm no new clippy lints on changed lines; validate with `openspec validate filter-file-tree-watch-events --strict`
