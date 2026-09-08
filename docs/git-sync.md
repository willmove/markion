# Git synchronization

Markion can synchronize a notes workspace through an ordinary Git repository. The everyday action is **Sync Now** in the Repository menu or status bar. It saves eligible named notes from that repository, creates a local snapshot commit when content changed, fetches the configured upstream branch, fast-forwards or merges ordinary divergent history, and pushes one exact commit to the configured branch.

## Prerequisites

- Install Git 2.39 or newer and make `git` available on `PATH`. An alternate executable can be set with `[git] executable` in `config.toml`.
- Use an ordinary, non-bare worktree on a named branch with one upstream remote branch.
- Markion asks for a repository-local author name and email only when Git does not already have them. Login credentials are separate from commit authorship.
- For HTTPS, use a secure platform credential helper such as Git Credential Manager or macOS Keychain. An explicit setup or sync can open the helper's system login prompt and offers Retry after authentication. Markion does not enable Git's plaintext `credential-store` helper or persist passwords and tokens.
- For SSH, configure the user's SSH agent, keys, `~/.ssh/config`, and `known_hosts`. Markion does not disable host-key verification.

Linked worktrees, submodules, Git LFS, bare, sparse, partial, shallow, detached-HEAD, and nested-repository workspaces are detected but remain read-only for synchronization.

## Start synchronizing notes

Choose **Repository → Set Up Git Sync** or choose **Sync Now** in a folder that has not been connected yet. The quick start offers the three ordinary ways to begin:

- **Use this folder** connects an already cloned Git repository. Markion discovers its branch and remote and enables the default notes policy.
- **Clone notes repository** asks for an HTTPS or SSH repository address. The destination is filled from the repository name, and the advanced section lets you override the branch. Markion keeps the current workspace open until the clone is complete, then switches to the cloned notes folder.
- **Start syncing this folder** initializes the open folder, attaches `origin`, creates a first version from eligible notes and attachments, and publishes `main`. It stages only the reviewed notes files; hidden files, private keys and unrelated binaries stay out of the first version.

The normal path needs only the repository address. Open **Advanced** when you need a different branch or repository-local author identity. If an empty remote is cloned, create and save the first note, then choose **Sync Now** to finish the first publication. If login or publication fails, the local repository and first commit remain intact and Retry resumes the same step; the remote is never silently replaced.

The initial policy permits changes to already tracked files and new Markdown, supported text, and supported image files in the repository. Hidden paths, symlinks, nested repositories, and other binaries stop one-click synchronization for review. A dedicated repository for notes is recommended.

## What Sync Now changes

Markion distinguishes four states:

1. The editor buffer may contain unsaved text.
2. The worktree contains the saved files.
3. The local Git branch contains snapshot commits.
4. The remote branch contains the last confirmed pushed commit.

**Sync Now** covers all four states for policy-approved named notes. **Commit Locally** only reaches local history; **Check Remote** only fetches; **Pull Updates** integrates a fetched commit; and **Push Commits** only uploads a fixed local commit. These actions are available in the Sync sidebar and Repository menu. They can be assigned shortcuts in Preferences → Shortcuts. Fetch and Push preserve staged and unstaged edits; Pull requires saved buffers and a clean repository. Network phases permit continued editing; integration reacquires the repository write guard and refuses a stale snapshot.

Markion never performs an automatic rebase, stash, force push, reset, clean, tag push, mirror push, or branch deletion. Existing foreign staging, disk conflicts, unexpected paths, active Git operations, target drift, or unsafe incoming paths stop synchronization without absorbing those changes.

## Create a deliberate local version

Use **Sync → Changes → Create local version** when you want a named checkpoint without contacting the remote. Select the whole files that belong together, enter a commit message, and choose **Create version**. The selection starts with policy-approved changes; **Select all** and **Clear selection** make it easy to adjust the group. Markion saves only selected named buffers, stages only those literal repository paths, and leaves unselected saved or unsaved work for a later version.

The draft is tied to the reviewed HEAD and file contents. If a selected file changes, the branch advances, a conflict starts, or another tool has staged content, Markion stops and asks you to refresh rather than committing different bytes. A hook or signing failure keeps the message and safe selection for retry and retains the existing recovery journal. Creating a local version never fetches, pulls, pushes, amends, rebases, resets, stashes, switches branches, creates tags, or rewrites history.

## Attachments

Local Markdown image references are checked against the repository and synchronization policy. Approved image files participate in the same commit as their note. Missing, ignored, out-of-repository, unsupported, or out-of-policy resources stop Sync Now with the affected authored URL. Repair the reference or use **Organize Images** to copy eligible local images into the note's asset folder, then retry. The Sync sidebar links to repair, scope settings and image organization. A specific omitted reference can be acknowledged; changing its URL invalidates that acknowledgment. Markion preserves authored relative URLs and never downloads a remote URL as part of synchronization.

## Conflicts and recovery

Concurrent edits on two computers can produce a merge conflict. The Sync sidebar shows the base, local and remote source for each actual index conflict. For text, choose individual hunks or a whole source, open a separate result draft in the editor, save the draft, and explicitly mark it resolved. Image/binary conflicts offer side selection and Keep Both at a validated unused path; delete/modify conflicts also offer deletion. The resolver verifies the operation, index, and conflicted worktree bytes before writing and staging a choice. Text that literally contains `<<<<<<<` is allowed; marker text is not used to decide whether a conflict is resolved.

Finish Merge preserves Git hooks and signing settings. Upload after manual resolution is explicit. Abort first verifies that the same operation still owns `HEAD`, `MERGE_HEAD`, and the journal. If external Git changes those identities, Markion leaves the repository and drafts untouched for manual inspection.

Before staging or worktree integration, Markion writes a versioned operation journal outside the repository. Required file preimages are bounded and saved before checkout or merge. On restart it restores an active Markion-owned conflict guard and exposes **Resolve Git Conflicts** again. The sidebar also exposes ownership-checked unstaging (retaining files), adoption of completed local operations, and remote verification of uncertain push delivery. Recovery discovers journals even after disconnect. Ambiguous checkout/external changes remain guarded with access to recovery copies and external inspection; they are not reset automatically. Unknown or user-created recovery files are never removed by operation cleanup.

An offline fetch keeps local commits. If a push response is lost, Markion fetches the target and verifies whether it contains the intended commit. It reports an uncertain result when delivery cannot be proven; retrying does not create a duplicate content snapshot commit.

## Optional remote checks

Background checking is disabled by default:

```toml
[git]
# executable = "C:/Program Files/Git/bin/git.exe"
background_check = false
```

The switch is available both globally under General Preferences and per repository under Sync settings. Local remotes, Git Credential Manager, the in-memory `cache` helper, Windows `wincred`, and macOS `osxkeychain` are eligible on their supported platforms. Arbitrary helper commands, `store`, `libsecret`, and SSH background checks remain disabled because Markion cannot guarantee that they will stay noninteractive. Background requests disable terminal, GCM and askpass prompts. A missing credential pauses further checks instead of repeatedly opening login UI. When enabled for an eligible repository, Markion checks only the active workspace, allows one request at a time, starts at a nominal five-minute interval, and backs off to thirty minutes. It stays quiet when unchanged. A background check never saves, commits, merges, pushes, changes an inactive workspace, or opens recurring login prompts.

Repository policy is stored in versioned `git-sync.toml`; operation journals, drafts, and preimages use the separate app-local Git sync data directory. Neither is synchronized through the notes repository. Resetting Preferences or evicting a recent workspace does not remove them. Disconnecting removes automatic binding while preserving the repository, remotes, commits, credentials, and unresolved recovery.


## Inspecting and configuring synchronization

Open **Sync** from the sidebar or the workspace header. The panel separates unsaved buffers, changed files and incoming/outgoing history, with the time of the last successful remote check. The file list includes changes excluded from the normal document tree and uses 50-item pages. It shows both index and worktree status; inspection lets you switch between staged and unstaged content.

**Recent history** shows bounded repository history. **File history** follows the active repository file through ordinary renames, and **Outgoing commits** shows local history awaiting upload. Entries include the commit subject, author, authored time, full object-backed identity, parent count, and changed-path drilldown. **Compare with current** shows a bounded diff from the selected commit to the working file with external diff drivers and text conversion disabled.

For a regular UTF-8 Markdown or supported text file, **Restore in editor** loads the historical source as one unsaved, undoable editor edit. It does not write the file, stage content, move HEAD, or create a commit. A dirty target requires explicit replacement confirmation, and a changed path, tab, or disk identity cancels a stale restore. Binary, symlink, deleted, invalid-UTF-8, and oversized historical content cannot replace an editor buffer. **Save a Copy** remains available when the historical bytes can be exported safely; it preserves extensions, refuses to replace an existing file, and respects repository write guards.

**Connection settings** supports semicolon-separated tracked roots and new-file roots (`.` means the repository root), the automatic commit message (`{count}` is expanded), optional local author name/email, and the repository background-check preference. File scope controls the new snapshot; pushes still transfer the branch's complete outgoing history.

Use Cancel to request cancellation. Closing or quitting during a synchronization request waits for the operation to return and reconcile before the usual unsaved-document flow. If a local mutation cannot be reconciled, its journal and path guards remain available for recovery.
