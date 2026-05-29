use serde::Serialize;
use tracing::{info, warn};

use crate::error::{AppError, Result};
use crate::state::RepoInfo;

#[derive(Serialize)]
struct ListReposQuery {
    #[serde(rename = "type")]
    repo_type: &'static str,
    per_page: u8,
    page: u32,
}

/// Query for the authenticated-user repos endpoint (`/user/repos`).
/// `visibility=all` so private repos are included; `affiliation=owner`
/// restricts to repos the user owns (matching the org-style listing).
#[derive(Serialize)]
struct ListAuthedReposQuery {
    visibility: &'static str,
    affiliation: &'static str,
    per_page: u8,
    page: u32,
}

/// Which GitHub endpoint to page through for a given configured "org".
#[derive(Clone, Copy, PartialEq)]
enum Route {
    /// `/orgs/{org}/repos` — the org is a real organization.
    Org,
    /// `/user/repos` — the org is the authenticated user's own account
    /// (includes their private repos).
    AuthedUser,
    /// `/users/{org}/repos` — some other user account (public repos only).
    OtherUser,
}

pub async fn fetch_org_repos(org: &str, token: &str) -> Result<Vec<RepoInfo>> {
    let crab = octocrab::OctocrabBuilder::new()
        .personal_token(token.to_string())
        .build()?;

    // If the configured "org" is actually the authenticated user's own
    // account, use /user/repos so we get their private repos too. The
    // public /users/{username}/repos route never returns private repos,
    // even for the authenticated user.
    let authed_login: Option<String> = crab
        .current()
        .user()
        .await
        .map(|u| u.login)
        .map_err(|e| {
            warn!(
                "could not fetch authenticated user: {}",
                github_error_message(&e)
            )
        })
        .ok();
    let is_self = authed_login
        .as_deref()
        .is_some_and(|login| login.eq_ignore_ascii_case(org));

    let mut all_repos = Vec::new();
    let mut page = 1u32;
    let mut route = Route::Org;

    loop {
        let result: std::result::Result<Vec<octocrab::models::Repository>, octocrab::Error> =
            match route {
                Route::AuthedUser => {
                    let query = ListAuthedReposQuery {
                        visibility: "all",
                        affiliation: "owner",
                        per_page: 100,
                        page,
                    };
                    crab.get("/user/repos", Some(&query)).await
                }
                Route::Org | Route::OtherUser => {
                    let query = ListReposQuery {
                        repo_type: "all",
                        per_page: 100,
                        page,
                    };
                    let path = if route == Route::Org {
                        format!("/orgs/{org}/repos")
                    } else {
                        format!("/users/{org}/repos")
                    };
                    crab.get(&path, Some(&query)).await
                }
            };

        let items = match result {
            Ok(items) => items,
            Err(e) => {
                let gh_message = github_error_message(&e);
                // The org listing 404s when "org" is really a user account.
                // Retry against the appropriate user endpoint.
                if route == Route::Org && gh_message == "Not Found" {
                    route = if is_self {
                        Route::AuthedUser
                    } else {
                        Route::OtherUser
                    };
                    info!(
                        "'{}' is not an org, retrying as {} account",
                        org,
                        if is_self {
                            "authenticated user"
                        } else {
                            "user"
                        }
                    );
                    continue;
                }
                warn!("org={org} page={page} error: {gh_message}");
                return Err(AppError::Config(format!("GitHub API error: {gh_message}")));
            }
        };

        let count = items.len();
        let via = match route {
            Route::Org => "org",
            Route::AuthedUser => "authed-user",
            Route::OtherUser => "user",
        };
        info!("org={org} page={page} count={count} via={via}");

        for repo in items {
            all_repos.push(RepoInfo {
                full_name: repo
                    .full_name
                    .unwrap_or_else(|| format!("{}/{}", org, repo.name)),
                name: repo.name,
                description: repo.description,
                default_branch: repo.default_branch.unwrap_or_else(|| "main".to_string()),
                clone_url: repo.clone_url.map(|u| u.to_string()).unwrap_or_default(),
                stars: repo.stargazers_count.unwrap_or(0),
                archived: repo.archived.unwrap_or(false),
                updated_at: repo.updated_at.unwrap_or_else(chrono::Utc::now),
            });
        }

        if count < 100 {
            break;
        }
        page += 1;
    }

    all_repos.sort_by_key(|a| a.name.to_lowercase());

    info!("Fetched {} repos total for org '{}'", all_repos.len(), org);
    if all_repos.len() < 100 {
        warn!(
            "Only {} repos for '{}' — verify token has 'repo' scope and is SSO-authorized if org uses SAML",
            all_repos.len(),
            org
        );
    }

    Ok(all_repos)
}

fn github_error_message(e: &octocrab::Error) -> String {
    match e {
        octocrab::Error::GitHub { source, .. } => source.message.clone(),
        _ => format!("{e:?}"),
    }
}
