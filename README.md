# github-repo-manager

A terminal UI for browsing and managing GitHub repositories across multiple organizations. Navigate repos, clone them into a local workspace, and keep everything up to date automatically.

## Features

- Browse repositories across multiple GitHub organizations
- Fuzzy search repos by name
- See which repos are already checked out locally (with branch and ahead/behind info)
- Clone repos into a configurable workspace (flat or nested directory structure)
- Manual `git fetch` for individual repos or all local repos at once
- Background scheduler that automatically fetches all local repos on a cron schedule
- GitHub API response caching (1 hour TTL) to avoid rate limits

## Prerequisites

- Rust (1.74+)
- `libgit2` — on macOS: `brew install libgit2`
- A GitHub personal access token (classic or fine-grained with repo read access)

## Installation

```bash
git clone <this-repo>
cd github-repo-manager
cargo build --release
# Binary will be at ./target/release/github-repo-manager
```

## Configuration

Create the config file before running:

```bash
mkdir -p ~/.config/github-repo-manager
```

`~/.config/github-repo-manager/config.toml`:

```toml
# Required: GitHub personal access token
github_token = "ghp_your_token_here"

# Root directory where repos are cloned (default: ~/workspace)
workspace_root = "/Users/you/workspace"

# Directory layout when cloning:
#   "nested" → ~/workspace/datarobot/some-repo  (default)
#   "flat"   → ~/workspace/some-repo
layout = "nested"

# Cron schedule for automatic git fetch of all local repos
# Format: seconds minutes hours day-of-month month day-of-week
# Default: daily at 5:00 AM
cron_schedule = "0 0 5 * * * *"

# GitHub organizations to browse
organizations = [
    "datarobot",
    "datarobot-community",
    "datarobot-oss",
]
```

## Running

```bash
cargo run
# or, after building:
./target/release/github-repo-manager
```

Logs are written to `~/.cache/github-repo-manager/app.log` (never to the terminal).

## UI Overview

```
 github-repo-manager   [Tab] switch  [/] search  [Enter] clone  [f] fetch  [F] fetch all  [r] refresh  [q] quit
┌────────────────────┬──────────────────────────────────────────────────────────────────────────────┐
│ Organizations      │ [datarobot] 42 repos                                                         │
│                    │ Search: [___________________]                                                 │
│ ▶ datarobot        │                                                                              │
│   datarobot-       │ ✓ some-repo                  (main) ↑2 ↓0                                   │
│   community        │ ○ another-repo                                                               │
│   datarobot-oss    │ ⊙ archived-repo [archived]                                                  │
│                    │ ○ yet-another-repo                                                           │
│                    │                                                                              │
 ~/workspace  │  Last fetch: 2026-05-28 05:00
```

**Repo indicators:**
- `✓` green — cloned locally (shows current branch and ahead/behind counts)
- `○` default — not yet cloned
- `⊙` yellow — archived on GitHub

## Key Bindings

| Key | Action |
|-----|--------|
| `Tab` | Switch focus between Organizations and Repositories panels |
| `h` / `l` | Move focus left (Orgs) / right (Repos) |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `/` | Activate fuzzy search in the Repos panel |
| `Esc` | Cancel search or close dialogs |
| `Enter` | Clone selected repo (if not local); show local path (if already cloned) |
| `f` | `git fetch` the selected repo |
| `F` | `git fetch` all locally cloned repos |
| `r` | Refresh repo list from GitHub API (bypasses cache) |
| `q` | Quit |
| `Ctrl+C` | Quit |

## Background Fetch Scheduler

The app runs a background job on the configured cron schedule to `git fetch` all repos found in your workspace. The default schedule is daily at 5:00 AM.

To test with a more frequent interval, temporarily change your config:
```toml
cron_schedule = "*/15 * * * * * *"  # every 15 seconds
```

The status bar shows the last fetch time and a `[fetching…]` indicator while a batch fetch is running.

## Caching

GitHub API responses are cached to disk at `~/.cache/github-repo-manager/<org>.json` and expire after 1 hour. Press `r` to force a refresh from the API for the current organization.
