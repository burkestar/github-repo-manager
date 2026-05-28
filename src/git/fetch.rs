use std::path::Path;

use crate::error::Result;

pub fn fetch_repo(repo_path: &Path, token: &str) -> Result<()> {
    let repo = git2::Repository::open(repo_path)?;
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

    remote.fetch(&[] as &[&str], Some(&mut fetch_opts), None)?;
    Ok(())
}
