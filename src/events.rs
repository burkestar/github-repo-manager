use std::path::PathBuf;

use crate::state::RepoInfo;

#[derive(Debug)]
pub enum AppEvent {
    ReposLoaded { org: String, repos: Vec<RepoInfo> },
    ReposFailed { org: String, error: String },
    CloneProgress { repo: String, progress: f64 },
    CloneCompleted { repo: String, path: PathBuf },
    CloneFailed { repo: String, error: String },
    FetchCompleted { repo: String },
    FetchFailed { repo: String, error: String },
    BatchFetchStarted,
    BatchFetchCompleted { fetched: usize, failed: usize },
}
