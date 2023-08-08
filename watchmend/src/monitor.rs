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
                            name: None,
                            group: None,
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
                    if let Some(status) = task.status {
                        if tt.sync {
                            if status == "interval" || status == "executing" {
                                info!("Execute periodic task: {}", id);
                                start(TaskFlag {
                                    id,
                                    name: None,
                                    group: None,
                                    mat: false,
                                })
                                .await?;
                            }
                        } else {
                            if status == "interval" {
                                info!("Execute periodic task: {}", id);
                                start(TaskFlag {
                                    id,
                                    name: None,
                                    group: None,
                                    mat: false,
                                })
                                .await?;
                            }
                        }
                    }
                }
            }
            common::task::TaskType::None => {}
        }
    }
    Ok(())
}

pub async fn run_monitor() -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_secs(5));
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
