# gitstare

A git-aware multi-repo dashboard for your terminal. One command, every repo on your machine, instant status at a glance.

```
          _ __       __
   ____ _(_) /______/ /_____ _________
  / __ `/ / __/ ___/ __/ __ `/ ___/ _ \
 / /_/ / / /_(__  ) /_/ /_/ / /  /  __/
 \__, /_/\__/____/\__/\__,_/_/   \___/
/____/
```

## The problem

Every developer with more than a handful of repos has the same experience: you forget what branch you left something on, you don't remember if you pushed, you have uncommitted changes rotting in three different projects, and stale branches from six months ago you never cleaned up. The only way to check is `cd`-ing into each one and running `git status`. Nobody does this, so stuff drifts.

**gitstare** fixes this. One command, and you see the state of every repo on your machine -- color-coded so problems jump out immediately.

## Features

- **Instant overview** -- sortable, filterable table showing every repo's branch, dirty/clean status, ahead/behind upstream, last commit time, and stale branch count
- **Color-coded rows** -- green = clean and synced, yellow = dirty or ahead, red = behind upstream. Open gitstare, see a wall of green with two red rows, and you know exactly where to go
- **Repo detail view** -- press Enter on any row to see full branch list, recent commit log, diff stats, and remote URLs
- **Parallel scanning** -- uses Rayon to scan repos in parallel. 100+ repos in under a second
- **SQLite caching** -- first scan populates a cache, subsequent launches are instant. Cache expires after 1 hour
- **Zero config** -- scans your home directory by default. No setup, no config file, no cloud account. Just run it
- **Optional config** -- power users can customize scan paths, depth, ignore patterns, and stale thresholds via TOML
- **Keyboard-driven** -- vim-style navigation (j/k), search (/), sort by any column (1-6), detail view (Enter)

## Install

```sh
cargo install gitstare
```

Or build from source:

```sh
git clone https://github.com/matteso1/gitstare.git
cd gitstare
cargo build --release
```

The binary will be at `target/release/gitstare`.

## Usage

```sh
# Scan your home directory (default)
gitstare

# Scan specific directories
gitstare -p ~/code -p ~/work

# Set scan depth (default: 4)
gitstare -d 6

# Force a fresh scan (skip cache)
gitstare --fresh
```

## Keyboard shortcuts

### Overview (main screen)

| Key | Action |
|-----|--------|
| `j` / `k` or arrow keys | Navigate up/down |
| `Enter` | Open repo detail view |
| `/` | Search/filter repos |
| `1` - `6` | Sort by column (press again to reverse) |
| `g` / `G` | Jump to top/bottom |
| `q` | Quit |

### Detail view

| Key | Action |
|-----|--------|
| `j` / `k` or arrow keys | Scroll commits |
| `Esc` / `Backspace` | Back to overview |
| `q` | Quit |

## Configuration

gitstare works out of the box with zero config. Optionally, create `~/.config/gitstare/config.toml`:

```toml
# Directories to scan (overrides default home directory scan)
scan_paths = ["~/code", "~/work", "~/personal"]

# Maximum directory depth to scan (default: 4)
max_depth = 4

# Directories to always skip
ignore = ["node_modules", "target", ".cache", "vendor", ".cargo"]

# Days since last commit before a branch is considered "stale" (default: 30)
stale_threshold = 30
```

## What the columns mean

| Column | Description |
|--------|-------------|
| **Repo** | Directory name of the repo |
| **Branch** | Current HEAD branch (or `detached@<hash>` if detached) |
| **Status** | `clean` or `NM NU` (N modified, N untracked files) |
| **+/-** | Commits ahead/behind upstream tracking branch |
| **Last Commit** | Relative time since last commit on current branch |
| **Stale** | Number of local branches with no commits in 30+ days |

## Color coding

- **Green** -- clean repo, synced with upstream
- **Yellow** -- dirty (uncommitted changes) or ahead of upstream
- **Red** -- behind upstream (you need to pull)
- **Peach** -- stale branch count > 0

## Tech stack

- **Rust** -- fast, single binary, no runtime dependencies
- **ratatui** + **crossterm** -- modern TUI rendering
- **git2** (libgit2 bindings) -- all git operations, no shelling out
- **rayon** -- parallel repo scanning
- **rusqlite** (bundled SQLite) -- scan result caching
- **clap** -- CLI argument parsing

## Testing

Run the integration test suite:

```sh
cargo test
```

To generate a test filesystem with repos covering every edge case (clean, dirty, detached HEAD, ahead/behind, merge conflicts, stale branches, empty repos, etc.):

```sh
bash tests/setup_test_repos.sh
cargo run -- -p ./test_repos --fresh
```

## Roadmap

### v0.2 -- actions
- Fetch, pull, push, delete branches from within the TUI
- Open repo in `$EDITOR` or `$SHELL`
- Watch mode with auto-refresh on filesystem changes
- Bookmarked/pinned repos

### v0.3 -- integrations
- GitHub/GitLab PR status per repo
- CI status badges inline
- `gitstare report` for markdown summary output

## License

MIT
