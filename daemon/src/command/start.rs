use std::{env, error::Error, path::Path, process::Stdio};
use tokio::process::{Child, Command};

use crate::entity;
use crate::global::{get_task_by_name, get_task_by_pid, update_pid_by_name, update_status_by_name};

async fn update_pid(name: String, pid: u32) -> Result<(), Box<dyn Error>> {
    update_pid_by_name(name, pid).await?;
    Ok(())
}

async fn update_status(name: String, status: String) -> Result<(), Box<dyn Error>> {
    update_status_by_name(name, status).await?;
    Ok(())
}

pub async fn start_task(args: Vec<String>) -> Result<entity::Response, Box<dyn Error>> {
    let len = args.len();
    let task;
    if len == 1 {
        task = get_task_by_name(args[0].clone()).await?;
    } else if len == 2 {
        if args[0] == "-n" || args[0] == "--name" {
            task = get_task_by_name(args[1].clone()).await?;
        } else if args[0] == "-p" || args[0] == "--pid" {
            let pid = args[1].parse::<u32>();
            if pid.is_err() {
                let res = entity::Response {
                    code: 40000,
                    msg: format!("Invalid pid: '{}'", args[1]),
                    data: None,
                };
                return Ok(res);
            }
            task = get_task_by_pid(pid.unwrap()).await?;
        } else {
            let res = entity::Response {
                code: 40000,
                msg: format!("Invalid args"),
                data: None,
            };
            return Ok(res);
        }
    } else {
        let res = entity::Response {
            code: 40000,
            msg: format!("Invalid args"),
            data: None,
        };
        return Ok(res);
    }
    // 声明指向文件的 stdout
    // TODO: 重定向到指定的文件
    let path = Path::new("/tmp/watchmen/tmp.log");
    // let file = std::fs::File::create(path)?;
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
        .stdout(stdout)
        .spawn()?;

    let result = child.id();

    match result {
        Some(pid) => {
            // 更改 task pid
            update_pid(task.name.clone(), pid).await?;
            // 更改 task status
            update_status(task.name.clone(), "running".to_string()).await?;
            // 异步等待子进程结束并更改 task status
            tokio::spawn(async move {
                child.wait().await.unwrap();
                update_status(task.name, "stopped".to_string())
                    .await
                    .unwrap();
            });
            Ok(entity::Response {
                code: 10000,
                msg: "success".to_string(),
                data: None,
            })
        }
        None => Ok(entity::Response {
            code: 40000,
            msg: "failed".to_string(),
            data: None,
        }),
    }
}
