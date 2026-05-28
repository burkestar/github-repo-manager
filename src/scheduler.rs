use tokio::sync::mpsc::UnboundedSender;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::config::Config;
use crate::error::{AppError, Result};
use crate::events::AppEvent;
use crate::git::{fetch::fetch_repo, workspace::scan_workspace};

pub async fn start_scheduler(
    config: Config,
    event_tx: UnboundedSender<AppEvent>,
) -> Result<JobScheduler> {
    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| AppError::Scheduler(e.to_string()))?;

    let cron_expr = config.cron_schedule.clone();
    info!("Starting fetch scheduler with cron: {}", cron_expr);

    let job = Job::new_async(cron_expr.as_str(), move |_uuid, _lock| {
        let config = config.clone();
        let tx = event_tx.clone();
        Box::pin(async move {
            info!("Scheduled batch fetch starting");
            let _ = tx.send(AppEvent::BatchFetchStarted);

            let checked_out = scan_workspace(&config.workspace_root, &config.layout);
            let (mut fetched, mut failed) = (0usize, 0usize);

            for info in checked_out.values() {
                let path = info.local_path.clone();
                let token = config.github_token.clone();
                let result = tokio::task::spawn_blocking(move || fetch_repo(&path, &token)).await;

                match result {
                    Ok(Ok(())) => fetched += 1,
                    Ok(Err(e)) => {
                        error!("Fetch failed for {}: {}", info.local_path.display(), e);
                        failed += 1;
                    }
                    Err(e) => {
                        error!("Spawn blocking error: {e}");
                        failed += 1;
                    }
                }
            }

            info!("Batch fetch complete: {} ok, {} failed", fetched, failed);
            let _ = tx.send(AppEvent::BatchFetchCompleted { fetched, failed });
        })
    })
    .map_err(|e| AppError::Scheduler(e.to_string()))?;

    scheduler
        .add(job)
        .await
        .map_err(|e| AppError::Scheduler(e.to_string()))?;

    scheduler
        .start()
        .await
        .map_err(|e| AppError::Scheduler(e.to_string()))?;

    Ok(scheduler)
}
