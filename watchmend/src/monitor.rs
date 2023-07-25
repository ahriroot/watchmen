use common::task::TaskFlag;
use std::time::{Duration, SystemTime};
use tokio::time;
use tracing::{error, info};

use crate::global::{get_all, start};

pub async fn rerun_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let tasks = get_all().await?;
    for (id, task) in tasks {
        match task.task_type {
            common::task::TaskType::Scheduled(_) => {}
            common::task::TaskType::Async(_) => {
                if let Some(status) = task.status {
                    if status == "auto restart" {
                        info!("Restart task: {}", id);
                        start(TaskFlag {
                            id,
                            name: "".to_string(),
                            mat: false,
                        })
                        .await?;
                    }
                }
            }
            common::task::TaskType::Periodic(tt) => {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("Failed to get timestamp")
                    .as_secs();
                if now >= tt.started_after && now - tt.last_run >= tt.interval {
                    info!("Run periodic task: {}", id);
                    start(TaskFlag {
                        id,
                        name: "".to_string(),
                        mat: false,
                    })
                    .await?;
                }
            }
            common::task::TaskType::None => {}
        }
    }
    Ok(())
}

pub async fn run_monitor() -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_secs(2));
    loop {
        match rerun_tasks().await {
            Ok(_) => {}
            Err(e) => {
                error!("Monitor tasks error: {}", e);
            }
        }
        interval.tick().await;
    }
}
