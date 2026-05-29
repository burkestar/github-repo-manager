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
    /// Terminal app to open with [t]. macOS: app name passed to `open -a`.
    /// Linux: binary name, invoked with `--working-directory`.
    /// Defaults: "Terminal" on macOS, "x-terminal-emulator" on Linux.
    pub terminal: Option<String>,
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
    vec![]
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        if !config_path.exists() {
            Self::write_default()?;
            return Err(AppError::Config(format!(
                "Created a default config file at {}.\n\
                You only need to create a GitHub access token and add it to that file:\n  \
                https://github.com/settings/tokens\n\
                Then set `github_token` and run this app again.",
                config_path.display(),
            )));
        }
        let contents = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        if config.github_token.is_empty() {
            return Err(AppError::Config(format!(
                "github_token must not be empty.\n\
                Create a GitHub access token and set it in {}:\n  \
                https://github.com/settings/tokens",
                config_path.display(),
            )));
        }
        Ok(config)
    }

    /// Create the config directory and write a default `config.toml`.
    /// Called on first run when no config file exists yet.
    fn write_default() -> Result<()> {
        let config_dir = Self::config_dir();
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(Self::config_path(), Self::default_contents())?;
        Ok(())
    }

    fn default_contents() -> String {
        format!(
            "# github-repo-manager configuration\n\
            # Create a GitHub access token and paste it below: https://github.com/settings/tokens\n\
            github_token = \"\"\n\
            workspace_root = \"{}\"\n\
            layout = \"nested\"\n\
            cron_schedule = \"{}\"\n\
            organizations = []\n",
            default_workspace_root().display(),
            default_cron_schedule(),
        )
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
