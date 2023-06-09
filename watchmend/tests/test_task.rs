#[cfg(test)]
mod tests {
    use common::{
        config::Config,
        handle::{Command, Request, Response},
        task::{AsyncTask, Task, TaskFlag},
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixStream,
    };

    #[tokio::test]
    async fn test() {
        let current = std::env::current_dir().unwrap();
        let parent = current.parent().unwrap().to_str().unwrap();

        let mut task = Task::default();
        task.command = "python".to_string();
        task.args = vec!["-u".to_string(), format!("{}/script/task.py", parent)];
        task.stdout = Some(format!("{}/logs/stdout.log", parent));
        task.stderr = Some(format!("{}/logs/stderr.log", parent));
        task.stdin = Some(true);
        task.task_type = common::task::TaskType::Async(AsyncTask {
            started_at: 0,
            stopped_at: 0,
        });

        let request = Request {
            command: Command::Run(task),
        };
        let req_str = serde_json::to_string(&[request]).unwrap();
        println!("{}", req_str);
    }

    #[tokio::test]
    async fn test_start_task() {
        let config: Config = Config::init(None).unwrap();
        let mut stream = UnixStream::connect(config.sock.path).await.unwrap();

        let task = Task::default();

        let request = Request {
            command: Command::Start(TaskFlag {
                name: task.name,
                mat: false,
            }),
        };

        let buf = serde_json::to_vec(&[request]).unwrap();
        stream.write_all(&buf).await.unwrap();

        let mut buf: [u8; 1024] = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        let res: Vec<Response> = serde_json::from_slice(&buf[..n]).unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_stop_task() {
        let config: Config = Config::init(None).unwrap();
        let mut stream = UnixStream::connect(config.sock.path).await.unwrap();

        let request = Request {
            command: Command::Stop(TaskFlag {
                name: "Default".to_string(),
                mat: false,
            }),
        };
        let buf = serde_json::to_vec(&[request]).unwrap();
        stream.write_all(&buf).await.unwrap();

        let mut buf: [u8; 1024] = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        let res: Vec<Response> = serde_json::from_slice(&buf[..n]).unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_write_task() {
        let config: Config = Config::init(None).unwrap();
        let mut stream = UnixStream::connect(config.sock.path).await.unwrap();

        let request = Request {
            command: Command::Write(
                TaskFlag {
                    name: "Default".to_string(),
                    mat: false,
                },
                "{\"key\": \"value\"}\n".to_string(),
            ),
        };
        let buf = serde_json::to_vec(&[request]).unwrap();
        stream.write_all(&buf).await.unwrap();

        let mut buf: [u8; 1024] = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        let res: Vec<Response> = serde_json::from_slice(&buf[..n]).unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_list_task() {
        let config: Config = Config::init(None).unwrap();
        let mut stream = UnixStream::connect(config.sock.path).await.unwrap();

        let request = Request {
            command: Command::List(None),
        };
        let buf = serde_json::to_vec(&[request]).unwrap();
        stream.write_all(&buf).await.unwrap();

        let mut buf: [u8; 1024] = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        let res: Vec<Response> = serde_json::from_slice(&buf[..n]).unwrap();
        println!("{:#?}", res);
    }
}
