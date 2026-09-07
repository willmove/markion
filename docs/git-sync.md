# Git synchronization

Markion can synchronize a notes workspace through an ordinary Git repository. The everyday action is **Sync Now** in the File menu or status bar. It saves eligible named notes from that repository, creates a local snapshot commit when content changed, fetches the configured upstream branch, fast-forwards or merges ordinary divergent history, and pushes one exact commit to the configured branch.

## Prerequisites

- Install Git 2.39 or newer and make `git` available on `PATH`. An alternate executable can be set with `[git] executable` in `config.toml`.
- Use an ordinary, non-bare worktree on a named branch with one upstream remote branch.
- Configure the repository author with `git config user.name` and `git config user.email`. Login credentials are separate from commit authorship.
- For HTTPS, configure a secure platform credential helper such as Git Credential Manager, macOS Keychain, or libsecret. Markion does not enable Git's plaintext `credential-store` helper.
- For SSH, configure the user's SSH agent, keys, `~/.ssh/config`, and `known_hosts`. Markion does not disable host-key verification.

Linked worktrees, submodules, Git LFS, bare, sparse, partial, shallow, detached-HEAD, and nested-repository workspaces are detected but remain read-only for synchronization.

## Set up an existing repository

1. Clone the repository with Git, or initialize it and add a remote.
2. Check out the local branch that should hold the notes and configure its upstream. For example:

   ```sh
   git switch main
   git branch --set-upstream-to=origin/main
   ```

3. Open the repository folder in Markion.
4. Choose **Sync Now**. On first use, Markion opens the same review as **File → Set Up Git Sync** so you can confirm the canonical worktree, branch, current changes, and file policy. Enabling it continues that synchronization automatically. You can also run **Set Up Git Sync** separately when you only want to save the connection policy.

The initial policy permits changes to already tracked files and new Markdown, supported text, and supported image files in the repository. Hidden paths, symlinks, nested repositories, and other binaries stop one-click synchronization for review. A dedicated repository for notes is recommended.

For an empty remote, make the first commit locally and publish the named branch explicitly before connecting it for routine synchronization. Markion's core onboarding path supports empty remotes and records the upstream only after publication succeeds.

## What Sync Now changes

Markion distinguishes four states:

1. The editor buffer may contain unsaved text.
2. The worktree contains the saved files.
3. The local Git branch contains snapshot commits.
4. The remote branch contains the last confirmed pushed commit.

**Sync Now** covers all four states for policy-approved named notes. **Commit Locally** only reaches local history; **Check Remote** only fetches; **Pull Updates** integrates a fetched commit; and **Push Commits** only uploads a fixed local commit. The secondary contracts exist in the Git core and are kept separate so a network failure cannot be reported as a local-save failure.

Markion never performs an automatic rebase, stash, force push, reset, clean, tag push, mirror push, or branch deletion. Existing foreign staging, disk conflicts, unexpected paths, active Git operations, target drift, or unsafe incoming paths stop synchronization without absorbing those changes.

## Attachments

Local Markdown image references are checked against the repository and synchronization policy. Approved image files participate in the same commit as their note. Missing, ignored, out-of-repository, unsupported, or out-of-policy resources stop Sync Now with the affected authored URL. Repair the reference or use **Organize Images** to copy eligible local images into the note's asset folder, then retry. Markion preserves authored relative URLs and never downloads a remote URL as part of synchronization.

## Conflicts and recovery

Concurrent edits on two computers can produce a merge conflict. Markion reads the actual Git index stages and asks which side to keep for each conflicted path: **This Computer**, **Remote Version**, or deletion when that side deleted the file. The resolver verifies the operation, index, and conflicted worktree bytes before writing and staging a choice. Text that literally contains `<<<<<<<` is allowed; marker text is not used to decide whether a conflict is resolved.

Finish Merge preserves Git hooks and signing settings. Upload after manual resolution is explicit. Abort first verifies that the same operation still owns `HEAD`, `MERGE_HEAD`, and the journal. If external Git changes those identities, Markion leaves the repository and drafts untouched for manual inspection.

Before staging or worktree integration, Markion writes a versioned operation journal outside the repository. Required file preimages are bounded and saved before checkout or merge. On restart it restores an active Markion-owned conflict guard and exposes **Resolve Git Conflicts** again. The recovery core also classifies owned staging, completed commits, completed integration, and uncertain push delivery against the real repository so later recovery UI can handle those states without guessing. Unknown or user-created recovery files are never removed by operation cleanup.

An offline fetch keeps local commits. If a push response is lost, Markion fetches the target and verifies whether it contains the intended commit. It reports an uncertain result when delivery cannot be proven; retrying does not create a duplicate content snapshot commit.

## Optional remote checks

Background checking is disabled by default:

```toml
[git]
# executable = "C:/Program Files/Git/bin/git.exe"
background_check = false
```

When enabled for a repository and supported by a noninteractive credential helper, Markion checks only the active workspace, allows one request at a time, starts at a nominal five-minute interval, and backs off to thirty minutes. It stays quiet when unchanged. A background check never saves, commits, merges, pushes, changes an inactive workspace, or opens recurring login prompts.

Repository policy is stored in versioned `git-sync.toml`; operation journals, drafts, and preimages use the separate app-local Git sync data directory. Neither is synchronized through the notes repository. Resetting Preferences or evicting a recent workspace does not remove them. Disconnecting removes automatic binding while preserving the repository, remotes, commits, credentials, and unresolved recovery.
