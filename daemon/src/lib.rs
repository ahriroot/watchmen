pub mod command;
pub mod engine;
pub mod entity;
pub mod macros;
// pub mod monitor;
pub mod utils;

pub mod global {

    use std::{collections::HashMap, error::Error, process::Stdio};

    use common::task::Task;
    use lazy_static::lazy_static;
    use tokio::{
        io::AsyncWriteExt,
        process::{Child, Command},
        sync::{mpsc, Mutex},
        task::JoinHandle,
    };

    static CHANNEL_SIZE: usize = 1024;

    struct TaskProcess {
        task: Task,
        joinhandle: Option<JoinHandle<Option<i32>>>,
        tx: Option<mpsc::Sender<Vec<u8>>>,
    }

    lazy_static! {
        static ref TASKS: Mutex<HashMap<String, TaskProcess>> = Mutex::new(HashMap::new());
    }

    pub async fn update(
        name: String,
        pid: Option<Option<u32>>,
        status: Option<Option<String>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", name),
            )));
        }
        let mut tp = tasks.get_mut(&name).unwrap();
        if let Some(pid) = pid {
            tp.task.pid = pid;
        }
        if let Some(status) = status {
            tp.task.status = status;
        }
        Ok(())
    }

    pub async fn add(task: Task) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        let name = task.name.clone();
        if tasks.contains_key(&name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] already exists", task.name),
            )));
        }
        let tp = TaskProcess {
            task,
            joinhandle: None,
            tx: None,
        };
        tasks.insert(name, tp);
        Ok(())
    }

    pub async fn remove(name: String) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", name),
            )));
        }
        let tp = tasks.get(&name).unwrap();
        if tp.task.pid.is_some() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is running", name),
            )));
        }
        if tp.joinhandle.is_some() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is running", name),
            )));
        }
        tasks.remove(&name);
        Ok(())
    }

    pub async fn delete(name: String) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", name),
            )));
        }
        let tp = tasks.get(&name).unwrap();
        if tp.task.pid.is_some() {
            stop(name.clone()).await?;
        }
        if let Some(jh) = &tp.joinhandle {
            jh.abort();
        }
        tasks.remove(&name);
        Ok(())
    }

    pub async fn start(name: String) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", name),
            )));
        }
        let mut tp = tasks.get_mut(&name).unwrap();

        let child = tp.task.start().await?;

        let rx = if Some(true) == tp.task.stdin {
            let (tx, rx) = mpsc::channel::<Vec<u8>>(CHANNEL_SIZE);
            tp.tx = Some(tx);
            Some(rx)
        } else {
            None
        };

        let task_name = tp.task.name.clone();
        let pid = child.id();
        let status = Some("running".to_string());

        let jh: JoinHandle<Option<i32>> = tokio::spawn(async move {
            let mut child = child;

            let cjh = if let Some(mut rx) = rx {
                let mut child_stdin = child.stdin.take().unwrap();
                let cjh = tokio::spawn(async move {
                    while let Some(message) = rx.recv().await {
                        child_stdin.write_all(&message).await.unwrap();
                    }
                });
                Some(cjh)
            } else {
                None
            };

            let res = child.wait().await.unwrap();

            update(name, Some(None), Some(Some("stopped".to_string())))
                .await
                .unwrap();

            if let Some(cjh) = cjh {
                cjh.await.unwrap();
            }

            return res.code();
        });

        tp.joinhandle = Some(jh);

        tokio::spawn(async move {
            update(task_name, Some(pid), Some(status)).await.unwrap();
        });

        Ok(())
    }

    pub async fn run(task: Task) -> Result<(), Box<dyn Error>> {
        let name = task.name.clone();
        add(task).await?;
        start(name).await
    }

    pub async fn stop(name: String) -> Result<(), Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", name),
            )));
        }
        let tp = tasks.get_mut(&name).unwrap();

        let pid = tp.task.pid;

        if let Some(pid) = pid {
            let mut child: Child = Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .envs(std::env::vars())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            child.wait().await?;
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not running", name),
            )))
        }
    }
}
