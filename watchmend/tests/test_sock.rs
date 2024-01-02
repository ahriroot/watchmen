#[cfg(test)]
mod tests {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::UnixStream,
    };
    use watchmend::common::{
        config::Config,
        handle::{Command, Request, Response},
        task::Task,
    };

    #[tokio::test]
    async fn test_request() {
        let config: Config = Config::init(None).unwrap();
        let mut stream = UnixStream::connect(config.sock.path).await.unwrap();

        let request = Request {
            command: Command::Run(Task::default()),
        };
        let buf = serde_json::to_vec(&[request]).unwrap();
        stream.write_all(&buf).await.unwrap();

        let mut buf: [u8; 1024] = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        let res: Vec<Response> = serde_json::from_slice(&buf[..n]).unwrap();
        println!("{:#?}", res);
    }
}
