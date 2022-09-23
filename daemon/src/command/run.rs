use std::{env, error::Error, path::Path, process::Stdio};
use tokio::process::{Child, Command};

use crate::entity::Task;
use crate::global::{add_task, update_pid_by_name, update_status_by_name};

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

pub async fn run_task(task: Task) -> Result<u32, Box<dyn Error>> {
    println!("task {} is running, pid is ", task.name);
    let t = task.clone();
    register(t).await?;
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

    let code = match result {
        Some(pid) => {
            // 更改 task pid
            update_pid(task.name.clone(), pid).await?;
            // 更改 task status
            update_status(task.name.clone(), "running".to_string()).await?;
            // 异步等待子进程结束并更改 task status
            tokio::spawn(async move {
                child.wait().await.unwrap();
            });
            10000
        }
        None => 40000,
    };

    Ok(code)
}
