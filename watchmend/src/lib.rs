pub mod command;
pub mod engine;
pub mod monitor;
pub mod utils;

pub mod global {

    use std::{
        collections::HashMap,
        error::Error,
        path::Path,
        process::Stdio,
        time::{SystemTime, UNIX_EPOCH},
    };

    use common::{
        config::{get_with_home, get_with_home_path},
        handle::{Data, Response, Status},
        task::{AsyncTask, Task, TaskFlag, TaskType},
    };
    use lazy_static::lazy_static;
    use regex::Regex;
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

    /// 设置缓存路径。
    ///
    /// # 参数
    ///
    /// - `path`: 缓存路径的字符串。
    ///
    /// # 示例
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// let mut rt = Runtime::new().unwrap();
    /// rt.block_on(async {
    ///     let path = "/path/to/cache".to_string();
    ///     set_cache(path).await;
    /// });
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 该函数会阻塞当前的异步任务执行线程。
    /// - 在调用该函数前，必须先初始化全局的缓存锁（CACHE）。
    pub async fn set_cache(path: String) {
        let mut cache = CACHE.lock().await;
        *cache = Some(path);
    }

    pub async fn get_all() -> Result<HashMap<String, Task>, Box<dyn Error>> {
        let tasks = TASKS.lock().await;
        let mut tasks_map: HashMap<String, Task> = HashMap::new();
        for (name, tp) in tasks.iter() {
            tasks_map.insert(name.clone(), tp.task.clone());
        }
        Ok(tasks_map)
    }

    pub async fn cache() -> Result<(), Box<dyn Error>> {
        // 启动协程写入缓存文件，避免阻塞对其他任务的操作
        tokio::spawn(async move {
            let path_mutex = CACHE.lock().await;
            let path = path_mutex.clone();
            drop(path_mutex); // 释放锁，避免阻塞对其他任务的操作
            if let Some(path) = path {
                let path = get_with_home(path.as_str());
                let path = Path::new(path.as_str());
                let parent = path.parent().unwrap();
                if !parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                let tasks = TASKS.lock().await;
                let tasks_cache: Vec<Task> = tasks.values().map(|tp| tp.task.clone()).collect();
                drop(tasks); // 释放锁，避免阻塞对其他任务的操作
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

        // 读取缓存文件序列化成任务列表
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
                    // 上次运行状态为 running 的染污加载后直接启动
                    if tp.task.status == Some("running".to_string()) {
                        let child = tp.task.start().await?;

                        // 配置了 stdin 时，启动一个协程用于向子进程 stdin 写入数据
                        let rx = if Some(true) == tp.task.stdin {
                            let (tx, rx) = mpsc::channel::<Vec<u8>>(CHANNEL_SIZE);
                            tp.tx = Some(tx);
                            Some(rx)
                        } else {
                            None
                        };

                        // 更新任务状态等数据
                        tp.task.pid = child.id();
                        tp.task.status = Some("running".to_string());
                        let now: u64 = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Failed to get timestamp")
                            .as_secs();
                        tp.task.task_type = TaskType::Async(AsyncTask {
                            max_restart: 0,
                            has_restart: 0,
                            started_at: now,
                            stopped_at: 0,
                        });

                        // 启动协程等待子进程退出
                        let jh: JoinHandle<Option<i32>> = tokio::spawn(async move {
                            let mut child = child;

                            // 接收到数据时写入子进程 stdin
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

                            // 等待子进程退出
                            let res = child.wait().await.unwrap();

                            update(
                                task.name,
                                Some(None),
                                Some(Some("stopped".to_string())),
                                Some(res.code()),
                                None,
                            )
                            .await
                            .unwrap();

                            // 等待 stdin 写入协程退出
                            if let Some(cjh) = cjh {
                                cjh.await.unwrap();
                            }

                            return res.code();
                        });

                        // 保存监控进程结束协程的句柄
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
        restart: Option<bool>,
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
            match status.clone() {
                Some(s) => match s.as_str() {
                    "running" => match tp.task.task_type.clone() {
                        TaskType::Async(tmp) => {
                            tp.task.task_type = TaskType::Async(AsyncTask {
                                max_restart: tmp.max_restart,
                                has_restart: tmp.has_restart,
                                started_at: tmp.started_at,
                                stopped_at: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .expect("Failed to get timestamp")
                                    .as_secs(),
                            });
                        }
                        TaskType::Periodic(tmp) => {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Failed to get timestamp")
                                .as_secs();
                            tp.task.task_type = TaskType::Periodic(common::task::PeriodicTask {
                                interval: tmp.interval,
                                last_run: now,
                                started_after: tmp.started_after,
                            });
                        }
                        _ => {}
                    },
                    _ => {}
                },
                None => {}
            }
            tp.task.status = status;
        }
        if let Some(restart) = restart {
            match tp.task.task_type.clone() {
                TaskType::Async(tmp) => {
                    let has = if restart {
                        if tmp.has_restart >= tmp.max_restart {
                            tp.task.status = Some("stopped".to_string());
                            tmp.has_restart
                        } else {
                            tmp.has_restart + 1
                        }
                    } else {
                        tp.task.status = Some("stopped".to_string());
                        tmp.has_restart
                    };
                    tp.task.task_type = TaskType::Async(AsyncTask {
                        max_restart: tmp.max_restart,
                        has_restart: has,
                        started_at: tmp.started_at,
                        stopped_at: tmp.stopped_at,
                    });
                }
                _ => {}
            }
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
            stop(tf.clone(), true).await?;
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

        match &tp.task.task_type {
            TaskType::Async(tt) => {
                let max = tt.max_restart;
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
                    let code = res.code();
                    let exit = if let Some(code) = code {
                        if code == 0 {
                            true
                        } else {
                            max == 0
                        }
                    } else {
                        true
                    };
                    if exit {
                        update(
                            tf.name,
                            Some(None),
                            Some(Some("stopped".to_string())),
                            Some(res.code()),
                            None,
                        )
                        .await
                        .unwrap();
                        cache().await.unwrap();

                        if let Some(cjh) = cjh {
                            cjh.await.unwrap();
                        }
                    } else {
                        update(
                            tf.name,
                            Some(None),
                            Some(Some("auto restart".to_string())),
                            Some(res.code()),
                            Some(true),
                        )
                        .await
                        .unwrap();
                        cache().await.unwrap();

                        if let Some(cjh) = cjh {
                            cjh.await.unwrap();
                        }
                    }

                    return res.code();
                });

                tp.joinhandle = Some(jh);

                tokio::spawn(async move {
                    update(task_name, Some(pid), Some(status), None, None)
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

    pub async fn restart(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        stop(tf.clone(), false).await?;
        start(tf).await
    }

    pub async fn run(task: Task) -> Result<Response, Box<dyn Error>> {
        let name = task.name.clone();
        add(task).await?;
        start(TaskFlag { name, mat: false }).await
    }

    pub async fn stop(tf: TaskFlag, to_cache: bool) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.lock().await;
        if !tasks.contains_key(&tf.name) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.name),
            )));
        }
        let tp = tasks.get_mut(&tf.name).unwrap();

        if tp.task.status != Some("running".to_string())
            && tp.task.status != Some("auto restart".to_string())
        {
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
            if to_cache {
                cache().await?;
            }
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
                if condition.mat {
                    for (name, tp) in tasks.iter() {
                        let regex: Regex = Regex::new(&condition.name)?;
                        if regex.is_match(name) {
                            status.push(tp.task.clone().into());
                        }
                    }
                    let response = Response::success(Some(Data::Status(status)));
                    Ok(response)
                } else {
                    if tasks.contains_key(&condition.name) {
                        let tp = tasks.get(&condition.name).unwrap();
                        status.push(tp.task.clone().into());
                        let response = Response::success(Some(Data::Status(status)));
                        Ok(response)
                    } else {
                        let response = Response::success(Some(Data::Status(status)));
                        Ok(response)
                    }
                }
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

#[cfg(test)]
mod tests {

    #[test]
    fn reg() {
        let regex: regex::Regex = regex::Regex::new("Test").unwrap();
        assert!(regex.is_match("TestPython"));
    }
}
