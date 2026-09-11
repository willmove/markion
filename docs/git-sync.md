# Backup and Sync

Markion can keep a notes folder backed up at a sync location and bring changes made on another computer into the same folder. The feature is powered by Git, but routine use does not require Git knowledge: use the **Backup and Sync** status in the bottom bar, then choose the one action Markion offers.

## Before you start

- Install Git 2.39 or later. Markion normally finds it automatically.
- Get a sync address from the service that will hold the notes. It is usually an HTTPS or SSH address and may require an account and access permission.
- A dedicated folder or repository for notes is recommended. Markion will not silently include unrelated or unsafe files.

Passwords and access tokens are handled by the operating system or Git credential helper. Markion does not store them in its preferences, notes folder, logs, or command history. SSH users should configure their SSH agent, keys, host settings, and trusted hosts before connecting.

## Turn on Backup and Sync

Open the notes folder and select **Backup and Sync → Turn on Backup and Sync**, or use the status-bar entry. Markion examines the folder and presents one recommended route:

- **Use this folder for Backup and Sync** when it is already a dedicated notes repository. Markion reuses its existing sync address, or you can paste one if it is not connected yet.
- **Enable Backup and Sync for this folder** when it contains local notes that have not been connected yet. Paste the sync address to send them.
- **Get notes from an existing sync address** when the open folder is empty and the notes already exist at a sync location. Paste the sync address and choose where the local copy should be created.

Other routes are available under **Other setup options**. A normal setup asks for at most the sync address. Branch, author identity, destination overrides, and transport details remain under **Advanced setup**.

If the folder is inside another repository or mixes notes with unrelated project content, Markion opens **Advanced repository setup** instead of guessing. Unsupported repository shapes remain inspectable but read-only for synchronization. These include linked worktrees, submodules, Git LFS, bare, sparse, partial, shallow, detached-branch, and nested-repository workspaces.

An empty sync location is valid. Finish setup, create and save a note, then choose **Sync Now**. If login or first publication fails, the local notes and version are retained; correct the account or permission problem and choose **Retry**.

## Everyday use

The bottom bar shows one plain-language state and one relevant action. Selecting the state opens the transient **Backup and Sync** center, which shows the sync location, local item count, incoming update count, last confirmed check, recent activity, version history, and settings.

Common states include:

- **Backup and sync is off** — choose **Turn on backup and sync**.
- **Local items waiting to sync** or **Updates available** — choose **Sync Now**.
- **Everything is synchronized** — the local and remote versions were confirmed equal by a recent remote operation.
- **Saved on this computer; sync will retry when online** — no local work was lost.
- **Reconnect to continue syncing** — complete the system login and retry.
- **Choose which note changes to keep** — open the guided conflict screen.
- **The sync location has not been checked** — choose **Sync Now** to check and synchronize. Markion never claims that everything is synchronized from stale or local-only information.

**Sync Now** saves eligible named notes, creates a local version when content changed, checks the sync location, safely combines compatible updates, and uploads the exact prepared version. Network work does not block editing. When Markion must update files, it briefly coordinates with saving and refuses stale or unsafe inputs.

Synchronization mirrors supported edits and deletions to the sync location; it is not an immutable archive. Use **Version history** to inspect or recover earlier content retained in the underlying history.

Closing the center does not cancel an operation. Use **Cancel operation** when available. Quitting while a synchronization request is active waits for it to reconcile before the normal unsaved-document flow.

## Version history

Open **Version history** from the File menu, a file-tree context menu, or a tab context menu to inspect versions of that file. The Backup and Sync center also exposes bounded recent history and outgoing history.

For a regular UTF-8 Markdown or supported text file, **Restore in editor** loads the old source as one unsaved, undoable edit. It does not immediately overwrite the file or move the repository to an older state. Replacing a dirty editor requires confirmation, and Markion cancels the restore if the path, tab, or disk version changed while it was being prepared. Binary, linked, deleted, invalid-UTF-8, and oversized historical content cannot replace an editor buffer. **Save a Copy** remains available when the bytes can be exported safely and never replaces an existing file.

## When two computers changed the same note

Markion opens a focused note-version chooser instead of putting conflict work in the sidebar. For text, compare **This Computer** and **Synced**, choose individual sections or a whole **Combined version**, and edit that result in a normal editor tab. Save the draft, choose **Use this version**, then choose **Finish and Continue Sync**. Image and other binary conflicts offer a side choice and **Keep Both** at a safe unused path; delete/change conflicts also offer deletion.

Markion checks that the same files and synchronization attempt are still active before applying a choice. You can undo the current synchronization attempt with **Abort**; if another Git tool changed the repository meanwhile, Markion leaves everything untouched for manual inspection. Drafts and recovery copies survive restart and disconnection.

## Attachments

Local Markdown image references are checked along with their note. Supported images are included in the same version. Missing, ignored, out-of-folder, unsupported, or excluded resources pause synchronization and identify the authored link. Repair the link or use **Organize Images**, then retry. A particular omitted attachment can be acknowledged, but editing its URL invalidates that acknowledgment. Markion preserves authored relative links and never downloads remote images during synchronization.

## Preferences and background checks

General Preferences contains a **Backup and Sync** section. **Background checks for workspaces** is the global default. Each connected workspace has a separate **Background checks for this workspace** option in Connection settings.

Background checking only looks for updates. It never saves or uploads notes, creates versions, combines updates, applies file changes, or opens repeated login prompts. It is off by default, checks only the active workspace, allows one request at a time, and backs off when the service is unavailable. It stays quiet when nothing changed. If the configured credential method cannot be guaranteed noninteractive, Markion disables background checks for that workspace.

Git executable selection and technical repository policy fields are under **Advanced Git settings**.

## Advanced Git details

The normal Backup and Sync center deliberately hides Git terminology. Users who work with Git can open **Advanced Git details**, and the Backup and Sync menu contains an **Advanced Git Tools** submenu with **Commit Locally**, **Check Remote**, **Pull Updates**, and **Push Commits**.

These layers are distinct:

1. An editor buffer may contain unsaved text.
2. The worktree contains saved files.
3. The local branch contains snapshot commits.
4. The upstream branch contains the last confirmed pushed commit.

**Sync Now** covers all four layers for policy-approved named notes. **Commit Locally** stops at local history, **Check Remote** only fetches, **Pull Updates** integrates a fetched commit, and **Push Commits** uploads one fixed local commit. Fetch and push preserve staged and unstaged edits. Pull requires saved buffers and a clean repository.

The default policy permits changes to tracked files and new Markdown, supported text, and supported images. Hidden paths, symlinks, nested repositories, unsupported binaries, foreign staging, disk conflicts, unexpected paths, active Git operations, unsafe incoming paths, or a changed upstream target stop one-click synchronization for review. Connection settings can narrow semicolon-separated tracked roots and new-file roots (`.` means repository root), change the automatic commit message (`{count}` is expanded), and supply a repository-local author name and email. Scope changes affect new snapshots; a push still transfers the branch's complete outgoing history.

Markion never automatically rebases, stashes, force-pushes, resets, cleans, pushes tags, mirrors, deletes branches, bypasses hooks/signing, replaces an existing remote, or absorbs unrelated staging. It pushes one exact object to one full branch reference and permits only one bounded retry after a non-fast-forward rejection.

Before worktree or index mutation, Markion writes a bounded operation journal and required preimages outside the notes repository. On restart it restores owned guards and can verify uncertain upload delivery without creating a duplicate content snapshot. If identities or paths changed externally, it keeps recovery copies and requires review rather than resetting anything.

Repository policy is stored in versioned `git-sync.toml`; operation journals, drafts, and preimages use the separate app-local Git sync data directory. Neither is synchronized through the notes repository. Resetting Preferences or removing a recent-workspace entry does not remove them. Disconnecting disables the automatic binding while preserving the repository, remote, commits, credentials, and unresolved recovery data.

For background checks, local remotes, Git Credential Manager, the in-memory `cache` helper, Windows `wincred`, and macOS `osxkeychain` are eligible on their supported platforms. Arbitrary helper commands, plaintext `store`, `libsecret`, and SSH background checks are rejected because Markion cannot guarantee that they remain noninteractive. Explicit foreground synchronization can still use supported HTTPS or SSH system authentication.
