use octocrab::params;
use octocrab::OctocrabBuilder;

use crate::error::Result;
use crate::state::RepoInfo;

pub async fn fetch_org_repos(org: &str, token: &str) -> Result<Vec<RepoInfo>> {
    let crab = OctocrabBuilder::new()
        .personal_token(token.to_string())
        .build()?;

    let mut all_repos = Vec::new();
    let mut page: u32 = 1;

    loop {
        let response = crab
            .orgs(org)
            .list_repos()
            .repo_type(params::repos::Type::All)
            .per_page(100)
            .page(page)
            .send()
            .await?;

        let is_last = response.next.is_none();

        for repo in response.items {
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

        if is_last {
            break;
        }
        page += 1;
    }

    all_repos.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(all_repos)
}
