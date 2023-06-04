#[cfg(test)]
mod tests {
    use common::{
        config::Config,
        handle::{Body, Command, Request, Response},
        task::Task,
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixStream,
    };

    #[tokio::test]
    async fn test_start_task() {
        let config: Config = Config::init(None).unwrap();
        let mut stream = UnixStream::connect(config.sock.path).await.unwrap();

        let current = std::env::current_dir().unwrap();
        let parent = current.parent().unwrap().to_str().unwrap();

        let mut task = Task::default();
        task.command = "python".to_string();
        task.args = vec!["-u".to_string(), format!("{}/script/task.py", parent)];
        task.stdout = Some(format!("{}/logs/stdout.log", parent));
        task.stdout = Some(format!("{}/logs/stderr.log", parent));
        task.stdin = Some(true);

        let request = Request {
            command: Command::Run,
            body: Body::Task(task),
        };
        let buf = serde_json::to_vec(&request).unwrap();
        stream.write_all(&buf).await.unwrap();

        let mut buf: [u8; 1024] = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        let res: Response<String> = serde_json::from_slice(&buf[..n]).unwrap();
        println!("{:#?}", res);
    }

    #[tokio::test]
    async fn test_stop_task() {
        let config: Config = Config::init(None).unwrap();
        let mut stream = UnixStream::connect(config.sock.path).await.unwrap();

        let request = Request {
            command: Command::Stop,
            body: Body::TaskFlag("Default".to_string()),
        };
        let buf = serde_json::to_vec(&request).unwrap();
        stream.write_all(&buf).await.unwrap();

        let mut buf: [u8; 1024] = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        let res: Response<String> = serde_json::from_slice(&buf[..n]).unwrap();
        println!("{:#?}", res);
    }
}
