use std::{env, error::Error, path::Path, process::Stdio};
use tokio::process::{Child, Command};

use crate::entity::{self, Task};
use crate::global::{
    add_task, check_exists, update_exit_code_by_name, update_pid_by_name, update_status_by_name,
};

async fn register(task: Task) -> Result<(), Box<dyn Error>> {
    add_task(task).await?;
    Ok(())
}

async fn update_pid(name: String, pid: u32) -> Result<(), Box<dyn Error>> {
    update_pid_by_name(name, pid).await?;
    Ok(())
}

async fn update_status(name: String, status: String) -> Result<(), Box<dyn Error>> {
    update_status_by_name(name, status).await?;
    Ok(())
}

pub async fn run_task(task: Task) -> Result<entity::Response, Box<dyn Error>> {
    let args_cmdline: Vec<String> = std::env::args().collect();
    if args_cmdline.len() < 3 {
        return Ok(entity::Response {
            code: 50000,
            msg: "Miss stdout path".to_string(),
            data: None,
        });
    }

    let exists = check_exists(task.name.clone()).await?;
    if exists {
        return Ok(entity::Response {
            code: 40000,
            msg: "Task already exists".to_string(),
            data: None,
        });
    }

    let t = task.clone();
    register(t).await?;

    // 声明指向文件的 stdout
    let path = Path::new(&args_cmdline[3]);
    let path = path.join(format!("{}.log", task.name));

    // 创建或追加文件
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    let stdout = Stdio::from(file);

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

    let code = match result {
        Some(pid) => {
            // 更改 task pid
            update_pid(task.name.clone(), pid).await?;
            // 更改 task status
            update_status(task.name.clone(), "running".to_string()).await?;
            // 异步等待子进程结束并更改 task status
            tokio::spawn(async move {
                let s = child.wait().await.unwrap();
                update_status(task.name.clone(), "stopped".to_string())
                    .await
                    .unwrap();
                update_pid(task.name.clone(), 0).await.unwrap();
                if let Some(code) = s.code() {
                    update_exit_code_by_name(task.name.clone(), code as u32)
                        .await
                        .unwrap();
                }
            });
            10000
        }
        None => 40000,
    };
    if code > 0 {
        return Ok(entity::Response {
            code: 10000,
            msg: "run success".to_string(),
            data: None,
        });
    } else {
        return Ok(entity::Response {
            code: 40000,
            msg: "run failed".to_string(),
            data: None,
        });
    }
}
