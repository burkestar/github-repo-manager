use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("GitHub API error: {0}")]
    GitHub(#[from] octocrab::Error),
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
