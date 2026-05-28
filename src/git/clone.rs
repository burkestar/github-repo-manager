use std::path::Path;

use tokio::sync::watch;

use crate::error::Result;

pub fn clone_repo(
    clone_url: &str,
    target_path: &Path,
    token: &str,
    progress_tx: watch::Sender<f64>,
) -> Result<()> {
    let mut callbacks = git2::RemoteCallbacks::new();

    let token_owned = token.to_string();
    callbacks.credentials(move |_url, username, allowed| {
        if allowed.contains(git2::CredentialType::SSH_KEY) {
            git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
        } else {
            git2::Cred::userpass_plaintext("oauth2", &token_owned)
        }
    });

    callbacks.transfer_progress(move |stats| {
        if stats.total_objects() > 0 {
            let pct = stats.received_objects() as f64 / stats.total_objects() as f64;
            let _ = progress_tx.send(pct);
        }
        true
    });

    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_opts);
    builder.clone(clone_url, target_path)?;

    Ok(())
}
