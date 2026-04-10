# gitstare

A git-aware multi-repo dashboard for your terminal. One command, every repo on your machine, instant status at a glance.

## What it is

`gitstare` is a single Rust binary. You run `gitstare` and it scans your filesystem (or configured directories), finds every git repo, and renders a beautiful ratatui TUI showing you the state of all of them at once. No config file needed -- it just works.

## The problem

Every developer with more than a handful of repos has the same experience: you forget what branch you left something on, you don't remember if you pushed, you have uncommitted changes rotting in three different projects, and you have stale branches from six months ago you never cleaned up. The only way to check is `cd`-ing into each repo one by one and running `git status`. Nobody does this, so stuff drifts out of sync.

## Core views (MVP)

### Overview table

The main screen. A sortable, filterable table where each row is a repo:

| Repo | Branch | Status | Ahead/Behind | Last Commit | Stale Branches |
|------|--------|--------|--------------|-------------|----------------|
| gitstare | main | clean | +0/-0 | 2h ago | 0 |
| website | feat/redesign | 3M 1U | +4/-0 | 3d ago | 2 |
| dotfiles | main | clean | +1/-0 | 2w ago | 0 |
| api-server | hotfix/db | 1M 0U | +0/-2 | 5h ago | 7 |

Columns:
- **Repo**: directory name (full path on hover/expand)
- **Branch**: current HEAD branch
- **Status**: clean, or NM NU (N modified, N untracked) -- color-coded green/yellow/red
- **Ahead/Behind**: vs upstream tracking branch. red if behind, yellow if ahead, green if synced
- **Last Commit**: relative time since last commit on current branch
- **Stale Branches**: count of local branches with no commits in 30+ days (configurable)

Color coding is the whole game here. A clean, synced repo row is green. A dirty, behind repo row is red. You open gitstare, see a wall of green with two red rows, and you know exactly where to go.

### Repo detail view

Press enter on a row to drill into a single repo. Shows:
- Full branch list with last commit date on each
- Recent commit log (last 10-20)
- Staged/unstaged diff stat (files changed, insertions, deletions)
- Remote URLs

### Actions (post-MVP but worth designing for)

From the detail view:
- `f` to fetch
- `p` to pull
- `P` to push
- `d` to delete a stale branch
- `o` to open the repo in your $EDITOR

These turn gitstare from a dashboard into a lightweight multi-repo manager. But MVP ships without actions -- read-only is fine.

## Tech stack

- **Rust** (obviously)
- **ratatui** for TUI rendering
- **git2** crate (libgit2 bindings) for all git operations -- no shelling out to `git`
- **SQLite** (via rusqlite) for caching scan results so subsequent launches are instant
- **walkdir** for filesystem scanning
- **crossterm** as the ratatui backend
- **clap** for CLI args
- **directories** crate for finding default scan paths (home dir, XDG dirs)
- **tokio** or **rayon** for parallel repo scanning (scanning 50+ repos serially would be slow)

## Config (optional)

gitstare works with zero config. By default it scans `~/` recursively for `.git` directories (with sane depth limits and ignoring node_modules, target, .cache, etc).

Optional `~/.config/gitstare/config.toml`:

```toml
# Directories to scan (overrides default home scan)
scan_paths = ["~/code", "~/work", "~/personal"]

# Max scan depth
max_depth = 4

# Directories to always ignore
ignore = ["node_modules", "target", ".cache", "vendor", ".cargo"]

# What counts as "stale" (days since last commit on branch)
stale_threshold = 30
```

## Data flow

1. On first run: walk configured directories, find all `.git` repos, read git state via git2, populate SQLite cache, render TUI
2. On subsequent runs: load from cache, render immediately, then rescan in background and update the table live
3. Within TUI: manual refresh with `r`, auto-refresh configurable

## What makes it star-worthy

**The screenshot.** A terminal full of color-coded repos with branch names, commit ages, and dirty/clean status is immediately compelling. Anyone who sees it thinks "I need this."

**Zero friction.** `cargo install gitstare` then `gitstare`. No setup, no config, no account, no cloud. It finds your repos and shows you the state.

**Speed story.** Rust + parallel scanning + SQLite caching. "Scans 100 repos in under a second." That line alone gets upvotes.

**Universal problem.** This isn't niche. Every developer with more than five repos has this exact pain point. The market is literally every developer.

## Ship plan

### v0.1 -- the screenshot release
- Filesystem scanning with configurable paths
- Overview table with all columns
- Color coding
- Sorting and filtering (by name, by status, by last commit)
- Repo detail view (read-only)
- SQLite caching
- Works on macOS and Linux (Windows is stretch)

### v0.2 -- the "actually useful daily" release
- Actions (fetch, pull, push, delete branch) from within TUI
- Keyboard-driven repo open ($EDITOR / $SHELL)
- Watch mode (auto-refresh on filesystem changes via notify crate)
- Bookmarked/pinned repos

### v0.3 -- the integrations release
- GitHub/GitLab PR status per repo (via API, opt-in with token)
- CI status badges inline
- Issue count per repo
- `gitstare report` command that outputs a markdown summary (for standups, for pasting into Slack)

## Name

`gitstare` -- because you're staring at all your repos at once. Cheeky, memorable, not taken on crates.io.

## Viral angle

LinkedIn post: "I had 40+ repos and no idea which ones had uncommitted changes. So I built gitstare -- a terminal dashboard that shows the state of every repo on your machine at a glance. Single binary, zero config, written in Rust."

Attach screenshot. That's the whole post. The screenshot does the selling.

HN post: "Show HN: gitstare -- a TUI dashboard for all your git repos" with a link to the GitHub repo that has a great README with an animated GIF (use vhs or asciinema to record).
