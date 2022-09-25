use std::{env, error::Error, path::Path, process::Stdio};
use tokio::process::{Child, Command};

use crate::entity;
use crate::global::{
    get_task_by_id, get_task_by_name, update_exit_code_by_name, update_pid_by_name,
    update_status_by_name,
};

async fn update_pid(name: String, pid: u32) -> Result<(), Box<dyn Error>> {
    update_pid_by_name(name, pid).await?;
    Ok(())
}

async fn update_status(name: String, status: String) -> Result<(), Box<dyn Error>> {
    update_status_by_name(name, status).await?;
    Ok(())
}

pub async fn start_task_by_task(task: entity::Task) -> Result<entity::Response, Box<dyn Error>> {
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

    match result {
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
            Ok(entity::Response {
                code: 10000,
                msg: "start success".to_string(),
                data: None,
            })
        }
        None => Ok(entity::Response {
            code: 40000,
            msg: "start failed".to_string(),
            data: None,
        }),
    }
}

pub async fn start_task(command: entity::Command) -> Result<entity::Response, Box<dyn Error>> {
    let task;
    if command.options.contains_key("id") {
        let id = command.options.get("id").unwrap();
        if let entity::Opt::U128(ref i) = id {
            task = get_task_by_id(*i).await?;
        } else {
            return Ok(entity::Response {
                code: 50000,
                msg: "Arg 'id' must be a usize".to_string(),
                data: None,
            });
        }
    } else if command.options.contains_key("name") {
        let name = command.options.get("name").unwrap();
        if let entity::Opt::Str(ref s) = name {
            task = get_task_by_name(s.clone()).await?;
        } else {
            return Ok(entity::Response {
                code: 50000,
                msg: "Arg 'name' must be a string".to_string(),
                data: None,
            });
        }
    } else {
        if command.args.len() == 0 {
            return Ok(entity::Response {
                code: 40000,
                msg: "Arg 'name' is required".to_string(),
                data: None,
            });
        } else {
            task = get_task_by_name(command.args[0].clone()).await?;
        }
    }

    if task.status != "stopped" || task.status != "added" {
        return Ok(entity::Response {
            code: 40000,
            msg: format!("Task with '{}' could not be started", task.status),
            data: None,
        });
    }

    println!("Start task: {:?}", task);

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

    match result {
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
            Ok(entity::Response {
                code: 10000,
                msg: format!("{} success", command.name),
                data: None,
            })
        }
        None => Ok(entity::Response {
            code: 40000,
            msg: format!("{} failed", command.name),
            data: None,
        }),
    }
}
