use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::{debug, warn};

use crate::config::WorkspaceLayout;
use crate::state::CheckoutInfo;

pub fn scan_workspace(root: &Path, layout: &WorkspaceLayout) -> HashMap<String, CheckoutInfo> {
    let mut result = HashMap::new();

    if !root.exists() {
        return result;
    }

    let max_depth = match layout {
        WorkspaceLayout::Flat => 1,
        WorkspaceLayout::Nested => 2,
    };

    scan_dir(root, 0, max_depth, &mut result);
    result
}

fn scan_dir(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    result: &mut HashMap<String, CheckoutInfo>,
) {
    let git_dir = dir.join(".git");
    if git_dir.exists() {
        if let Some(info) = read_checkout_info(dir) {
            result.insert(info.0, info.1);
        }
        return;
    }

    if depth >= max_depth {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to read directory {}: {}", dir.display(), e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, depth + 1, max_depth, result);
        }
    }
}

fn read_checkout_info(repo_path: &Path) -> Option<(String, CheckoutInfo)> {
    let repo = git2::Repository::open(repo_path)
        .map_err(|e| {
            debug!("Could not open repo at {}: {}", repo_path.display(), e);
            e
        })
        .ok()?;

    let full_name = infer_full_name(&repo, repo_path)?;

    let head = repo.head().ok()?;
    let current_branch = head.shorthand().filter(|s| *s != "HEAD").map(String::from);

    let (ahead, behind) = compute_ahead_behind(&repo, &head);

    Some((
        full_name,
        CheckoutInfo {
            local_path: repo_path.to_path_buf(),
            current_branch,
            ahead,
            behind,
        },
    ))
}

fn infer_full_name(repo: &git2::Repository, fallback_path: &Path) -> Option<String> {
    if let Ok(remote) = repo.find_remote("origin") {
        if let Some(url) = remote.url() {
            if let Some(name) = extract_full_name_from_url(url) {
                return Some(name);
            }
        }
    }
    fallback_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| format!("unknown/{n}"))
}

fn extract_full_name_from_url(url: &str) -> Option<String> {
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    if url.contains("github.com") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            let org = parts[parts.len() - 2];
            let repo = parts[parts.len() - 1];
            return Some(format!("{org}/{repo}"));
        }
    }
    None
}

fn compute_ahead_behind(repo: &git2::Repository, head: &git2::Reference) -> (u32, u32) {
    let local_oid = match head.target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    let branch_name = match head.shorthand() {
        Some(b) => b,
        None => return (0, 0),
    };

    let upstream_ref = format!("refs/remotes/origin/{branch_name}");
    let upstream_oid = match repo.refname_to_id(&upstream_ref) {
        Ok(oid) => oid,
        Err(_) => return (0, 0),
    };

    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((ahead, behind)) => (ahead as u32, behind as u32),
        Err(_) => (0, 0),
    }
}

pub fn repo_target_path(
    workspace_root: &Path,
    layout: &WorkspaceLayout,
    full_name: &str,
) -> PathBuf {
    match layout {
        WorkspaceLayout::Flat => {
            let repo_name = full_name.split('/').next_back().unwrap_or(full_name);
            workspace_root.join(repo_name)
        }
        WorkspaceLayout::Nested => workspace_root.join(full_name),
    }
}
