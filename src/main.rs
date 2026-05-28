#![allow(clippy::result_large_err)]

mod app;
mod config;
mod error;
mod events;
mod git;
mod github;
mod scheduler;
mod state;
mod ui;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

use crate::app::App;
use crate::config::Config;
use crate::error::Result;

fn init_tracing() -> WorkerGuard {
    let log_dir = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("github-repo-manager");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .init();

    guard
}

#[tokio::main]
async fn main() -> Result<()> {
    let _log_guard = init_tracing();

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        ratatui::restore();
        default_panic(info);
    }));

    let config = Config::load().map_err(|e| {
        eprintln!("Configuration error:\n  {e}");
        eprintln!(
            "\nConfig file should be at: {}",
            Config::config_path().display()
        );
        e
    })?;

    let terminal = ratatui::init();
    let result = App::new(config).run(terminal).await;
    ratatui::restore();

    result
}
