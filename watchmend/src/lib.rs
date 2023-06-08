pub mod command;
pub mod engine;
// pub mod monitor;
pub mod utils;

pub mod global {

    use std::{collections::HashMap, error::Error, path::Path, process::Stdio};

    use common::{
        config::{get_with_home, get_with_home_path},
        handle::{Data, Response, Status},
        task::{Task, TaskFlag, TaskType},
    };
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
        static ref CACHE: Mutex<Option<String>> = Mutex::new(None);
        static ref TASKS: Mutex<HashMap<String, TaskProcess>> = Mutex::new(HashMap::new());
    }

    pub async fn set_cache(path: String) {
        let mut cache = CACHE.lock().await;
        *cache = Some(path);
    }

    pub async fn cache() -> Result<(), Box<dyn Error>> {
        tokio::spawn(async move {
            let path_mutex = CACHE.lock().await;
            let path = path_mutex.clone();
            drop(path_mutex);
            if let Some(path) = path {
                let path = get_with_home(path.as_str());
                let path = Path::new(path.as_str());
                let parent = path.parent().unwrap();
                if !parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                let tasks = TASKS.lock().await;
                let tasks_cache: Vec<Task> = tasks.values().map(|tp| tp.task.clone()).collect();
                drop(tasks);
                let tasks_cache_str = serde_json::to_string(&tasks_cache).unwrap();
                tokio::fs::write(path, tasks_cache_str).await.unwrap();
            }
        });
        Ok(())
    }

    pub async fn load(path: &str) -> Result<(), Box<dyn Error>> {
        let path = get_with_home(path);
        let path = Path::new(path.as_str());
        if !path.exists() || !path.is_file() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Config file [{}] not a valid file", path.to_str().unwrap()),
            )));
        }
        let tasks_cache: Vec<Task> = serde_json::from_str(&std::fs::read_to_string(path).unwrap())?;
        let mut tasks = TASKS.lock().await;
        for task in tasks_cache {
            let name = task.name.clone();
            let mut tp = TaskProcess {
                task: task.clone(),
                joinhandle: None,
                tx: None,
            };
            match tp.task.task_type {
                TaskType::Async(_) => {
                    if tp.task.status == Some("running".to_string()) {
                        let child = tp.task.start().await?;

                        let rx = if Some(true) == tp.task.stdin {
                            let (tx, rx) = mpsc::channel::<Vec<u8>>(CHANNEL_SIZE);
                            tp.tx = Some(tx);
                            Some(rx)
                        } else {
                            None
                        };

                        tp.task.pid = child.id();
                        tp.task.status = Some("running".to_string());

                        let jh: JoinHandle<Option<i32>> = tokio::spawn(async move {
                            let mut child = child;

                            let cjh = if let Some(mut rx) = rx {
                                let mut child_stdin = child.stdin.take().unwrap();
                                // let mut stdin_writer = tokio::io::BufWriter::new(child_stdin);
                                let cjh = tokio::spawn(async move {
                                    while let Some(message) = rx.recv().await {
                                        child_stdin.write_all(&message).await.unwrap();
                                        child_stdin.flush().await.unwrap();
                                    }
                                });
                                Some(cjh)
                            } else {
                                None
                            };

                            let res = child.wait().await.unwrap();

                            update(
                                task.name,
                                Some(None),
                                Some(Some("stopped".to_string())),
                                Some(res.code()),
                            )
                            .await
                            .unwrap();

                            if let Some(cjh) = cjh {
                                cjh.await.unwrap();
                            }

                            return res.code();
                        });

                        tp.joinhandle = Some(jh);
                    }
                }
                _ => {}
            }
            tasks.insert(name, tp);
        }
        Ok(())
    }

    pub async fn update(
        name: String,
        pid: Option<Option<u32>>,
        status: Option<Option<String>>,
        code: Option<Option<i32>>,
    ) -> Result<Response, Box<dyn Error>> {
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
        if let Some(code) = code {
            tp.task.code = code;
        }
        Ok(Response::success(None))
    }

    pub async fn add(mut task: Task) -> Result<Response, Box<dyn Error>> {
        if let Some(so) = &task.stdout {
            let stdout = get_with_home_path(so);
            let parent = stdout.parent().unwrap();
            if parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
            task.stdout = Some(stdout.to_str().unwrap().to_string());
        }
        if let Some(se) = &task.stderr {
            let stderr = get_with_home_path(se);
            let parent = stderr.parent().unwrap();
            if parent.exists() {
                std::fs::create_dir_all(parent).unwrap();
            }
            task.stderr = Some(stderr.to_str().unwrap().to_string());
        }

        let mut args = task.args.clone();
        for i in 0..args.len() {
            args[i] = get_with_home(&args[i]);
        }
        task.args = args;

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
        cache().await?;
        Ok(Response::success(None))
    }

    pub async fn remove(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&tf.name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.name),
            )));
        }
        let tp = tasks.get(&tf.name).unwrap();
        if Some("running".to_string()) == tp.task.status {
            return Ok(Response::wrong(
                "Task is running, please stop it first".to_string(),
            ));
        }
        tasks.remove(&tf.name);
        cache().await?;
        Ok(Response::success(None))
    }

    pub async fn delete(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&tf.name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.name),
            )));
        }
        let tp = tasks.get(&tf.name).unwrap();
        if tp.task.pid.is_some() {
            stop(tf.clone()).await?;
        }
        if let Some(jh) = &tp.joinhandle {
            jh.abort();
        }
        tasks.remove(&tf.name);
        Ok(Response::success(None))
    }

    pub async fn start(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&tf.name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.name),
            )));
        }
        let mut tp = tasks.get_mut(&tf.name).unwrap();

        match tp.task.task_type {
            TaskType::Async(_) => {
                if tp.task.status == Some("running".to_string()) {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Task [{}] is running", tf.name),
                    )));
                }

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
                        // let mut stdin_writer = tokio::io::BufWriter::new(child_stdin);
                        let cjh = tokio::spawn(async move {
                            while let Some(message) = rx.recv().await {
                                child_stdin.write_all(&message).await.unwrap();
                                child_stdin.flush().await.unwrap();
                            }
                        });
                        Some(cjh)
                    } else {
                        None
                    };

                    let res = child.wait().await.unwrap();

                    update(
                        tf.name,
                        Some(None),
                        Some(Some("stopped".to_string())),
                        Some(res.code()),
                    )
                    .await
                    .unwrap();
                    cache().await.unwrap();

                    if let Some(cjh) = cjh {
                        cjh.await.unwrap();
                    }

                    return res.code();
                });

                tp.joinhandle = Some(jh);

                tokio::spawn(async move {
                    update(task_name, Some(pid), Some(status), None)
                        .await
                        .unwrap();
                });

                cache().await?;
                Ok(Response::success(None))
            }
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Task is not an async task",
            ))),
        }
    }

    pub async fn run(task: Task) -> Result<Response, Box<dyn Error>> {
        let name = task.name.clone();
        add(task).await?;
        start(TaskFlag { name }).await
    }

    pub async fn stop(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&tf.name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.name),
            )));
        }
        let tp = tasks.get_mut(&tf.name).unwrap();

        if tp.task.status != Some("running".to_string()) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is not running", tf.name),
            )));
        }

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
            tp.task.status = Some("stopped".to_string());
            tp.task.code = Some(9);
            tp.joinhandle = None;
            tp.tx = None;
            drop(tasks);
            cache().await?;
            Ok(Response::success(None))
        } else {
            Ok(Response::wrong("Task is not running".to_string()))
        }
    }

    pub async fn write(tf: TaskFlag, data: String) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&tf.name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.name),
            )));
        }
        let tp = tasks.get_mut(&tf.name).unwrap();

        if tp.task.status != Some("running".to_string()) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is not running", tf.name),
            )));
        }

        let tx = &tp.tx.clone().unwrap();

        let data: Vec<u8> = data.into_bytes();

        tx.send(data).await?;

        Ok(Response::success(None))
    }

    pub async fn list(condition: Option<TaskFlag>) -> Result<Response, Box<dyn Error>> {
        let tasks = TASKS.lock().await;
        let mut status: Vec<Status> = Vec::new();
        match condition {
            Some(condition) => {
                for (name, tp) in tasks.iter() {
                    if name.contains(&condition.name) {
                        status.push(tp.task.clone().into());
                    }
                }
                let response = Response::success(Some(Data::Status(status)));
                Ok(response)
            }
            None => {
                for (_, tp) in tasks.iter() {
                    status.push(tp.task.clone().into());
                }
                let response = Response::success(Some(Data::Status(status)));
                Ok(response)
            }
        }
    }
}
