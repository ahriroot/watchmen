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

pub async fn run_monitor() -> Result<entity::Response, Box<dyn std::error::Error>> {
    let mut interval = time::interval(Duration::from_secs(10));
    loop {
        let tasks = get_all_tasks().await.unwrap();
        for task in tasks {
            let time: DateTime<Local> = Local::now();
            let now = time.timestamp_millis() as u128;
            let dft = time.to_string();

            if task.status == "interval" && now >= task.origin {
                if now - task.laststart_at < task.interval {
                    continue;
                }
                let args_cmdline: Vec<String> = std::env::args().collect();
                if args_cmdline.len() < 3 {
                    return Ok(entity::Response {
                        code: 50000,
                        msg: "Miss stdout path".to_string(),
                        data: None,
                    });
                }

                // 声明指向文件的 stdout
                let path = Path::new(&args_cmdline[3]);
                let path = path.join(format!("{}.log", task.name));

                // 创建或追加文件
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?;
                let stdout = Stdio::from(file);

                println!("Execute {}: {:?}", dft, task);
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
                        println!(
                            "Finish {} Exit code {} {:?}",
                            Local::now().to_string(),
                            s,
                            task
                        );
                    });
                }
            }
        }
        interval.tick().await;
    }
}
