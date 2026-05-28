use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github_token: String,
    #[serde(default = "default_workspace_root")]
    pub workspace_root: PathBuf,
    #[serde(default)]
    pub layout: WorkspaceLayout,
    #[serde(default = "default_cron_schedule")]
    pub cron_schedule: String,
    #[serde(default = "default_organizations")]
    pub organizations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceLayout {
    Flat,
    #[default]
    Nested,
}

fn default_workspace_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("workspace")
}

fn default_cron_schedule() -> String {
    "0 0 5 * * *".to_string()
}

fn default_organizations() -> Vec<String> {
    vec![
        "datarobot".to_string(),
        "datarobot-community".to_string(),
        "datarobot-oss".to_string(),
    ]
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        if !config_path.exists() {
            return Err(AppError::Config(format!(
                "Config file not found at {}.\n\
                Create it with:\n\
                  github_token = \"ghp_your_token_here\"\n\
                  workspace_root = \"{}\"\n\
                  layout = \"nested\"\n\
                  cron_schedule = \"0 0 5 * * *\"\n\
                  organizations = [\"datarobot\", \"datarobot-community\", \"datarobot-oss\"]",
                config_path.display(),
                default_workspace_root().display(),
            )));
        }
        let contents = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        if config.github_token.is_empty() {
            return Err(AppError::Config(
                "github_token must not be empty".to_string(),
            ));
        }
        Ok(config)
    }

    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".config")
            .join("github-repo-manager")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }
}
