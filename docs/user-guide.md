# User Guide

How to install, configure, and use github-repo-manager.

## 1. Install

```bash
# From crates.io (recommended) — requires Rust; installs to ~/.cargo/bin
cargo install github-repo-manager

# …or the latest from git
cargo install --git https://github.com/burkestar/github-repo-manager
```

Prebuilt binaries for macOS, Linux, and Windows are attached to each [GitHub Release](https://github.com/burkestar/github-repo-manager/releases) if you'd rather not compile from source.

## 2. Create a GitHub access token

Create a **classic** token with the **`repo`** scope at **github.com/settings/tokens**, then run `github-repo-manager` once: it writes `~/.config/github-repo-manager/config.toml` and exits. Paste your token into `github_token`, list your `organizations`, and run it again.

That's it. The rest of this guide is reference material.

## Configuration

`~/.config/github-repo-manager/config.toml`:

```toml
# Required: GitHub personal access token
github_token = "ghp_your_token_here"

# Root directory where repos are cloned (default: ~/workspace)
workspace_root = "/Users/you/workspace"

# Directory layout when cloning:
#   "nested" → ~/workspace/org-a/some-repo  (default)
#   "flat"   → ~/workspace/some-repo
layout = "nested"

# Cron schedule for automatic git fetch of all local repos
# Format: seconds minutes hours day-of-month month day-of-week
# Default: daily at 5:00 AM
cron_schedule = "0 0 5 * * *"

# GitHub organizations (and/or your own username) to browse
organizations = [
    "org-a",
    "org-b",
]
```

Logs are written to `~/.config/github-repo-manager/app.log` (never to the terminal).

## Token details

The token needs to list repositories (including private ones) for each configured organization.

- **Classic token (recommended):** select the **`repo`** scope — read/write access to code and metadata for all repos you have access to, including private ones.
- **Fine-grained token:** select **All repositories** (or specific repos) and enable the **Contents** (read) and **Metadata** (read) permissions. Some organizations disable fine-grained tokens by policy, in which case a classic token is required.

**SSO authorization (SAML-protected orgs):** if an organization enforces SAML SSO, API calls are rejected even with a valid token unless it has been SSO-authorized. Go to **github.com/settings/tokens**, click **Configure SSO** on your token, **Authorize** each org that requires it, then press `r` in the TUI to refresh.

## UI Overview

```
 github-repo-manager   [Tab] switch  [/] search  [Enter] clone  [f] fetch  [F] fetch all  [r] refresh  [q] quit
┌────────────────────┬──────────────────────────────────────────────────────────────────────────────┐
│ Organizations      │ [org-a] 42 repos                                                             │
│                    │ Search: [___________________]                                                │
│ ▶ org-a            │                                                                              │
│   org-b            │ ✓ some-repo                  (main) ↑2 ↓0                                    │
│                    │ ○ another-repo                                                               │
│                    │ ⊙ archived-repo [archived]                                                   │
│                    │ ○ yet-another-repo                                                           │
│                    │                                                                              │
 ~/workspace  │  Last fetch: 2026-05-28 05:00
```

**Repo indicators:**
- `✓` green — cloned locally (shows current branch and ahead/behind counts)
- `○` default — not yet cloned
- `⊙` yellow — archived on GitHub

## Key Bindings

### Navigation

| Key | Action |
|-----|--------|
| `Tab` | Switch focus between Organizations and Repositories panels |
| `h` / `l` | Move focus left (Orgs) / right (Repos) |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Switch to Repos panel (from Orgs); clone selected repo or show local path (from Repos) |

### Search

| Key | Action |
|-----|--------|
| `/` | Activate fuzzy search in the Repos panel |
| `↓` / `↑` | Navigate results while search is active |
| `Backspace` | Delete last character in search query |
| `Enter` | Commit search and act on selected repo |
| `Esc` | Clear search and return to normal mode |

### Repo Actions

| Key | Action |
|-----|--------|
| `f` | `git fetch` the selected repo |
| `F` | `git fetch` all locally cloned repos |
| `m` | Update selected repo: stash changes, checkout default branch, pull latest |
| `o` | Open selected repo on GitHub in the browser |
| `t` | Open a terminal at the selected repo's local path |
| `d` | Show the selected repo's description in a popup |

### Display

| Key | Action |
|-----|--------|
| `a` | Toggle visibility of archived repos |
| `s` | Cycle sort field: Name → Last Updated → Name |
| `S` | Toggle sort order: Ascending ↔ Descending |
| `r` | Refresh repo list from GitHub API (bypasses cache) |

### Dialogs & Popups

| Key | Action |
|-----|--------|
| `Esc` | Dismiss clone progress dialog (clone continues in background) |
| `Esc` / `Enter` | Dismiss failed-clone dialog |
| Any key | Dismiss error or info popups |

### Application

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Ctrl+C` | Quit |

## Background Fetch Scheduler

The app runs a background job on the configured cron schedule to `git fetch` all repos found in your workspace. The default schedule is daily at 5:00 AM.

To test with a more frequent interval, temporarily change your config:
```toml
cron_schedule = "*/15 * * * * *"  # every 15 seconds
```

The status bar shows the last fetch time and a `[fetching…]` indicator while a batch fetch is running.

## Caching

GitHub API responses are cached to disk at `~/.config/github-repo-manager/<org>.json` and expire after 1 hour. Press `r` to force a refresh from the API for the current organization.

The local workspace scan result is also cached at `~/.config/github-repo-manager/workspace_cache.json` and refreshed on startup and after every clone or fetch.
