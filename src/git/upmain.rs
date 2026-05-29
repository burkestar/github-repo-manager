use std::path::Path;

use crate::error::{AppError, Result};

pub fn upmain(repo_path: &Path, token: &str, default_branch: &str) -> Result<String> {
    let mut repo = git2::Repository::open(repo_path)?;

    // Stash if there are any staged or unstaged modifications to tracked files.
    let dirty = repo
        .statuses(Some(
            git2::StatusOptions::new()
                .include_untracked(false)
                .include_ignored(false),
        ))?
        .iter()
        .any(|e| e.status() != git2::Status::CURRENT);

    let stashed = if dirty {
        let sig = repo.signature()?;
        repo.stash_save(&sig, "upmain auto-stash", None)?;
        true
    } else {
        false
    };

    // Switch to the default branch.
    let branch_ref = format!("refs/heads/{default_branch}");
    let obj = repo
        .revparse_single(&branch_ref)
        .map_err(|_| AppError::Config(format!("branch '{default_branch}' not found locally")))?;
    repo.checkout_tree(&obj, Some(git2::build::CheckoutBuilder::default().safe()))?;
    repo.set_head(&branch_ref)?;

    // Fetch origin.
    let mut remote = repo.find_remote("origin")?;
    let mut callbacks = git2::RemoteCallbacks::new();
    let token_owned = token.to_string();
    callbacks.credentials(move |_url, username, allowed| {
        if allowed.contains(git2::CredentialType::SSH_KEY) {
            git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
        } else {
            git2::Cred::userpass_plaintext("oauth2", &token_owned)
        }
    });
    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    remote.fetch(&[default_branch], Some(&mut fetch_opts), None)?;

    // Fast-forward only.
    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;

    let stash_note = if stashed {
        ", stashed local changes"
    } else {
        ""
    };

    if analysis.is_up_to_date() {
        return Ok(format!(
            "Already up to date on '{default_branch}'{stash_note}"
        ));
    }

    if !analysis.is_fast_forward() {
        return Err(AppError::Config(format!(
            "'{default_branch}' has diverged from origin — cannot fast-forward"
        )));
    }

    let mut reference = repo.find_reference(&branch_ref)?;
    reference.set_target(fetch_commit.id(), "upmain fast-forward")?;
    repo.set_head(&branch_ref)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    Ok(format!("Updated '{default_branch}'{stash_note}"))
}
