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

pub async fn start_task(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    let task;
    if command.options.contains_key("name") {
        let name = command.options.get("name").unwrap();
        if let entity::Opt::Str(ref s) = name.value {
            task = get_task_by_name(s.clone()).await?;
        } else {
            return Ok(entity::Response {
                code: 1,
                msg: "Arg 'name' must be a string".to_string(),
                data: None,
            });
        }
    } else if command.options.contains_key("pid") {
        let name = command.options.get("pid").unwrap();
        if let entity::Opt::U32(ref s) = name.value {
            task = get_task_by_pid(*s).await?;
        } else {
            return Ok(entity::Response {
                code: 1,
                msg: "Arg 'pid' must be a number".to_string(),
                data: None,
            });
        }
    } else {
        if command.args.len() == 0 {
            return Ok(entity::Response {
                code: 1,
                msg: "Arg 'name' or 'pid' is required".to_string(),
                data: None,
            });
        } else {
            task = get_task_by_name(command.args[0].clone()).await?;
        }
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
