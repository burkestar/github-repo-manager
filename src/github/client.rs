use serde::Serialize;
use tracing::{info, warn};

use crate::error::Result;
use crate::state::RepoInfo;

#[derive(Serialize)]
struct ListReposQuery {
    #[serde(rename = "type")]
    repo_type: &'static str,
    per_page: u8,
    page: u32,
}

pub async fn fetch_org_repos(org: &str, token: &str) -> Result<Vec<RepoInfo>> {
    let crab = octocrab::OctocrabBuilder::new()
        .personal_token(token.to_string())
        .build()?;

    let mut all_repos = Vec::new();
    let mut page = 1u32;

    loop {
        let query = ListReposQuery {
            repo_type: "all",
            per_page: 100,
            page,
        };
        let items: Vec<octocrab::models::Repository> = crab
            .get(format!("/orgs/{org}/repos"), Some(&query))
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("SAML")
                    || msg.contains("enforcement")
                    || msg.contains("GitHub")
                {
                    // Log the raw error so we can diagnose the exact message
                    warn!("org={org} page={page} raw error: {e:?}");
                }
                e
            })?;

        let count = items.len();
        info!("org={} page={} count={}", org, page, count);

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

    all_repos.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

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
