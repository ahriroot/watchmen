pub mod command;
pub mod common {
    include!("../../common.rs");
}
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

    use crate::common::{
        config::{get_with_home, get_with_home_path},
        handle::{Data, Response, Status},
        task::{AsyncTask, Task, TaskFlag, TaskType},
    };
    use lazy_static::lazy_static;
    use log::info;
    use regex::Regex;
    use tokio::{
        io::AsyncWriteExt,
        process::{Child, Command},
        sync::{mpsc, RwLock},
        task::JoinHandle,
    };

    static CHANNEL_SIZE: usize = 1024;

    struct TaskProcess {
        task: Task,
        joinhandle: Option<JoinHandle<Option<i32>>>,
        tx: Option<mpsc::Sender<Vec<u8>>>,
    }

    lazy_static! {
        static ref CACHE: RwLock<Option<String>> = RwLock::new(None);
        static ref TASKS: RwLock<HashMap<i64, TaskProcess>> = RwLock::new(HashMap::new());
    }

    /// Set cache path
    ///
    /// # params
    ///
    /// - `path`: cache file path
    ///
    /// # example
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
    pub async fn set_cache(path: String) {
        let mut cache = CACHE.write().await;
        *cache = Some(path);
    }

    pub async fn get_all() -> Result<HashMap<i64, Task>, Box<dyn Error>> {
        let tasks = TASKS.read().await;
        let mut tasks_map: HashMap<i64, Task> = HashMap::new();
        for (id, tp) in tasks.iter() {
            tasks_map.insert(*id, tp.task.clone());
        }
        Ok(tasks_map)
    }

    pub async fn cache() -> Result<(), Box<dyn Error>> {
        // 启动协程写入缓存文件，避免阻塞对其他任务的操作
        tokio::spawn(async move {
            let path_mutex = CACHE.read().await;
            let path = path_mutex.clone();
            drop(path_mutex); // 释放锁，避免阻塞对其他任务的操作
            if let Some(path) = path {
                let path = get_with_home(path.as_str());
                let path = Path::new(path.as_str());
                let parent = path.parent().unwrap();
                if !parent.exists() {
                    std::fs::create_dir_all(parent).unwrap();
                }
                let tasks = TASKS.read().await;
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
                format!("Cache file [{}] is not valid", path.to_str().unwrap()),
            )));
        }

        // 读取缓存文件序列化成任务列表
        let tasks_cache: Vec<Task> = serde_json::from_str(&std::fs::read_to_string(path).unwrap())?;
        let mut tasks = TASKS.write().await;
        for task in tasks_cache {
            let mut tp = TaskProcess {
                task: task.clone(),
                joinhandle: None,
                tx: None,
            };
            match &tp.task.task_type {
                TaskType::Async(tt) => {
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
                            max_restart: tt.max_restart,
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
                                task.id,
                                Some(None),
                                Some(Some("stopped".to_string())),
                                Some(res.code()),
                                None,
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
            tasks.insert(task.id, tp);
        }
        Ok(())
    }

    pub async fn update(
        id: i64,
        pid: Option<Option<u32>>,
        status: Option<Option<String>>,
        code: Option<Option<i32>>,
        restart: Option<bool>,
        from_status: Option<Vec<&str>>,
    ) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if !tasks.contains_key(&id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", id),
            )));
        }
        let tp = tasks.get_mut(&id).unwrap();
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
                            tp.task.task_type =
                                TaskType::Periodic(crate::common::task::PeriodicTask {
                                    interval: tmp.interval,
                                    last_run: now,
                                    started_after: tmp.started_after,
                                    sync: tmp.sync,
                                });
                        }
                        _ => {}
                    },
                    _ => {}
                },
                None => {}
            }
            if let Some(from) = from_status {
                let now_status = tp.task.status.as_ref().unwrap().as_str();
                if from.contains(&now_status) {
                    tp.task.status = status;
                }
            } else {
                tp.task.status = status;
            }
        }
        if let Some(restart) = restart {
            match tp.task.task_type.clone() {
                TaskType::Async(tmp) => {
                    let has = if restart {
                        if let Some(max) = tmp.max_restart {
                            if tmp.has_restart >= max {
                                tp.task.status = Some("stopped".to_string());
                                tmp.has_restart
                            } else {
                                tmp.has_restart + 1
                            }
                        } else {
                            0
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
        let mut tasks = TASKS.write().await;
        let id = task.id;
        if tasks.contains_key(&id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] already exists", id),
            )));
        }

        match task.task_type {
            TaskType::Scheduled(_) => {
                task.status = Some("waiting".to_string());
            }
            _ => {
                task.status = Some("added".to_string());
            }
        }

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
        let tn = task.name.clone();
        let tp = TaskProcess {
            task,
            joinhandle: None,
            tx: None,
        };
        tasks.insert(id, tp);
        cache().await?;
        Ok(Response::success(Some(Data::String(format!(
            "Task [{}] added",
            tn
        )))))
    }

    pub async fn reload(task: Task) -> Result<Response, Box<dyn Error>> {
        remove(
            TaskFlag {
                id: task.id,
                name: None,
                group: None,
                mat: false,
            },
            false,
        )
        .await?;
        add(task).await
    }

    pub async fn remove(tf: TaskFlag, to_cache: bool) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if tf.id > 0 {
            if !tasks.contains_key(&tf.id) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Task [{}] not exists", tf.id),
                )));
            }
            let tp = tasks.get(&tf.id).unwrap();
            if Some("running".to_string()) == tp.task.status {
                return Ok(Response::wrong(
                    "Task is running, please stop it first".to_string(),
                ));
            }
            if let Some(removed) = tasks.remove(&tf.id) {
                if to_cache {
                    cache().await?;
                }
                Ok(Response::success(Some(Data::String(format!(
                    "Task [{}:{}] removed",
                    removed.task.id, removed.task.name
                )))))
            } else {
                Ok(Response::wrong(format!("Task [{}:] not exists", tf.id)))
            }
        } else if let Some(name) = tf.name {
            // find id by name
            let id = tasks
                .iter()
                .find(|(_, tp)| tp.task.name == name)
                .map(|(id, _)| *id);
            if let Some(id) = id {
                let tp = tasks.get(&id).unwrap();
                if Some("running".to_string()) == tp.task.status {
                    return Ok(Response::wrong(
                        "Task is running, please stop it first".to_string(),
                    ));
                }
                if let Some(removed) = tasks.remove(&id) {
                    if to_cache {
                        cache().await?;
                    }
                    Ok(Response::success(Some(Data::String(format!(
                        "Task [{}:{}] removed",
                        removed.task.id, removed.task.name
                    )))))
                } else {
                    Ok(Response::wrong(format!(
                        "Task [{}:{}] not exists",
                        tf.id, name
                    )))
                }
            } else {
                return Ok(Response::wrong(format!("Task [:{}] not exists", name)));
            }
        } else if let Some(group) = &tf.group {
            let ids = tasks
                .iter()
                .filter(|(_, tp)| {
                    if let Some(g) = &tp.task.group {
                        g == group
                    } else {
                        false
                    }
                })
                .map(|(id, _)| *id)
                .collect::<Vec<i64>>();
            if ids.is_empty() {
                return Ok(Response::wrong(format!(
                    "Task group [{}] not exists",
                    group
                )));
            }
            let mut removed = vec![];
            for id in ids {
                let tp = tasks.get(&id).unwrap();
                if Some("running".to_string()) == tp.task.status {
                    return Ok(Response::wrong(
                        "Task is running, please stop it first".to_string(),
                    ));
                }
                if let Some(r) = tasks.remove(&id) {
                    removed.push(format!("{}:{}", r.task.id, r.task.name));
                }
            }
            Ok(Response::success(Some(Data::String(format!(
                "Task group [{}({})] removed",
                group,
                removed.len()
            )))))
        } else {
            Ok(Response::wrong(
                "Task id or name or group is required".to_string(),
            ))
        }
    }

    pub async fn delete(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if !tasks.contains_key(&tf.id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.id),
            )));
        }
        let tp = tasks.get(&tf.id).unwrap();
        if tp.task.pid.is_some() {
            stop(tf.clone(), true).await?;
        }
        if let Some(jh) = &tp.joinhandle {
            jh.abort();
        }
        let tn = tf.id.clone();
        tasks.remove(&tf.id);
        Ok(Response::success(Some(Data::String(format!(
            "Task [{}] deleted",
            tn
        )))))
    }

    pub async fn start(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if !tasks.contains_key(&tf.id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.id),
            )));
        }
        let tp = tasks.get_mut(&tf.id).unwrap();

        match &tp.task.task_type {
            TaskType::Scheduled(_) => {
                let id = tf.id;
                let name = tf.name.clone();

                let mut child = tp.task.start().await?;
                let pid = child.id();
                let jh: JoinHandle<Option<i32>> = tokio::spawn(async move {
                    let res = child.wait().await.unwrap();
                    let code = res.code();
                    info!(
                        "Task [{}:{}] exited with code: {:?}",
                        id,
                        name.unwrap_or(String::new()),
                        code
                    );

                    update(
                        tf.id,
                        Some(None),
                        Some(Some("waiting".to_string())),
                        Some(res.code()),
                        Some(true),
                        Some(vec!["processing"]),
                    )
                    .await
                    .unwrap();

                    cache().await.unwrap();

                    return res.code();
                });

                tp.joinhandle = Some(jh);

                tokio::spawn(async move {
                    update(
                        id,
                        Some(pid),
                        Some(Some("processing".to_string())),
                        None,
                        None,
                        None,
                    )
                    .await
                    .unwrap();
                });
                Ok(Response::success(Some(Data::String(format!(
                    "Task [{}] started",
                    id
                )))))
            }
            TaskType::Async(tt) => {
                let max = tt.max_restart;
                if tp.task.status == Some("running".to_string()) {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Task [{}] is running", tf.id),
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
                    let exit = if let Some(max) = max {
                        if let Some(code) = code {
                            if code == 0 {
                                true
                            } else {
                                max == 0
                            }
                        } else {
                            true
                        }
                    } else {
                        true
                    };
                    if exit {
                        update(
                            tf.id,
                            Some(None),
                            Some(Some("stopped".to_string())),
                            Some(res.code()),
                            None,
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
                            tf.id,
                            Some(None),
                            Some(Some("auto restart".to_string())),
                            Some(res.code()),
                            Some(true),
                            None,
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

                let id = tf.id;
                tokio::spawn(async move {
                    update(id, Some(pid), Some(status), None, None, None)
                        .await
                        .unwrap();
                });

                cache().await?;
                Ok(Response::success(Some(Data::String(format!(
                    "Task [{}] started",
                    id
                )))))
            }
            TaskType::Periodic(_) => {
                let id = tf.id;
                let name = tf.name.clone();

                let mut child = tp.task.start().await?;
                let pid = child.id();
                let jh: JoinHandle<Option<i32>> = tokio::spawn(async move {
                    let res = child.wait().await.unwrap();
                    let code = res.code();
                    info!(
                        "Task [{}:{}] exited with code: {:?}",
                        id,
                        name.unwrap_or(String::new()),
                        code
                    );

                    let _ = update(
                        tf.id,
                        Some(None),
                        Some(Some("interval".to_string())),
                        Some(res.code()),
                        Some(true),
                        Some(vec!["executing"]),
                    )
                    .await;

                    cache().await.unwrap();

                    return res.code();
                });

                tp.joinhandle = Some(jh);

                tokio::spawn(async move {
                    update(
                        id,
                        Some(pid),
                        Some(Some("executing".to_string())),
                        None,
                        None,
                        None,
                    )
                    .await
                    .unwrap();
                });
                Ok(Response::success(Some(Data::String(format!(
                    "Task [{}] started",
                    id
                )))))
            }
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Task type not supported",
            ))),
        }
    }

    pub async fn restart(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        stop(tf.clone(), false).await?;
        start(tf).await
    }

    pub async fn run(task: Task) -> Result<Response, Box<dyn Error>> {
        let id = task.id;
        let typ = task.task_type.clone();
        let res = add(task).await?;
        match typ {
            TaskType::Async(_) | TaskType::Periodic(_) => {
                start(TaskFlag {
                    id,
                    name: None,
                    group: None,
                    mat: false,
                })
                .await
            }
            _ => return Ok(res),
        }
    }

    pub async fn stop(tf: TaskFlag, to_cache: bool) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if !tasks.contains_key(&tf.id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.id),
            )));
        }
        let tp = tasks.get_mut(&tf.id).unwrap();

        if tp.task.status != Some("running".to_string())
            && tp.task.status != Some("auto restart".to_string())
        {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is not running", tf.id),
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
            Ok(Response::success(Some(Data::String(format!(
                "Task [{}] stopped",
                tf.id
            )))))
        } else {
            Ok(Response::wrong(format!("Task [{}] is not running", tf.id)))
        }
    }

    pub async fn write(tf: TaskFlag, data: String) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if !tasks.contains_key(&tf.id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.id),
            )));
        }
        let tp = tasks.get_mut(&tf.id).unwrap();

        if tp.task.status != Some("running".to_string()) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is not running", tf.id),
            )));
        }

        let tx = &tp.tx.clone().unwrap();

        let data: Vec<u8> = data.into_bytes();

        tx.send(data).await?;

        Ok(Response::success(None))
    }

    pub async fn list(condition: Option<TaskFlag>) -> Result<Response, Box<dyn Error>> {
        let tasks = TASKS.write().await;

        let res = match condition {
            Some(condition) => {
                let mut status: Vec<Status> = Vec::new();
                if condition.id > 0 {
                    if tasks.contains_key(&condition.id) {
                        let tp = tasks.get(&condition.id).unwrap();
                        status.push(tp.task.clone().into());
                    }
                } else if condition.mat {
                    let name = condition.name.unwrap_or(String::new());
                    for (_id, tp) in tasks.iter() {
                        let regex: Regex = Regex::new(&name)?;
                        if regex.is_match(&tp.task.name) {
                            status.push(tp.task.clone().into());
                        }
                    }
                } else if condition.group.is_some() {
                    let group = condition.group.unwrap_or(String::new());
                    for (_id, tp) in tasks.iter() {
                        let regex: Regex = Regex::new(&group)?;
                        if tp.task.group.is_some()
                            && regex.is_match(&tp.task.group.clone().unwrap())
                        {
                            status.push(tp.task.clone().into());
                        }
                    }
                } else {
                    let name = condition.name.unwrap_or(String::new());
                    for (_id, tp) in tasks.iter() {
                        if tp.task.name == name {
                            status.push(tp.task.clone().into());
                        }
                    }
                }
                status
            }
            None => {
                let mut status: Vec<Status> = Vec::new();
                for (_, tp) in tasks.iter() {
                    status.push(tp.task.clone().into());
                }
                status
            }
        };
        Ok(Response::success(Some(Data::Status(res))))
    }

    pub async fn pause(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if !tasks.contains_key(&tf.id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.id),
            )));
        }
        let tp = tasks.get_mut(&tf.id).unwrap();

        if tp.task.status != Some("interval".to_string())
            && tp.task.status != Some("executing".to_string())
        {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is not interval", tf.id),
            )));
        }

        tp.task.status = Some("paused".to_string());

        cache().await?;
        Ok(Response::success(Some(Data::String(format!(
            "Task [{}] paused",
            tf.id
        )))))
    }

    pub async fn resume(tf: TaskFlag) -> Result<Response, Box<dyn Error>> {
        let mut tasks = TASKS.write().await;
        if !tasks.contains_key(&tf.id) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] not exists", tf.id),
            )));
        }
        let tp = tasks.get_mut(&tf.id).unwrap();

        if tp.task.status != Some("paused".to_string()) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Task [{}] is not paused", tf.id),
            )));
        }

        tp.task.status = Some("interval".to_string());

        cache().await?;
        Ok(Response::success(Some(Data::String(format!(
            "Task [{}] resumed",
            tf.id
        )))))
    }
}
