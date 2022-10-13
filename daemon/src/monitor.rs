use chrono::prelude::{DateTime, Local};
use std::{env, path::Path, process::Stdio, time::Duration};
use tokio::{
    process::{Child, Command},
    time,
};

use crate::{
    entity,
    global::{get_all_tasks, update_laststart_at_by_id},
};

pub async fn rerun_tasks(home_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let tasks = get_all_tasks().await?;
    for task in tasks {
        let time: DateTime<Local> = Local::now();
        let now = time.timestamp_millis() as u128;

        if task.status == "interval" && now >= task.origin {
            if now - task.laststart_at < task.interval {
                continue;
            }

            // 声明指向文件的 stdout
            let path = Path::new(&home_path);
            let path = path.join("stdout");
            let path = path.join(format!("{}.log", task.name));

            // 创建或追加文件
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            let stdout = Stdio::from(file);

            crate::info!("TASK\tEXECUTE\t{:?}", task);

            // 获取环境变量 PATH
            let env_path = env::var("PATH")?;
            let mut child: Child = Command::new(&task.command)
                .args(&task.args)
                .env("PATH", env_path)
                .stdin(Stdio::null())
                .stdout(stdout)
                .stderr(Stdio::null())
                .spawn()?;

            let result = child.id();
            if result.is_some() {
                update_laststart_at_by_id(task.id, now).await?;
                tokio::spawn(async move {
                    let s = child.wait().await.unwrap();
                    let exit_code = s.code().unwrap_or(-1);
                    crate::info!("TASK\tFINISH\t{}\t{:?}", exit_code, task);
                });
            }
        }
    }
    Ok(())
}

pub async fn run_monitor(
    home_path: String,
) -> Result<entity::Response, Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_secs(10));
    loop {
        match rerun_tasks(home_path.clone()).await {
            Ok(_) => {}
            Err(e) => {
                crate::error!("Monitor tasks error: {}", e);
            }
        }
        interval.tick().await;
    }
}
