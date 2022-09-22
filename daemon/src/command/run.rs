use std::{env, error::Error, path::Path, process::Stdio};
use tokio::process::{Child, Command};

use crate::entity::Task;
use crate::global::{add_task, remove_task_by_name, update_pid_by_name, update_status_by_name};

async fn register(task: Task) -> Result<(), Box<dyn Error>> {
    add_task(task).await?;
    Ok(())
}

async fn unregister(name: String) -> Result<Option<Task>, Box<dyn Error>> {
    let task = remove_task_by_name(name).await?;
    Ok(task)
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
            // 注册任务
            update_pid(task.name.clone(), pid).await?;
            update_status(task.name.clone(), "running".to_string()).await?;
            // 异步等待子进程退出并注销任务
            tokio::spawn(async move {
                child.wait().await.unwrap();
                unregister(task.name).await.unwrap();
            });
            10000
        }
        None => 50000,
    };

    Ok(code)
}
