use crate::common::task::TaskFlag;
use chrono::Datelike;
use chrono::Timelike;
use std::time::{Duration, SystemTime};
use tokio::time;
use tracing::{error, info};

use crate::global::{get_all, start};

pub async fn rerun_tasks(delay: u64) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = get_all().await?;
    let now = chrono::Local::now();
    for (id, task) in tasks {
        match task.task_type {
            crate::common::task::TaskType::Scheduled(scheduled) => {
                let nd = chrono::NaiveDate::from_ymd_opt(
                    scheduled.year.unwrap_or(now.year()),
                    scheduled.month.unwrap_or(now.month()),
                    scheduled.day.unwrap_or(now.day()),
                );
                let nt = chrono::NaiveTime::from_hms_opt(
                    scheduled.hour.unwrap_or(now.hour()),
                    scheduled.minute.unwrap_or(now.minute()),
                    scheduled.second.unwrap_or(now.second()),
                );
                match (nd, nt) {
                    (Some(nd), Some(nt)) => {
                        let exec = chrono::NaiveDateTime::new(nd, nt);
                        let exec_timestamp_utc = exec.timestamp();
                        let now_timestamp_utc = now.naive_local().timestamp();
                        let diff = (exec_timestamp_utc - now_timestamp_utc).abs() as u64;
                        if diff < delay && exec_timestamp_utc <= now_timestamp_utc {
                            if let Some(status) = task.status {
                                if status == "waiting" {
                                    tokio::spawn(async move {
                                        info!("Execute scheduled task: {}", id);
                                        let mut interval =
                                            time::interval(Duration::from_secs(diff));
                                        interval.tick().await;
                                        let _ = start(TaskFlag {
                                            id,
                                            name: None,
                                            group: None,
                                            mat: false,
                                        })
                                        .await;
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        error!("Invalid scheduled task: {}", id);
                    }
                }
            }
            crate::common::task::TaskType::Async(_) => {
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
            crate::common::task::TaskType::Periodic(tt) => {
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
            crate::common::task::TaskType::None => {}
        }
    }
    Ok(())
}

pub async fn run_monitor(delay: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let delay = delay.unwrap_or(5);
    let mut interval = time::interval(Duration::from_secs(delay));
    loop {
        match rerun_tasks(delay).await {
            Ok(_) => {}
            Err(e) => {
                error!("Monitor tasks error: {}", e);
            }
        }
        interval.tick().await;
    }
}
